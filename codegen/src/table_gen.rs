use super::field_type_index;
use super::type_map::{get_base_type, get_element_type, get_index};
use flatc_rs_schema::{self as schema, BaseType};

use super::code_writer::CodeWriter;
use super::type_map;
use super::CodeGenOptions;

/// Generate Rust code for the table at `schema.objects[index]`.
pub fn generate(w: &mut CodeWriter, schema: &schema::Schema, index: usize, opts: &CodeGenOptions) {
    let obj = &schema.objects[index];
    let name = obj.name.as_deref().unwrap_or("");
    let current_ns = type_map::object_namespace(obj);

    gen_offset_marker(w, name);
    gen_reader_struct(w, name);
    w.blank();
    gen_follow_impl(w, name);
    w.blank();
    gen_impl_block(w, schema, obj, name, current_ns);
    w.blank();
    gen_verifiable_impl(w, schema, obj, name, current_ns);
    gen_args_struct(w, schema, obj, name, current_ns);
    w.blank();
    gen_builder(w, schema, obj, name, current_ns);
    w.blank();
    gen_debug_impl(w, schema, obj, name, opts);
    w.blank();
    gen_create_fn(w, schema, obj, name, current_ns);

    // Object API: owned T type with pack/unpack
    if opts.gen_object_api {
        w.blank();
        gen_object_api(w, schema, obj, name, current_ns, opts);
    }
}

/// `pub enum FooOffset {}`
fn gen_offset_marker(w: &mut CodeWriter, name: &str) {
    w.line(&format!("pub enum {name}Offset {{}}"));
    w.line("#[derive(Copy, Clone, PartialEq)]");
}

/// Reader struct with lifetime.
fn gen_reader_struct(w: &mut CodeWriter, name: &str) {
    w.blank();
    w.block(&format!("pub struct {name}<'a>"), |w| {
        w.line("pub _tab: ::flatbuffers::Table<'a>,");
    });
}

/// Follow impl for the reader.
fn gen_follow_impl(w: &mut CodeWriter, name: &str) {
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
fn gen_impl_block(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    obj: &schema::Object,
    name: &str,
    current_ns: &str,
) {
    w.block(&format!("impl<'a> {name}<'a>"), |w| {
        // VTable offset constants
        for (i, field) in obj.fields.iter().enumerate() {
            let fname = field.name.as_deref().unwrap_or("");
            let escaped = type_map::escape_keyword(fname);
            let upper = type_map::to_upper_snake_case(&escaped);
            let slot = field.id.unwrap_or(i as u32);
            let vt_offset = 4 + 2 * slot;
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
        gen_create_method(w, schema, obj, name, current_ns);
        w.blank(); // C++ emits a double blank after create()
        w.blank();

        // Field accessors - key methods are emitted right after the key field
        for (i, field) in obj.fields.iter().enumerate() {
            gen_field_accessor(w, schema, obj, field, i, name, current_ns);
            // Key comparison methods come right after the key field accessor
            if has_key_attribute(field) {
                w.blank();
                gen_key_methods(w, schema, field, name, current_ns);
            }
        }
    });
}

/// Generate an accessor method for a table field.
fn gen_field_accessor(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    _obj: &schema::Object,
    field: &schema::Field,
    _field_idx: usize,
    table_name: &str,
    current_ns: &str,
) {
    let fname = field.name.as_deref().unwrap_or("");
    let escaped = type_map::escape_keyword(fname);
    let accessor_name = type_map::to_snake_case(&escaped);
    let upper = type_map::to_upper_snake_case(&escaped);

    let bt = get_base_type(field.type_.as_ref());

    let is_required = is_field_required(field);
    let is_optional_scalar = field.is_optional == Some(true);
    let is_deprecated = field.is_deprecated == Some(true);

    if is_deprecated {
        w.line("#[deprecated]");
    }
    w.line("#[inline]");

    match bt {
        bt if type_map::is_scalar(bt) => {
            gen_scalar_accessor(
                w,
                schema,
                field,
                &accessor_name,
                &upper,
                bt,
                is_optional_scalar,
                table_name,
                current_ns,
            );
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
            );
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
            );
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
            );
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
            );
        }
        _ => {
            // Fallback for unhandled types
            w.line(&format!(
                "// TODO: accessor for {accessor_name} (type {bt:?})"
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn gen_scalar_accessor(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    field: &schema::Field,
    accessor_name: &str,
    upper_name: &str,
    bt: BaseType,
    is_optional: bool,
    table_name: &str,
    current_ns: &str,
) {
    // Check if this is an enum field (has index pointing to an enum)
    let has_enum = get_index(field.type_.as_ref())
        .map(|i| i >= 0)
        .unwrap_or(false);

    if has_enum {
        let enum_idx = field_type_index(field);
        let enum_name = type_map::resolve_enum_name(schema, current_ns, enum_idx);
        let is_bitflags = type_map::is_bitflags_enum(schema, enum_idx);

        // Determine default
        let default = if let Some(ref ds) = field.default_string {
            format!("{enum_name}::{ds}")
        } else {
            let dv = field.default_integer.unwrap_or(0);
            if is_bitflags {
                format!("{enum_name}::from_bits_retain({dv})")
            } else {
                format!("{enum_name}({dv})")
            }
        };

        if is_optional {
            w.line(&format!(
                "pub fn {accessor_name}(&self) -> Option<{enum_name}> {{"
            ));
            w.indent();
            w.line("// Safety:");
            w.line("// Created from valid Table for this object");
            w.line("// which contains a valid value in this slot");
            w.line(&format!(
                "unsafe {{ self._tab.get::<{enum_name}>({table_name}::VT_{upper_name}, None) }}"
            ));
        } else {
            w.line(&format!("pub fn {accessor_name}(&self) -> {enum_name} {{"));
            w.indent();
            w.line("// Safety:");
            w.line("// Created from valid Table for this object");
            w.line("// which contains a valid value in this slot");
            w.line(&format!(
                "unsafe {{ self._tab.get::<{enum_name}>({table_name}::VT_{upper_name}, Some({default})).unwrap()}}"
            ));
        }
        w.dedent();
        w.line("}");
    } else {
        let rust_type = type_map::scalar_rust_type(bt);
        let default = scalar_default(field, bt);

        if is_optional {
            w.line(&format!(
                "pub fn {accessor_name}(&self) -> Option<{rust_type}> {{"
            ));
            w.indent();
            w.line("// Safety:");
            w.line("// Created from valid Table for this object");
            w.line("// which contains a valid value in this slot");
            w.line(&format!(
                "unsafe {{ self._tab.get::<{rust_type}>({table_name}::VT_{upper_name}, None) }}"
            ));
        } else {
            w.line(&format!("pub fn {accessor_name}(&self) -> {rust_type} {{"));
            w.indent();
            w.line("// Safety:");
            w.line("// Created from valid Table for this object");
            w.line("// which contains a valid value in this slot");
            w.line(&format!(
                "unsafe {{ self._tab.get::<{rust_type}>({table_name}::VT_{upper_name}, Some({default})).unwrap()}}"
            ));
        }
        w.dedent();
        w.line("}");
    }
}

fn gen_string_accessor(
    w: &mut CodeWriter,
    field: &schema::Field,
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
    schema: &schema::Schema,
    field: &schema::Field,
    accessor_name: &str,
    upper_name: &str,
    table_name: &str,
    current_ns: &str,
) {
    let struct_idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
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
}

fn gen_table_field_accessor(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    field: &schema::Field,
    accessor_name: &str,
    upper_name: &str,
    table_name: &str,
    current_ns: &str,
) {
    let table_idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
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
}

fn gen_vector_accessor(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    field: &schema::Field,
    accessor_name: &str,
    upper_name: &str,
    table_name: &str,
    current_ns: &str,
) {
    let element_bt = get_element_type(field.type_.as_ref());
    let has_default = field.default_string.is_some();

    let vector_inner = vector_element_type(schema, field, element_bt, "'a", current_ns);
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
    if let Some(nested_type) = get_nested_flatbuffer_attr(field) {
        if let Some(table_idx) = find_table_by_name(schema, &nested_type) {
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
}

fn gen_union_accessor(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    field: &schema::Field,
    accessor_name: &str,
    upper_name: &str,
    table_name: &str,
    current_ns: &str,
) {
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
    let enum_idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
    if enum_idx < schema.enums.len() {
        let union_enum = &schema.enums[enum_idx];
        let enum_name = type_map::resolve_enum_name(schema, current_ns, enum_idx);

        for val in &union_enum.values {
            let vname = val.name.as_deref().unwrap_or("");
            if vname == "NONE" {
                continue;
            }
            // Sanitize FQN for enum constant reference and accessor name
            let const_name = type_map::escape_keyword(&type_map::sanitize_union_const_name(vname));
            let variant_snake = type_map::to_snake_case(&const_name);
            let variant_bt = get_base_type(val.union_type.as_ref());

            if variant_bt == BaseType::BASE_TYPE_TABLE {
                let table_idx = val.union_type.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
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
                let struct_idx =
                    val.union_type.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
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
}

/// Verifiable impl for the table.
fn gen_verifiable_impl(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    obj: &schema::Object,
    name: &str,
    current_ns: &str,
) {
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
            for field in &obj.fields {
                let fname = field.name.as_deref().unwrap_or("");
                let escaped = type_map::escape_keyword(fname);
                let upper = type_map::to_upper_snake_case(&escaped);
                let bt = get_base_type(field.type_.as_ref());
                let is_required = field.is_required == Some(true)
                    || (has_key_attribute(field) && bt == BaseType::BASE_TYPE_STRING);
                // the type discriminant is verified as a scalar.
                if bt == BaseType::BASE_TYPE_UNION {
                    continue;
                }
                let verify_type = verifier_type_str(schema, field, current_ns);
                w.line(&format!(
                    " .visit_field::<{verify_type}>(\"{fname}\", Self::VT_{upper}, {is_required})?"
                ));
            }
            w.line(" .finish();");
            w.line("Ok(())");
            w.dedent();
            w.line("}");
        },
    );
}

/// Debug impl for the table.
fn gen_debug_impl(
    w: &mut CodeWriter,
    _schema: &schema::Schema,
    obj: &schema::Object,
    name: &str,
    opts: &CodeGenOptions,
) {
    w.block(&format!("impl ::core::fmt::Debug for {name}<'_>"), |w| {
        w.block(
            "fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result",
            |w| {
                w.line(&format!("let mut ds = f.debug_struct(\"{name}\");"));
                for field in &obj.fields {
                    if is_union_field(field) {
                        continue;
                    }
                    let fname = field.name.as_deref().unwrap_or("");
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
                    let n = obj.fields.iter().filter(|f| !is_union_field(f)).count();
                    w.line("use ::serde::ser::SerializeStruct;");
                    w.line(&format!("let mut s = serializer.serialize_struct(\"{name}\", {n})?;"));
                    for field in &obj.fields {
                        if is_union_field(field) {
                            continue;
                        }
                        let fname = field.name.as_deref().unwrap_or("");
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

/// Generate the builder struct.
fn gen_builder(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    obj: &schema::Object,
    name: &str,
    current_ns: &str,
) {
    // Builder struct
    w.block(
        &format!("pub struct {name}Builder<'a: 'b, 'b, A: ::flatbuffers::Allocator + 'a>"),
        |w| {
            w.line("fbb_: &'b mut ::flatbuffers::FlatBufferBuilder<'a, A>,");
            w.line("start_: ::flatbuffers::WIPOffset<::flatbuffers::TableUnfinishedWIPOffset>,");
        },
    );

    // Builder impl (immediately follows struct, no blank line)
    w.block(
        &format!("impl<'a: 'b, 'b, A: ::flatbuffers::Allocator + 'a> {name}Builder<'a, 'b, A>"),
        |w| {
            // add_* methods for each field
            for (i, field) in obj.fields.iter().enumerate() {
                gen_builder_add_method(w, schema, field, name, i, current_ns);
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
                        let fbt = get_base_type(field.type_.as_ref());
                        let is_key_string = has_key_attribute(field) && fbt == BaseType::BASE_TYPE_STRING;
                        if field.is_required == Some(true) || is_key_string {
                            let fname = field.name.as_deref().unwrap_or("");
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
        },
    );
}

fn gen_builder_add_method(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    field: &schema::Field,
    table_name: &str,
    _field_idx: usize,
    current_ns: &str,
) {
    let fname = field.name.as_deref().unwrap_or("");
    let escaped = type_map::escape_keyword(fname);
    let accessor = type_map::to_snake_case(&escaped);
    let upper = type_map::to_upper_snake_case(&escaped);

    let bt = get_base_type(field.type_.as_ref());

    w.line("#[inline]");

    match bt {
        bt if type_map::is_scalar(bt) => {
            let (param_type, use_default) = scalar_builder_type(schema, field, bt, current_ns);
            if use_default {
                let default = scalar_builder_default(schema, field, bt, current_ns);
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
            let struct_idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
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
            let table_idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
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
            let element_bt = get_element_type(field.type_.as_ref());
            let vec_inner = vector_element_type(schema, field, element_bt, "'b", current_ns);
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
            w.line(&format!("// TODO: add_{accessor} (type {bt:?})"));
        }
    }
}

/// Generate the Args struct for convenience table creation.
fn gen_args_struct(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    obj: &schema::Object,
    name: &str,
    current_ns: &str,
) {
    let needs_lifetime = obj.fields.iter().any(|f| {
        let bt = get_base_type(f.type_.as_ref());
        !type_map::is_scalar(bt)
    });

    let lifetime = if needs_lifetime { "<'a>" } else { "" };

    // C++ flatc uses 4-space indentation for struct fields (different from rest of code)
    w.line(&format!("pub struct {name}Args{lifetime} {{"));
    for field in &obj.fields {
        let fname = field.name.as_deref().unwrap_or("");
        let escaped = type_map::escape_keyword(fname);
        let accessor = type_map::to_snake_case(&escaped);
        let arg_type = args_field_type(schema, field, current_ns);
        w.line(&format!("    pub {accessor}: {arg_type},"));
    }
    w.line("}");

    // Default impl - C++ always uses <'a> lifetime on the impl, even for non-lifetime structs
    w.block(&format!("impl<'a> Default for {name}Args{lifetime}"), |w| {
        w.line("#[inline]");
        w.block("fn default() -> Self", |w| {
            w.line(&format!("{name}Args {{"));
            w.indent();
            for field in &obj.fields {
                let fname = field.name.as_deref().unwrap_or("");
                let escaped = type_map::escape_keyword(fname);
                let accessor = type_map::to_snake_case(&escaped);
                let default = args_field_default(schema, field, current_ns);
                let is_required = field.is_required == Some(true) || has_key_attribute(field);
                if is_required {
                    w.line(&format!("{accessor}: {default}, // required field"));
                } else {
                    w.line(&format!("{accessor}: {default},"));
                }
            }
            w.dedent();
            w.line("}");
        });
    });
}

/// Generate the convenience `create()` function.
/// Generate the inline `create()` method inside the impl block (C++ flatc style).
fn gen_create_method(
    w: &mut CodeWriter,
    _schema: &schema::Schema,
    obj: &schema::Object,
    name: &str,
    _current_ns: &str,
) {
    let needs_lifetime = obj.fields.iter().any(|f| {
        let bt = get_base_type(f.type_.as_ref());
        !type_map::is_scalar(bt)
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

    // Build field add calls — C++ sorts scalars by alignment size descending,
    // then by field index descending within same size. Non-scalars first (reversed).
    let mut non_scalar_fields: Vec<(usize, &schema::Field)> = Vec::new();
    let mut scalar_fields: Vec<(usize, &schema::Field)> = Vec::new();

    for (i, field) in obj.fields.iter().enumerate() {
        let bt = get_base_type(field.type_.as_ref());
        if type_map::is_scalar(bt) {
            scalar_fields.push((i, field));
        } else {
            non_scalar_fields.push((i, field));
        }
    }

    // C++ emits: last non-scalar first, then scalars sorted by size desc then index desc
    for (_, field) in non_scalar_fields.iter().rev() {
        let fname = field.name.as_deref().unwrap_or("");
        let escaped = type_map::escape_keyword(fname);
        let accessor = type_map::to_snake_case(&escaped);
        w.line(&format!(
            "if let Some(x) = args.{accessor} {{ builder.add_{accessor}(x); }}"
        ));
    }

    // Sort scalars by alignment size descending, then field index descending
    scalar_fields.sort_by(|a, b| {
        let sz_a = scalar_alignment_size(get_base_type(a.1.type_.as_ref()));
        let sz_b = scalar_alignment_size(get_base_type(b.1.type_.as_ref()));
        sz_b.cmp(&sz_a).then(b.0.cmp(&a.0))
    });

    for (_, field) in &scalar_fields {
        let fname = field.name.as_deref().unwrap_or("");
        let escaped = type_map::escape_keyword(fname);
        let accessor = type_map::to_snake_case(&escaped);
        if field.is_optional == Some(true) {
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

fn gen_create_fn(
    w: &mut CodeWriter,
    _schema: &schema::Schema,
    obj: &schema::Object,
    name: &str,
    _current_ns: &str,
) {
    let needs_lifetime = obj.fields.iter().any(|f| {
        let bt = get_base_type(f.type_.as_ref());
        !type_map::is_scalar(bt)
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
    let mut non_scalar_fields: Vec<(usize, &schema::Field)> = Vec::new();
    let mut scalar_fields: Vec<(usize, &schema::Field)> = Vec::new();

    for (i, field) in obj.fields.iter().enumerate() {
        let bt = get_base_type(field.type_.as_ref());
        if type_map::is_scalar(bt) {
            scalar_fields.push((i, field));
        } else {
            non_scalar_fields.push((i, field));
        }
    }

    // Non-scalar fields: reversed order, wrap in if let Some
    for (_, field) in non_scalar_fields.iter().rev() {
        let fname = field.name.as_deref().unwrap_or("");
        let escaped = type_map::escape_keyword(fname);
        let accessor = type_map::to_snake_case(&escaped);
        w.line(&format!(
            "if let Some(x) = args.{accessor} {{ builder.add_{accessor}(x); }}"
        ));
    }

    // Scalar fields: sort by alignment size desc then index desc (matches C++ ordering)
    scalar_fields.sort_by(|a, b| {
        let sz_a = scalar_alignment_size(get_base_type(a.1.type_.as_ref()));
        let sz_b = scalar_alignment_size(get_base_type(b.1.type_.as_ref()));
        sz_b.cmp(&sz_a).then(b.0.cmp(&a.0))
    });

    for (_, field) in &scalar_fields {
        let fname = field.name.as_deref().unwrap_or("");
        let escaped = type_map::escape_keyword(fname);
        let accessor = type_map::to_snake_case(&escaped);
        if field.is_optional == Some(true) {
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

// ---- Helper functions ----

/// Returns the alignment/size in bytes of a scalar type.
/// Used to sort fields for optimal vtable packing (matching C++ flatc ordering).
fn scalar_alignment_size(bt: BaseType) -> u32 {
    match bt {
        BaseType::BASE_TYPE_BOOL
        | BaseType::BASE_TYPE_BYTE
        | BaseType::BASE_TYPE_U_BYTE
        | BaseType::BASE_TYPE_U_TYPE => 1,
        BaseType::BASE_TYPE_SHORT | BaseType::BASE_TYPE_U_SHORT => 2,
        BaseType::BASE_TYPE_INT | BaseType::BASE_TYPE_U_INT | BaseType::BASE_TYPE_FLOAT => 4,
        BaseType::BASE_TYPE_LONG | BaseType::BASE_TYPE_U_LONG | BaseType::BASE_TYPE_DOUBLE => 8,
        _ => 0,
    }
}

/// Get the Rust type string for a vector element.
fn vector_element_type(
    schema: &schema::Schema,
    field: &schema::Field,
    element_bt: BaseType,
    lifetime: &str,
    current_ns: &str,
) -> String {
    match element_bt {
        bt if type_map::is_scalar(bt) => {
            // Check if vector of enum
            let has_index = field
                .type_
                .as_ref()
                .and_then(|t| t.index)
                .map(|i| i >= 0)
                .unwrap_or(false);
            if has_index {
                let enum_idx = field_type_index(field);
                if enum_idx < schema.enums.len() {
                    return type_map::resolve_enum_name(schema, current_ns, enum_idx);
                }
            }
            type_map::scalar_rust_type(bt).to_string()
        }
        BaseType::BASE_TYPE_STRING => {
            // C++ uses double space for &'b  str (builder lifetime) but single space for &'a str
            let space = if lifetime == "'b" { "  " } else { " " };
            format!("::flatbuffers::ForwardsUOffset<&{lifetime}{space}str>")
        }
        BaseType::BASE_TYPE_TABLE => {
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
            let tname = type_map::resolve_object_name(schema, current_ns, idx);
            format!("::flatbuffers::ForwardsUOffset<{tname}<{lifetime}>>")
        }
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
            type_map::resolve_object_name(schema, current_ns, idx)
        }
        _ => format!("u8 /* TODO: element type {element_bt:?} */"),
    }
}

/// Get the type string for verifier field visitation.
fn verifier_type_str(schema: &schema::Schema, field: &schema::Field, current_ns: &str) -> String {
    let bt = get_base_type(field.type_.as_ref());

    match bt {
        bt if type_map::is_scalar(bt) => {
            let has_enum = field
                .type_
                .as_ref()
                .and_then(|t| t.index)
                .map(|i| i >= 0)
                .unwrap_or(false);
            if has_enum {
                let idx = field_type_index(field);
                type_map::resolve_enum_name(schema, current_ns, idx)
            } else {
                type_map::scalar_rust_type(bt).to_string()
            }
        }
        BaseType::BASE_TYPE_STRING => "::flatbuffers::ForwardsUOffset<&str>".to_string(),
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
            type_map::resolve_object_name(schema, current_ns, idx)
        }
        BaseType::BASE_TYPE_TABLE => {
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
            let tname = type_map::resolve_object_name(schema, current_ns, idx);
            format!("::flatbuffers::ForwardsUOffset<{tname}>")
        }
        BaseType::BASE_TYPE_VECTOR => {
            let element_bt = get_element_type(field.type_.as_ref());
            let inner = vector_element_type(schema, field, element_bt, "'_", current_ns);
            format!("::flatbuffers::ForwardsUOffset<::flatbuffers::Vector<'_, {inner}>>")
        }
        BaseType::BASE_TYPE_UNION => {
            "::flatbuffers::ForwardsUOffset<::flatbuffers::Table<'_>>".to_string()
        }
        _ => "u8".to_string(),
    }
}

/// Get the scalar default value as a Rust expression.
fn scalar_default(field: &schema::Field, bt: BaseType) -> String {
    if let Some(ref ds) = field.default_string {
        // Enum default handled elsewhere
        return ds.clone();
    }

    if type_map::is_float(bt) {
        let val = field.default_real.unwrap_or(0.0);
        type_map::format_default_real(val, bt)
    } else if bt == BaseType::BASE_TYPE_BOOL {
        let val = field.default_integer.unwrap_or(0);
        type_map::format_default_integer(val, bt)
    } else {
        let val = field.default_integer.unwrap_or(0);
        format!("{val}")
    }
}

/// Get the Rust type for a scalar builder parameter.
/// Returns (type_str, use_push_slot_with_default).
fn scalar_builder_type(
    schema: &schema::Schema,
    field: &schema::Field,
    bt: BaseType,
    current_ns: &str,
) -> (String, bool) {
    let has_enum = get_index(field.type_.as_ref())
        .map(|i| i >= 0)
        .unwrap_or(false);

    let is_optional = field.is_optional == Some(true);

    if has_enum {
        let idx = field_type_index(field);
        let enum_name = type_map::resolve_enum_name(schema, current_ns, idx);
        (enum_name, !is_optional)
    } else {
        (type_map::scalar_rust_type(bt).to_string(), !is_optional)
    }
}

/// Get the default value string for a scalar builder push_slot call.
fn scalar_builder_default(
    schema: &schema::Schema,
    field: &schema::Field,
    bt: BaseType,
    current_ns: &str,
) -> String {
    let has_enum = get_index(field.type_.as_ref())
        .map(|i| i >= 0)
        .unwrap_or(false);

    if has_enum {
        let idx = field_type_index(field);
        let enum_name = type_map::resolve_enum_name(schema, current_ns, idx);
        let is_bitflags = type_map::is_bitflags_enum(schema, idx);
        if let Some(ref ds) = field.default_string {
            format!("{enum_name}::{ds}")
        } else {
            let dv = field.default_integer.unwrap_or(0);
            if is_bitflags {
                format!("{enum_name}::from_bits_retain({dv})")
            } else {
                format!("{enum_name}({dv})")
            }
        }
    } else {
        scalar_default(field, bt)
    }
}

/// Get the Rust type for an Args struct field.
fn args_field_type(schema: &schema::Schema, field: &schema::Field, current_ns: &str) -> String {
    let bt = get_base_type(field.type_.as_ref());
    let is_optional = field.is_optional == Some(true);

    match bt {
        bt if type_map::is_scalar(bt) => {
            let has_enum = field
                .type_
                .as_ref()
                .and_then(|t| t.index)
                .map(|i| i >= 0)
                .unwrap_or(false);
            let base = if has_enum {
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
        BaseType::BASE_TYPE_STRING => "Option<::flatbuffers::WIPOffset<&'a str>>".to_string(),
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
            let sname = type_map::resolve_object_name(schema, current_ns, idx);
            format!("Option<&'a {sname}>")
        }
        BaseType::BASE_TYPE_TABLE => {
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
            let tname = type_map::resolve_object_name(schema, current_ns, idx);
            format!("Option<::flatbuffers::WIPOffset<{tname}<'a>>>")
        }
        BaseType::BASE_TYPE_VECTOR => {
            let element_bt = get_element_type(field.type_.as_ref());
            let inner = vector_element_type(schema, field, element_bt, "'a", current_ns);
            format!("Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, {inner}>>>")
        }
        BaseType::BASE_TYPE_UNION => {
            "Option<::flatbuffers::WIPOffset<::flatbuffers::UnionWIPOffset>>".to_string()
        }
        _ => "u8".to_string(),
    }
}

/// Get the default value for an Args struct field.
fn args_field_default(schema: &schema::Schema, field: &schema::Field, current_ns: &str) -> String {
    let bt = get_base_type(field.type_.as_ref());

    match bt {
        bt if type_map::is_scalar(bt) => {
            if field.is_optional == Some(true) {
                "None".to_string()
            } else {
                scalar_builder_default(schema, field, bt, current_ns)
            }
        }
        _ => "None".to_string(),
    }
}

/// Extract the `nested_flatbuffer` attribute value from a field, if present.
/// The value is the type name of the nested table (e.g., "Monster").
fn get_nested_flatbuffer_attr(field: &schema::Field) -> Option<String> {
    field.attributes.as_ref().and_then(|attrs| {
        attrs.entries.iter().find_map(|e| {
            if e.key.as_deref() == Some("nested_flatbuffer") {
                e.value.as_ref().map(|v| {
                    // Strip surrounding quotes if present (parser preserves them)
                    v.trim_matches('"').to_string()
                })
            } else {
                None
            }
        })
    })
}

/// Find a table in the schema by its short name (not FQN).
fn find_table_by_name(schema: &schema::Schema, name: &str) -> Option<usize> {
    schema
        .objects
        .iter()
        .position(|obj| obj.is_struct != Some(true) && obj.name.as_deref() == Some(name))
}

/// Check if a field has the `key` attribute.
fn has_key_attribute(field: &schema::Field) -> bool {
    field.attributes.as_ref().is_some_and(|attrs| {
        attrs
            .entries
            .iter()
            .any(|e| e.key.as_deref() == Some("key"))
    })
}

/// Generate key_compare_less_than and key_compare_with_value methods.
fn gen_key_methods(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    field: &schema::Field,
    table_name: &str,
    current_ns: &str,
) {
    let fname = field.name.as_deref().unwrap_or("");
    let escaped = type_map::escape_keyword(fname);
    let accessor = type_map::to_snake_case(&escaped);
    let bt = get_base_type(field.type_.as_ref());

    // Determine the key type and comparison style
    let (key_type, is_string) = if bt == BaseType::BASE_TYPE_STRING {
        ("& str".to_string(), true)
    } else if type_map::is_scalar(bt) {
        let has_enum = get_index(field.type_.as_ref())
            .map(|i| i >= 0)
            .unwrap_or(false);
        if has_enum {
            let idx = field_type_index(field);
            (type_map::resolve_enum_name(schema, current_ns, idx), false)
        } else {
            (type_map::scalar_rust_type(bt).to_string(), false)
        }
    } else {
        return; // Unsupported key type
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
}

/// Generate the Object API for a table: owned `{Name}T` type with `pack`/`unpack`.
fn gen_object_api(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    obj: &schema::Object,
    name: &str,
    current_ns: &str,
    opts: &CodeGenOptions,
) {
    let t_name = format!("{name}T");

    // --- Struct definition ---
    w.line("#[non_exhaustive]");
    let mut derives = vec!["Debug", "Clone", "PartialEq"];
    if opts.rust_serialize {
        derives.push("::serde::Serialize");
        derives.push("::serde::Deserialize");
    }
    w.line(&format!("#[derive({})]", derives.join(", ")));
    w.block(&format!("pub struct {t_name}"), |w| {
        for field in &obj.fields {
            let bt = get_base_type(field.type_.as_ref());
            // Skip union type discriminator fields (they're handled by the union T enum)
            if is_union_type_field(schema, field) {
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
                if is_union_type_field(schema, field) {
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

/// Check if a field's type is a union.
fn is_union_field(field: &schema::Field) -> bool {
    get_base_type(field.type_.as_ref()) == BaseType::BASE_TYPE_UNION
}

/// Check if a field is a union type discriminator (the `_type` field for a union).
fn is_union_type_field(schema: &schema::Schema, field: &schema::Field) -> bool {
    let bt = get_base_type(field.type_.as_ref());
    if !type_map::is_scalar(bt) {
        return false;
    }
    let has_index = get_index(field.type_.as_ref())
        .map(|i| i >= 0)
        .unwrap_or(false);
    if !has_index {
        return false;
    }
    let idx = field_type_index(field);
    if idx >= schema.enums.len() {
        return false;
    }
    schema.enums[idx].is_union == Some(true)
}

/// Check if a field is required (explicitly or implicitly as a string key).
fn is_field_required(field: &schema::Field) -> bool {
    if field.is_required == Some(true) {
        return true;
    }
    // String key fields are implicitly required (C++ flatc behavior)
    if field.is_key == Some(true) {
        let bt = get_base_type(field.type_.as_ref());
        if bt == BaseType::BASE_TYPE_STRING {
            return true;
        }
    }
    false
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
            let has_enum = get_index(field.type_.as_ref())
                .map(|i| i >= 0)
                .unwrap_or(false);
            let base = if has_enum {
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
            if is_field_required(field) || field.default_string.is_some() {
                "alloc::string::String".to_string()
            } else {
                "Option<alloc::string::String>".to_string()
            }
        }
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
            let sname = type_map::resolve_object_name(schema, current_ns, idx);
            format!("Option<{sname}T>")
        }
        BaseType::BASE_TYPE_TABLE => {
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
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
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
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
            let has_index = field
                .type_
                .as_ref()
                .and_then(|t| t.index)
                .map(|i| i >= 0)
                .unwrap_or(false);
            if has_index {
                let enum_idx = field_type_index(field);
                if enum_idx < schema.enums.len() {
                    return type_map::resolve_enum_name(schema, current_ns, enum_idx);
                }
            }
            type_map::scalar_rust_type(bt).to_string()
        }
        BaseType::BASE_TYPE_STRING => "alloc::string::String".to_string(),
        BaseType::BASE_TYPE_TABLE => {
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
            let tname = type_map::resolve_object_name(schema, current_ns, idx);
            format!("{tname}T")
        }
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
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
                scalar_builder_default(schema, field, bt, current_ns)
            }
        }
        BaseType::BASE_TYPE_STRING => {
            if is_field_required(field) {
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
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
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
        if is_union_type_field(schema, field) {
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
                if is_field_required(field) || field.default_string.is_some() {
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
                let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
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
        if bt == BaseType::BASE_TYPE_UNION || is_union_type_field(schema, field) {
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
        if is_union_type_field(schema, field) {
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
                if is_field_required(field) {
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
        if is_union_type_field(schema, field) {
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
    let enum_idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
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
