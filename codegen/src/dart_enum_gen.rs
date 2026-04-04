use flatc_rs_schema::resolved::{ResolvedEnum, ResolvedSchema};

use super::code_writer::CodeWriter;
use super::dart_type_map;
use super::CodeGenError;

/// Generate Dart code for the enum at `schema.enums[index]`.
pub fn generate(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    index: usize,
    _gen_object_api: bool,
) -> Result<(), CodeGenError> {
    let enum_def = &schema.enums[index];
    let is_union = enum_def.is_union;

    if is_union {
        generate_union_enum(w, schema, index)?;
    } else {
        generate_regular_enum(w, enum_def);
    }
    Ok(())
}

/// Generate a regular Dart enum.
fn generate_regular_enum(w: &mut CodeWriter, enum_def: &ResolvedEnum) {
    let name = &enum_def.name;
    let is_bitflags = enum_def
        .attributes
        .as_ref()
        .is_some_and(|attrs| attrs.has("bit_flags"));

    dart_type_map::gen_doc_comment(w, enum_def.documentation.as_ref());

    w.line(&format!("class {name} {{"));
    w.indent();
    w.line("final int value;");
    w.line(&format!("const {name}._(this.value);"));
    w.blank();

    // factory constructor fromValue
    w.line(&format!("factory {name}.fromValue(int value) {{",));
    w.indent();
    w.line("final result = values[value];");
    w.line("if (result == null) {");
    w.indent();
    w.line(&format!(
        "throw StateError('Invalid value $value for bit flag enum {name}',",
    ));
    w.line(");");
    w.dedent();
    w.line("}");
    w.line("return result;");
    w.dedent();
    w.line("}");
    w.blank();

    // static helper for nullable values
    w.line(&format!("static {name}? _createOrNull(int? value) =>",));
    w.indent();
    w.line(&format!("value == null ? null : {name}.fromValue(value);"));
    w.dedent();
    w.blank();

    // min/max values for bitflags
    if is_bitflags {
        let min_val = enum_def.values.first().map(|v| v.value).unwrap_or(0);
        let max_val = enum_def.values.last().map(|v| v.value).unwrap_or(0);
        w.line(&format!("static const int minValue = {min_val};"));
        w.line(&format!("static const int maxValue = {max_val};"));
        w.line("static bool containsValue(int value) => values.containsKey(value);");
        w.blank();
    }

    // Values
    for val in &enum_def.values {
        let vname = &val.name;
        let vval = val.value;
        dart_type_map::gen_doc_comment(w, val.documentation.as_ref());
        w.line(&format!("static const {name} {vname} = {name}._({vval});"));
    }
    w.blank();

    // values map
    w.line(&format!("static const Map<int, {name}> values = {{"));
    w.indent();
    for val in &enum_def.values {
        let vname = &val.name;
        let vval = val.value;
        w.line(&format!("{vval}: {vname},"));
    }
    w.dedent();
    w.line("};");
    w.blank();

    // Reader
    w.line(&format!(
        "static const fb.Reader<{name}> reader = _{name}Reader();"
    ));
    w.blank();

    // toString
    w.line("@override");
    w.line("String toString() {");
    w.indent();
    w.line(&format!("return '{name}{{value: $value}}';"));
    w.dedent();
    w.line("}");

    w.dedent();
    w.line("}");
    w.blank();

    // Reader class
    w.line(&format!("class _{name}Reader extends fb.Reader<{name}> {{"));
    w.indent();
    w.line(&format!("const _{name}Reader();"));
    w.blank();
    w.line("@override");
    w.line("int get size => 1;");
    w.blank();
    w.line("@override");
    w.line(&format!("{name} read(fb.BufferContext bc, int offset) =>",));
    w.indent();
    w.line(&format!(
        "{name}.fromValue(const fb.Int8Reader().read(bc, offset));",
    ));
    w.dedent();
    w.dedent();
    w.line("}");
}

/// Generate a union enum plus helper functions.
fn generate_union_enum(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    index: usize,
) -> Result<(), CodeGenError> {
    let enum_def = &schema.enums[index];
    let name = &enum_def.name;

    dart_type_map::gen_doc_comment(w, enum_def.documentation.as_ref());

    w.line(&format!("class {name} {{"));
    w.indent();
    w.line("final int value;");
    w.line(&format!("const {name}._(this.value);"));
    w.blank();

    // factory constructor fromValue
    w.line(&format!("factory {name}.fromValue(int value) {{",));
    w.indent();
    w.line("final result = values[value];");
    w.line("if (result == null) {");
    w.indent();
    w.line(&format!(
        "throw StateError('Invalid value $value for bit flag enum {name}',",
    ));
    w.line(");");
    w.dedent();
    w.line("}");
    w.line("return result;");
    w.dedent();
    w.line("}");
    w.blank();

    // static helper for nullable values
    w.line(&format!("static {name}? _createOrNull(int? value) =>",));
    w.indent();
    w.line(&format!("value == null ? null : {name}.fromValue(value);"));
    w.dedent();
    w.blank();

    // min/max values
    let min_val = enum_def.values.first().map(|v| v.value).unwrap_or(0);
    let max_val = enum_def.values.last().map(|v| v.value).unwrap_or(0);
    w.line(&format!("static const int minValue = {min_val};"));
    w.line(&format!("static const int maxValue = {max_val};"));
    w.line("static bool containsValue(int value) => values.containsKey(value);");
    w.blank();

    // Values
    for val in &enum_def.values {
        let vname = &val.name;
        let vval = val.value;
        dart_type_map::gen_doc_comment(w, val.documentation.as_ref());
        w.line(&format!("static const {name} {vname} = {name}._({vval});"));
    }
    w.blank();

    // values map
    w.line(&format!("static const Map<int, {name}> values = {{"));
    w.indent();
    for val in &enum_def.values {
        let vname = &val.name;
        let vval = val.value;
        w.line(&format!("{vval}: {vname},"));
    }
    w.dedent();
    w.line("};");
    w.blank();

    // Reader
    w.line(&format!(
        "static const fb.Reader<{name}> reader = _{name}Reader();"
    ));
    w.blank();

    // toString
    w.line("@override");
    w.line("String toString() {");
    w.indent();
    w.line(&format!("return '{name}{{value: $value}}';"));
    w.dedent();
    w.line("}");

    w.dedent();
    w.line("}");
    w.blank();

    // Reader class
    w.line(&format!("class _{name}Reader extends fb.Reader<{name}> {{"));
    w.indent();
    w.line(&format!("const _{name}Reader();"));
    w.blank();
    w.line("@override");
    w.line("int get size => 1;");
    w.blank();
    w.line("@override");
    w.line(&format!("{name} read(fb.BufferContext bc, int offset) =>",));
    w.indent();
    w.line(&format!(
        "{name}.fromValue(const fb.Uint8Reader().read(bc, offset));",
    ));
    w.dedent();
    w.dedent();
    w.line("}");

    Ok(())
}
