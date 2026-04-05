use crate::field_type_index;
use crate::type_map::has_type_index;
use flatc_rs_schema::resolved::{ResolvedField, ResolvedObject, ResolvedSchema};
use flatc_rs_schema::BaseType;

use crate::type_map;
use crate::{type_visibility, CodeGenError, CodeGenOptions};
use codegen_core::CodeWriter;

use super::helpers;

/// Generate the Object API for a table: owned `{Name}T` type with `pack`/`unpack`.
pub(super) fn gen_object_api(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
    current_ns: &str,
    opts: &CodeGenOptions,
) -> Result<(), CodeGenError> {
    let vis = type_visibility(obj.attributes.as_ref(), opts);
    let t_name = format!("{name}T");

    // Pre-compute owned types and defaults so we don't need Result inside closures
    let field_info: Vec<Option<(String, String, String)>> = obj
        .fields
        .iter()
        .map(|field| {
            let bt = field.type_.base_type;
            if helpers::is_union_type_field(schema, field) {
                return Ok(None);
            }
            let fname = type_map::to_snake_case(&type_map::escape_keyword(&field.name));
            let owned_type = table_owned_field_type(schema, field, bt, current_ns)?;
            let default = table_owned_field_default(schema, field, bt, current_ns)?;
            Ok(Some((fname, owned_type, default)))
        })
        .collect::<Result<Vec<_>, CodeGenError>>()?;

    // --- Struct definition ---
    w.line("#[non_exhaustive]");
    let mut derives = vec!["Debug", "Clone", "PartialEq"];
    if opts.rust_serialize {
        derives.push("::serde::Serialize");
        derives.push("::serde::Deserialize");
    }
    w.line(&format!("#[derive({})]", derives.join(", ")));
    w.block(&format!("{vis} struct {t_name}"), |w| {
        for (fname, owned_type, _) in field_info.iter().flatten() {
            w.line(&format!("pub {fname}: {owned_type},"));
        }
    });
    w.blank();

    // --- Default impl ---
    w.block(&format!("impl Default for {t_name}"), |w| {
        w.block("fn default() -> Self", |w| {
            w.line("Self {");
            w.indent();
            for (fname, _, default) in field_info.iter().flatten() {
                w.line(&format!("{fname}: {default},"));
            }
            w.dedent();
            w.line("}");
        });
    });
    w.blank();

    // --- Pack method ---
    w.try_block(&format!("impl {t_name}"), |w| {
        w.try_block(
            &format!("pub fn pack<'b, A: ::flatbuffers::Allocator + 'b>(\n    &self,\n    _fbb: &mut ::flatbuffers::FlatBufferBuilder<'b, A>,\n  ) -> ::flatbuffers::WIPOffset<{name}<'b>>"),
            |w| {
                gen_pack_body(w, schema, obj, name, current_ns)
            },
        )
    })?;
    w.blank();

    // --- Unpack method ---
    w.try_block(&format!("impl {name}<'_>"), |w| {
        w.try_block(&format!("pub fn unpack(&self) -> {t_name}"), |w| {
            gen_unpack_body(w, schema, obj, &t_name, current_ns)
        })
    })?;
    Ok(())
}

/// Get the owned Rust type for a table field in the Object API.
fn table_owned_field_type(
    schema: &ResolvedSchema,
    field: &ResolvedField,
    bt: BaseType,
    current_ns: &str,
) -> Result<String, CodeGenError> {
    let is_optional = field.is_optional;
    match bt {
        bt if type_map::is_scalar(bt) => {
            let base = if has_type_index(field) {
                let idx = field_type_index(field)?;
                type_map::resolve_enum_name(schema, current_ns, idx)
            } else {
                type_map::scalar_rust_type(bt).to_string()
            };
            if is_optional {
                Ok(format!("Option<{base}>"))
            } else {
                Ok(base)
            }
        }
        BaseType::BASE_TYPE_STRING => {
            if helpers::is_field_required(field) || field.default_string.is_some() {
                Ok("alloc::string::String".to_string())
            } else {
                Ok("Option<alloc::string::String>".to_string())
            }
        }
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field_type_index(field)?;
            let sname = type_map::resolve_object_name(schema, current_ns, idx);
            Ok(format!("Option<{sname}T>"))
        }
        BaseType::BASE_TYPE_TABLE => {
            let idx = field_type_index(field)?;
            let tname = type_map::resolve_object_name(schema, current_ns, idx);
            Ok(format!("Option<alloc::boxed::Box<{tname}T>>"))
        }
        BaseType::BASE_TYPE_VECTOR => {
            let element_bt = field.type_.element_type_or_none();
            let inner = vector_owned_element_type(schema, field, element_bt, current_ns)?;
            if field.default_string.is_some() {
                Ok(format!("alloc::vec::Vec<{inner}>"))
            } else {
                Ok(format!("Option<alloc::vec::Vec<{inner}>>"))
            }
        }
        BaseType::BASE_TYPE_UNION => {
            let idx = field_type_index(field)?;
            let ename = type_map::resolve_enum_name(schema, current_ns, idx);
            Ok(format!("{ename}T"))
        }
        other => Err(CodeGenError::Internal(format!(
            "unsupported base type {:?} for Object API field '{}'",
            other, &field.name,
        ))),
    }
}

/// Get the owned element type for a vector field in the Object API.
fn vector_owned_element_type(
    schema: &ResolvedSchema,
    field: &ResolvedField,
    element_bt: BaseType,
    current_ns: &str,
) -> Result<String, CodeGenError> {
    match element_bt {
        bt if type_map::is_scalar(bt) => {
            if has_type_index(field) {
                let enum_idx = field_type_index(field)?;
                if enum_idx < schema.enums.len() {
                    return Ok(type_map::resolve_enum_name(schema, current_ns, enum_idx));
                }
            }
            Ok(type_map::scalar_rust_type(bt).to_string())
        }
        BaseType::BASE_TYPE_STRING => Ok("alloc::string::String".to_string()),
        BaseType::BASE_TYPE_TABLE => {
            let idx = field_type_index(field)?;
            let tname = type_map::resolve_object_name(schema, current_ns, idx);
            Ok(format!("{tname}T"))
        }
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field_type_index(field)?;
            let sname = type_map::resolve_object_name(schema, current_ns, idx);
            Ok(format!("{sname}T"))
        }
        other => Err(CodeGenError::Internal(format!(
            "unsupported vector element base type {:?} for Object API field '{}'",
            other, &field.name,
        ))),
    }
}

/// Get the default value for an Object API field.
fn table_owned_field_default(
    schema: &ResolvedSchema,
    field: &ResolvedField,
    bt: BaseType,
    current_ns: &str,
) -> Result<String, CodeGenError> {
    match bt {
        bt if type_map::is_scalar(bt) => {
            if field.is_optional {
                Ok("None".to_string())
            } else {
                helpers::scalar_builder_default(schema, field, bt, current_ns)
            }
        }
        BaseType::BASE_TYPE_STRING => {
            if helpers::is_field_required(field) {
                Ok("alloc::string::String::new()".to_string())
            } else if let Some(default_val) = &field.default_string {
                Ok(format!(
                    "alloc::string::ToString::to_string(\"{}\")",
                    default_val
                ))
            } else {
                Ok("None".to_string())
            }
        }
        BaseType::BASE_TYPE_VECTOR => {
            if field.default_string.is_some() {
                Ok("Default::default()".to_string())
            } else {
                Ok("None".to_string())
            }
        }
        BaseType::BASE_TYPE_UNION => {
            let idx = field_type_index(field)?;
            let ename = type_map::resolve_enum_name(schema, current_ns, idx);
            Ok(format!("{ename}T::NONE"))
        }
        _ => Ok("None".to_string()),
    }
}

/// Generate the pack method body.
fn gen_pack_body(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
    current_ns: &str,
) -> Result<(), CodeGenError> {
    // Phase 1: Pre-build all non-scalar fields into local variables
    for field in &obj.fields {
        let bt = field.type_.base_type;
        if helpers::is_union_type_field(schema, field) {
            continue;
        }
        let fname = type_map::to_snake_case(&type_map::escape_keyword(&field.name));

        match bt {
            bt if type_map::is_scalar(bt) => {
                w.line(&format!("let {fname} = self.{fname};"));
            }
            BaseType::BASE_TYPE_STRING => {
                if helpers::is_field_required(field) || field.default_string.is_some() {
                    w.line(&format!(
                        "let {fname} = Some(_fbb.create_string(&self.{fname}));"
                    ));
                } else {
                    w.line(&format!(
                        "let {fname} = self.{fname}.as_ref().map(|x| _fbb.create_string(x));"
                    ));
                }
            }
            BaseType::BASE_TYPE_STRUCT => {
                w.line(&format!(
                    "let {fname}_tmp = self.{fname}.as_ref().map(|x| x.pack());"
                ));
                w.line(&format!("let {fname} = {fname}_tmp.as_ref();"));
            }
            BaseType::BASE_TYPE_TABLE => {
                w.line(&format!(
                    "let {fname} = self.{fname}.as_ref().map(|x| x.pack(_fbb));"
                ));
            }
            BaseType::BASE_TYPE_VECTOR => {
                gen_pack_vector_field(w, field, &fname);
            }
            BaseType::BASE_TYPE_UNION => {
                let idx = field_type_index(field)?;
                let ename = type_map::resolve_enum_name(schema, current_ns, idx);
                let snake = type_map::to_snake_case(ename.split("::").last().unwrap_or(&ename));
                w.line(&format!("let {fname}_type = self.{fname}.{snake}_type();"));
                w.line(&format!("let {fname} = self.{fname}.pack(_fbb);"));
            }
            _ => {}
        }
    }

    // Phase 2: Assemble into Args struct and create
    w.line(&format!("create{name}(_fbb, &{name}Args {{"));
    w.indent();
    for field in &obj.fields {
        let bt = field.type_.base_type;
        let fname = type_map::to_snake_case(&type_map::escape_keyword(&field.name));
        if bt == BaseType::BASE_TYPE_UNION || helpers::is_union_type_field(schema, field) {
            // Union generates two Args fields: {name}_type and {name}
            // The discriminator field has the _type suffix in args
        }
        // Just emit field name = local variable
        w.line(&format!("{fname},"));
    }
    w.dedent();
    w.line("})");
    Ok(())
}

/// Generate pack code for a vector field.
fn gen_pack_vector_field(w: &mut CodeWriter, field: &ResolvedField, fname: &str) {
    let has_default = field.default_string.is_some();
    let element_bt = field.type_.element_type_or_none();

    if has_default {
        // Non-optional vector: wrap in Some(...) directly
        match element_bt {
            BaseType::BASE_TYPE_STRING => {
                w.line(&format!("let {fname} = Some({{"));
                w.indent();
                w.line(&format!("let x = &self.{fname};"));
                w.line("let w: alloc::vec::Vec<_> = x.iter().map(|s| _fbb.create_string(s)).collect();");
                w.line("_fbb.create_vector(&w)");
                w.dedent();
                w.line("});");
            }
            BaseType::BASE_TYPE_TABLE => {
                w.line(&format!("let {fname} = Some({{"));
                w.indent();
                w.line(&format!("let x = &self.{fname};"));
                w.line("let w: alloc::vec::Vec<_> = x.iter().map(|t| t.pack(_fbb)).collect();");
                w.line("_fbb.create_vector(&w)");
                w.dedent();
                w.line("});");
            }
            BaseType::BASE_TYPE_STRUCT => {
                w.line(&format!("let {fname} = Some({{"));
                w.indent();
                w.line(&format!("let x = &self.{fname};"));
                w.line("let w: alloc::vec::Vec<_> = x.iter().map(|t| t.pack()).collect();");
                w.line("_fbb.create_vector(&w)");
                w.dedent();
                w.line("});");
            }
            _ => {
                // Scalar, enum, bool vectors
                w.line(&format!("let {fname} = Some({{"));
                w.indent();
                w.line(&format!("let x = &self.{fname};"));
                w.line("_fbb.create_vector(x)");
                w.dedent();
                w.line("});");
            }
        }
    } else {
        // Optional vector: use .as_ref().map(...)
        match element_bt {
            bt if type_map::is_scalar(bt) => {
                // Scalar vectors (including enum vectors): create_vector directly
                w.line(&format!(
                    "let {fname} = self.{fname}.as_ref().map(|x| _fbb.create_vector(x));"
                ));
            }
            BaseType::BASE_TYPE_STRING => {
                w.line(&format!("let {fname} = self.{fname}.as_ref().map(|x| {{"));
                w.indent();
                w.line("let w: alloc::vec::Vec<_> = x.iter().map(|s| _fbb.create_string(s)).collect();");
                w.line("_fbb.create_vector(&w)");
                w.dedent();
                w.line("});");
            }
            BaseType::BASE_TYPE_TABLE => {
                w.line(&format!("let {fname} = self.{fname}.as_ref().map(|x| {{"));
                w.indent();
                w.line("let w: alloc::vec::Vec<_> = x.iter().map(|t| t.pack(_fbb)).collect();");
                w.line("_fbb.create_vector(&w)");
                w.dedent();
                w.line("});");
            }
            BaseType::BASE_TYPE_STRUCT => {
                w.line(&format!("let {fname} = self.{fname}.as_ref().map(|x| {{"));
                w.indent();
                w.line("let w: alloc::vec::Vec<_> = x.iter().map(|t| t.pack()).collect();");
                w.line("_fbb.create_vector(&w)");
                w.dedent();
                w.line("});");
            }
            _ => {
                w.line(&format!(
                    "let {fname} = self.{fname}.as_ref().map(|x| _fbb.create_vector(x));"
                ));
            }
        }
    }
}

/// Generate the unpack method body.
fn gen_unpack_body(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    t_name: &str,
    current_ns: &str,
) -> Result<(), CodeGenError> {
    for field in &obj.fields {
        let bt = field.type_.base_type;
        if helpers::is_union_type_field(schema, field) {
            continue;
        }
        let fname = type_map::to_snake_case(&type_map::escape_keyword(&field.name));

        match bt {
            bt if type_map::is_scalar(bt) => {
                w.line(&format!("let {fname} = self.{fname}();"));
            }
            BaseType::BASE_TYPE_STRING => {
                if helpers::is_field_required(field) {
                    w.line(&format!("let {fname} = self.{fname}().to_string();"));
                } else if field.default_string.is_some() {
                    w.line(&format!("let {fname} = {{"));
                    w.indent();
                    w.line(&format!("let x = self.{fname}();"));
                    w.line("alloc::string::ToString::to_string(x)");
                    w.dedent();
                    w.line("};");
                } else {
                    w.line(&format!(
                        "let {fname} = self.{fname}().map(|x| x.to_string());"
                    ));
                }
            }
            BaseType::BASE_TYPE_STRUCT => {
                w.line(&format!(
                    "let {fname} = self.{fname}().map(|x| x.unpack());"
                ));
            }
            BaseType::BASE_TYPE_TABLE => {
                w.line(&format!(
                    "let {fname} = self.{fname}().map(|x| alloc::boxed::Box::new(x.unpack()));"
                ));
            }
            BaseType::BASE_TYPE_VECTOR => {
                gen_unpack_vector_field(w, field, &fname);
            }
            BaseType::BASE_TYPE_UNION => {
                gen_unpack_union_field(w, schema, field, &fname, current_ns)?;
            }
            _ => {}
        }
    }

    // Assemble the T struct
    w.line(&format!("{t_name} {{"));
    w.indent();
    for field in &obj.fields {
        if helpers::is_union_type_field(schema, field) {
            continue;
        }
        let fname = type_map::to_snake_case(&type_map::escape_keyword(&field.name));
        w.line(&format!("{fname},"));
    }
    w.dedent();
    w.line("}");
    Ok(())
}

/// Generate unpack code for a vector field.
fn gen_unpack_vector_field(w: &mut CodeWriter, field: &ResolvedField, fname: &str) {
    let has_default = field.default_string.is_some();
    let element_bt = field.type_.element_type_or_none();

    if has_default {
        // Non-optional vector: accessor returns Vector directly
        match element_bt {
            BaseType::BASE_TYPE_STRING => {
                w.line(&format!("let {fname} = {{"));
                w.indent();
                w.line(&format!("let x = self.{fname}();"));
                w.line("x.iter().map(|s| s.to_string()).collect()");
                w.dedent();
                w.line("};");
            }
            BaseType::BASE_TYPE_TABLE => {
                w.line(&format!("let {fname} = {{"));
                w.indent();
                w.line(&format!("let x = self.{fname}();"));
                w.line("x.iter().map(|t| t.unpack()).collect()");
                w.dedent();
                w.line("};");
            }
            BaseType::BASE_TYPE_STRUCT => {
                w.line(&format!("let {fname} = {{"));
                w.indent();
                w.line(&format!("let x = self.{fname}();"));
                w.line("x.iter().map(|t| t.unpack()).collect()");
                w.dedent();
                w.line("};");
            }
            _ => {
                // Scalar, enum, bool
                w.line(&format!("let {fname} = {{"));
                w.indent();
                w.line(&format!("let x = self.{fname}();"));
                w.line("x.into_iter().collect()");
                w.dedent();
                w.line("};");
            }
        }
    } else {
        // Optional vector: accessor returns Option<Vector>
        match element_bt {
            bt if type_map::is_scalar(bt) => {
                w.line(&format!(
                    "let {fname} = self.{fname}().map(|x| x.into_iter().collect());"
                ));
            }
            BaseType::BASE_TYPE_STRING => {
                w.line(&format!(
                    "let {fname} = self.{fname}().map(|x| x.iter().map(|s| s.to_string()).collect());"
                ));
            }
            BaseType::BASE_TYPE_TABLE => {
                w.line(&format!(
                    "let {fname} = self.{fname}().map(|x| x.iter().map(|t| t.unpack()).collect());"
                ));
            }
            BaseType::BASE_TYPE_STRUCT => {
                w.line(&format!(
                    "let {fname} = self.{fname}().map(|x| x.iter().map(|t| t.unpack()).collect());"
                ));
            }
            _ => {
                w.line(&format!(
                    "let {fname} = self.{fname}().map(|x| x.into_iter().collect());"
                ));
            }
        }
    }
}

/// Generate unpack code for a union field.
fn gen_unpack_union_field(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    fname: &str,
    current_ns: &str,
) -> Result<(), CodeGenError> {
    let enum_idx = field_type_index(field)?;
    if enum_idx >= schema.enums.len() {
        w.line(&format!("let {fname} = Default::default();"));
        return Ok(());
    }
    let union_enum = &schema.enums[enum_idx];
    let ename = type_map::resolve_enum_name(schema, current_ns, enum_idx);

    w.line(&format!("let {fname} = match self.{fname}_type() {{"));
    w.indent();
    for val in &union_enum.values {
        let vname = &val.name;
        if vname == "NONE" {
            w.line(&format!("{ename}::NONE => {ename}T::NONE,"));
            continue;
        }
        // Sanitize FQN: const uses underscores, T variant uses PascalCase
        let const_name = type_map::escape_keyword(&type_map::sanitize_union_const_name(vname));
        let t_variant = type_map::escape_keyword(&type_map::fqn_to_pascal(vname));
        let variant_snake = type_map::to_snake_case(&const_name);
        let variant_bt = val
            .union_type
            .as_ref()
            .map(|t| t.base_type)
            .unwrap_or(BaseType::BASE_TYPE_NONE);

        if variant_bt == BaseType::BASE_TYPE_TABLE {
            w.line(&format!(
                "{ename}::{const_name} => {ename}T::{t_variant}(alloc::boxed::Box::new(self.{fname}_as_{variant_snake}().unwrap().unpack())),"
            ));
        }
    }
    w.line(&format!("_ => {ename}T::NONE,"));
    w.dedent();
    w.line("};");
    Ok(())
}
