use crate::field_type_index;
use flatc_rs_schema::resolved::{ResolvedField, ResolvedObject, ResolvedSchema};
use flatc_rs_schema::BaseType;

use crate::type_map;
use crate::{CodeGenError, CodeGenOptions};
use codegen_core::CodeWriter;

use super::helpers;

fn table_vt_ref(table_name: &str, upper: &str, opts: &CodeGenOptions) -> String {
    if opts.rust_pluggable_buffer {
        format!("{table_name}::<'_, [u8]>::VT_{upper}")
    } else {
        format!("{table_name}::VT_{upper}")
    }
}

/// Generate the builder struct.
pub(super) fn gen_builder(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
    current_ns: &str,
    opts: &CodeGenOptions,
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
                if field.is_deprecated {
                    continue;
                }
                gen_builder_add_method(w, schema, field, name, current_ns, opts)?;
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
                            let upper = type_map::rust_field_offset_name(fname);
                            w.line(&format!(
                                "self.fbb_.required(o, {},\"{fname}\");",
                                table_vt_ref(name, &upper, opts)
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
    opts: &CodeGenOptions,
) -> Result<(), CodeGenError> {
    let fname = &field.name;
    let escaped = type_map::escape_keyword(fname);
    let accessor = escaped;
    let upper = type_map::rust_field_offset_name(fname);

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
                    "self.fbb_.push_slot::<{param_type}>({}, {accessor}, {default});",
                    table_vt_ref(table_name, &upper, opts)
                ));
            } else {
                w.line(&format!(
                    "pub fn add_{accessor}(&mut self, {accessor}: {param_type}) {{"
                ));
                w.indent();
                w.line(&format!(
                    "self.fbb_.push_slot_always::<{param_type}>({}, {accessor});",
                    table_vt_ref(table_name, &upper, opts)
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
                "self.fbb_.push_slot_always::<::flatbuffers::WIPOffset<_>>({}, {accessor});",
                table_vt_ref(table_name, &upper, opts)
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
                "self.fbb_.push_slot_always::<&{struct_name}>({}, {accessor});",
                table_vt_ref(table_name, &upper, opts)
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
                "self.fbb_.push_slot_always::<::flatbuffers::WIPOffset<{table_name_ref}>>({}, {accessor});",
                table_vt_ref(table_name, &upper, opts)
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
                "self.fbb_.push_slot_always::<::flatbuffers::WIPOffset<_>>({}, {accessor});",
                table_vt_ref(table_name, &upper, opts)
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
                "self.fbb_.push_slot_always::<::flatbuffers::WIPOffset<_>>({}, {accessor});",
                table_vt_ref(table_name, &upper, opts)
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

    let lifetime = if needs_lifetime { "<'a>" } else { "" };

    // Pre-compute types and defaults so we don't need Result inside closures
    let field_info: Vec<(String, String, String, bool)> = obj
        .fields
        .iter()
        .filter(|field| !field.is_deprecated)
        .map(|field| {
            let fname = &field.name;
            let escaped = type_map::escape_keyword(fname);
            let accessor = escaped;
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
