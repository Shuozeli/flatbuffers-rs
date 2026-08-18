use crate::type_map::has_type_index;
use crate::{field_id, field_type_index, union_variant_type_index};
use flatc_rs_schema::resolved::{ResolvedField, ResolvedObject, ResolvedSchema};
use flatc_rs_schema::BaseType;

use crate::type_map;
use crate::{CodeGenError, CodeGenOptions};
use codegen_core::CodeWriter;

use super::helpers;

/// Context for generating a scalar accessor.
struct GenScalarAccessorContext<'a> {
    schema: &'a ResolvedSchema,
    field: &'a ResolvedField,
    accessor_name: &'a str,
    upper_name: &'a str,
    bt: BaseType,
    is_optional: bool,
    table_name: &'a str,
    current_ns: &'a str,
}

/// `pub enum FooOffset {}`
pub(super) fn gen_offset_marker(w: &mut CodeWriter, name: &str, vis: &str) {
    w.line(&format!("{vis} enum {name}Offset {{}}"));
}

/// Reader struct with lifetime.
pub(super) fn gen_reader_struct(w: &mut CodeWriter, name: &str, vis: &str, opts: &CodeGenOptions) {
    w.line("#[derive(Copy, Clone, PartialEq)]");
    if opts.rust_pluggable_buffer {
        w.block(
            &format!(
                "{vis} struct {name}<'a, B: ?Sized + __flatc_rs_runtime::FlatBufferRead = [u8]>"
            ),
            |w| {
                w.line("_buf: &'a B,");
                w.line("_loc: usize,");
            },
        );
    } else {
        w.block(&format!("{vis} struct {name}<'a>"), |w| {
            w.line("pub _tab: ::flatbuffers::Table<'a>,");
        });
    }
}

/// Follow impl for the reader.
pub(super) fn gen_follow_impl(w: &mut CodeWriter, name: &str, opts: &CodeGenOptions) {
    w.block(
        &format!("impl<'a> ::flatbuffers::Follow<'a> for {name}<'a>"),
        |w| {
            w.line(&format!("type Inner = {name}<'a>;"));
            w.line("#[inline]");
            w.block(
                "unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner",
                |w| {
                    if opts.rust_pluggable_buffer {
                        w.line("Self { _buf: buf, _loc: loc }");
                    } else {
                        w.line("Self { _tab: unsafe { ::flatbuffers::Table::new(buf, loc) } }");
                    }
                },
            );
        },
    );
    if opts.rust_pluggable_buffer {
        w.blank();
        w.block(
            &format!("unsafe impl<'a, B: ?Sized + __flatc_rs_runtime::FlatBufferRead> __flatc_rs_runtime::FollowIn<'a, B> for ::flatbuffers::ForwardsUOffset<{name}<'a, B>>"),
            |w| {
                w.line(&format!("type Inner = {name}<'a, B>;"));
                w.line("#[inline]");
                w.block("unsafe fn follow_in(buf: &'a B, loc: usize) -> Self::Inner", |w| {
                    w.line(&format!(
                        "{name}::init_from_buffer(buf, __flatc_rs_runtime::uoffset_target(buf, loc))"
                    ));
                });
            },
        );
    }
}

/// Main impl block with VT constants and accessors.
pub(super) fn gen_impl_block(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
    current_ns: &str,
    opts: &CodeGenOptions,
) -> Result<(), CodeGenError> {
    // Pre-compute VT offsets so we don't need Result inside the closure
    let vt_offsets: Vec<(String, u32)> = obj
        .fields
        .iter()
        .map(|field| {
            let fname = &field.name;
            let upper = type_map::rust_field_offset_name(fname);
            let slot = field_id(field)?;
            let vt_offset = 4 + 2 * slot;
            Ok((upper, vt_offset))
        })
        .collect::<Result<Vec<_>, CodeGenError>>()?;

    let impl_header = if opts.rust_pluggable_buffer {
        format!("impl<'a, B: ?Sized + __flatc_rs_runtime::FlatBufferRead> {name}<'a, B>")
    } else {
        format!("impl<'a> {name}<'a>")
    };

    w.try_block(&impl_header, |w| -> Result<(), CodeGenError> {
        // VTable offset constants
        for (upper, vt_offset) in &vt_offsets {
            w.line(&format!(
                "pub const VT_{upper}: ::flatbuffers::VOffsetT = {vt_offset};"
            ));
        }
        w.blank();
        // init_from_table (used by union accessors)
        w.line("#[inline]");
        if opts.rust_pluggable_buffer {
            w.block(
                "pub unsafe fn init_from_buffer(buf: &'a B, loc: usize) -> Self",
                |w| {
                    w.line(&format!("{name} {{ _buf: buf, _loc: loc }}"));
                },
            );
            w.blank();
            w.line("#[inline]");
            w.block(
                "pub unsafe fn init_from_table(table: __flatc_rs_runtime::Table<'a, B>) -> Self",
                |w| {
                    w.line(&format!("{name} {{ _buf: table.buf, _loc: table.loc }}"));
                },
            );
        } else {
            w.block(
                "pub unsafe fn init_from_table(table: ::flatbuffers::Table<'a>) -> Self",
                |w| {
                    w.line(&format!("{name} {{ _tab: table }}"));
                },
            );
        }
        if !opts.rust_pluggable_buffer {
            gen_create_method(w, obj, name);
            w.blank(); // C++ emits a double blank after create()
            w.blank();
        }

        // Field accessors - key methods are emitted right after the key field
        for field in &obj.fields {
            gen_field_accessor(w, schema, field, name, current_ns, opts)?;
            // Key comparison methods come right after the key field accessor
            if helpers::has_key_attribute(field) {
                w.blank();
                gen_key_methods(w, schema, field, name, current_ns)?;
            }
        }
        Ok(())
    })?;

    if opts.rust_pluggable_buffer {
        w.blank();
        w.block(&format!("impl<'a> {name}<'a, [u8]>"), |w| {
            gen_create_method(w, obj, name);
        });
    }

    Ok(())
}

/// Generate an accessor method for a table field.
fn gen_field_accessor(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    table_name: &str,
    current_ns: &str,
    opts: &CodeGenOptions,
) -> Result<(), CodeGenError> {
    let fname = &field.name;
    let accessor_name = type_map::rust_field_name(fname);
    let upper = type_map::rust_field_offset_name(fname);

    let bt = field.type_.base_type;

    let is_required = helpers::is_field_required(field);
    let is_optional_scalar = field.is_optional;
    let is_deprecated = field.is_deprecated;

    if is_deprecated {
        w.line("#[deprecated]");
    }
    type_map::gen_rust_doc_comment(w, field.documentation.as_ref());
    w.line("#[inline]");

    match bt {
        bt if type_map::is_scalar(bt) => {
            gen_scalar_accessor(
                w,
                GenScalarAccessorContext {
                    schema,
                    field,
                    accessor_name: &accessor_name,
                    upper_name: &upper,
                    bt,
                    is_optional: is_optional_scalar,
                    table_name,
                    current_ns,
                },
                opts,
            )?;
        }
        BaseType::BASE_TYPE_STRING => {
            gen_string_accessor(
                w,
                field,
                &accessor_name,
                &upper,
                table_name,
                is_required,
                opts,
            );
        }
        BaseType::BASE_TYPE_STRUCT => {
            gen_struct_field_accessor(
                w,
                schema,
                field,
                &accessor_name,
                &upper,
                table_name,
                current_ns,
                opts,
            )?;
        }
        BaseType::BASE_TYPE_TABLE => {
            gen_table_field_accessor(
                w,
                schema,
                field,
                &accessor_name,
                &upper,
                table_name,
                current_ns,
                opts,
            )?;
        }
        BaseType::BASE_TYPE_VECTOR => {
            gen_vector_accessor(
                w,
                schema,
                field,
                &accessor_name,
                &upper,
                table_name,
                current_ns,
                opts,
            )?;
        }
        BaseType::BASE_TYPE_UNION => {
            gen_union_accessor(
                w,
                schema,
                field,
                &accessor_name,
                &upper,
                table_name,
                current_ns,
                opts,
            )?;
        }
        _ => {
            return Err(CodeGenError::Internal(format!(
                "unhandled BaseType {bt:?} for accessor '{accessor_name}'"
            )));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn gen_scalar_accessor(
    w: &mut CodeWriter,
    ctx: GenScalarAccessorContext<'_>,
    opts: &CodeGenOptions,
) -> Result<(), CodeGenError> {
    // Check if this is an enum field (has index pointing to an enum)
    if has_type_index(ctx.field) {
        let enum_idx = field_type_index(ctx.field)?;
        let enum_name = type_map::resolve_enum_name(ctx.schema, ctx.current_ns, enum_idx);
        let default =
            helpers::scalar_builder_default(ctx.schema, ctx.field, ctx.bt, ctx.current_ns)?;

        if ctx.is_optional {
            w.line(&format!(
                "pub fn {}(&self) -> Option<{enum_name}> {{",
                ctx.accessor_name
            ));
            w.indent();
            w.line("// Safety:");
            w.line("// Created from valid Table for this object");
            w.line("// which contains a valid value in this slot");
            if opts.rust_pluggable_buffer {
                w.line(&format!(
                    "unsafe {{ __flatc_rs_runtime::table_get::<{enum_name}, B>(self._buf, self._loc, Self::VT_{}, None) }}",
                    ctx.upper_name
                ));
            } else {
                w.line(&format!(
                    "unsafe {{ self._tab.get::<{enum_name}>({}::VT_{}, None) }}",
                    ctx.table_name, ctx.upper_name
                ));
            }
        } else {
            w.line(&format!(
                "pub fn {}(&self) -> {enum_name} {{",
                ctx.accessor_name
            ));
            w.indent();
            w.line("// Safety:");
            w.line("// Created from valid Table for this object");
            w.line("// which contains a valid value in this slot");
            if opts.rust_pluggable_buffer {
                w.line(&format!(
                    "unsafe {{ __flatc_rs_runtime::table_get::<{enum_name}, B>(self._buf, self._loc, Self::VT_{}, Some({default})).unwrap()}}",
                    ctx.upper_name
                ));
            } else {
                w.line(&format!(
                    "unsafe {{ self._tab.get::<{enum_name}>({}::VT_{}, Some({default})).unwrap()}}",
                    ctx.table_name, ctx.upper_name
                ));
            }
        }
        w.dedent();
        w.line("}");
    } else {
        let rust_type = type_map::scalar_rust_type(ctx.bt);
        let default = helpers::scalar_default(ctx.field, ctx.bt);

        if ctx.is_optional {
            w.line(&format!(
                "pub fn {}(&self) -> Option<{rust_type}> {{",
                ctx.accessor_name
            ));
            w.indent();
            w.line("// Safety:");
            w.line("// Created from valid Table for this object");
            w.line("// which contains a valid value in this slot");
            if opts.rust_pluggable_buffer {
                w.line(&format!(
                    "unsafe {{ __flatc_rs_runtime::table_get::<{rust_type}, B>(self._buf, self._loc, Self::VT_{}, None) }}",
                    ctx.upper_name
                ));
            } else {
                w.line(&format!(
                    "unsafe {{ self._tab.get::<{rust_type}>({}::VT_{}, None) }}",
                    ctx.table_name, ctx.upper_name
                ));
            }
        } else {
            w.line(&format!(
                "pub fn {}(&self) -> {rust_type} {{",
                ctx.accessor_name
            ));
            w.indent();
            w.line("// Safety:");
            w.line("// Created from valid Table for this object");
            w.line("// which contains a valid value in this slot");
            if opts.rust_pluggable_buffer {
                w.line(&format!(
                    "unsafe {{ __flatc_rs_runtime::table_get::<{rust_type}, B>(self._buf, self._loc, Self::VT_{}, Some({default})).unwrap()}}",
                    ctx.upper_name
                ));
            } else {
                w.line(&format!(
                    "unsafe {{ self._tab.get::<{rust_type}>({}::VT_{}, Some({default})).unwrap()}}",
                    ctx.table_name, ctx.upper_name
                ));
            }
        }
        w.dedent();
        w.line("}");
    }
    Ok(())
}

fn gen_string_accessor(
    w: &mut CodeWriter,
    field: &ResolvedField,
    accessor_name: &str,
    upper_name: &str,
    table_name: &str,
    is_required: bool,
    opts: &CodeGenOptions,
) {
    let has_default = field.default_string.is_some();

    if is_required {
        w.line(&format!("pub fn {accessor_name}(&self) -> &'a str {{"));
        w.indent();
        w.line("// Safety:");
        w.line("// Created from valid Table for this object");
        w.line("// which contains a valid value in this slot");
        if opts.rust_pluggable_buffer {
            w.line(&format!(
                "unsafe {{ __flatc_rs_runtime::table_get_string(self._buf, self._loc, Self::VT_{upper_name}, None).unwrap()}}"
            ));
        } else {
            w.line(&format!(
                "unsafe {{ self._tab.get::<::flatbuffers::ForwardsUOffset<&str>>({table_name}::VT_{upper_name}, None).unwrap()}}"
            ));
        }
    } else if has_default {
        let default_val = field.default_string.as_deref().unwrap_or("");
        w.line(&format!("pub fn {accessor_name}(&self) -> &'a str {{"));
        w.indent();
        w.line("// Safety:");
        w.line("// Created from valid Table for this object");
        w.line("// which contains a valid value in this slot");
        if opts.rust_pluggable_buffer {
            w.line(&format!(
                "unsafe {{ __flatc_rs_runtime::table_get_string(self._buf, self._loc, Self::VT_{upper_name}, Some(&\"{default_val}\")).unwrap()}}"
            ));
        } else {
            w.line(&format!(
                "unsafe {{ self._tab.get::<::flatbuffers::ForwardsUOffset<&str>>({table_name}::VT_{upper_name}, Some(&\"{default_val}\")).unwrap()}}"
            ));
        }
    } else {
        w.line(&format!(
            "pub fn {accessor_name}(&self) -> Option<&'a str> {{"
        ));
        w.indent();
        w.line("// Safety:");
        w.line("// Created from valid Table for this object");
        w.line("// which contains a valid value in this slot");
        if opts.rust_pluggable_buffer {
            w.line(&format!(
                "unsafe {{ __flatc_rs_runtime::table_get_string(self._buf, self._loc, Self::VT_{upper_name}, None)}}"
            ));
        } else {
            w.line(&format!(
                "unsafe {{ self._tab.get::<::flatbuffers::ForwardsUOffset<&str>>({table_name}::VT_{upper_name}, None)}}"
            ));
        }
    }
    w.dedent();
    w.line("}");
}

#[allow(clippy::too_many_arguments)]
fn gen_struct_field_accessor(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    accessor_name: &str,
    upper_name: &str,
    table_name: &str,
    current_ns: &str,
    opts: &CodeGenOptions,
) -> Result<(), CodeGenError> {
    let struct_idx = field_type_index(field)?;
    let struct_name = type_map::resolve_object_name(schema, current_ns, struct_idx);

    w.line(&format!(
        "pub fn {accessor_name}(&self) -> Option<&'a {struct_name}> {{"
    ));
    w.indent();
    w.line("// Safety:");
    w.line("// Created from valid Table for this object");
    w.line("// which contains a valid value in this slot");
    if opts.rust_pluggable_buffer {
        w.line(&format!(
            "unsafe {{ __flatc_rs_runtime::table_get_struct::<{struct_name}, B>(self._buf, self._loc, Self::VT_{upper_name})}}"
        ));
    } else {
        w.line(&format!(
            "unsafe {{ self._tab.get::<{struct_name}>({table_name}::VT_{upper_name}, None)}}"
        ));
    }
    w.dedent();
    w.line("}");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn gen_table_field_accessor(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    accessor_name: &str,
    upper_name: &str,
    table_name: &str,
    current_ns: &str,
    opts: &CodeGenOptions,
) -> Result<(), CodeGenError> {
    let table_idx = field_type_index(field)?;
    let field_table_name = type_map::resolve_object_name(schema, current_ns, table_idx);

    if opts.rust_pluggable_buffer {
        w.line(&format!(
            "pub fn {accessor_name}(&self) -> Option<{field_table_name}<'a, B>> {{"
        ));
    } else {
        w.line(&format!(
            "pub fn {accessor_name}(&self) -> Option<{field_table_name}<'a>> {{"
        ));
    }
    w.indent();
    w.line("// Safety:");
    w.line("// Created from valid Table for this object");
    w.line("// which contains a valid value in this slot");
    if opts.rust_pluggable_buffer {
        w.line(&format!(
            "unsafe {{ __flatc_rs_runtime::table_field_loc(self._buf, self._loc, Self::VT_{upper_name}).map(|loc| {field_table_name}::init_from_buffer(self._buf, __flatc_rs_runtime::uoffset_target(self._buf, loc))) }}"
        ));
    } else {
        w.line(&format!(
            "unsafe {{ self._tab.get::<::flatbuffers::ForwardsUOffset<{field_table_name}>>({table_name}::VT_{upper_name}, None)}}"
        ));
    }
    w.dedent();
    w.line("}");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn gen_vector_accessor(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    accessor_name: &str,
    upper_name: &str,
    table_name: &str,
    current_ns: &str,
    opts: &CodeGenOptions,
) -> Result<(), CodeGenError> {
    let element_bt = field.type_.element_type_or_none();
    let has_default = field.default_string.is_some();

    let vector_inner = if opts.rust_pluggable_buffer {
        helpers::pluggable_vector_element_type(schema, field, element_bt, current_ns)?
    } else {
        helpers::vector_element_type(schema, field, element_bt, "'a", current_ns)?
    };
    let full_type = if opts.rust_pluggable_buffer {
        format!("__flatc_rs_runtime::Vector<'a, B, {vector_inner}>")
    } else {
        let follow_inner =
            helpers::vector_follow_element_type(schema, field, element_bt, "'a", current_ns)?;
        format!("::flatbuffers::ForwardsUOffset<::flatbuffers::Vector<'a, {follow_inner}>>")
    };

    if has_default {
        w.line(&format!(
            "pub fn {accessor_name}(&self) -> {} {{",
            if opts.rust_pluggable_buffer {
                format!("__flatc_rs_runtime::Vector<'a, B, {vector_inner}>")
            } else {
                format!("::flatbuffers::Vector<'a, {vector_inner}>")
            }
        ));
        w.indent();
        w.line("// Safety:");
        w.line("// Created from valid Table for this object");
        w.line("// which contains a valid value in this slot");
        if opts.rust_pluggable_buffer {
            w.line(&format!(
                "unsafe {{ __flatc_rs_runtime::table_get_vector::<B, {vector_inner}>(self._buf, self._loc, Self::VT_{upper_name}).unwrap()}}"
            ));
        } else {
            w.line(&format!(
                "unsafe {{ self._tab.get::<{full_type}>({table_name}::VT_{upper_name}, Some(Default::default())).unwrap()}}"
            ));
        }
    } else {
        w.line(&format!(
            "pub fn {accessor_name}(&self) -> Option<{}> {{",
            if opts.rust_pluggable_buffer {
                format!("__flatc_rs_runtime::Vector<'a, B, {vector_inner}>")
            } else {
                format!("::flatbuffers::Vector<'a, {vector_inner}>")
            }
        ));
        w.indent();
        w.line("// Safety:");
        w.line("// Created from valid Table for this object");
        w.line("// which contains a valid value in this slot");
        if opts.rust_pluggable_buffer {
            w.line(&format!(
                "unsafe {{ __flatc_rs_runtime::table_get_vector::<B, {vector_inner}>(self._buf, self._loc, Self::VT_{upper_name})}}"
            ));
        } else {
            w.line(&format!(
                "unsafe {{ self._tab.get::<{full_type}>({table_name}::VT_{upper_name}, None)}}"
            ));
        }
    }
    w.dedent();
    w.line("}");

    // Generate typed accessor for nested_flatbuffer attribute
    if let Some(nested_type) = helpers::get_nested_flatbuffer_attr(field) {
        if let Some(table_idx) = helpers::find_table_by_name(schema, &nested_type) {
            let nested_table_name = type_map::resolve_object_name(schema, current_ns, table_idx);
            w.blank();
            w.line("#[inline]");
            let nested_return = if opts.rust_pluggable_buffer {
                format!("{nested_table_name}<'a, [u8]>")
            } else {
                format!("{nested_table_name}<'a>")
            };
            w.line(&format!(
                "pub fn {accessor_name}_nested_flatbuffer(&'a self) -> Option<{nested_return}> {{"
            ));
            w.indent();
            if opts.rust_pluggable_buffer {
                w.line(&format!("let data = self.{accessor_name}()?;"));
                w.line("let bytes = data.bytes()?;");
                w.line("use ::flatbuffers::Follow;");
                w.line(&format!(
                    "Some(unsafe {{ <::flatbuffers::ForwardsUOffset<{nested_table_name}<'a>>>::follow(bytes, 0) }})"
                ));
            } else {
                w.line(&format!("self.{accessor_name}().map(|data| {{"));
                w.indent();
                w.line("use ::flatbuffers::Follow;");
                w.line(&format!(
                    "unsafe {{ <::flatbuffers::ForwardsUOffset<{nested_table_name}<'a>>>::follow(data.bytes(), 0) }}"
                ));
                w.dedent();
                w.line("})");
            }
            w.dedent();
            w.line("}");
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn gen_union_accessor(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    accessor_name: &str,
    upper_name: &str,
    table_name: &str,
    current_ns: &str,
    opts: &CodeGenOptions,
) -> Result<(), CodeGenError> {
    // The union value field. The type field (u8 discriminant) is a separate field
    // handled as a scalar enum accessor.
    if opts.rust_pluggable_buffer {
        w.line(&format!(
            "pub fn {accessor_name}(&self) -> Option<__flatc_rs_runtime::Table<'a, B>> {{"
        ));
    } else {
        w.line(&format!(
            "pub fn {accessor_name}(&self) -> Option<::flatbuffers::Table<'a>> {{"
        ));
    }
    w.indent();
    w.line("// Safety:");
    w.line("// Created from valid Table for this object");
    w.line("// which contains a valid value in this slot");
    if opts.rust_pluggable_buffer {
        w.line(&format!(
            "__flatc_rs_runtime::table_field_loc(self._buf, self._loc, Self::VT_{upper_name}).map(|loc| __flatc_rs_runtime::Table {{ buf: self._buf, loc: __flatc_rs_runtime::uoffset_target(self._buf, loc) }})"
        ));
    } else {
        w.line(&format!(
            "unsafe {{ self._tab.get::<::flatbuffers::ForwardsUOffset<::flatbuffers::Table<'a>>>({table_name}::VT_{upper_name}, None)}}"
        ));
    }
    w.dedent();
    w.line("}");

    if field.is_deprecated {
        return Ok(());
    }

    // Generate typed accessors for each union variant
    let enum_idx = field_type_index(field)?;
    if enum_idx < schema.enums.len() {
        let union_enum = &schema.enums[enum_idx];
        let enum_name = type_map::resolve_enum_name(schema, current_ns, enum_idx);

        for val in &union_enum.values {
            let vname = &val.name;
            if vname == "NONE" {
                continue;
            }
            // Sanitize FQN for enum constant reference and accessor name
            let const_name = type_map::escape_keyword(&type_map::sanitize_union_const_name(vname));
            let variant_snake = type_map::to_rust_snake_case(&const_name);
            let variant_bt = val
                .union_type
                .as_ref()
                .map(|t| t.base_type)
                .unwrap_or(BaseType::BASE_TYPE_NONE);

            if variant_bt == BaseType::BASE_TYPE_TABLE {
                let table_idx = union_variant_type_index(val)?;
                let table_name = type_map::resolve_object_name(schema, current_ns, table_idx);

                w.blank();
                w.line("#[inline]");
                w.line("#[allow(non_snake_case)]");
                w.line(&format!(
                    "pub fn {accessor_name}_as_{variant_snake}(&self) -> Option<{}> {{",
                    if opts.rust_pluggable_buffer {
                        format!("{table_name}<'a, B>")
                    } else {
                        format!("{table_name}<'a>")
                    }
                ));
                w.indent();
                w.line(&format!(
                    "if self.{accessor_name}_type() == {enum_name}::{const_name} {{"
                ));
                w.indent();
                w.line(&format!("self.{accessor_name}().map(|t| {{"));
                w.indent();
                w.line("// Safety:");
                w.line("// Created from a valid Table for this object");
                w.line("// Which contains a valid union in this slot");
                w.line(&format!("unsafe {{ {table_name}::init_from_table(t) }}"));
                w.dedent();
                w.line("})");
                w.dedent();
                w.line("} else {");
                w.indent();
                w.line("None");
                w.dedent();
                w.line("}");
                w.dedent();
                w.line("}");
            } else if variant_bt == BaseType::BASE_TYPE_STRUCT {
                let struct_idx = union_variant_type_index(val)?;
                let struct_name = type_map::resolve_object_name(schema, current_ns, struct_idx);

                w.blank();
                w.line("#[inline]");
                w.line("#[allow(non_snake_case)]");
                w.line(&format!(
                    "pub fn {accessor_name}_as_{variant_snake}(&self) -> Option<&'a {struct_name}> {{"
                ));
                w.indent();
                w.line(&format!(
                    "if self.{accessor_name}_type() == {enum_name}::{const_name} {{"
                ));
                w.indent();
                if opts.rust_pluggable_buffer {
                    w.line(&format!(
                        "self.{accessor_name}().and_then(|t| unsafe {{ __flatc_rs_runtime::follow_struct::<{struct_name}, B>(t.buf, t.loc) }})"
                    ));
                } else {
                    w.line(&format!("self.{accessor_name}().map(|t| {{"));
                    w.indent();
                    w.line("// Safety:");
                    w.line("// Created from a valid Table for this object");
                    w.line("// Which contains a valid union in this slot");
                    w.line(&format!(
                        "unsafe {{ <&'a {struct_name} as ::flatbuffers::Follow<'a>>::follow(t.buf(), t.loc()) }}"
                    ));
                    w.dedent();
                    w.line("})");
                }
                w.dedent();
                w.line("} else {");
                w.indent();
                w.line("None");
                w.dedent();
                w.line("}");
                w.dedent();
                w.line("}");
            }
        }
    }
    Ok(())
}

/// Verifiable impl for the table.
pub(super) fn gen_verifiable_impl(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
    current_ns: &str,
) -> Result<(), CodeGenError> {
    enum VerifyEntry {
        Field {
            name: String,
            upper: String,
            verify_type: String,
            is_required: bool,
        },
        Union {
            type_name: String,
            type_upper: String,
            value_name: String,
            value_upper: String,
            enum_name: String,
            is_required: bool,
            variants: Vec<(String, String)>,
        },
    }

    // Pre-compute verifier metadata so no fallible schema lookup occurs while
    // writing the impl body.
    let verify_entries: Vec<Option<VerifyEntry>> = obj
        .fields
        .iter()
        .map(|field| {
            let bt = field.type_.base_type;

            let is_paired_union_type = helpers::is_union_type_field(schema, field)
                && obj.fields.iter().any(|value_field| {
                    value_field.type_.base_type == BaseType::BASE_TYPE_UNION
                        && field.name == format!("{}_type", value_field.name)
                        && field.type_.index == value_field.type_.index
                });
            if is_paired_union_type {
                return Ok(None);
            }

            let fname = &field.name;
            let upper = type_map::rust_field_offset_name(fname);
            let is_required = field.is_required
                || (helpers::has_key_attribute(field) && bt == BaseType::BASE_TYPE_STRING);

            if bt == BaseType::BASE_TYPE_UNION {
                let enum_idx = field_type_index(field)?;
                let union_enum = schema.enums.get(enum_idx).ok_or_else(|| {
                    CodeGenError::Internal(format!(
                        "union field '{}.{}' references missing enum index {enum_idx}",
                        obj.name, field.name
                    ))
                })?;
                let type_name = format!("{fname}_type");
                let type_field = obj
                    .fields
                    .iter()
                    .find(|candidate| {
                        candidate.name == type_name
                            && helpers::is_union_type_field(schema, candidate)
                            && candidate.type_.index == field.type_.index
                    })
                    .ok_or_else(|| {
                        CodeGenError::Internal(format!(
                            "union field '{}.{}' has no matching discriminator field",
                            obj.name, field.name
                        ))
                    })?;
                let type_upper = type_map::rust_field_offset_name(&type_field.name);
                let enum_name = type_map::resolve_enum_name(schema, current_ns, enum_idx);
                let variants = union_enum
                    .values
                    .iter()
                    .filter(|variant| variant.name != "NONE")
                    .map(|variant| {
                        let variant_type = variant
                            .union_type
                            .as_ref()
                            .map(|ty| ty.base_type)
                            .unwrap_or(BaseType::BASE_TYPE_NONE);
                        if variant_type != BaseType::BASE_TYPE_TABLE
                            && variant_type != BaseType::BASE_TYPE_STRUCT
                        {
                            return Err(CodeGenError::Internal(format!(
                                "union variant '{}::{}' has unsupported verifier type {variant_type:?}",
                                union_enum.name, variant.name
                            )));
                        }
                        let object_idx = union_variant_type_index(variant)?;
                        let object_name =
                            type_map::resolve_object_name(schema, current_ns, object_idx);
                        let const_name = type_map::escape_keyword(
                            &type_map::sanitize_union_const_name(&variant.name),
                        );
                        Ok((
                            format!("{enum_name}::{const_name}"),
                            format!("::flatbuffers::ForwardsUOffset<{object_name}>"),
                        ))
                    })
                    .collect::<Result<Vec<_>, CodeGenError>>()?;

                return Ok(Some(VerifyEntry::Union {
                    type_name: type_map::rust_field_name(&type_field.name),
                    type_upper,
                    value_name: type_map::rust_field_name(fname),
                    value_upper: upper,
                    enum_name,
                    is_required,
                    variants,
                }));
            }

            let verify_type = helpers::verifier_type_str(schema, field, current_ns)?;
            Ok(Some(VerifyEntry::Field {
                name: type_map::rust_field_name(fname),
                upper,
                verify_type,
                is_required,
            }))
        })
        .collect::<Result<Vec<_>, CodeGenError>>()?;

    w.block(
        &format!("impl ::flatbuffers::Verifiable for {name}<'_>"),
        |w| {
            w.line("#[inline]");
            // C++ flatc uses multi-line run_verifier signature
            w.line("fn run_verifier(");
            w.indent();
            w.line("v: &mut ::flatbuffers::Verifier, pos: usize");
            w.dedent();
            w.line(") -> Result<(), ::flatbuffers::InvalidFlatbuffer> {");
            w.indent();
            w.line("v.visit_table(pos)?");
            for entry in verify_entries.iter().flatten() {
                match entry {
                    VerifyEntry::Field {
                        name,
                        upper,
                        verify_type,
                        is_required,
                    } => {
                        w.line(&format!(
                            " .visit_field::<{verify_type}>(\"{name}\", Self::VT_{upper}, {is_required})?"
                        ));
                    }
                    VerifyEntry::Union {
                        type_name,
                        type_upper,
                        value_name,
                        value_upper,
                        enum_name,
                        is_required,
                        variants,
                    } => {
                        w.line(&format!(
                            " .visit_union::<{enum_name}, _>(\"{type_name}\", Self::VT_{type_upper}, \"{value_name}\", Self::VT_{value_upper}, {is_required}, |key, v, pos| {{"
                        ));
                        w.indent();
                        w.line("match key {");
                        w.indent();
                        for (variant_name, verify_type) in variants {
                            w.line(&format!(
                                "{variant_name} => v.verify_union_variant::<{verify_type}>(\"{variant_name}\", pos),"
                            ));
                        }
                        w.line("_ => Ok(()),");
                        w.dedent();
                        w.line("}");
                        w.dedent();
                        w.line(" })?");
                    }
                }
            }
            w.line(" .finish();");
            w.line("Ok(())");
            w.dedent();
            w.line("}");
        },
    );
    Ok(())
}

/// Debug impl for the table.
pub(super) fn gen_debug_impl(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
    current_ns: &str,
    opts: &CodeGenOptions,
) -> Result<(), CodeGenError> {
    let union_fields = obj
        .fields
        .iter()
        .filter(|field| !field.is_deprecated && helpers::is_union_field(field))
        .map(|field| {
            let enum_idx = field_type_index(field)?;
            let union_enum = schema.enums.get(enum_idx).ok_or_else(|| {
                CodeGenError::Internal(format!(
                    "union field '{}' references missing enum index {enum_idx}",
                    field.name
                ))
            })?;
            let enum_name = type_map::resolve_enum_name(schema, current_ns, enum_idx);
            let accessor = type_map::rust_field_name(&field.name);
            let variants = union_enum
                .values
                .iter()
                .filter(|value| value.name != "NONE")
                .map(|value| {
                    let constant =
                        type_map::escape_keyword(&type_map::sanitize_union_const_name(&value.name));
                    let method = type_map::to_rust_snake_case(&constant);
                    (constant, method)
                })
                .collect::<Vec<_>>();
            Ok((field.name.clone(), accessor, enum_name, variants))
        })
        .collect::<Result<Vec<_>, CodeGenError>>()?;

    let debug_impl = if opts.rust_pluggable_buffer {
        format!("impl<B: ?Sized + __flatc_rs_runtime::FlatBufferRead> ::core::fmt::Debug for {name}<'_, B>")
    } else {
        format!("impl ::core::fmt::Debug for {name}<'_>")
    };
    w.block(&debug_impl, |w| {
        w.block(
            "fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result",
            |w| {
                w.line(&format!("let mut ds = f.debug_struct(\"{name}\");"));
                for field in &obj.fields {
                    if field.is_deprecated {
                        continue;
                    }
                    let fname = &field.name;
                    let accessor = type_map::rust_field_name(fname);
                    if !helpers::is_union_field(field) {
                        w.line(&format!(
                            "ds.field(\"{accessor}\", &self.{accessor}());"
                        ));
                        continue;
                    }

                    let (_, _, enum_name, variants) = union_fields
                        .iter()
                        .find(|(field_name, _, _, _)| field_name == fname)
                        .expect("union field metadata must be precomputed");
                    w.line(&format!("match self.{accessor}_type() {{"));
                    w.indent();
                    for (constant, method) in variants {
                        w.line(&format!("{enum_name}::{constant} => {{"));
                        w.indent();
                        w.line(&format!(
                            "if let Some(x) = self.{accessor}_as_{method}() {{"
                        ));
                        w.indent();
                        w.line(&format!("ds.field(\"{accessor}\", &x)"));
                        w.dedent();
                        w.line("} else {");
                        w.indent();
                        w.line(&format!(
                            "ds.field(\"{accessor}\", &\"InvalidFlatbuffer: Union discriminant does not match value.\")"
                        ));
                        w.dedent();
                        w.line("}");
                        w.dedent();
                        w.line("},");
                    }
                    w.line("_ => {");
                    w.indent();
                    w.line("let x: Option<()> = None;");
                    w.line(&format!("ds.field(\"{accessor}\", &x)"));
                    w.dedent();
                    w.line("},");
                    w.dedent();
                    w.line("};");
                }
                w.line("ds.finish()");
            },
        );
    });

    if opts.rust_serialize {
        w.blank();
        let serialize_impl = if opts.rust_pluggable_buffer {
            format!("impl<B: ?Sized + __flatc_rs_runtime::FlatBufferRead> ::serde::Serialize for {name}<'_, B>")
        } else {
            format!("impl ::serde::Serialize for {name}<'_>")
        };
        w.block(&serialize_impl, |w| {
            w.block(
                "fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>\nwhere S: ::serde::Serializer",
                |w| {
                    let n = obj
                        .fields
                        .iter()
                        .filter(|f| !f.is_deprecated)
                        .count();
                    w.line("use ::serde::ser::SerializeStruct;");
                    let mutability = if n > 0 { "mut " } else { "" };
                    w.line(&format!(
                        "let {mutability}s = serializer.serialize_struct(\"{name}\", {n})?;"
                    ));
                    for field in &obj.fields {
                        if field.is_deprecated {
                            continue;
                        }
                        let fname = &field.name;
                        let accessor = type_map::rust_field_name(fname);
                        if helpers::is_union_field(field) {
                            let (_, _, enum_name, variants) = union_fields
                                .iter()
                                .find(|(field_name, _, _, _)| field_name == fname)
                                .expect("union field metadata must be precomputed");
                            w.line(&format!("match self.{accessor}_type() {{"));
                            w.indent();
                            w.line(&format!("{enum_name}::NONE => (),"));
                            for (constant, method) in variants {
                                w.line(&format!("{enum_name}::{constant} => {{"));
                                w.indent();
                                w.line(&format!(
                                    "let value = self.{accessor}_as_{method}().expect(\"Invalid union table, expected `{enum_name}::{constant}`.\");"
                                ));
                                w.line(&format!(
                                    "s.serialize_field(\"{accessor}\", &value)?;"
                                ));
                                w.dedent();
                                w.line("}");
                            }
                            w.line("_ => unimplemented!(),");
                            w.dedent();
                            w.line("}");
                        } else if field.type_.base_type == BaseType::BASE_TYPE_VECTOR {
                            let tmp = format!("{accessor}_vec");
                            if field.default_string.is_some() {
                                w.line(&format!(
                                    "let {tmp}: ::std::vec::Vec<_> = self.{accessor}().iter().collect();"
                                ));
                            } else {
                                w.line(&format!(
                                    "let {tmp} = self.{accessor}().map(|v| v.iter().collect::<::std::vec::Vec<_>>());"
                                ));
                            }
                            w.line(&format!(
                                "s.serialize_field(\"{accessor}\", &{tmp})?;"
                            ));
                        } else {
                            w.line(&format!(
                                "s.serialize_field(\"{accessor}\", &self.{accessor}())?;"
                            ));
                        }
                    }
                    w.line("s.end()");
                },
            );
        });
    }
    Ok(())
}

/// Generate the inline `create()` method inside the impl block (C++ flatc style).
fn gen_create_method(w: &mut CodeWriter, obj: &ResolvedObject, name: &str) {
    let has_fields = obj.fields.iter().any(|field| !field.is_deprecated);
    let needs_lifetime = obj.fields.iter().filter(|f| !f.is_deprecated).any(|f| {
        let bt = f.type_.base_type;
        matches!(
            bt,
            BaseType::BASE_TYPE_STRING
                | BaseType::BASE_TYPE_STRUCT
                | BaseType::BASE_TYPE_TABLE
                | BaseType::BASE_TYPE_VECTOR
        )
    });

    let args_lifetime = if needs_lifetime { "<'args>" } else { "" };

    if has_fields {
        w.line("#[allow(unused_mut)]");
    }
    w.line("pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr, A: ::flatbuffers::Allocator + 'bldr>(");
    w.indent();
    w.line("_fbb: &'mut_bldr mut ::flatbuffers::FlatBufferBuilder<'bldr, A>,");
    let args_name = if has_fields { "args" } else { "_args" };
    w.line(&format!("{args_name}: &'args {name}Args{args_lifetime}"));
    w.dedent();
    w.line(&format!(") -> ::flatbuffers::WIPOffset<{name}<'bldr>> {{"));
    w.indent();
    let mutability = if has_fields { "mut " } else { "" };
    w.line(&format!(
        "let {mutability}builder = {name}Builder::new(_fbb);"
    ));

    // Build field add calls -- C++ sorts scalars by alignment size descending,
    // then by field index descending within same size. Non-scalars first (reversed).
    let mut non_scalar_fields: Vec<(usize, &ResolvedField)> = Vec::new();
    let mut scalar_fields: Vec<(usize, &ResolvedField)> = Vec::new();

    for (i, field) in obj.fields.iter().enumerate() {
        if field.is_deprecated {
            continue;
        }
        let bt = field.type_.base_type;
        if type_map::is_scalar(bt) {
            scalar_fields.push((i, field));
        } else {
            non_scalar_fields.push((i, field));
        }
    }

    // C++ emits: last non-scalar first, then scalars sorted by size desc then index desc
    for (_, field) in non_scalar_fields.iter().rev() {
        let fname = &field.name;
        let accessor = type_map::rust_field_name(fname);
        w.line(&format!(
            "if let Some(x) = args.{accessor} {{ builder.add_{accessor}(x); }}"
        ));
    }

    // Sort scalars by alignment size descending, then field index descending
    scalar_fields.sort_by(|a, b| {
        let sz_a = helpers::scalar_alignment_size(a.1.type_.base_type);
        let sz_b = helpers::scalar_alignment_size(b.1.type_.base_type);
        sz_b.cmp(&sz_a).then(b.0.cmp(&a.0))
    });

    for (_, field) in &scalar_fields {
        let fname = &field.name;
        let accessor = type_map::rust_field_name(fname);
        if field.is_optional {
            w.line(&format!(
                "if let Some(x) = args.{accessor} {{ builder.add_{accessor}(x); }}"
            ));
        } else {
            w.line(&format!("builder.add_{accessor}(args.{accessor});"));
        }
    }

    w.line("builder.finish()");
    w.dedent();
    w.line("}");
}

/// Generate key_compare_less_than and key_compare_with_value methods.
fn gen_key_methods(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    table_name: &str,
    current_ns: &str,
) -> Result<(), CodeGenError> {
    let fname = &field.name;
    let accessor = type_map::rust_field_name(fname);
    let bt = field.type_.base_type;

    // Determine the key type and comparison style
    let (key_type, is_string) = if bt == BaseType::BASE_TYPE_STRING {
        ("& str".to_string(), true)
    } else if type_map::is_scalar(bt) {
        if has_type_index(field) {
            let idx = field_type_index(field)?;
            (type_map::resolve_enum_name(schema, current_ns, idx), false)
        } else {
            (type_map::scalar_rust_type(bt).to_string(), false)
        }
    } else {
        return Ok(()); // Unsupported key type
    };

    // key_compare_less_than
    w.line("#[inline]");
    w.block(
        &format!("pub fn key_compare_less_than(&self, o: &{table_name}) -> bool"),
        |w| {
            w.line(&format!("self.{accessor}() < o.{accessor}()"));
        },
    );
    w.blank();

    // key_compare_with_value
    w.line("#[inline]");
    w.block(
        &format!("pub fn key_compare_with_value(&self, val: {key_type}) -> ::core::cmp::Ordering"),
        |w| {
            w.line(&format!("let key = self.{accessor}();"));
            if is_string {
                // String key: accessor returns &str, C++ does key.cmp(val)
                w.line("key.cmp(val)");
            } else {
                // Scalar comparison
                w.line("key.cmp(&val)");
            }
        },
    );
    Ok(())
}
