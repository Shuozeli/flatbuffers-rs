use crate::field_type_index;
use flatc_rs_schema::resolved::{ResolvedField, ResolvedObject, ResolvedSchema};
use flatc_rs_schema::BaseType;

use crate::type_map;
use crate::CodeGenError;
use codegen_core::CodeWriter;

use super::helpers;

/// Generate the builder struct.
pub(super) fn gen_builder(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
    current_ns: &str,
) -> Result<(), CodeGenError> {
    // Builder struct
    w.block(
        &format!("pub struct {name}Builder<'a: 'b, 'b, A: ::flatbuffers::Allocator + 'a>"),
        |w| {
            w.line("fbb_: &'b mut ::flatbuffers::FlatBufferBuilder<'a, A>,");
            w.line("start_: ::flatbuffers::WIPOffset<::flatbuffers::TableUnfinishedWIPOffset>,");
        },
    );

    // Builder impl (immediately follows struct, no blank line)
    w.try_block(
        &format!("impl<'a: 'b, 'b, A: ::flatbuffers::Allocator + 'a> {name}Builder<'a, 'b, A>"),
        |w| {
            // add_* methods for each field
            for field in &obj.fields {
                gen_builder_add_method(w, schema, field, name, current_ns)?;
            }

            // new()
            w.line("#[inline]");
            w.block(
                &format!("pub fn new(_fbb: &'b mut ::flatbuffers::FlatBufferBuilder<'a, A>) -> {name}Builder<'a, 'b, A>"),
                |w| {
                    w.line("let start = _fbb.start_table();");
                    w.line(&format!("{name}Builder {{"));
                    w.indent();
                    w.line("fbb_: _fbb,");
                    w.line("start_: start,");
                    w.dedent();
                    w.line("}");
                },
            );

            // finish()
            w.line("#[inline]");
            w.block(
                &format!("pub fn finish(self) -> ::flatbuffers::WIPOffset<{name}<'a>>"),
                |w| {
                    w.line("let o = self.fbb_.end_table(self.start_);");
                    // Required field assertions (explicit required or string key fields)
                    for field in &obj.fields {
                        let fbt = field.type_.base_type;
                        let is_key_string = helpers::has_key_attribute(field) && fbt == BaseType::BASE_TYPE_STRING;
                        if field.is_required || is_key_string {
                            let fname = &field.name;
                            let escaped = type_map::escape_keyword(fname);
                            let upper = type_map::to_upper_snake_case(&escaped);
                            w.line(&format!(
                                "self.fbb_.required(o, {name}::VT_{upper},\"{fname}\");"
                            ));
                        }
                    }
                    w.line("::flatbuffers::WIPOffset::new(o.value())");
                },
            );
            Ok(())
        },
    )
}

fn gen_builder_add_method(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    table_name: &str,
    current_ns: &str,
) -> Result<(), CodeGenError> {
    let fname = &field.name;
    let escaped = type_map::escape_keyword(fname);
    let accessor = type_map::to_snake_case(&escaped);
    let upper = type_map::to_upper_snake_case(&escaped);

    let bt = field.type_.base_type;

    w.line("#[inline]");

    match bt {
        bt if type_map::is_scalar(bt) => {
            let (param_type, use_default) =
                helpers::scalar_builder_type(schema, field, bt, current_ns)?;
            if use_default {
                let default = helpers::scalar_builder_default(schema, field, bt, current_ns)?;
                w.line(&format!(
                    "pub fn add_{accessor}(&mut self, {accessor}: {param_type}) {{"
                ));
                w.indent();
                w.line(&format!(
                    "self.fbb_.push_slot::<{param_type}>({table_name}::VT_{upper}, {accessor}, {default});"
                ));
            } else {
                w.line(&format!(
                    "pub fn add_{accessor}(&mut self, {accessor}: {param_type}) {{"
                ));
                w.indent();
                w.line(&format!(
                    "self.fbb_.push_slot_always::<{param_type}>({table_name}::VT_{upper}, {accessor});"
                ));
            }
            w.dedent();
            w.line("}");
        }
        BaseType::BASE_TYPE_STRING => {
            w.line(&format!(
                "pub fn add_{accessor}(&mut self, {accessor}: ::flatbuffers::WIPOffset<&'b  str>) {{"
            ));
            w.indent();
            w.line(&format!(
                "self.fbb_.push_slot_always::<::flatbuffers::WIPOffset<_>>({table_name}::VT_{upper}, {accessor});"
            ));
            w.dedent();
            w.line("}");
        }
        BaseType::BASE_TYPE_STRUCT => {
            let struct_idx = field_type_index(field)?;
            let struct_name = type_map::resolve_object_name(schema, current_ns, struct_idx);
            w.line(&format!(
                "pub fn add_{accessor}(&mut self, {accessor}: &{struct_name}) {{"
            ));
            w.indent();
            w.line(&format!(
                "self.fbb_.push_slot_always::<&{struct_name}>({table_name}::VT_{upper}, {accessor});"
            ));
            w.dedent();
            w.line("}");
        }
        BaseType::BASE_TYPE_TABLE => {
            let table_idx = field_type_index(field)?;
            let table_name_ref = type_map::resolve_object_name(schema, current_ns, table_idx);
            w.line(&format!(
                "pub fn add_{accessor}(&mut self, {accessor}: ::flatbuffers::WIPOffset<{table_name_ref}<'b >>) {{"
            ));
            w.indent();
            w.line(&format!(
                "self.fbb_.push_slot_always::<::flatbuffers::WIPOffset<{table_name_ref}>>({table_name}::VT_{upper}, {accessor});"
            ));
            w.dedent();
            w.line("}");
        }
        BaseType::BASE_TYPE_VECTOR => {
            let element_bt = field.type_.element_type_or_none();
            let vec_inner =
                helpers::vector_element_type(schema, field, element_bt, "'b", current_ns)?;
            w.line(&format!(
                "pub fn add_{accessor}(&mut self, {accessor}: ::flatbuffers::WIPOffset<::flatbuffers::Vector<'b , {vec_inner}>>) {{"
            ));
            w.indent();
            w.line(&format!(
                "self.fbb_.push_slot_always::<::flatbuffers::WIPOffset<_>>({table_name}::VT_{upper}, {accessor});"
            ));
            w.dedent();
            w.line("}");
        }
        BaseType::BASE_TYPE_UNION => {
            w.line(&format!(
                "pub fn add_{accessor}(&mut self, {accessor}: ::flatbuffers::WIPOffset<::flatbuffers::UnionWIPOffset>) {{"
            ));
            w.indent();
            w.line(&format!(
                "self.fbb_.push_slot_always::<::flatbuffers::WIPOffset<_>>({table_name}::VT_{upper}, {accessor});"
            ));
            w.dedent();
            w.line("}");
        }
        _ => {
            return Err(CodeGenError::Internal(format!(
                "unhandled BaseType {bt:?} for builder add_{accessor}"
            )));
        }
    }
    Ok(())
}

/// Generate the Args struct for convenience table creation.
pub(super) fn gen_args_struct(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
    current_ns: &str,
) -> Result<(), CodeGenError> {
    let needs_lifetime = obj.fields.iter().any(|f| {
        let bt = f.type_.base_type;
        matches!(
            bt,
            BaseType::BASE_TYPE_STRING
                | BaseType::BASE_TYPE_STRUCT
                | BaseType::BASE_TYPE_TABLE
                | BaseType::BASE_TYPE_VECTOR
        )
    });

    let lifetime = if needs_lifetime { "<'a>" } else { "" };

    // Pre-compute types and defaults so we don't need Result inside closures
    let field_info: Vec<(String, String, String, bool)> = obj
        .fields
        .iter()
        .map(|field| {
            let fname = &field.name;
            let escaped = type_map::escape_keyword(fname);
            let accessor = type_map::to_snake_case(&escaped);
            let arg_type = helpers::args_field_type(schema, field, current_ns)?;
            let default = helpers::args_field_default(schema, field, current_ns)?;
            let is_required = field.is_required || helpers::has_key_attribute(field);
            Ok((accessor, arg_type, default, is_required))
        })
        .collect::<Result<Vec<_>, CodeGenError>>()?;

    // C++ flatc uses 4-space indentation for struct fields (different from rest of code)
    w.line(&format!("pub struct {name}Args{lifetime} {{"));
    for (accessor, arg_type, _, _) in &field_info {
        w.line(&format!("    pub {accessor}: {arg_type},"));
    }
    w.line("}");

    // Default impl - C++ always uses <'a> lifetime on the impl, even for non-lifetime structs
    w.block(&format!("impl<'a> Default for {name}Args{lifetime}"), |w| {
        w.line("#[inline]");
        w.block("fn default() -> Self", |w| {
            w.line(&format!("{name}Args {{"));
            w.indent();
            for (accessor, _, default, is_required) in &field_info {
                if *is_required {
                    w.line(&format!("{accessor}: {default}, // required field"));
                } else {
                    w.line(&format!("{accessor}: {default},"));
                }
            }
            w.dedent();
            w.line("}");
        });
    });
    Ok(())
}

/// Generate the standalone `create*()` function.
pub(super) fn gen_create_fn(w: &mut CodeWriter, obj: &ResolvedObject, name: &str) {
    let needs_lifetime = obj.fields.iter().any(|f| {
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

    w.line("#[inline]");
    w.line(&format!(
        "pub fn create{name}<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr, A: ::flatbuffers::Allocator + 'bldr>("
    ));
    w.indent();
    w.line("fbb: &'mut_bldr mut ::flatbuffers::FlatBufferBuilder<'bldr, A>,");
    w.line(&format!("args: &'args {name}Args{args_lifetime},"));
    w.dedent();
    w.line(&format!(") -> ::flatbuffers::WIPOffset<{name}<'bldr>> {{"));
    w.indent();
    w.line(&format!("let mut builder = {name}Builder::new(fbb);"));

    // Add fields - non-scalars first (they have larger offsets), then scalars
    // This matches the C++ codegen ordering for better vtable packing
    let mut non_scalar_fields: Vec<(usize, &ResolvedField)> = Vec::new();
    let mut scalar_fields: Vec<(usize, &ResolvedField)> = Vec::new();

    for (i, field) in obj.fields.iter().enumerate() {
        let bt = field.type_.base_type;
        if type_map::is_scalar(bt) {
            scalar_fields.push((i, field));
        } else {
            non_scalar_fields.push((i, field));
        }
    }

    // Non-scalar fields: reversed order, wrap in if let Some
    for (_, field) in non_scalar_fields.iter().rev() {
        let fname = &field.name;
        let escaped = type_map::escape_keyword(fname);
        let accessor = type_map::to_snake_case(&escaped);
        w.line(&format!(
            "if let Some(x) = args.{accessor} {{ builder.add_{accessor}(x); }}"
        ));
    }

    // Scalar fields: sort by alignment size desc then index desc (matches C++ ordering)
    scalar_fields.sort_by(|a, b| {
        let sz_a = helpers::scalar_alignment_size(a.1.type_.base_type);
        let sz_b = helpers::scalar_alignment_size(b.1.type_.base_type);
        sz_b.cmp(&sz_a).then(b.0.cmp(&a.0))
    });

    for (_, field) in &scalar_fields {
        let fname = &field.name;
        let escaped = type_map::escape_keyword(fname);
        let accessor = type_map::to_snake_case(&escaped);
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
