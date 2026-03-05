use crate::field_type_index;
use crate::type_map::{get_base_type, get_element_type, has_enum_index};
use flatc_rs_schema::{self as schema, BaseType};

use crate::code_writer::CodeWriter;
use crate::type_map;
use crate::{type_visibility, CodeGenOptions};

use super::helpers;

/// Generate the Object API for a table: owned `{Name}T` type with `pack`/`unpack`.
pub(super) fn gen_object_api(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    obj: &schema::Object,
    name: &str,
    current_ns: &str,
    opts: &CodeGenOptions,
) {
    let vis = type_visibility(obj.attributes.as_ref(), opts);
    let t_name = format!("{name}T");

    // --- Struct definition ---
    w.line("#[non_exhaustive]");
    let mut derives = vec!["Debug", "Clone", "PartialEq"];
    if opts.rust_serialize {
        derives.push("::serde::Serialize");
        derives.push("::serde::Deserialize");
    }
    w.line(&format!("#[derive({})]", derives.join(", ")));
    w.block(&format!("{vis} struct {t_name}"), |w| {
        for field in &obj.fields {
            let bt = get_base_type(field.type_.as_ref());
            // Skip union type discriminator fields (they're handled by the union T enum)
            if helpers::is_union_type_field(schema, field) {
                continue;
            }
            let fname = type_map::to_snake_case(&type_map::escape_keyword(
                field.name.as_deref().unwrap_or(""),
            ));
            let owned_type = table_owned_field_type(schema, field, bt, current_ns);
            w.line(&format!("pub {fname}: {owned_type},"));
        }
    });
    w.blank();

    // --- Default impl ---
    w.block(&format!("impl Default for {t_name}"), |w| {
        w.block("fn default() -> Self", |w| {
            w.line("Self {");
            w.indent();
            for field in &obj.fields {
                let bt = get_base_type(field.type_.as_ref());
                if helpers::is_union_type_field(schema, field) {
                    continue;
                }
                let fname = type_map::to_snake_case(&type_map::escape_keyword(
                    field.name.as_deref().unwrap_or(""),
                ));
                let default = table_owned_field_default(schema, field, bt, current_ns);
                w.line(&format!("{fname}: {default},"));
            }
            w.dedent();
            w.line("}");
        });
    });
    w.blank();

    // --- Pack method ---
    w.block(&format!("impl {t_name}"), |w| {
        w.block(
            &format!("pub fn pack<'b, A: ::flatbuffers::Allocator + 'b>(\n    &self,\n    _fbb: &mut ::flatbuffers::FlatBufferBuilder<'b, A>,\n  ) -> ::flatbuffers::WIPOffset<{name}<'b>>"),
            |w| {
                gen_pack_body(w, schema, obj, name, current_ns);
            },
        );
    });
    w.blank();

    // --- Unpack method ---
    w.block(&format!("impl {name}<'_>"), |w| {
        w.block(&format!("pub fn unpack(&self) -> {t_name}"), |w| {
            gen_unpack_body(w, schema, obj, &t_name, current_ns);
        });
    });
}

/// Get the owned Rust type for a table field in the Object API.
fn table_owned_field_type(
    schema: &schema::Schema,
    field: &schema::Field,
    bt: BaseType,
    current_ns: &str,
) -> String {
    let is_optional = field.is_optional == Some(true);
    match bt {
        bt if type_map::is_scalar(bt) => {
            let base = if has_enum_index(field) {
                let idx = field_type_index(field);
                type_map::resolve_enum_name(schema, current_ns, idx)
            } else {
                type_map::scalar_rust_type(bt).to_string()
            };
            if is_optional {
                format!("Option<{base}>")
            } else {
                base
            }
        }
        BaseType::BASE_TYPE_STRING => {
            if helpers::is_field_required(field) || field.default_string.is_some() {
                "alloc::string::String".to_string()
            } else {
                "Option<alloc::string::String>".to_string()
            }
        }
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field_type_index(field);
            let sname = type_map::resolve_object_name(schema, current_ns, idx);
            format!("Option<{sname}T>")
        }
        BaseType::BASE_TYPE_TABLE => {
            let idx = field_type_index(field);
            let tname = type_map::resolve_object_name(schema, current_ns, idx);
            format!("Option<alloc::boxed::Box<{tname}T>>")
        }
        BaseType::BASE_TYPE_VECTOR => {
            let element_bt = get_element_type(field.type_.as_ref());
            let inner = vector_owned_element_type(schema, field, element_bt, current_ns);
            if field.default_string.is_some() {
                format!("alloc::vec::Vec<{inner}>")
            } else {
                format!("Option<alloc::vec::Vec<{inner}>>")
            }
        }
        BaseType::BASE_TYPE_UNION => {
            let idx = field_type_index(field);
            let ename = type_map::resolve_enum_name(schema, current_ns, idx);
            format!("{ename}T")
        }
        _ => "u8".to_string(),
    }
}

/// Get the owned element type for a vector field in the Object API.
fn vector_owned_element_type(
    schema: &schema::Schema,
    field: &schema::Field,
    element_bt: BaseType,
    current_ns: &str,
) -> String {
    match element_bt {
        bt if type_map::is_scalar(bt) => {
            if has_enum_index(field) {
                let enum_idx = field_type_index(field);
                if enum_idx < schema.enums.len() {
                    return type_map::resolve_enum_name(schema, current_ns, enum_idx);
                }
            }
            type_map::scalar_rust_type(bt).to_string()
        }
        BaseType::BASE_TYPE_STRING => "alloc::string::String".to_string(),
        BaseType::BASE_TYPE_TABLE => {
            let idx = field_type_index(field);
            let tname = type_map::resolve_object_name(schema, current_ns, idx);
            format!("{tname}T")
        }
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field_type_index(field);
            let sname = type_map::resolve_object_name(schema, current_ns, idx);
            format!("{sname}T")
        }
        _ => "u8".to_string(),
    }
}

/// Get the default value for an Object API field.
fn table_owned_field_default(
    schema: &schema::Schema,
    field: &schema::Field,
    bt: BaseType,
    current_ns: &str,
) -> String {
    match bt {
        bt if type_map::is_scalar(bt) => {
            if field.is_optional == Some(true) {
                "None".to_string()
            } else {
                helpers::scalar_builder_default(schema, field, bt, current_ns)
            }
        }
        BaseType::BASE_TYPE_STRING => {
            if helpers::is_field_required(field) {
                "alloc::string::String::new()".to_string()
            } else if let Some(default_val) = &field.default_string {
                format!("alloc::string::ToString::to_string(\"{}\")", default_val)
            } else {
                "None".to_string()
            }
        }
        BaseType::BASE_TYPE_VECTOR => {
            if field.default_string.is_some() {
                "Default::default()".to_string()
            } else {
                "None".to_string()
            }
        }
        BaseType::BASE_TYPE_UNION => {
            let idx = field_type_index(field);
            let ename = type_map::resolve_enum_name(schema, current_ns, idx);
            format!("{ename}T::NONE")
        }
        _ => "None".to_string(),
    }
}

/// Generate the pack method body.
fn gen_pack_body(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    obj: &schema::Object,
    name: &str,
    current_ns: &str,
) {
    // Phase 1: Pre-build all non-scalar fields into local variables
    for field in &obj.fields {
        let bt = get_base_type(field.type_.as_ref());
        if helpers::is_union_type_field(schema, field) {
            continue;
        }
        let fname = type_map::to_snake_case(&type_map::escape_keyword(
            field.name.as_deref().unwrap_or(""),
        ));

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
                gen_pack_vector_field(w, schema, field, &fname, current_ns);
            }
            BaseType::BASE_TYPE_UNION => {
                let idx = field_type_index(field);
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
        let bt = get_base_type(field.type_.as_ref());
        let fname = type_map::to_snake_case(&type_map::escape_keyword(
            field.name.as_deref().unwrap_or(""),
        ));
        if bt == BaseType::BASE_TYPE_UNION || helpers::is_union_type_field(schema, field) {
            // Union generates two Args fields: {name}_type and {name}
            // The discriminator field has the _type suffix in args
        }
        // Just emit field name = local variable
        w.line(&format!("{fname},"));
    }
    w.dedent();
    w.line("})");
}

/// Generate pack code for a vector field.
fn gen_pack_vector_field(
    w: &mut CodeWriter,
    _schema: &schema::Schema,
    field: &schema::Field,
    fname: &str,
    _current_ns: &str,
) {
    let has_default = field.default_string.is_some();
    let element_bt = get_element_type(field.type_.as_ref());

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
    schema: &schema::Schema,
    obj: &schema::Object,
    t_name: &str,
    current_ns: &str,
) {
    for field in &obj.fields {
        let bt = get_base_type(field.type_.as_ref());
        if helpers::is_union_type_field(schema, field) {
            continue;
        }
        let fname = type_map::to_snake_case(&type_map::escape_keyword(
            field.name.as_deref().unwrap_or(""),
        ));

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
                gen_unpack_vector_field(w, schema, field, &fname, current_ns);
            }
            BaseType::BASE_TYPE_UNION => {
                gen_unpack_union_field(w, schema, field, &fname, current_ns);
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
        let fname = type_map::to_snake_case(&type_map::escape_keyword(
            field.name.as_deref().unwrap_or(""),
        ));
        w.line(&format!("{fname},"));
    }
    w.dedent();
    w.line("}");
}

/// Generate unpack code for a vector field.
fn gen_unpack_vector_field(
    w: &mut CodeWriter,
    _schema: &schema::Schema,
    field: &schema::Field,
    fname: &str,
    _current_ns: &str,
) {
    let has_default = field.default_string.is_some();
    let element_bt = get_element_type(field.type_.as_ref());

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
    schema: &schema::Schema,
    field: &schema::Field,
    fname: &str,
    current_ns: &str,
) {
    let enum_idx = field_type_index(field);
    if enum_idx >= schema.enums.len() {
        w.line(&format!("let {fname} = Default::default();"));
        return;
    }
    let union_enum = &schema.enums[enum_idx];
    let ename = type_map::resolve_enum_name(schema, current_ns, enum_idx);

    w.line(&format!("let {fname} = match self.{fname}_type() {{"));
    w.indent();
    for val in &union_enum.values {
        let vname = val.name.as_deref().unwrap_or("");
        if vname == "NONE" {
            w.line(&format!("{ename}::NONE => {ename}T::NONE,"));
            continue;
        }
        // Sanitize FQN: const uses underscores, T variant uses PascalCase
        let const_name = type_map::escape_keyword(&type_map::sanitize_union_const_name(vname));
        let t_variant = type_map::escape_keyword(&type_map::fqn_to_pascal(vname));
        let variant_snake = type_map::to_snake_case(&const_name);
        let variant_bt = type_map::get_base_type(val.union_type.as_ref());

        if variant_bt == BaseType::BASE_TYPE_TABLE {
            w.line(&format!(
                "{ename}::{const_name} => {ename}T::{t_variant}(alloc::boxed::Box::new(self.{fname}_as_{variant_snake}().unwrap().unpack())),"
            ));
        }
    }
    w.line(&format!("_ => {ename}T::NONE,"));
    w.dedent();
    w.line("};");
}
