use crate::type_map::has_type_index;
use crate::{field_id, field_type_index, union_variant_type_index};
use flatc_rs_schema::resolved::{ResolvedField, ResolvedObject, ResolvedSchema};
use flatc_rs_schema::BaseType;

use crate::code_writer::CodeWriter;
use crate::type_map;
use crate::{CodeGenError, CodeGenOptions};

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
    w.line("#[derive(Copy, Clone, PartialEq)]");
}

/// Reader struct with lifetime.
pub(super) fn gen_reader_struct(w: &mut CodeWriter, name: &str, vis: &str) {
    w.blank();
    w.block(&format!("{vis} struct {name}<'a>"), |w| {
        w.line("pub _tab: ::flatbuffers::Table<'a>,");
    });
}

/// Follow impl for the reader.
pub(super) fn gen_follow_impl(w: &mut CodeWriter, name: &str) {
    w.block(
        &format!("impl<'a> ::flatbuffers::Follow<'a> for {name}<'a>"),
        |w| {
            w.line(&format!("type Inner = {name}<'a>;"));
            w.line("#[inline]");
            w.block(
                "unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner",
                |w| {
                    w.line("Self { _tab: unsafe { ::flatbuffers::Table::new(buf, loc) } }");
                },
            );
        },
    );
}

/// Main impl block with VT constants and accessors.
pub(super) fn gen_impl_block(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
    current_ns: &str,
) -> Result<(), CodeGenError> {
    // Pre-compute VT offsets so we don't need Result inside the closure
    let vt_offsets: Vec<(String, u32)> = obj
        .fields
        .iter()
        .map(|field| {
            let fname = &field.name;
            let escaped = type_map::escape_keyword(fname);
            let upper = type_map::to_upper_snake_case(&escaped);
            let slot = field_id(field)?;
            let vt_offset = 4 + 2 * slot;
            Ok((upper, vt_offset))
        })
        .collect::<Result<Vec<_>, CodeGenError>>()?;

    w.try_block(&format!("impl<'a> {name}<'a>"), |w| {
        // VTable offset constants
        for (upper, vt_offset) in &vt_offsets {
            w.line(&format!(
                "pub const VT_{upper}: ::flatbuffers::VOffsetT = {vt_offset};"
            ));
        }
        w.blank();
        // init_from_table (used by union accessors)
        w.line("#[inline]");
        w.block(
            "pub unsafe fn init_from_table(table: ::flatbuffers::Table<'a>) -> Self",
            |w| {
                w.line(&format!("{name} {{ _tab: table }}"));
            },
        );
        gen_create_method(w, obj, name);
        w.blank(); // C++ emits a double blank after create()
        w.blank();

        // Field accessors - key methods are emitted right after the key field
        for field in &obj.fields {
            gen_field_accessor(w, schema, field, name, current_ns)?;
            // Key comparison methods come right after the key field accessor
            if helpers::has_key_attribute(field) {
                w.blank();
                gen_key_methods(w, schema, field, name, current_ns)?;
            }
        }
        Ok(())
    })
}

/// Generate an accessor method for a table field.
fn gen_field_accessor(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    table_name: &str,
    current_ns: &str,
) -> Result<(), CodeGenError> {
    let fname = &field.name;
    let escaped = type_map::escape_keyword(fname);
    let accessor_name = type_map::to_snake_case(&escaped);
    let upper = type_map::to_upper_snake_case(&escaped);

    let bt = field.type_.base_type;

    let is_required = helpers::is_field_required(field);
    let is_optional_scalar = field.is_optional;
    let is_deprecated = field.is_deprecated;

    if is_deprecated {
        w.line("#[deprecated]");
    }
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
            )?;
        }
        BaseType::BASE_TYPE_STRING => {
            gen_string_accessor(w, field, &accessor_name, &upper, table_name, is_required);
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
) -> Result<(), CodeGenError> {
    // Check if this is an enum field (has index pointing to an enum)
    if has_type_index(ctx.field) {
        let enum_idx = field_type_index(ctx.field)?;
        let enum_name = type_map::resolve_enum_name(ctx.schema, ctx.current_ns, enum_idx);
        let is_bitflags = type_map::is_bitflags_enum(ctx.schema, enum_idx);

        // Determine default
        let default = if let Some(ref ds) = ctx.field.default_string {
            format!("{enum_name}::{ds}")
        } else {
            let dv = ctx.field.default_integer.unwrap_or(0);
            if is_bitflags {
                format!("{enum_name}::from_bits_retain({dv})")
            } else {
                format!("{enum_name}({dv})")
            }
        };

        if ctx.is_optional {
            w.line(&format!(
                "pub fn {}(&self) -> Option<{enum_name}> {{",
                ctx.accessor_name
            ));
            w.indent();
            w.line("// Safety:");
            w.line("// Created from valid Table for this object");
            w.line("// which contains a valid value in this slot");
            w.line(&format!(
                "unsafe {{ self._tab.get::<{enum_name}>({}::VT_{}, None) }}",
                ctx.table_name, ctx.upper_name
            ));
        } else {
            w.line(&format!(
                "pub fn {}(&self) -> {enum_name} {{",
                ctx.accessor_name
            ));
            w.indent();
            w.line("// Safety:");
            w.line("// Created from valid Table for this object");
            w.line("// which contains a valid value in this slot");
            w.line(&format!(
                "unsafe {{ self._tab.get::<{enum_name}>({}::VT_{}, Some({default})).unwrap()}}",
                ctx.table_name, ctx.upper_name
            ));
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
            w.line(&format!(
                "unsafe {{ self._tab.get::<{rust_type}>({}::VT_{}, None) }}",
                ctx.table_name, ctx.upper_name
            ));
        } else {
            w.line(&format!(
                "pub fn {}(&self) -> {rust_type} {{",
                ctx.accessor_name
            ));
            w.indent();
            w.line("// Safety:");
            w.line("// Created from valid Table for this object");
            w.line("// which contains a valid value in this slot");
            w.line(&format!(
                "unsafe {{ self._tab.get::<{rust_type}>({}::VT_{}, Some({default})).unwrap()}}",
                ctx.table_name, ctx.upper_name
            ));
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
) {
    let has_default = field.default_string.is_some();

    if is_required {
        w.line(&format!("pub fn {accessor_name}(&self) -> &'a str {{"));
        w.indent();
        w.line("// Safety:");
        w.line("// Created from valid Table for this object");
        w.line("// which contains a valid value in this slot");
        w.line(&format!(
            "unsafe {{ self._tab.get::<::flatbuffers::ForwardsUOffset<&str>>({table_name}::VT_{upper_name}, None).unwrap()}}"
        ));
    } else if has_default {
        let default_val = field.default_string.as_deref().unwrap_or("");
        w.line(&format!("pub fn {accessor_name}(&self) -> &'a str {{"));
        w.indent();
        w.line("// Safety:");
        w.line("// Created from valid Table for this object");
        w.line("// which contains a valid value in this slot");
        w.line(&format!(
            "unsafe {{ self._tab.get::<::flatbuffers::ForwardsUOffset<&str>>({table_name}::VT_{upper_name}, Some(&\"{default_val}\")).unwrap()}}"
        ));
    } else {
        w.line(&format!(
            "pub fn {accessor_name}(&self) -> Option<&'a str> {{"
        ));
        w.indent();
        w.line("// Safety:");
        w.line("// Created from valid Table for this object");
        w.line("// which contains a valid value in this slot");
        w.line(&format!(
            "unsafe {{ self._tab.get::<::flatbuffers::ForwardsUOffset<&str>>({table_name}::VT_{upper_name}, None)}}"
        ));
    }
    w.dedent();
    w.line("}");
}

fn gen_struct_field_accessor(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    accessor_name: &str,
    upper_name: &str,
    table_name: &str,
    current_ns: &str,
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
    w.line(&format!(
        "unsafe {{ self._tab.get::<{struct_name}>({table_name}::VT_{upper_name}, None)}}"
    ));
    w.dedent();
    w.line("}");
    Ok(())
}

fn gen_table_field_accessor(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    accessor_name: &str,
    upper_name: &str,
    table_name: &str,
    current_ns: &str,
) -> Result<(), CodeGenError> {
    let table_idx = field_type_index(field)?;
    let field_table_name = type_map::resolve_object_name(schema, current_ns, table_idx);

    w.line(&format!(
        "pub fn {accessor_name}(&self) -> Option<{field_table_name}<'a>> {{"
    ));
    w.indent();
    w.line("// Safety:");
    w.line("// Created from valid Table for this object");
    w.line("// which contains a valid value in this slot");
    w.line(&format!(
        "unsafe {{ self._tab.get::<::flatbuffers::ForwardsUOffset<{field_table_name}>>({table_name}::VT_{upper_name}, None)}}"
    ));
    w.dedent();
    w.line("}");
    Ok(())
}

fn gen_vector_accessor(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    accessor_name: &str,
    upper_name: &str,
    table_name: &str,
    current_ns: &str,
) -> Result<(), CodeGenError> {
    let element_bt = field.type_.element_type_or_none();
    let has_default = field.default_string.is_some();

    let vector_inner = helpers::vector_element_type(schema, field, element_bt, "'a", current_ns)?;
    let full_type =
        format!("::flatbuffers::ForwardsUOffset<::flatbuffers::Vector<'a, {vector_inner}>>");

    if has_default {
        w.line(&format!(
            "pub fn {accessor_name}(&self) -> ::flatbuffers::Vector<'a, {vector_inner}> {{"
        ));
        w.indent();
        w.line("// Safety:");
        w.line("// Created from valid Table for this object");
        w.line("// which contains a valid value in this slot");
        w.line(&format!(
            "unsafe {{ self._tab.get::<{full_type}>({table_name}::VT_{upper_name}, Some(Default::default())).unwrap()}}"
        ));
    } else {
        w.line(&format!(
            "pub fn {accessor_name}(&self) -> Option<::flatbuffers::Vector<'a, {vector_inner}>> {{"
        ));
        w.indent();
        w.line("// Safety:");
        w.line("// Created from valid Table for this object");
        w.line("// which contains a valid value in this slot");
        w.line(&format!(
            "unsafe {{ self._tab.get::<{full_type}>({table_name}::VT_{upper_name}, None)}}"
        ));
    }
    w.dedent();
    w.line("}");

    // Generate typed accessor for nested_flatbuffer attribute
    if let Some(nested_type) = helpers::get_nested_flatbuffer_attr(field) {
        if let Some(table_idx) = helpers::find_table_by_name(schema, &nested_type) {
            let nested_table_name = type_map::resolve_object_name(schema, current_ns, table_idx);
            w.blank();
            w.line("#[inline]");
            w.line(&format!(
                "pub fn {accessor_name}_nested_flatbuffer(&'a self) -> Option<{nested_table_name}<'a>> {{"
            ));
            w.indent();
            w.line(&format!("self.{accessor_name}().map(|data| {{"));
            w.indent();
            w.line("use ::flatbuffers::Follow;");
            w.line(&format!(
                "unsafe {{ <::flatbuffers::ForwardsUOffset<{nested_table_name}<'a>>>::follow(data.bytes(), 0) }}"
            ));
            w.dedent();
            w.line("})");
            w.dedent();
            w.line("}");
        }
    }
    Ok(())
}

fn gen_union_accessor(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    accessor_name: &str,
    upper_name: &str,
    table_name: &str,
    current_ns: &str,
) -> Result<(), CodeGenError> {
    // The union value field. The type field (u8 discriminant) is a separate field
    // handled as a scalar enum accessor.
    w.line(&format!(
        "pub fn {accessor_name}(&self) -> Option<::flatbuffers::Table<'a>> {{"
    ));
    w.indent();
    w.line(&format!(
        "unsafe {{ self._tab.get::<::flatbuffers::ForwardsUOffset<::flatbuffers::Table<'a>>>({table_name}::VT_{upper_name}, None)}}"
    ));
    w.dedent();
    w.line("}");

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
            let variant_snake = type_map::to_snake_case(&const_name);
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
                w.line(&format!(
                    "pub fn {accessor_name}_as_{variant_snake}(&self) -> Option<{table_name}<'a>> {{"
                ));
                w.indent();
                w.line(&format!(
                    "if self.{accessor_name}_type() == {enum_name}::{const_name} {{"
                ));
                w.indent();
                w.line(&format!(
                    "self.{accessor_name}().map(|t| unsafe {{ {table_name}::init_from_table(t) }})"
                ));
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
                w.line(&format!(
                    "pub fn {accessor_name}_as_{variant_snake}(&self) -> Option<&'a {struct_name}> {{"
                ));
                w.indent();
                w.line(&format!(
                    "if self.{accessor_name}_type() == {enum_name}::{const_name} {{"
                ));
                w.indent();
                w.line(&format!(
                    "self.{accessor_name}().map(|t| unsafe {{ <&{struct_name}>::follow(t.buf, t.loc) }})"
                ));
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
    // Pre-compute verifier type strings so we don't need Result inside the closure
    let verify_fields: Vec<Option<(String, String, bool)>> = obj
        .fields
        .iter()
        .map(|field| {
            let bt = field.type_.base_type;
            if bt == BaseType::BASE_TYPE_UNION {
                return Ok(None);
            }
            let fname = &field.name;
            let escaped = type_map::escape_keyword(fname);
            let upper = type_map::to_upper_snake_case(&escaped);
            let is_required = field.is_required
                || (helpers::has_key_attribute(field) && bt == BaseType::BASE_TYPE_STRING);
            let verify_type = helpers::verifier_type_str(schema, field, current_ns)?;
            Ok(Some((upper, verify_type, is_required)))
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
            for (i, field) in obj.fields.iter().enumerate() {
                if let Some((upper, verify_type, is_required)) = &verify_fields[i] {
                    let fname = &field.name;
                    w.line(&format!(
                        " .visit_field::<{verify_type}>(\"{fname}\", Self::VT_{upper}, {is_required})?"
                    ));
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
    obj: &ResolvedObject,
    name: &str,
    opts: &CodeGenOptions,
) {
    w.block(&format!("impl ::core::fmt::Debug for {name}<'_>"), |w| {
        w.block(
            "fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result",
            |w| {
                w.line(&format!("let mut ds = f.debug_struct(\"{name}\");"));
                for field in &obj.fields {
                    if helpers::is_union_field(field) {
                        continue;
                    }
                    let fname = &field.name;
                    let escaped = type_map::escape_keyword(fname);
                    let accessor = type_map::to_snake_case(&escaped);
                    w.line(&format!("ds.field(\"{fname}\", &self.{accessor}());"));
                }
                w.line("ds.finish()");
            },
        );
    });

    if opts.rust_serialize {
        w.blank();
        w.block(&format!("impl ::serde::Serialize for {name}<'_>"), |w| {
            w.block(
                "fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>\nwhere S: ::serde::Serializer",
                |w| {
                    let n = obj.fields.iter().filter(|f| !helpers::is_union_field(f)).count();
                    w.line("use ::serde::ser::SerializeStruct;");
                    w.line(&format!("let mut s = serializer.serialize_struct(\"{name}\", {n})?;"));
                    for field in &obj.fields {
                        if helpers::is_union_field(field) {
                            continue;
                        }
                        let fname = &field.name;
                        let escaped = type_map::escape_keyword(fname);
                        let accessor = type_map::to_snake_case(&escaped);
                        w.line(&format!("s.serialize_field(\"{fname}\", &self.{accessor}())?;"));
                    }
                    w.line("s.end()");
                },
            );
        });
    }
}

/// Generate the inline `create()` method inside the impl block (C++ flatc style).
fn gen_create_method(w: &mut CodeWriter, obj: &ResolvedObject, name: &str) {
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

    w.line("#[allow(unused_mut)]");
    w.line("pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr, A: ::flatbuffers::Allocator + 'bldr>(");
    w.indent();
    w.line("_fbb: &'mut_bldr mut ::flatbuffers::FlatBufferBuilder<'bldr, A>,");
    w.line(&format!("args: &'args {name}Args{args_lifetime}"));
    w.dedent();
    w.line(&format!(") -> ::flatbuffers::WIPOffset<{name}<'bldr>> {{"));
    w.indent();
    w.line(&format!("let mut builder = {name}Builder::new(_fbb);"));

    // Build field add calls -- C++ sorts scalars by alignment size descending,
    // then by field index descending within same size. Non-scalars first (reversed).
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

    // C++ emits: last non-scalar first, then scalars sorted by size desc then index desc
    for (_, field) in non_scalar_fields.iter().rev() {
        let fname = &field.name;
        let escaped = type_map::escape_keyword(fname);
        let accessor = type_map::to_snake_case(&escaped);
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

/// Generate key_compare_less_than and key_compare_with_value methods.
fn gen_key_methods(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    table_name: &str,
    current_ns: &str,
) -> Result<(), CodeGenError> {
    let fname = &field.name;
    let escaped = type_map::escape_keyword(fname);
    let accessor = type_map::to_snake_case(&escaped);
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
