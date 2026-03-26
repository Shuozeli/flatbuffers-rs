use super::type_map::{escape_keyword, has_type_index};
use super::{field_offset, field_type_index, obj_byte_size, obj_min_align, type_index};
use flatc_rs_schema::resolved::{ResolvedField, ResolvedObject, ResolvedSchema};
use flatc_rs_schema::BaseType;

use super::code_writer::CodeWriter;
use super::type_map;
use super::{type_visibility, CodeGenError, CodeGenOptions};

/// Generate Rust code for the struct at `schema.objects[index]`.
pub fn generate(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    index: usize,
    opts: &CodeGenOptions,
) -> Result<(), CodeGenError> {
    let obj = &schema.objects[index];
    let name = &obj.name;
    let byte_size = obj_byte_size(obj)?;
    let min_align = obj_min_align(obj)?;

    let vis = type_visibility(obj.attributes.as_ref(), opts);

    // Struct definition
    w.line(&format!("// struct {name}, aligned to {min_align}"));
    w.line("#[repr(transparent)]");
    w.line("#[derive(Clone, Copy, PartialEq)]");
    w.line(&format!("{vis} struct {name}(pub [u8; {byte_size}]);"));
    w.blank();

    // Default impl
    w.block(&format!("impl Default for {name}"), |w| {
        w.block("fn default() -> Self", |w| {
            w.line(&format!("Self([0; {byte_size}])"));
        });
    });
    w.blank();

    // Debug impl
    w.block(&format!("impl ::core::fmt::Debug for {name}"), |w| {
        w.block(
            "fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result",
            |w| {
                w.line(&format!("let mut s = f.debug_struct(\"{name}\");"));
                for field in &obj.fields {
                    let fname = &field.name;
                    let accessor = escape_keyword(&type_map::to_snake_case(fname));
                    w.line(&format!("s.field(\"{fname}\", &self.{accessor}());"));
                }
                w.line("s.finish()");
            },
        );
    });

    if opts.rust_serialize {
        w.blank();
        w.block(&format!("impl ::serde::Serialize for {name}"), |w| {
            w.block(
                "fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>\nwhere S: ::serde::Serializer",
                |w| {
                    let n = obj.fields.len();
                    w.line("use ::serde::ser::SerializeStruct;");
                    w.line(&format!("let mut s = serializer.serialize_struct(\"{name}\", {n})?;"));
                    for field in &obj.fields {
                        let fname = &field.name;
                        let accessor = type_map::to_snake_case(fname);
                        w.line(&format!("s.serialize_field(\"{fname}\", &self.{accessor}())?;"));
                    }
                    w.line("s.end()");
                },
            );
        });
    }

    w.blank();

    // Main impl block: constructor + accessors
    // Use impl<'a> if there are array fields (array getters return Array<'a, T, N>)
    let impl_header = if has_array_fields(obj) {
        format!("impl<'a> {name}")
    } else {
        format!("impl {name}")
    };
    w.try_block(&impl_header, |w| {
        gen_constructor(w, schema, obj)?;
        w.blank();

        // Generate getters and setters for each field
        for field in &obj.fields {
            gen_field_getter(w, schema, field)?;
            w.blank();
            gen_field_setter(w, schema, field)?;
            w.blank();
        }

        // Key comparison methods (for fields with `key` attribute)
        if let Some(key_field) = find_key_field(obj) {
            gen_struct_key_methods(w, schema, key_field, name)?;
        }
        Ok(())
    })?;
    w.blank();

    // Follow<'a> for T (returns &T)
    w.block(
        &format!("impl<'a> ::flatbuffers::Follow<'a> for {name}"),
        |w| {
            w.line(&format!("type Inner = &'a {name};"));
            w.line("#[inline]");
            w.block(
                "unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner",
                |w| {
                    w.line(&format!("unsafe {{ <&'a {name}>::follow(buf, loc) }}"));
                },
            );
        },
    );
    w.blank();

    // Follow<'a> for &T (returns &T)
    w.block(
        &format!("impl<'a> ::flatbuffers::Follow<'a> for &'a {name}"),
        |w| {
            w.line(&format!("type Inner = &'a {name};"));
            w.line("#[inline]");
            w.block(
                "unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner",
                |w| {
                    w.line(&format!(
                        "unsafe {{ ::flatbuffers::follow_cast_ref::<{name}>(buf, loc) }}"
                    ));
                },
            );
        },
    );
    w.blank();

    // Push impl
    w.block(&format!("impl<'b> ::flatbuffers::Push for {name}"), |w| {
        w.line(&format!("type Output = {name};"));
        w.line("#[inline]");
        w.block(
            "unsafe fn push(&self, dst: &mut [u8], _written_len: usize)",
            |w| {
                w.line(&format!(
                    "let src = unsafe {{ ::core::slice::from_raw_parts(self as *const {name} as *const u8, <Self as ::flatbuffers::Push>::size()) }};"
                ));
                w.line("dst.copy_from_slice(src);");
            },
        );
        w.line("#[inline]");
        w.block("fn alignment() -> ::flatbuffers::PushAlignment", |w| {
            w.line(&format!(
                "::flatbuffers::PushAlignment::new({min_align})"
            ));
        });
    });
    w.blank();

    // Verifiable impl
    w.block(
        &format!("impl<'a> ::flatbuffers::Verifiable for {name}"),
        |w| {
            w.line("#[inline]");
            w.block(
                "fn run_verifier(v: &mut ::flatbuffers::Verifier, pos: usize) -> Result<(), ::flatbuffers::InvalidFlatbuffer>",
                |w| {
                    w.line("v.in_buffer::<Self>(pos)");
                },
            );
        },
    );
    w.blank();

    // SimpleToVerifyInSlice marker
    w.line(&format!(
        "impl ::flatbuffers::SimpleToVerifyInSlice for {name} {{}}"
    ));

    // Object API: owned T type with pack/unpack
    if opts.gen_object_api {
        w.blank();
        gen_object_api(w, schema, index, opts)?;
    }
    Ok(())
}

/// Generate the `new(...)` constructor.
fn gen_constructor(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
) -> Result<(), CodeGenError> {
    if obj.fields.is_empty() {
        return Ok(());
    }

    // Build parameter list
    let params: Vec<String> = obj
        .fields
        .iter()
        .map(|f| {
            let fname = escape_keyword(&type_map::to_snake_case(&f.name));
            let bt = f.type_.base_type;
            if bt == BaseType::BASE_TYPE_ARRAY {
                let (elem_type_str, fixed_len) = array_element_info(schema, f)?;
                Ok(format!("{fname}: &[{elem_type_str}; {fixed_len}]"))
            } else if bt == BaseType::BASE_TYPE_STRUCT {
                let ftype = field_rust_type(schema, f)?;
                Ok(format!("{fname}: &{ftype}"))
            } else {
                let ftype = field_rust_type(schema, f)?;
                Ok(format!("{fname}: {ftype}"))
            }
        })
        .collect::<Result<Vec<_>, CodeGenError>>()?;

    let byte_size = obj_byte_size(obj)?;
    w.block(&format!("pub fn new({}) -> Self", params.join(", ")), |w| {
        w.line(&format!("let mut s = Self([0; {byte_size}]);"));
        for field in &obj.fields {
            let fname = escape_keyword(&type_map::to_snake_case(&field.name));
            w.line(&format!("s.set_{fname}({fname});"));
        }
        w.line("s");
    });
    Ok(())
}

/// Generate a getter for a struct field.
fn gen_field_getter(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
) -> Result<(), CodeGenError> {
    let fname = escape_keyword(&type_map::to_snake_case(&field.name));
    let offset = field_offset(field)?;

    let bt = field.type_.base_type;

    // Array fields return ::flatbuffers::Array<'a, T, N>
    if bt == BaseType::BASE_TYPE_ARRAY {
        let (elem_type_str, fixed_len) = array_element_info(schema, field)?;
        w.line(&format!(
            "pub fn {fname}(&'a self) -> ::flatbuffers::Array<'a, {elem_type_str}, {fixed_len}> {{"
        ));
        w.indent();
        w.line("use ::flatbuffers::Follow;");
        w.line(&format!(
            "unsafe {{ ::flatbuffers::Array::follow(&self.0, {offset}) }}"
        ));
        w.dedent();
        w.line("}");
        return Ok(());
    }

    let ftype = field_rust_type(schema, field)?;

    // Struct fields inside structs are read via nested struct accessor
    if bt == BaseType::BASE_TYPE_STRUCT {
        let struct_idx = field_type_index(field)?;
        let struct_name = &schema.objects[struct_idx].name;
        let struct_size = obj_byte_size(&schema.objects[struct_idx])?;

        w.line(&format!("pub fn {fname}(&self) -> &{struct_name} {{"));
        w.indent();
        w.line(&format!(
            "unsafe {{ &*(self.0[{offset}..{offset}+{struct_size}].as_ptr() as *const {struct_name}) }}"
        ));
        w.dedent();
        w.line("}");
        return Ok(());
    }

    // Check if this is an enum-typed field (scalar base type with index)
    if type_map::is_scalar(bt) && has_type_index(field) {
        let enum_idx = field_type_index(field)?;
        let enum_name = &schema.enums[enum_idx].name;

        // Use EndianScalar trait methods on the enum type directly.
        // This works for both regular enums and bitflags (avoids private field access).
        w.line(&format!("pub fn {fname}(&self) -> {enum_name} {{"));
        w.indent();
        w.line(&format!(
            "let mut mem = ::core::mem::MaybeUninit::<<{enum_name} as ::flatbuffers::EndianScalar>::Scalar>::uninit();"
        ));
        w.block("unsafe", |w| {
            w.line("::core::ptr::copy_nonoverlapping(");
            w.indent();
            w.line(&format!("self.0[{offset}..].as_ptr(),"));
            w.line("mem.as_mut_ptr() as *mut u8,");
            w.line(&format!(
                "::core::mem::size_of::<<{enum_name} as ::flatbuffers::EndianScalar>::Scalar>(),"
            ));
            w.dedent();
            w.line(");");
        });
        w.line(&format!("<{enum_name} as ::flatbuffers::EndianScalar>::from_little_endian(unsafe {{ mem.assume_init() }})"));
        w.dedent();
        w.line("}");
        return Ok(());
    }

    // Regular scalar field
    w.line(&format!("pub fn {fname}(&self) -> {ftype} {{"));
    w.indent();
    w.line(&format!(
        "let mut mem = ::core::mem::MaybeUninit::<<{ftype} as ::flatbuffers::EndianScalar>::Scalar>::uninit();"
    ));
    w.line("::flatbuffers::EndianScalar::from_little_endian(unsafe {");
    w.indent();
    w.line("::core::ptr::copy_nonoverlapping(");
    w.indent();
    w.line(&format!("self.0[{offset}..].as_ptr(),"));
    w.line("mem.as_mut_ptr() as *mut u8,");
    w.line(&format!(
        "::core::mem::size_of::<<{ftype} as ::flatbuffers::EndianScalar>::Scalar>(),"
    ));
    w.dedent();
    w.line(");");
    w.line("mem.assume_init()");
    w.dedent();
    w.line("})");
    w.dedent();
    w.line("}");
    Ok(())
}

/// Generate a setter for a struct field.
fn gen_field_setter(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
) -> Result<(), CodeGenError> {
    let fname = escape_keyword(&type_map::to_snake_case(&field.name));
    let offset = field_offset(field)?;

    let bt = field.type_.base_type;

    // Array fields
    if bt == BaseType::BASE_TYPE_ARRAY {
        let et = field.type_.element_type_or_none();
        let (elem_type_str, fixed_len) = array_element_info(schema, field)?;

        if et == BaseType::BASE_TYPE_STRUCT {
            // Struct arrays: raw byte copy
            let struct_idx = field_type_index(field)?;
            let struct_size = obj_byte_size(&schema.objects[struct_idx])?;
            let total_bytes = struct_size * fixed_len;
            w.line(&format!(
                "pub fn set_{fname}(&mut self, x: &[{elem_type_str}; {fixed_len}]) {{"
            ));
            w.indent();
            w.block("unsafe", |w| {
                w.line("::core::ptr::copy(");
                w.indent();
                w.line("x.as_ptr() as *const u8,");
                w.line(&format!("self.0.as_mut_ptr().add({offset}),"));
                w.line(&format!("{total_bytes},"));
                w.dedent();
                w.line(");");
            });
            w.dedent();
            w.line("}");
        } else {
            // Scalar/enum arrays: use emplace_scalar_array for endian safety
            w.line(&format!(
                "pub fn set_{fname}(&mut self, items: &[{elem_type_str}; {fixed_len}]) {{"
            ));
            w.indent();
            w.line(&format!(
                "unsafe {{ ::flatbuffers::emplace_scalar_array(&mut self.0, {offset}, items) }};"
            ));
            w.dedent();
            w.line("}");
        }
        return Ok(());
    }

    // Struct fields inside structs
    if bt == BaseType::BASE_TYPE_STRUCT {
        let struct_idx = field_type_index(field)?;
        let struct_name = &schema.objects[struct_idx].name;
        let struct_size = obj_byte_size(&schema.objects[struct_idx])?;

        w.line(&format!(
            "pub fn set_{fname}(&mut self, {fname}: &{struct_name}) {{"
        ));
        w.indent();
        w.line(&format!(
            "self.0[{offset}..{offset}+{struct_size}].copy_from_slice(&{fname}.0);"
        ));
        w.dedent();
        w.line("}");
        return Ok(());
    }

    let ftype = field_rust_type(schema, field)?;

    // Enum-typed field: use EndianScalar trait to avoid private field access (bitflags)
    if type_map::is_scalar(bt) && has_type_index(field) {
        let enum_idx = field_type_index(field)?;
        let enum_name = &schema.enums[enum_idx].name;

        w.line(&format!(
            "pub fn set_{fname}(&mut self, {fname}: {enum_name}) {{"
        ));
        w.indent();
        w.line(&format!(
            "let {fname}_le = <{enum_name} as ::flatbuffers::EndianScalar>::to_little_endian({fname});"
        ));
        w.block("unsafe", |w| {
            w.line("::core::ptr::copy_nonoverlapping(");
            w.indent();
            w.line(&format!("&{fname}_le as *const _ as *const u8,"));
            w.line(&format!("self.0[{offset}..].as_mut_ptr(),"));
            w.line(&format!(
                "::core::mem::size_of::<<{enum_name} as ::flatbuffers::EndianScalar>::Scalar>(),"
            ));
            w.dedent();
            w.line(");");
        });
        w.dedent();
        w.line("}");
        return Ok(());
    }

    // Regular scalar
    w.line(&format!(
        "pub fn set_{fname}(&mut self, {fname}: {ftype}) {{"
    ));
    w.indent();
    w.line(&format!(
        "let {fname}_le = ::flatbuffers::EndianScalar::to_little_endian({fname});"
    ));
    w.block("unsafe", |w| {
        w.line("::core::ptr::copy_nonoverlapping(");
        w.indent();
        w.line(&format!("&{fname}_le as *const _ as *const u8,"));
        w.line(&format!("self.0[{offset}..].as_mut_ptr(),"));
        w.line(&format!(
            "::core::mem::size_of::<<{ftype} as ::flatbuffers::EndianScalar>::Scalar>(),"
        ));
        w.dedent();
        w.line(");");
    });
    w.dedent();
    w.line("}");
    Ok(())
}

/// Get the Rust type string for a struct field (scalars only, or nested struct reference).
fn field_rust_type(schema: &ResolvedSchema, field: &ResolvedField) -> Result<String, CodeGenError> {
    let bt = field.type_.base_type;

    if bt == BaseType::BASE_TYPE_STRUCT {
        let idx = field_type_index(field)?;
        return Ok(schema.objects[idx].name.clone());
    }

    // Check if this is an enum-typed field
    if type_map::is_scalar(bt) && has_type_index(field) {
        let enum_idx = field_type_index(field)?;
        return Ok(schema.enums[enum_idx].name.clone());
    }

    Ok(type_map::scalar_rust_type(bt).to_string())
}

/// Get the element Rust type name and fixed length for an array field.
fn array_element_info(
    schema: &ResolvedSchema,
    field: &ResolvedField,
) -> Result<(String, usize), CodeGenError> {
    let ty = &field.type_;
    let et = ty.element_type.unwrap_or(BaseType::BASE_TYPE_NONE);
    let fixed_len = ty.fixed_length.ok_or_else(|| {
        CodeGenError::Internal(format!("array field '{}' has no fixed_length", &field.name))
    })? as usize;

    let elem_type_str = if et == BaseType::BASE_TYPE_STRUCT {
        let idx = type_index(ty, "array element struct lookup")?;
        schema.objects[idx].name.clone()
    } else if type_map::is_scalar(et) {
        // Check for enum index on the array type
        if let Some(idx) = ty.index {
            if idx >= 0 {
                schema.enums[idx as usize].name.clone()
            } else {
                type_map::scalar_rust_type(et).to_string()
            }
        } else {
            type_map::scalar_rust_type(et).to_string()
        }
    } else {
        type_map::scalar_rust_type(et).to_string()
    };

    Ok((elem_type_str, fixed_len))
}

/// Returns true if any field in the struct is an array type.
fn has_array_fields(obj: &ResolvedObject) -> bool {
    obj.fields
        .iter()
        .any(|f| f.type_.base_type == BaseType::BASE_TYPE_ARRAY)
}

/// Find the key field in a struct.
fn find_key_field(obj: &ResolvedObject) -> Option<&ResolvedField> {
    obj.fields
        .iter()
        .find(|f| super::rust_table_gen::helpers::has_key_attribute(f))
}

/// Generate the Object API for a struct: owned `{Name}T` type with `pack`/`unpack`.
fn gen_object_api(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    index: usize,
    opts: &CodeGenOptions,
) -> Result<(), CodeGenError> {
    let obj = &schema.objects[index];
    let name = &obj.name;
    let vis = type_visibility(obj.attributes.as_ref(), opts);
    let t_name = format!("{name}T");

    // Skip if it has array fields (complex; defer to future work)
    if has_array_fields(obj) {
        eprintln!(
            "warning: Object API not generated for struct '{}' (contains array fields)",
            name
        );
        return Ok(());
    }

    // Pre-compute owned types for fields so we don't need Result inside closures
    let field_owned_types: Vec<String> = obj
        .fields
        .iter()
        .map(|field| {
            let bt = field.type_.base_type;
            struct_owned_field_type(schema, field, bt)
        })
        .collect::<Result<Vec<_>, CodeGenError>>()?;

    // Generate the owned struct definition
    let mut derives = vec!["Debug", "Clone", "PartialEq", "Default"];
    if opts.rust_serialize {
        derives.push("::serde::Serialize");
        derives.push("::serde::Deserialize");
    }
    w.line(&format!("#[derive({})]", derives.join(", ")));
    w.block(&format!("{vis} struct {t_name}"), |w| {
        for (i, field) in obj.fields.iter().enumerate() {
            let fname = escape_keyword(&type_map::to_snake_case(&field.name));
            let owned_type = &field_owned_types[i];
            w.line(&format!("pub {fname}: {owned_type},"));
        }
    });
    w.blank();

    // Generate pack: {Name}T -> {Name}
    w.block(&format!("impl {t_name}"), |w| {
        w.block(&format!("pub fn pack(&self) -> {name}"), |w| {
            // Build argument list for the struct constructor
            let args: Vec<String> = obj
                .fields
                .iter()
                .map(|f| {
                    let fname = escape_keyword(&type_map::to_snake_case(&f.name));
                    let bt = f.type_.base_type;
                    if bt == BaseType::BASE_TYPE_STRUCT {
                        // Nested struct: pack and pass by reference
                        format!("&self.{fname}.pack()")
                    } else {
                        format!("self.{fname}")
                    }
                })
                .collect();
            w.line(&format!("{name}::new({})", args.join(", ")));
        });
    });
    w.blank();

    // Generate unpack: {Name} -> {Name}T
    w.block(&format!("impl {name}"), |w| {
        w.block(&format!("pub fn unpack(&self) -> {t_name}"), |w| {
            w.line(&format!("{t_name} {{"));
            w.indent();
            for field in &obj.fields {
                let fname = escape_keyword(&type_map::to_snake_case(&field.name));
                let bt = field.type_.base_type;
                if bt == BaseType::BASE_TYPE_STRUCT {
                    w.line(&format!("{fname}: self.{fname}().unpack(),"));
                } else {
                    w.line(&format!("{fname}: self.{fname}(),"));
                }
            }
            w.dedent();
            w.line("}");
        });
    });
    Ok(())
}

/// Get the owned Rust type for a struct field in the Object API.
fn struct_owned_field_type(
    schema: &ResolvedSchema,
    field: &ResolvedField,
    bt: BaseType,
) -> Result<String, CodeGenError> {
    if bt == BaseType::BASE_TYPE_STRUCT {
        let idx = field_type_index(field)?;
        let struct_name = &schema.objects[idx].name;
        Ok(format!("{struct_name}T"))
    } else if type_map::is_scalar(bt) {
        // Check for enum-typed field
        if has_type_index(field) {
            let enum_idx = field_type_index(field)?;
            Ok(schema.enums[enum_idx].name.clone())
        } else {
            Ok(type_map::scalar_rust_type(bt).to_string())
        }
    } else {
        Ok(type_map::scalar_rust_type(bt).to_string())
    }
}

/// Generate key comparison methods for a struct.
fn gen_struct_key_methods(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    struct_name: &str,
) -> Result<(), CodeGenError> {
    let fname = &field.name;
    let accessor = escape_keyword(&type_map::to_snake_case(fname));
    let rust_type = field_rust_type(schema, field)?;

    w.line("#[inline]");
    w.block(
        &format!("pub fn key_compare_less_than(&self, o: &{struct_name}) -> bool"),
        |w| {
            w.line(&format!("self.{accessor}() < o.{accessor}()"));
        },
    );
    w.blank();

    w.line("#[inline]");
    w.block(
        &format!("pub fn key_compare_with_value(&self, val: {rust_type}) -> ::core::cmp::Ordering"),
        |w| {
            w.line(&format!("let key = self.{accessor}();"));
            w.line("key.cmp(&val)");
        },
    );
    Ok(())
}
