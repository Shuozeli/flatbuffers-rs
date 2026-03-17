use flatc_rs_schema::resolved::{ResolvedField, ResolvedObject, ResolvedSchema};
use flatc_rs_schema::BaseType;

use super::code_writer::CodeWriter;
use super::ts_type_map;
use super::type_map;
use super::{field_offset, field_type_index, obj_byte_size, obj_min_align, type_index};

/// Generate TypeScript code for the struct at `schema.objects[index]`.
pub fn generate(w: &mut CodeWriter, schema: &ResolvedSchema, index: usize, gen_object_api: bool) {
    let obj = &schema.objects[index];
    let name = &obj.name;
    let byte_size = obj_byte_size(obj).unwrap();
    let fqn = ts_type_map::build_fqn(obj);

    // Accessor class
    let implements = if gen_object_api {
        format!(" implements flatbuffers.IUnpackableObject<{name}T>")
    } else {
        String::new()
    };

    // Documentation
    ts_type_map::gen_doc_comment(w, obj.documentation.as_ref());

    w.block(&format!("export class {name}{implements}"), |w| {
        // bb and bb_pos properties
        w.line("bb: flatbuffers.ByteBuffer|null = null;");
        w.line("bb_pos = 0;");
        w.blank();

        // __init
        w.block(
            &format!("__init(i:number, bb:flatbuffers.ByteBuffer):{name}"),
            |w| {
                w.line("this.bb_pos = i;");
                w.line("this.bb = bb;");
                w.line("return this;");
            },
        );
        w.blank();

        // getFullyQualifiedName
        w.block(&format!("static getFullyQualifiedName(): \"{fqn}\""), |w| {
            w.line(&format!("return '{fqn}';"));
        });
        w.blank();

        // Field accessors
        for field in &obj.fields {
            ts_type_map::gen_doc_comment(w, field.documentation.as_ref());
            gen_field_accessor(w, schema, field);
            w.blank();
        }

        // Field mutators (skip struct and array fields)
        for field in &obj.fields {
            let bt = type_map::get_base_type(&field.type_);
            if bt == BaseType::BASE_TYPE_STRUCT || bt == BaseType::BASE_TYPE_ARRAY {
                continue;
            }
            gen_field_mutator(w, schema, field);
            w.blank();
        }

        // static sizeOf
        w.block("static sizeOf():number", |w| {
            w.line(&format!("return {byte_size};"));
        });
        w.blank();

        // static createXxx
        gen_static_create(w, schema, obj, name);

        // unpack / unpackTo (Object API)
        if gen_object_api {
            w.blank();
            gen_unpack(w, schema, obj, name);
            w.blank();
            gen_unpack_to(w, schema, obj, name);
        }
    });

    // Object API T class
    if gen_object_api {
        w.blank();
        gen_object_api_class(w, schema, obj, name);
    }
}

/// Generate a field accessor for a struct.
fn gen_field_accessor(w: &mut CodeWriter, schema: &ResolvedSchema, field: &ResolvedField) {
    let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(&field.name));
    let offset = field_offset(field).unwrap();
    let bt = type_map::get_base_type(&field.type_);

    if bt == BaseType::BASE_TYPE_ARRAY {
        gen_array_accessor(w, schema, &fname, field, offset);
        return;
    }

    if bt == BaseType::BASE_TYPE_STRUCT {
        let idx = field_type_index(field).unwrap();
        let struct_name = &schema.objects[idx].name;
        w.block(
            &format!("{fname}(obj?:{struct_name}):{struct_name}|null"),
            |w| {
                w.line(&format!(
                    "return (obj || new {struct_name}()).__init(this.bb_pos + {offset}, this.bb!);"
                ));
            },
        );
        return;
    }

    let ts_type = field_ts_type(schema, field, bt);
    let read_method = ts_type_map::bb_read_method(bt);

    if bt == BaseType::BASE_TYPE_BOOL {
        w.block(&format!("{fname}():{ts_type}"), |w| {
            w.line(&format!(
                "return !!this.bb!.{read_method}(this.bb_pos + {offset});"
            ));
        });
    } else {
        w.block(&format!("{fname}():{ts_type}"), |w| {
            w.line(&format!(
                "return this.bb!.{read_method}(this.bb_pos + {offset});"
            ));
        });
    }
}

/// Generate an index-based accessor for a fixed-length array field.
fn gen_array_accessor(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    fname: &str,
    field: &ResolvedField,
    offset: usize,
) {
    let (et, _fixed_len, elem_size) = array_element_info(schema, field);

    if et == BaseType::BASE_TYPE_STRUCT {
        let idx = field_type_index(field).unwrap();
        let struct_name = &schema.objects[idx].name;
        let struct_size = obj_byte_size(&schema.objects[idx]).unwrap();

        w.block(
            &format!("{fname}(index: number, obj?:{struct_name}):{struct_name}|null"),
            |w| {
                w.line(&format!(
                    "return (obj || new {struct_name}()).__init(this.bb_pos + {offset} + index * {struct_size}, this.bb!);"
                ));
            },
        );
    } else {
        let ts_type = array_elem_ts_type(schema, field, et);
        let read_method = ts_type_map::bb_read_method(et);

        if et == BaseType::BASE_TYPE_BOOL {
            w.block(&format!("{fname}(index: number):{ts_type}|null"), |w| {
                w.line(&format!(
                    "return !!this.bb!.{read_method}(this.bb_pos + {offset} + index * {elem_size});"
                ));
            });
        } else {
            w.block(&format!("{fname}(index: number):{ts_type}|null"), |w| {
                w.line(&format!(
                    "return this.bb!.{read_method}(this.bb_pos + {offset} + index * {elem_size});"
                ));
            });
        }
    }
}

/// Generate a field mutator for a struct.
fn gen_field_mutator(w: &mut CodeWriter, schema: &ResolvedSchema, field: &ResolvedField) {
    let offset = field_offset(field).unwrap();
    let bt = type_map::get_base_type(&field.type_);

    // Struct and array fields don't get simple mutators
    if bt == BaseType::BASE_TYPE_STRUCT || bt == BaseType::BASE_TYPE_ARRAY {
        return;
    }

    let ts_type = field_ts_type(schema, field, bt);
    let write_method = ts_type_map::bb_write_method(bt);
    let pascal = ts_type_map::escape_ts_keyword(&ts_type_map::to_pascal_case(&field.name));

    if bt == BaseType::BASE_TYPE_BOOL {
        w.block(&format!("mutate{pascal}(value:{ts_type}):boolean"), |w| {
            w.line(&format!(
                "this.bb!.{write_method}(this.bb_pos + {offset}, +value);"
            ));
            w.line("return true;");
        });
    } else {
        w.block(&format!("mutate{pascal}(value:{ts_type}):boolean"), |w| {
            w.line(&format!(
                "this.bb!.{write_method}(this.bb_pos + {offset}, value);"
            ));
            w.line("return true;");
        });
    }
}

/// Generate static createXxx method.
fn gen_static_create(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
) {
    let byte_size = obj_byte_size(obj).unwrap();
    let min_align = obj_min_align(obj).unwrap();

    // Build parameter list
    let params: Vec<String> = obj
        .fields
        .iter()
        .map(|f| {
            let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(&f.name));
            let bt = type_map::get_base_type(&f.type_);
            if bt == BaseType::BASE_TYPE_ARRAY {
                let ts_type = create_param_array_type(schema, f);
                format!("{fname}:{ts_type}")
            } else {
                let ts_type = field_ts_type(schema, f, bt);
                format!("{fname}:{ts_type}")
            }
        })
        .collect();

    let param_str = if params.is_empty() {
        "builder:flatbuffers.Builder".to_string()
    } else {
        format!("builder:flatbuffers.Builder, {}", params.join(", "))
    };

    w.block(
        &format!("static create{name}({param_str}):flatbuffers.Offset"),
        |w| {
            w.line(&format!("builder.prep({min_align}, {byte_size});"));

            // Write fields in reverse order (FlatBuffers convention)
            for field in obj.fields.iter().rev() {
                gen_struct_field_write(w, schema, field);
            }

            w.line("return builder.offset();");
        },
    );
}

/// Generate a field write in the static create method (structs are written in reverse).
fn gen_struct_field_write(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
) {
    let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(&field.name));
    let bt = type_map::get_base_type(&field.type_);

    let padding = field.padding.unwrap_or(0) as usize;

    if bt == BaseType::BASE_TYPE_ARRAY {
        gen_array_field_write(w, schema, field, &fname, padding);
        return;
    }

    if bt == BaseType::BASE_TYPE_STRUCT {
        // Nested struct: handled by caller passing struct offset (noop here except padding)
        if padding > 0 {
            w.line(&format!("builder.pad({padding});"));
        }
        return;
    }

    if padding > 0 {
        w.line(&format!("builder.pad({padding});"));
    }

    let write_method = ts_type_map::builder_write_method(bt);

    if bt == BaseType::BASE_TYPE_BOOL {
        w.line(&format!("builder.{write_method}(+{fname});"));
    } else {
        w.line(&format!("builder.{write_method}({fname});"));
    }
}

/// Generate the write loop for a fixed-length array in createXxx.
fn gen_array_field_write(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
    fname: &str,
    padding: usize,
) {
    let (et, fixed_len, _elem_size) = array_element_info(schema, field);

    if padding > 0 {
        w.line(&format!("builder.pad({padding});"));
    }

    let max_idx = fixed_len - 1;

    if et == BaseType::BASE_TYPE_STRUCT {
        let idx = field_type_index(field).unwrap();
        let struct_name = &schema.objects[idx].name;
        let t_name = format!("{struct_name}T");

        w.block(&format!("for (let i = {max_idx}; i >= 0; --i)"), |w| {
            w.line(&format!("const item = {fname}?.[i];"));
            w.block(&format!("if (item instanceof {t_name})"), |w| {
                w.line("item.pack(builder);");
                w.line("continue;");
            });
            // Fallback: create default struct
            w.line(&format!("new {t_name}().pack(builder);"));
        });
    } else {
        // Scalar/enum array
        let write_method = ts_type_map::builder_write_method(et);
        let default_val = if et == BaseType::BASE_TYPE_LONG || et == BaseType::BASE_TYPE_U_LONG {
            "BigInt(0)"
        } else {
            "0"
        };

        w.block(&format!("for (let i = {max_idx}; i >= 0; --i)"), |w| {
            w.line(&format!(
                "builder.{write_method}({fname}?.[i] ?? {default_val});"
            ));
        });
    }
}

/// Generate unpack() method.
fn gen_unpack(w: &mut CodeWriter, schema: &ResolvedSchema, obj: &ResolvedObject, name: &str) {
    w.block(&format!("unpack():{name}T"), |w| {
        let args: Vec<String> = obj
            .fields
            .iter()
            .map(|f| unpack_struct_field_expr(schema, f))
            .collect();

        w.line(&format!("return new {name}T("));
        w.indent();
        for (i, arg) in args.iter().enumerate() {
            let comma = if i < args.len() - 1 { "," } else { "" };
            w.line(&format!("{arg}{comma}"));
        }
        w.dedent();
        w.line(");");
    });
}

/// Generate unpackTo() method.
fn gen_unpack_to(w: &mut CodeWriter, schema: &ResolvedSchema, obj: &ResolvedObject, name: &str) {
    w.block(&format!("unpackTo(_o:{name}T):void"), |w| {
        for field in &obj.fields {
            let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(&field.name));
            let expr = unpack_struct_field_expr(schema, field);
            w.line(&format!("_o.{fname} = {expr};"));
        }
    });
}

/// Generate the Object API T class for a struct.
fn gen_object_api_class(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
) {
    let t_name = format!("{name}T");

    w.block(
        &format!("export class {t_name} implements flatbuffers.IGeneratedObject"),
        |w| {
            // Constructor with all fields + defaults
            let ctor_params: Vec<String> = obj
                .fields
                .iter()
                .map(|f| {
                    let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(&f.name));
                    let bt = type_map::get_base_type(&f.type_);
                    let (ts_type, default) = object_api_field_type_and_default(schema, f, bt);
                    format!("public {fname}:{ts_type} = {default}")
                })
                .collect();

            if ctor_params.is_empty() {
                w.line("constructor() {}");
            } else {
                w.line("constructor(");
                w.indent();
                for (i, param) in ctor_params.iter().enumerate() {
                    let comma = if i < ctor_params.len() - 1 { "," } else { "" };
                    w.line(&format!("{param}{comma}"));
                }
                w.dedent();
                w.line(") {}");
            }
            w.blank();

            // pack method
            w.block(
                "pack(builder:flatbuffers.Builder):flatbuffers.Offset",
                |w| {
                    let args: Vec<String> = obj
                        .fields
                        .iter()
                        .map(|f| {
                            let fname = ts_type_map::escape_ts_keyword(
                                &ts_type_map::to_camel_case(&f.name),
                            );
                            let bt = type_map::get_base_type(&f.type_);
                            if bt == BaseType::BASE_TYPE_STRUCT {
                                format!("(this.{fname} !== null ? this.{fname}!.pack(builder) : 0)")
                            } else {
                                format!("this.{fname}")
                            }
                        })
                        .collect();
                    w.line(&format!(
                        "return {name}.create{name}(builder, {});",
                        args.join(", ")
                    ));
                },
            );
        },
    );
}

/// Get TypeScript type and default value for a struct's Object API field.
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
            (format!("({struct_name}T)[]"), "[]".to_string())
        } else {
            let ts_type = array_elem_ts_type(schema, field, et);
            (format!("({ts_type})[]"), "[]".to_string())
        }
    } else if bt == BaseType::BASE_TYPE_STRUCT {
        let idx = field_type_index(field).unwrap();
        let struct_name = &schema.objects[idx].name;
        (format!("{struct_name}T|null"), "null".to_string())
    } else {
        let ts_type = field_ts_type(schema, field, bt);
        let default = field_default_ts(field, bt);
        (ts_type, default)
    }
}

/// Get the TypeScript type for a struct field.
fn field_ts_type(schema: &ResolvedSchema, field: &ResolvedField, bt: BaseType) -> String {
    if bt == BaseType::BASE_TYPE_STRUCT {
        let idx = field_type_index(field).unwrap();
        return schema.objects[idx].name.clone();
    }

    // Check for enum type
    if type_map::is_scalar(bt) && type_map::has_enum_index(field) {
        let enum_idx = field_type_index(field).unwrap();
        let enum_def = &schema.enums[enum_idx];
        return enum_def.name.clone();
    }

    ts_type_map::scalar_ts_type(bt).to_string()
}

/// Get the TypeScript default value string for a struct field.
fn field_default_ts(_field: &ResolvedField, bt: BaseType) -> String {
    match bt {
        BaseType::BASE_TYPE_BOOL => "false".to_string(),
        BaseType::BASE_TYPE_LONG | BaseType::BASE_TYPE_U_LONG => "BigInt('0')".to_string(),
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
        ts_type_map::scalar_size(et)
    };

    (et, fixed_len, elem_size)
}

/// Get the TS type name for an array element.
fn array_elem_ts_type(schema: &ResolvedSchema, field: &ResolvedField, et: BaseType) -> String {
    // Check for enum index on the array type
    if type_map::is_scalar(et) && type_map::has_enum_index(field) {
        let enum_idx = field_type_index(field).unwrap();
        schema.enums[enum_idx].name.clone()
    } else {
        ts_type_map::scalar_ts_type(et).to_string()
    }
}

/// Get the parameter type for an array field in createXxx.
fn create_param_array_type(schema: &ResolvedSchema, field: &ResolvedField) -> String {
    let (et, _fixed_len, _elem_size) = array_element_info(schema, field);
    if et == BaseType::BASE_TYPE_STRUCT {
        let idx = field_type_index(field).unwrap();
        let struct_name = &schema.objects[idx].name;
        let t_name = format!("{struct_name}T");
        format!("(any|{t_name})[]|null")
    } else {
        let ts_type = array_elem_ts_type(schema, field, et);
        format!("{ts_type}[]|null")
    }
}

/// Generate the unpack expression for a struct field.
fn unpack_struct_field_expr(schema: &ResolvedSchema, field: &ResolvedField) -> String {
    let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(&field.name));
    let bt = type_map::get_base_type(&field.type_);

    if bt == BaseType::BASE_TYPE_ARRAY {
        let (et, fixed_len, _elem_size) = array_element_info(schema, field);
        if et == BaseType::BASE_TYPE_STRUCT {
            let idx = field_type_index(field).unwrap();
            let struct_name = &schema.objects[idx].name;
            let t_name = format!("{struct_name}T");
            format!(
                "this.bb!.createObjList<{struct_name}, {t_name}>(this.{fname}.bind(this), {fixed_len})"
            )
        } else {
            let ts_type = array_elem_ts_type(schema, field, et);
            format!("this.bb!.createScalarList<{ts_type}>(this.{fname}.bind(this), {fixed_len})")
        }
    } else if bt == BaseType::BASE_TYPE_STRUCT {
        format!("(this.{fname}() !== null ? this.{fname}()!.unpack() : null)")
    } else {
        format!("this.{fname}()")
    }
}
