use flatc_rs_schema::resolved::{ResolvedField, ResolvedObject, ResolvedSchema};
use flatc_rs_schema::BaseType;

use super::dart_type_map;
use super::type_map;
use super::{field_offset, field_type_index, obj_byte_size, type_index};
use codegen_core::CodeWriter;

/// Context for generating struct field write code.
struct StructFieldWriteCtx<'a> {
    schema: &'a ResolvedSchema,
    field: &'a ResolvedField,
    /// If true, use underscore-prefixed private field names (e.g., `_x`).
    /// If false, use public field names (e.g., `x`).
    use_underscore_prefix: bool,
}

/// Generate Dart code for the struct at `schema.objects[index]`.
pub fn generate(w: &mut CodeWriter, schema: &ResolvedSchema, index: usize, gen_object_api: bool) {
    let obj = &schema.objects[index];
    let name = &obj.name;
    let byte_size = obj_byte_size(obj).unwrap();
    let _fqn = dart_type_map::build_fqn(obj);

    dart_type_map::gen_doc_comment(w, obj.documentation.as_ref());

    // Main class with private constructor
    w.line(&format!("class {name} {{"));
    w.indent();

    // Private fields
    w.line(&format!("{name}._(this._bc, this._bcOffset);"));
    w.blank();

    // Static reader
    w.line(&format!(
        "static const fb.Reader<{name}> reader = _{name}Reader();"
    ));
    w.blank();

    // Private fields
    w.line("final fb.BufferContext _bc;");
    w.line("final int _bcOffset;");
    w.blank();

    // Field accessors
    for field in &obj.fields {
        dart_type_map::gen_doc_comment(w, field.documentation.as_ref());
        gen_field_accessor(w, schema, field);
        w.blank();
    }

    // toString
    w.line("@override");
    w.line("String toString() {");
    w.indent();
    let fields_str = obj
        .fields
        .iter()
        .map(|f| {
            let fname = dart_type_map::escape_dart_keyword(&dart_type_map::to_camel_case(&f.name));
            format!("{fname}: ${fname}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    w.line(&format!("return '{name}{{{fields_str}}}';"));
    w.dedent();
    w.line("}");
    w.blank();

    // unpack
    if gen_object_api {
        w.line(&format!(
            "{name}T unpack() => {name}T({});",
            obj.fields
                .iter()
                .map(|f| {
                    let fname =
                        dart_type_map::escape_dart_keyword(&dart_type_map::to_camel_case(&f.name));
                    format!("{fname}: {fname}")
                })
                .collect::<Vec<_>>()
                .join(", ")
        ));
        w.blank();

        // pack static method
        w.line(&format!(
            "static int pack(fb.Builder fbBuilder, {name}T? object) {{",
        ));
        w.indent();
        w.line("if (object == null) return 0;");
        w.line("return object.pack(fbBuilder);");
        w.dedent();
        w.line("}");
    }

    w.dedent();
    w.line("}");
    w.blank();

    // Object API T class
    if gen_object_api {
        gen_object_api_class(w, schema, obj, name);
    }

    // Reader class
    w.blank();
    w.line(&format!(
        "class _{name}Reader extends fb.StructReader<{name}> {{"
    ));
    w.indent();
    w.line(&format!("const _{name}Reader();"));
    w.blank();
    w.line("@override");
    w.line(&format!("int get size => {byte_size};"));
    w.blank();
    w.line("@override");
    w.line(&format!(
        "{name} createObject(fb.BufferContext bc, int offset) =>",
    ));
    w.indent();
    w.line(&format!("{name}._(bc, offset);"));
    w.dedent();
    w.dedent();
    w.line("}");

    // Builder class
    w.blank();
    w.line(&format!("class {name}Builder {{"));
    w.indent();
    w.line(&format!(
        "{name}Builder(this.fbBuilder, this._{});",
        obj.fields
            .iter()
            .map(|f| dart_type_map::escape_dart_keyword(&dart_type_map::to_camel_case(&f.name)))
            .collect::<Vec<_>>()
            .join(", this._")
    ));
    w.blank();
    w.line("final fb.Builder fbBuilder;");
    for field in &obj.fields {
        let fname = dart_type_map::escape_dart_keyword(&dart_type_map::to_camel_case(&field.name));
        let bt = field.type_.base_type;
        let dart_type = field_dart_type(schema, field, bt);
        w.line(&format!("final {dart_type} _{fname};"));
    }
    w.blank();

    gen_struct_builder_methods(w, schema, obj);

    w.dedent();
    w.line("}");

    // ObjectBuilder class
    if gen_object_api {
        w.blank();
        gen_object_builder(w, schema, obj, name);
    }
}

/// Generate a field accessor for a struct.
fn gen_field_accessor(w: &mut CodeWriter, schema: &ResolvedSchema, field: &ResolvedField) {
    let fname = dart_type_map::escape_dart_keyword(&dart_type_map::to_camel_case(&field.name));
    let bt = field.type_.base_type;

    if bt == BaseType::BASE_TYPE_ARRAY {
        gen_array_accessor(w, schema, &fname, field);
        return;
    }

    if bt == BaseType::BASE_TYPE_STRUCT {
        let idx = field_type_index(field).unwrap();
        let struct_name = &schema.objects[idx].name;
        w.line(&format!(
            "{struct_name}? get {fname} => {struct_name}.reader.vTableGetNullable(",
        ));
        w.indent();
        w.line(&format!(
            "_bc, _bcOffset, {});",
            field_offset(field).unwrap()
        ));
        w.dedent();
        return;
    }

    let dart_type = field_dart_type(schema, field, bt);
    let read_method = dart_type_map::bb_read_method(bt);

    if bt == BaseType::BASE_TYPE_BOOL {
        w.line(&format!(
            "bool get {fname} => const fb.{read_method}().read(_bc, _bcOffset + {}) != 0;",
            field_offset(field).unwrap()
        ));
    } else {
        w.line(&format!(
            "{dart_type} get {fname} => const fb.{read_method}().read(_bc, _bcOffset + {});",
            field_offset(field).unwrap()
        ));
    }
}

/// Generate an index-based accessor for a fixed-length array field.
fn gen_array_accessor(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    fname: &str,
    field: &ResolvedField,
) {
    let (et, _fixed_len, _elem_size) = array_element_info(schema, field);

    if et == BaseType::BASE_TYPE_STRUCT {
        let idx = field_type_index(field).unwrap();
        let struct_name = &schema.objects[idx].name;
        let _struct_size = obj_byte_size(&schema.objects[idx]).unwrap();
        w.line(&format!(
            "{struct_name}? get {fname} => {struct_name}.reader.vTableGetNullable(",
        ));
        w.indent();
        w.line(&format!(
            "_bc, _bcOffset, {});",
            field_offset(field).unwrap()
        ));
        w.dedent();
    } else {
        let dart_type = array_elem_dart_type(schema, field, et);
        let read_method = dart_type_map::bb_read_method(et);

        if et == BaseType::BASE_TYPE_BOOL {
            w.line(&format!(
                "bool? get {fname} => const fb.{read_method}().vTableGetNullable(_bc, _bcOffset, {});",
                field_offset(field).unwrap()
            ));
        } else {
            w.line(&format!(
                "{dart_type}? get {fname} => const fb.{read_method}().vTableGetNullable(_bc, _bcOffset, {});",
                field_offset(field).unwrap()
            ));
        }
    }
}

/// Generate builder methods for a struct.
fn gen_struct_builder_methods(w: &mut CodeWriter, schema: &ResolvedSchema, obj: &ResolvedObject) {
    // finish method - writes all fields in reverse order (FlatBuffers convention)
    w.line("int finish() {");
    w.indent();
    // Write fields in reverse order - Builder uses private fields with underscore prefix
    for field in obj.fields.iter().rev() {
        let ctx = StructFieldWriteCtx {
            schema,
            field,
            use_underscore_prefix: true,
        };
        gen_struct_field_write(w, &ctx);
    }
    w.line("return fbBuilder.offset;");
    w.dedent();
    w.line("}");
}

/// Generate the Object API T class for a struct.
fn gen_object_api_class(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
) {
    let t_name = format!("{name}T");

    w.line(&format!("class {t_name} implements fb.Packable {{"));
    w.indent();

    // Fields
    for field in &obj.fields {
        let fname = dart_type_map::escape_dart_keyword(&dart_type_map::to_camel_case(&field.name));
        let bt = field.type_.base_type;
        let (dart_type, default) = object_api_field_type_and_default(schema, field, bt);
        if default == "null" {
            w.line(&format!("{dart_type}? {fname};"));
        } else {
            w.line(&format!("{dart_type} {fname};"));
        }
    }
    w.blank();

    // Constructor
    let ctor_params: Vec<String> = obj
        .fields
        .iter()
        .map(|f| {
            let fname = dart_type_map::escape_dart_keyword(&dart_type_map::to_camel_case(&f.name));
            let bt = f.type_.base_type;
            let (_, default) = object_api_field_type_and_default(schema, f, bt);
            if default == "null" {
                format!("this.{fname}")
            } else {
                format!("this.{fname} = {default}")
            }
        })
        .collect();

    w.line(&format!("{}(", t_name));
    w.line("{");
    w.indent();
    for (i, param) in ctor_params.iter().enumerate() {
        let comma = if i < ctor_params.len() - 1 { "," } else { "" };
        w.line(&format!("{}{}", param, comma));
    }
    w.dedent();
    w.line("});");
    w.blank();

    // pack method
    w.line("@override");
    w.line("int pack(fb.Builder fbBuilder) {");
    w.indent();
    // Write fields in reverse order (FlatBuffers convention)
    // T class uses public field names (no underscore prefix)
    for field in obj.fields.iter().rev() {
        let ctx = StructFieldWriteCtx {
            schema,
            field,
            use_underscore_prefix: false,
        };
        gen_struct_field_write(w, &ctx);
    }
    w.line("return fbBuilder.offset;");
    w.dedent();
    w.line("}");
    w.blank();

    // toString
    w.line("@override");
    w.line("String toString() {");
    w.indent();
    let fields_str = obj
        .fields
        .iter()
        .map(|f| {
            let fname = dart_type_map::escape_dart_keyword(&dart_type_map::to_camel_case(&f.name));
            format!("{fname}: ${fname}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    w.line(&format!("return '{t_name}{{{fields_str}}}';"));
    w.dedent();
    w.line("}");

    w.dedent();
    w.line("}");
}

/// Generate a field write in the pack method (structs are written in reverse).
fn gen_struct_field_write(w: &mut CodeWriter, ctx: &StructFieldWriteCtx) {
    let fname = dart_type_map::escape_dart_keyword(&dart_type_map::to_camel_case(&ctx.field.name));
    let field_ref = if ctx.use_underscore_prefix {
        format!("_{}", fname)
    } else {
        fname.to_string()
    };
    let bt = ctx.field.type_.base_type;

    if bt == BaseType::BASE_TYPE_STRUCT {
        // Nested struct: pack and write
        w.line(&format!(
            "fbBuilder.putInt32({}?.pack(fbBuilder) ?? 0);",
            field_ref
        ));
        return;
    }

    if bt == BaseType::BASE_TYPE_ARRAY {
        gen_array_field_write(w, ctx, &fname);
        return;
    }

    let write_method = dart_type_map::bb_write_method(bt);

    if bt == BaseType::BASE_TYPE_BOOL {
        w.line(&format!(
            "fbBuilder.{write_method}({field_ref} == true ? 1 : 0);",
        ));
    } else {
        w.line(&format!("fbBuilder.{write_method}({field_ref});"));
    }
}

/// Generate field write code for an array field (writes elements in reverse order).
fn gen_array_field_write(w: &mut CodeWriter, ctx: &StructFieldWriteCtx, fname: &str) {
    let (et, fixed_len, _elem_size) = array_element_info(ctx.schema, ctx.field);
    let field_ref = if ctx.use_underscore_prefix {
        format!("_{}", fname)
    } else {
        fname.to_string()
    };

    if et == BaseType::BASE_TYPE_STRUCT {
        let _idx = field_type_index(ctx.field).unwrap();
        // Write struct array elements in reverse order
        w.line(&format!("for (int i = {fixed_len} - 1; i >= 0; i--) {{"));
        w.indent();
        w.line(&format!(
            "fbBuilder.putInt32({field_ref}?[i]?.pack(fbBuilder) ?? 0);"
        ));
        w.dedent();
        w.line("}");
    } else {
        let write_method = dart_type_map::bb_write_method(et);
        // Write scalar array elements in reverse order
        w.line(&format!("for (int i = {fixed_len} - 1; i >= 0; i--) {{"));
        w.indent();
        if et == BaseType::BASE_TYPE_BOOL {
            w.line(&format!(
                "fbBuilder.{write_method}({field_ref}?[i] == true ? 1 : 0);"
            ));
        } else {
            w.line(&format!("fbBuilder.{write_method}({field_ref}?[i] ?? 0);"));
        }
        w.dedent();
        w.line("}");
    }
}

/// Get Dart type for a struct field.
fn field_dart_type(schema: &ResolvedSchema, field: &ResolvedField, bt: BaseType) -> String {
    if bt == BaseType::BASE_TYPE_STRUCT {
        let idx = field_type_index(field).unwrap();
        return schema.objects[idx].name.clone();
    }

    if bt == BaseType::BASE_TYPE_STRING {
        return "String".to_string();
    }

    if bt == BaseType::BASE_TYPE_UNION {
        return "dynamic".to_string();
    }

    if bt == BaseType::BASE_TYPE_ARRAY {
        let (et, _fixed_len, _elem_size) = array_element_info(schema, field);
        return format!("{}?", array_elem_dart_type(schema, field, et));
    }

    // Check for enum type
    if type_map::is_scalar(bt) && type_map::has_type_index(field) {
        let enum_idx = field_type_index(field).unwrap();
        let enum_def = &schema.enums[enum_idx];
        return enum_def.name.clone();
    }

    dart_type_map::scalar_dart_type(bt).to_string()
}

/// Get Dart type and default value for Object API field.
fn object_api_field_type_and_default(
    schema: &ResolvedSchema,
    field: &ResolvedField,
    bt: BaseType,
) -> (String, String) {
    if bt == BaseType::BASE_TYPE_ARRAY {
        let (et, _fixed_len, _elem_size) = array_element_info(schema, field);
        if et == BaseType::BASE_TYPE_STRUCT {
            let idx = field_type_index(field).unwrap();
            let struct_name = &schema.objects[idx].name;
            (format!("({struct_name}T)"), "null".to_string())
        } else {
            let dart_type = array_elem_dart_type(schema, field, et);
            (format!("({dart_type})"), "null".to_string())
        }
    } else if bt == BaseType::BASE_TYPE_STRUCT {
        let idx = field_type_index(field).unwrap();
        let struct_name = &schema.objects[idx].name;
        (format!("{struct_name}T?"), "null".to_string())
    } else if bt == BaseType::BASE_TYPE_UNION {
        ("dynamic".to_string(), "null".to_string())
    } else {
        let dart_type = field_dart_type(schema, field, bt);
        let default = field_default(field, bt);
        (dart_type, default)
    }
}

/// Get the Dart default value for a struct field.
fn field_default(_field: &ResolvedField, bt: BaseType) -> String {
    match bt {
        BaseType::BASE_TYPE_BOOL => "false".to_string(),
        BaseType::BASE_TYPE_FLOAT | BaseType::BASE_TYPE_DOUBLE => "0.0".to_string(),
        _ => "0".to_string(),
    }
}

/// Extract array element info: (element_base_type, fixed_length, element_size_in_bytes).
fn array_element_info(schema: &ResolvedSchema, field: &ResolvedField) -> (BaseType, usize, usize) {
    let ty = &field.type_;
    let et = ty.element_type.unwrap_or(BaseType::BASE_TYPE_NONE);
    let fixed_len = ty
        .fixed_length
        .expect("BUG: array field has no fixed_length") as usize;

    let elem_size = if et == BaseType::BASE_TYPE_STRUCT {
        let idx = type_index(ty, "array element struct lookup").unwrap();
        obj_byte_size(&schema.objects[idx]).unwrap()
    } else {
        et.scalar_byte_size()
    };

    (et, fixed_len, elem_size)
}

/// Get the Dart type name for an array element.
fn array_elem_dart_type(schema: &ResolvedSchema, field: &ResolvedField, et: BaseType) -> String {
    if et == BaseType::BASE_TYPE_STRUCT {
        let idx = field_type_index(field).unwrap();
        return schema.objects[idx].name.clone();
    }

    if et == BaseType::BASE_TYPE_ARRAY {
        // Nested array - get element type from the array's element type
        let ty = &field.type_;
        let inner_et = ty.element_type.unwrap_or(BaseType::BASE_TYPE_NONE);
        return format!("{}?", array_elem_dart_type(schema, field, inner_et));
    }

    // Check for enum index on the array type
    if type_map::is_scalar(et) && type_map::has_type_index(field) {
        let enum_idx = field_type_index(field).unwrap();
        schema.enums[enum_idx].name.clone()
    } else {
        dart_type_map::scalar_dart_type(et).to_string()
    }
}

/// Generate the ObjectBuilder class for a struct.
fn gen_object_builder(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
) {
    let _t_name = format!("{name}T");

    w.line(&format!(
        "class {name}ObjectBuilder extends fb.ObjectBuilder {{"
    ));
    w.indent();

    // Fields for the builder
    for field in &obj.fields {
        let fname = dart_type_map::escape_dart_keyword(&dart_type_map::to_camel_case(&field.name));
        let bt = field.type_.base_type;
        let dart_type = field_dart_type(schema, field, bt);
        w.line(&format!("final {dart_type} _{fname};"));
    }
    w.blank();

    // Constructor - use positional parameters to initialize private fields
    w.line(&format!("{name}ObjectBuilder("));
    w.indent();
    for (i, field) in obj.fields.iter().enumerate() {
        let fname = dart_type_map::escape_dart_keyword(&dart_type_map::to_camel_case(&field.name));
        if i < obj.fields.len() - 1 {
            w.line(&format!("this._{fname},"));
        } else {
            w.line(&format!("this._{fname}"));
        }
    }
    w.dedent();
    w.line(");");
    w.blank();

    // finish method
    w.line("@override");
    w.line("int finish(fb.Builder fbBuilder) {");
    w.indent();
    // Write fields in reverse order (FlatBuffers convention)
    // ObjectBuilder uses private field names (with underscore prefix)
    for field in obj.fields.iter().rev() {
        let ctx = StructFieldWriteCtx {
            schema,
            field,
            use_underscore_prefix: true,
        };
        gen_struct_field_write(w, &ctx);
    }
    w.line("return fbBuilder.offset;");
    w.dedent();
    w.line("}");
    w.blank();

    // toBytes method
    w.line("@override");
    w.line("Uint8List toBytes([String? fileIdentifier]) {");
    w.indent();
    w.line("final fbBuilder = fb.Builder(deduplicateTables: false);");
    w.line("fbBuilder.finish(finish(fbBuilder), fileIdentifier);");
    w.line("return fbBuilder.buffer;");
    w.dedent();
    w.line("}");

    w.dedent();
    w.line("}");
}
