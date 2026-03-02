use flatc_rs_schema::{self as schema, BaseType};

use super::code_writer::CodeWriter;
use super::field_type_index;
use super::ts_type_map;
use super::type_map;

/// Generate TypeScript code for the table at `schema.objects[index]`.
pub fn generate(w: &mut CodeWriter, schema: &schema::Schema, index: usize, gen_object_api: bool) {
    let obj = &schema.objects[index];
    let name = obj.name.as_deref().unwrap_or("");
    let fqn = build_fqn(obj);

    // Check if this table is the root table (compare FQN to disambiguate same-named tables)
    let is_root = schema
        .root_table
        .as_ref()
        .map(|rt| {
            let rt_fqn = build_fqn(rt);
            rt_fqn == fqn
        })
        .unwrap_or(false);
    let file_ident = schema.file_ident.as_deref().unwrap_or("");

    let implements = if gen_object_api {
        format!(" implements flatbuffers.IUnpackableObject<{name}T>")
    } else {
        String::new()
    };

    // Documentation
    gen_doc_comment(w, obj.documentation.as_ref());

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

        // getRootAsXxx
        gen_get_root_as(w, name);
        w.blank();

        // getSizePrefixedRootAsXxx
        gen_get_size_prefixed_root_as(w, name);
        w.blank();

        // bufferHasIdentifier (only for root table with file_ident)
        if is_root && !file_ident.is_empty() {
            w.block(
                "static bufferHasIdentifier(bb:flatbuffers.ByteBuffer):boolean",
                |w| {
                    w.line(&format!("return bb.__has_identifier('{file_ident}');"));
                },
            );
            w.blank();
        }

        // getFullyQualifiedName
        w.block(
            &format!("static getFullyQualifiedName(): \"{fqn}\""),
            |w| {
                w.line(&format!("return '{fqn}';"));
            },
        );
        w.blank();

        // Field accessors
        for field in &obj.fields {
            if field.is_deprecated == Some(true) {
                continue;
            }
            gen_doc_comment(w, field.documentation.as_ref());
            gen_field_accessor(w, schema, obj, field);
            w.blank();
        }

        // Field mutators (for mutable fields)
        for field in &obj.fields {
            if field.is_deprecated == Some(true) {
                continue;
            }
            let bt = type_map::get_base_type(field.type_.as_ref());
            if type_map::is_scalar(bt) || bt == BaseType::BASE_TYPE_BOOL {
                gen_field_mutator(w, schema, field);
                w.blank();
            }
        }

        // Static builder methods
        gen_start_method(w, obj, name);
        w.blank();

        for field in &obj.fields {
            if field.is_deprecated == Some(true) {
                continue;
            }
            gen_add_method(w, schema, obj, field, name);
            w.blank();

            // Vector create/start helpers
            let bt = type_map::get_base_type(field.type_.as_ref());
            if bt == BaseType::BASE_TYPE_VECTOR {
                gen_vector_helpers(w, schema, field);
                w.blank();
            }
        }

        gen_end_method(w, obj, name);
        w.blank();

        // finishXxxBuffer / finishSizePrefixedXxxBuffer (root table with file_ident)
        if is_root && !file_ident.is_empty() {
            w.block(
                &format!("static finish{name}Buffer(builder:flatbuffers.Builder, offset:flatbuffers.Offset)"),
                |w| {
                    w.line(&format!("builder.finish(offset, '{file_ident}');"));
                },
            );
            w.blank();
            w.block(
                &format!("static finishSizePrefixed{name}Buffer(builder:flatbuffers.Builder, offset:flatbuffers.Offset)"),
                |w| {
                    w.line(&format!("builder.finish(offset, '{file_ident}', true);"));
                },
            );
            w.blank();
        }

        // Convenience create function
        gen_create_fn(w, schema, obj, name);

        // serialize / deserialize
        w.blank();
        w.block("serialize():Uint8Array", |w| {
            w.line("return this.bb!.bytes();");
        });
        w.blank();
        w.block(
            &format!("static deserialize(buffer: Uint8Array):{name}"),
            |w| {
                w.line(&format!(
                    "return {name}.getRootAs{name}(new flatbuffers.ByteBuffer(buffer))"
                ));
            },
        );

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

/// Build FQN like "MyGame.Example.Monster".
fn build_fqn(obj: &schema::Object) -> String {
    let name = obj.name.as_deref().unwrap_or("");
    let ns = type_map::object_namespace(obj);
    if ns.is_empty() {
        name.to_string()
    } else {
        format!("{ns}.{name}")
    }
}

fn gen_get_root_as(w: &mut CodeWriter, name: &str) {
    w.block(
        &format!(
            "static getRootAs{name}(bb:flatbuffers.ByteBuffer, obj?:{name}):{name}"
        ),
        |w| {
            w.line(&format!(
                "return (obj || new {name}()).__init(bb.readInt32(bb.position()) + bb.position(), bb);"
            ));
        },
    );
}

fn gen_get_size_prefixed_root_as(w: &mut CodeWriter, name: &str) {
    w.block(
        &format!(
            "static getSizePrefixedRootAs{name}(bb:flatbuffers.ByteBuffer, obj?:{name}):{name}"
        ),
        |w| {
            w.line("bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);");
            w.line(&format!(
                "return (obj || new {name}()).__init(bb.readInt32(bb.position()) + bb.position(), bb);"
            ));
        },
    );
}

/// Generate a field accessor.
fn gen_field_accessor(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    _obj: &schema::Object,
    field: &schema::Field,
) {
    let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(
        field.name.as_deref().unwrap_or(""),
    ));
    let bt = type_map::get_base_type(field.type_.as_ref());
    let slot = field.id.unwrap_or(0);
    let vt_offset = 4 + 2 * slot;

    match bt {
        BaseType::BASE_TYPE_BOOL => {
            gen_scalar_accessor(w, &fname, bt, field, vt_offset, schema);
        }
        bt if type_map::is_scalar(bt) => {
            // Check for enum type
            let has_enum = type_map::get_index(field.type_.as_ref())
                .map(|i| i >= 0)
                .unwrap_or(false);
            if has_enum {
                gen_enum_accessor(w, schema, &fname, bt, field, vt_offset);
            } else {
                gen_scalar_accessor(w, &fname, bt, field, vt_offset, schema);
            }
        }
        BaseType::BASE_TYPE_STRING => {
            gen_string_accessor(w, &fname, field, vt_offset);
        }
        BaseType::BASE_TYPE_STRUCT => {
            gen_struct_accessor(w, schema, &fname, field, vt_offset);
        }
        BaseType::BASE_TYPE_TABLE => {
            gen_table_accessor(w, schema, &fname, field, vt_offset);
        }
        BaseType::BASE_TYPE_UNION => {
            gen_union_accessor(w, schema, &fname, field, vt_offset);
        }
        BaseType::BASE_TYPE_VECTOR => {
            gen_vector_accessor(w, schema, &fname, field, vt_offset);
        }
        _ => {
            // Unknown type, skip
        }
    }
}

fn gen_scalar_accessor(
    w: &mut CodeWriter,
    fname: &str,
    bt: BaseType,
    field: &schema::Field,
    vt_offset: u32,
    schema: &schema::Schema,
) {
    let ts_type = ts_type_map::scalar_ts_type(bt);
    let read_method = ts_type_map::bb_read_method(bt);
    let is_optional = field.is_optional == Some(true);

    let default_val = if is_optional {
        "null".to_string()
    } else {
        scalar_default_value(field, bt, schema)
    };

    let return_type = if is_optional {
        format!("{ts_type}|null")
    } else {
        ts_type.to_string()
    };

    let value_expr = if bt == BaseType::BASE_TYPE_BOOL {
        format!("!!this.bb!.{read_method}(this.bb_pos + offset)")
    } else {
        format!("this.bb!.{read_method}(this.bb_pos + offset)")
    };

    w.block(&format!("{fname}():{return_type}"), |w| {
        w.line(&format!(
            "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
        ));
        w.line(&format!("return offset ? {value_expr} : {default_val};"));
    });
}

fn gen_enum_accessor(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    fname: &str,
    bt: BaseType,
    field: &schema::Field,
    vt_offset: u32,
) {
    let enum_idx = field_type_index(field);
    let enum_def = &schema.enums[enum_idx];
    let enum_name = enum_def.name.as_deref().unwrap_or("number");
    let read_method = ts_type_map::bb_read_method(bt);

    let default_val = scalar_default_value(field, bt, schema);

    w.block(&format!("{fname}():{enum_name}"), |w| {
        w.line(&format!(
            "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
        ));
        w.line(&format!(
            "return offset ? this.bb!.{read_method}(this.bb_pos + offset) : {default_val};"
        ));
    });
}

fn gen_string_accessor(w: &mut CodeWriter, fname: &str, field: &schema::Field, vt_offset: u32) {
    let has_default = field.default_string.is_some();

    if has_default {
        let default_str = field.default_string.as_deref().unwrap_or("");
        if default_str.is_empty() {
            // default = "" -> return empty string when absent
            w.block(&format!("{fname}():string"), |w| {
                w.line(&format!(
                    "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
                ));
                w.line("return offset ? this.bb!.__string(this.bb_pos + offset)! : '';");
            });
        } else {
            w.block(&format!("{fname}():string"), |w| {
                w.line(&format!(
                    "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
                ));
                w.line(&format!(
                    "return offset ? this.bb!.__string(this.bb_pos + offset)! : '{default_str}';"
                ));
            });
        }
    } else {
        // Regular nullable string
        w.line(&format!("{fname}():string|null"));
        w.line(&format!(
            "{fname}(optionalEncoding:flatbuffers.Encoding):string|Uint8Array|null"
        ));
        w.block(
            &format!("{fname}(optionalEncoding?:any):string|Uint8Array|null"),
            |w| {
                w.line(&format!(
                    "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
                ));
                w.line(
                    "return offset ? this.bb!.__string(this.bb_pos + offset, optionalEncoding) : null;",
                );
            },
        );
    }
}

fn gen_struct_accessor(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    fname: &str,
    field: &schema::Field,
    vt_offset: u32,
) {
    let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
    let struct_name = schema.objects[idx].name.as_deref().unwrap_or("");

    w.block(
        &format!("{fname}(obj?:{struct_name}):{struct_name}|null"),
        |w| {
            w.line(&format!(
                "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
            ));
            w.line(&format!(
                "return offset ? (obj || new {struct_name}()).__init(this.bb_pos + offset, this.bb!) : null;"
            ));
        },
    );
}

fn gen_table_accessor(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    fname: &str,
    field: &schema::Field,
    vt_offset: u32,
) {
    let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
    let table_name = schema.objects[idx].name.as_deref().unwrap_or("");

    w.block(
        &format!("{fname}(obj?:{table_name}):{table_name}|null"),
        |w| {
            w.line(&format!(
                "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
            ));
            w.line(&format!(
                "return offset ? (obj || new {table_name}()).__init(this.bb!.__indirect(this.bb_pos + offset), this.bb!) : null;"
            ));
        },
    );
}

fn gen_union_accessor(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    fname: &str,
    field: &schema::Field,
    vt_offset: u32,
) {
    // Check if the union has string variants - use __union_with_string in that case
    let has_string = type_map::get_index(field.type_.as_ref())
        .and_then(|idx| {
            if idx >= 0 {
                Some(&schema.enums[idx as usize])
            } else {
                None
            }
        })
        .map(|enum_def| {
            enum_def.values.iter().any(|val| {
                let vname = val.name.as_deref().unwrap_or("");
                vname != "NONE"
                    && type_map::get_base_type(val.union_type.as_ref())
                        == BaseType::BASE_TYPE_STRING
            })
        })
        .unwrap_or(false);

    if has_string {
        w.block(
            &format!("{fname}<T extends flatbuffers.Table>(obj:any|string):any|string|null"),
            |w| {
                w.line(&format!(
                    "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
                ));
                w.line(
                    "return offset ? this.bb!.__union_with_string(obj, this.bb_pos + offset) : null;",
                );
            },
        );
    } else {
        w.block(
            &format!("{fname}<T extends flatbuffers.Table>(obj:T):T|null"),
            |w| {
                w.line(&format!(
                    "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
                ));
                w.line("return offset ? this.bb!.__union(obj, this.bb_pos + offset) : null;");
            },
        );
    }
}

fn gen_vector_accessor(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    fname: &str,
    field: &schema::Field,
    vt_offset: u32,
) {
    let et = type_map::get_element_type(field.type_.as_ref());

    match et {
        et if type_map::is_scalar(et) => {
            gen_scalar_vector_accessor(w, schema, fname, field, et, vt_offset);
        }
        BaseType::BASE_TYPE_STRING => {
            gen_string_vector_accessor(w, fname, vt_offset);
        }
        BaseType::BASE_TYPE_TABLE => {
            gen_table_vector_accessor(w, schema, fname, field, vt_offset);
        }
        BaseType::BASE_TYPE_STRUCT => {
            gen_struct_vector_accessor(w, schema, fname, field, vt_offset);
        }
        BaseType::BASE_TYPE_UNION => {
            gen_union_vector_accessor(w, fname, vt_offset);
        }
        _ => {}
    }

    // Length accessor
    w.blank();
    w.block(&format!("{fname}Length():number"), |w| {
        w.line(&format!(
            "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
        ));
        w.line("return offset ? this.bb!.__vector_len(this.bb_pos + offset) : 0;");
    });
}

fn gen_scalar_vector_accessor(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    fname: &str,
    field: &schema::Field,
    et: BaseType,
    vt_offset: u32,
) {
    let ts_type = ts_type_map::scalar_ts_type(et);
    let read_method = ts_type_map::bb_read_method(et);
    let elem_size = ts_type_map::scalar_size(et);

    // Check for enum element type
    let has_enum = type_map::get_index(field.type_.as_ref())
        .map(|i| i >= 0)
        .unwrap_or(false);
    let return_type = if has_enum {
        let enum_idx = field_type_index(field);
        schema.enums[enum_idx]
            .name
            .as_deref()
            .unwrap_or(ts_type)
            .to_string()
    } else {
        ts_type.to_string()
    };

    // Element accessor
    if et == BaseType::BASE_TYPE_BOOL {
        w.block(&format!("{fname}(index: number):{return_type}|null"), |w| {
            w.line(&format!(
                "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
            ));
            w.line(&format!(
                "return offset ? !!this.bb!.{read_method}(this.bb!.__vector(this.bb_pos + offset) + index * {elem_size}) : false;"
            ));
        });
    } else {
        w.block(&format!("{fname}(index: number):{return_type}|null"), |w| {
            w.line(&format!(
                "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
            ));
            w.line(&format!(
                "return offset ? this.bb!.{read_method}(this.bb!.__vector(this.bb_pos + offset) + index * {elem_size}) : 0;"
            ));
        });
    }

    // Typed array accessor
    let array_name = ts_type_map::typed_array_name(et);
    w.blank();
    w.block(&format!("{fname}Array():{array_name}|null"), |w| {
        w.line(&format!(
            "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
        ));
        w.line(&format!(
            "return offset ? new {array_name}(this.bb!.bytes().buffer, this.bb!.bytes().byteOffset + this.bb!.__vector(this.bb_pos + offset), this.bb!.__vector_len(this.bb_pos + offset)) : null;"
        ));
    });
}

fn gen_string_vector_accessor(w: &mut CodeWriter, fname: &str, vt_offset: u32) {
    w.line(&format!("{fname}(index: number):string"));
    w.line(&format!(
        "{fname}(index: number, optionalEncoding:flatbuffers.Encoding):string|Uint8Array"
    ));
    w.block(
        &format!("{fname}(index: number, optionalEncoding?:any):string|Uint8Array|null"),
        |w| {
            w.line(&format!(
                "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
            ));
            w.line("return offset ? this.bb!.__string(this.bb!.__indirect(this.bb!.__vector(this.bb_pos + offset) + index * 4), optionalEncoding) : null;");
        },
    );
}

fn gen_table_vector_accessor(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    fname: &str,
    field: &schema::Field,
    vt_offset: u32,
) {
    let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
    let table_name = schema.objects[idx].name.as_deref().unwrap_or("");

    w.block(
        &format!("{fname}(index: number, obj?:{table_name}):{table_name}|null"),
        |w| {
            w.line(&format!(
                "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
            ));
            w.line(&format!(
                "return offset ? (obj || new {table_name}()).__init(this.bb!.__indirect(this.bb!.__vector(this.bb_pos + offset) + index * 4), this.bb!) : null;"
            ));
        },
    );
}

fn gen_struct_vector_accessor(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    fname: &str,
    field: &schema::Field,
    vt_offset: u32,
) {
    let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
    let struct_name = schema.objects[idx].name.as_deref().unwrap_or("");
    let struct_size = schema.objects[idx].byte_size.unwrap_or(0);

    w.block(
        &format!("{fname}(index: number, obj?:{struct_name}):{struct_name}|null"),
        |w| {
            w.line(&format!(
                "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
            ));
            w.line(&format!(
                "return offset ? (obj || new {struct_name}()).__init(this.bb!.__vector(this.bb_pos + offset) + index * {struct_size}, this.bb!) : null;"
            ));
        },
    );
}

fn gen_union_vector_accessor(w: &mut CodeWriter, fname: &str, vt_offset: u32) {
    w.block(
        &format!("{fname}<T extends flatbuffers.Table>(index: number, obj:T):T|null"),
        |w| {
            w.line(&format!(
                "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
            ));
            w.line(
                "return offset ? this.bb!.__union(obj, this.bb!.__vector(this.bb_pos + offset) + index * 4) : null;",
            );
        },
    );
}

/// Generate field mutator for scalar table fields.
fn gen_field_mutator(w: &mut CodeWriter, schema: &schema::Schema, field: &schema::Field) {
    let pascal = ts_type_map::escape_ts_keyword(&ts_type_map::to_pascal_case(
        field.name.as_deref().unwrap_or(""),
    ));
    let bt = type_map::get_base_type(field.type_.as_ref());
    let slot = field.id.unwrap_or(0);
    let vt_offset = 4 + 2 * slot;

    let ts_type = scalar_field_ts_type(schema, field, bt);
    let write_method = ts_type_map::bb_write_method(bt);

    if bt == BaseType::BASE_TYPE_BOOL {
        w.block(&format!("mutate{pascal}(value:{ts_type}):boolean"), |w| {
            w.line(&format!(
                "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
            ));
            w.blank();
            w.line("if (offset === 0) {");
            w.indent();
            w.line("return false;");
            w.dedent();
            w.line("}");
            w.blank();
            w.line(&format!(
                "this.bb!.{write_method}(this.bb_pos + offset, +value);"
            ));
            w.line("return true;");
        });
    } else {
        w.block(&format!("mutate{pascal}(value:{ts_type}):boolean"), |w| {
            w.line(&format!(
                "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
            ));
            w.blank();
            w.line("if (offset === 0) {");
            w.indent();
            w.line("return false;");
            w.dedent();
            w.line("}");
            w.blank();
            w.line(&format!(
                "this.bb!.{write_method}(this.bb_pos + offset, value);"
            ));
            w.line("return true;");
        });
    }
}

fn gen_start_method(w: &mut CodeWriter, obj: &schema::Object, name: &str) {
    let field_count = obj.fields.len();
    w.block(
        &format!("static start{name}(builder:flatbuffers.Builder)"),
        |w| {
            w.line(&format!("builder.startObject({field_count});"));
        },
    );
}

fn gen_add_method(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    _obj: &schema::Object,
    field: &schema::Field,
    _table_name: &str,
) {
    let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(
        field.name.as_deref().unwrap_or(""),
    ));
    let pascal = ts_type_map::escape_ts_keyword(&ts_type_map::to_pascal_case(
        field.name.as_deref().unwrap_or(""),
    ));
    let bt = type_map::get_base_type(field.type_.as_ref());
    let slot = field.id.unwrap_or(0);

    match bt {
        bt if type_map::is_scalar(bt) => {
            let add_method = ts_type_map::builder_add_field_method(bt);
            let is_optional = field.is_optional == Some(true);

            // Use enum type name if this is an enum field
            let param_type = scalar_field_ts_type(schema, field, bt);

            let default_val = if is_optional {
                "null".to_string()
            } else {
                scalar_default_value(field, bt, schema)
            };

            if bt == BaseType::BASE_TYPE_BOOL {
                let default_expr = if is_optional {
                    "null".to_string()
                } else {
                    format!("+{default_val}")
                };
                w.block(
                    &format!(
                        "static add{pascal}(builder:flatbuffers.Builder, {fname}:{param_type})"
                    ),
                    |w| {
                        w.line(&format!(
                            "builder.{add_method}({slot}, +{fname}, {default_expr});"
                        ));
                    },
                );
            } else {
                w.block(
                    &format!(
                        "static add{pascal}(builder:flatbuffers.Builder, {fname}:{param_type})"
                    ),
                    |w| {
                        w.line(&format!(
                            "builder.{add_method}({slot}, {fname}, {default_val});"
                        ));
                    },
                );
            }
        }
        BaseType::BASE_TYPE_STRING | BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_VECTOR => {
            w.block(
                &format!(
                    "static add{pascal}(builder:flatbuffers.Builder, {fname}Offset:flatbuffers.Offset)"
                ),
                |w| {
                    w.line(&format!(
                        "builder.addFieldOffset({slot}, {fname}Offset, 0);"
                    ));
                },
            );
        }
        BaseType::BASE_TYPE_STRUCT => {
            w.block(
                &format!(
                    "static add{pascal}(builder:flatbuffers.Builder, {fname}Offset:flatbuffers.Offset)"
                ),
                |w| {
                    w.line(&format!(
                        "builder.addFieldStruct({slot}, {fname}Offset, 0);"
                    ));
                },
            );
        }
        BaseType::BASE_TYPE_UNION => {
            w.block(
                &format!(
                    "static add{pascal}(builder:flatbuffers.Builder, {fname}Offset:flatbuffers.Offset)"
                ),
                |w| {
                    w.line(&format!(
                        "builder.addFieldOffset({slot}, {fname}Offset, 0);"
                    ));
                },
            );
        }
        _ => {}
    }
}

fn gen_vector_helpers(w: &mut CodeWriter, schema: &schema::Schema, field: &schema::Field) {
    let pascal = ts_type_map::escape_ts_keyword(&ts_type_map::to_pascal_case(
        field.name.as_deref().unwrap_or(""),
    ));
    let et = type_map::get_element_type(field.type_.as_ref());

    if type_map::is_scalar(et) {
        // createXxxVector for scalar types
        let elem_size = ts_type_map::scalar_size(et);
        let add_method = ts_type_map::builder_add_method(et);
        let ts_type = ts_type_map::scalar_ts_type(et);
        let array_name = ts_type_map::typed_array_name(et);

        w.block(
            &format!(
                "static create{pascal}Vector(builder:flatbuffers.Builder, data:{ts_type}[]|{array_name}):flatbuffers.Offset"
            ),
            |w| {
                w.line(&format!(
                    "builder.startVector({elem_size}, data.length, {elem_size});"
                ));
                w.block(
                    "for (let i = data.length - 1; i >= 0; i--)",
                    |w| {
                        w.line(&format!("builder.{add_method}(data[i]!);"));
                    },
                );
                w.line("return builder.endVector();");
            },
        );
        w.blank();
    }

    // startXxxVector for all vector types
    let (elem_size, alignment) = match et {
        et if type_map::is_scalar(et) => {
            let s = ts_type_map::scalar_size(et);
            (s, s)
        }
        BaseType::BASE_TYPE_STRING | BaseType::BASE_TYPE_TABLE => (4, 4),
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
            let struct_size = schema.objects[idx].byte_size.unwrap_or(0) as usize;
            let struct_align = schema.objects[idx].min_align.unwrap_or(1) as usize;
            (struct_size, struct_align)
        }
        _ => (4, 4),
    };

    w.block(
        &format!("static start{pascal}Vector(builder:flatbuffers.Builder, numElems:number)"),
        |w| {
            w.line(&format!(
                "builder.startVector({elem_size}, numElems, {alignment});"
            ));
        },
    );
}

fn gen_end_method(w: &mut CodeWriter, obj: &schema::Object, name: &str) {
    w.block(
        &format!("static end{name}(builder:flatbuffers.Builder):flatbuffers.Offset"),
        |w| {
            w.line("const offset = builder.endObject();");
            // Add required field checks
            for field in &obj.fields {
                if field.is_required == Some(true) {
                    let slot = field.id.unwrap_or(0);
                    let vt_offset = 4 + 2 * slot;
                    let fname = field.name.as_deref().unwrap_or("");
                    w.line(&format!(
                        "builder.requiredField(offset, {vt_offset}); // {fname}"
                    ));
                }
            }
            w.line("return offset;");
        },
    );
}

fn gen_create_fn(w: &mut CodeWriter, schema: &schema::Schema, obj: &schema::Object, name: &str) {
    // Build parameter list (skip deprecated fields)
    let params: Vec<String> = obj
        .fields
        .iter()
        .filter(|f| f.is_deprecated != Some(true))
        .map(|f| {
            let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(
                f.name.as_deref().unwrap_or(""),
            ));
            let bt = type_map::get_base_type(f.type_.as_ref());
            let is_optional = f.is_optional == Some(true);
            let param_type = create_fn_param_type(schema, f, bt);
            if is_optional && type_map::is_scalar(bt) {
                format!("{fname}:{param_type}|null")
            } else {
                format!("{fname}:{param_type}")
            }
        })
        .collect();

    if params.is_empty() {
        w.block(
            &format!("static create{name}(builder:flatbuffers.Builder):flatbuffers.Offset"),
            |w| {
                w.line(&format!("{name}.start{name}(builder);"));
                w.line(&format!("return {name}.end{name}(builder);"));
            },
        );
        return;
    }

    w.block(
        &format!(
            "static create{name}(builder:flatbuffers.Builder, {}):flatbuffers.Offset",
            params.join(", ")
        ),
        |w| {
            w.line(&format!("{name}.start{name}(builder);"));
            for field in &obj.fields {
                if field.is_deprecated == Some(true) {
                    continue;
                }
                let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(
                    field.name.as_deref().unwrap_or(""),
                ));
                let pascal = ts_type_map::escape_ts_keyword(&ts_type_map::to_pascal_case(
                    field.name.as_deref().unwrap_or(""),
                ));
                let bt = type_map::get_base_type(field.type_.as_ref());
                let is_optional = field.is_optional == Some(true);

                match bt {
                    bt if type_map::is_scalar(bt) => {
                        if is_optional {
                            // Optional scalars: only add if not null
                            w.line(&format!("if ({fname} !== null)"));
                            w.indent();
                            w.line(&format!("{name}.add{pascal}(builder, {fname});"));
                            w.dedent();
                        } else {
                            w.line(&format!("{name}.add{pascal}(builder, {fname});"));
                        }
                    }
                    BaseType::BASE_TYPE_STRING
                    | BaseType::BASE_TYPE_TABLE
                    | BaseType::BASE_TYPE_VECTOR
                    | BaseType::BASE_TYPE_STRUCT
                    | BaseType::BASE_TYPE_UNION => {
                        w.line(&format!("{name}.add{pascal}(builder, {fname});"));
                    }
                    _ => {}
                }
            }
            w.line(&format!("return {name}.end{name}(builder);"));
        },
    );
}

fn create_fn_param_type(schema: &schema::Schema, field: &schema::Field, bt: BaseType) -> String {
    match bt {
        bt if type_map::is_scalar(bt) => {
            let has_enum = type_map::get_index(field.type_.as_ref())
                .map(|i| i >= 0)
                .unwrap_or(false);
            if has_enum {
                let enum_idx = field_type_index(field);
                schema.enums[enum_idx]
                    .name
                    .as_deref()
                    .unwrap_or("number")
                    .to_string()
            } else {
                ts_type_map::scalar_ts_type(bt).to_string()
            }
        }
        BaseType::BASE_TYPE_STRING
        | BaseType::BASE_TYPE_TABLE
        | BaseType::BASE_TYPE_VECTOR
        | BaseType::BASE_TYPE_STRUCT
        | BaseType::BASE_TYPE_UNION => "flatbuffers.Offset".to_string(),
        _ => "number".to_string(),
    }
}

/// Get the TypeScript type for a scalar field, using enum name if applicable.
fn scalar_field_ts_type(schema: &schema::Schema, field: &schema::Field, bt: BaseType) -> String {
    let has_enum = type_map::get_index(field.type_.as_ref())
        .map(|i| i >= 0)
        .unwrap_or(false);
    if has_enum {
        let enum_idx = field_type_index(field);
        schema.enums[enum_idx]
            .name
            .as_deref()
            .unwrap_or("number")
            .to_string()
    } else {
        ts_type_map::scalar_ts_type(bt).to_string()
    }
}

/// Get the TypeScript default value for a scalar field.
fn scalar_default_value(field: &schema::Field, bt: BaseType, schema: &schema::Schema) -> String {
    let has_enum = type_map::get_index(field.type_.as_ref())
        .map(|i| i >= 0)
        .unwrap_or(false);

    if type_map::is_float(bt) {
        // Parser may store integer-valued float defaults (like 42.0) in default_integer
        let val = field
            .default_real
            .unwrap_or_else(|| field.default_integer.unwrap_or(0) as f64);
        ts_type_map::format_default_real_ts(val, bt)
    } else if has_enum {
        // Enum default: check default_string first (e.g., "Blue"), format as EnumName.ValueName
        let enum_idx = field_type_index(field);
        let enum_def = &schema.enums[enum_idx];
        let enum_name = enum_def.name.as_deref().unwrap_or("unknown");

        if let Some(ref ds) = field.default_string {
            format!("{enum_name}.{ds}")
        } else {
            let val = field.default_integer.unwrap_or(0);
            format!("{val}")
        }
    } else {
        let val = field.default_integer.unwrap_or(0);
        ts_type_map::format_default_ts(val, bt)
    }
}

// --- Object API generation ---

fn gen_unpack(w: &mut CodeWriter, schema: &schema::Schema, obj: &schema::Object, name: &str) {
    let non_deprecated: Vec<_> = obj
        .fields
        .iter()
        .filter(|f| f.is_deprecated != Some(true))
        .collect();
    w.block(&format!("unpack():{name}T"), |w| {
        w.line(&format!("return new {name}T("));
        w.indent();
        for (i, field) in non_deprecated.iter().enumerate() {
            let arg = unpack_field_expr(schema, field);
            let comma = if i < non_deprecated.len() - 1 {
                ","
            } else {
                ""
            };
            w.line(&format!("{arg}{comma}"));
        }
        w.dedent();
        w.line(");");
    });
}

fn gen_unpack_to(w: &mut CodeWriter, schema: &schema::Schema, obj: &schema::Object, name: &str) {
    w.block(&format!("unpackTo(_o:{name}T):void"), |w| {
        for field in &obj.fields {
            if field.is_deprecated == Some(true) {
                continue;
            }
            let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(
                field.name.as_deref().unwrap_or(""),
            ));
            let expr = unpack_field_expr(schema, field);
            w.line(&format!("_o.{fname} = {expr};"));
        }
    });
}

fn unpack_field_expr(schema: &schema::Schema, field: &schema::Field) -> String {
    let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(
        field.name.as_deref().unwrap_or(""),
    ));
    let bt = type_map::get_base_type(field.type_.as_ref());

    match bt {
        BaseType::BASE_TYPE_STRUCT => {
            format!("(this.{fname}() !== null ? this.{fname}()!.unpack() : null)")
        }
        BaseType::BASE_TYPE_TABLE => {
            format!("(this.{fname}() !== null ? this.{fname}()!.unpack() : null)")
        }
        BaseType::BASE_TYPE_STRING => {
            format!("this.{fname}()")
        }
        BaseType::BASE_TYPE_VECTOR => {
            let et = type_map::get_element_type(field.type_.as_ref());
            match et {
                et if type_map::is_scalar(et) => {
                    format!(
                        "this.bb!.createScalarList<{ts_type}>(this.{fname}.bind(this), this.{fname}Length())",
                        ts_type = ts_type_map::scalar_ts_type(et)
                    )
                }
                BaseType::BASE_TYPE_STRING => {
                    format!(
                        "this.bb!.createScalarList<string>(this.{fname}.bind(this), this.{fname}Length())"
                    )
                }
                BaseType::BASE_TYPE_TABLE => {
                    let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
                    let table_name = schema.objects[idx].name.as_deref().unwrap_or("");
                    format!(
                        "this.bb!.createObjList<{table_name}, {table_name}T>(this.{fname}.bind(this), this.{fname}Length())"
                    )
                }
                BaseType::BASE_TYPE_STRUCT => {
                    let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
                    let struct_name = schema.objects[idx].name.as_deref().unwrap_or("");
                    format!(
                        "this.bb!.createObjList<{struct_name}, {struct_name}T>(this.{fname}.bind(this), this.{fname}Length())"
                    )
                }
                _ => format!("this.{fname}()"),
            }
        }
        BaseType::BASE_TYPE_UNION => {
            // Unpack union using unionToXxx helper for proper type dispatch
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(-1);
            if idx >= 0 && (idx as usize) < schema.enums.len() {
                let enum_name = schema.enums[idx as usize].name.as_deref().unwrap_or("");
                format!(
                    "(() => {{ const temp = unionTo{enum_name}(this.{fname}Type(), (obj:any) => this.{fname}(obj)); return temp === null ? null : temp.unpack() }})()"
                )
            } else {
                format!("this.{fname}()")
            }
        }
        _ => format!("this.{fname}()"),
    }
}

fn gen_object_api_class(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    obj: &schema::Object,
    name: &str,
) {
    let t_name = format!("{name}T");

    w.block(
        &format!("export class {t_name} implements flatbuffers.IGeneratedObject"),
        |w| {
            // Constructor with all fields + defaults (skip deprecated)
            let ctor_params: Vec<String> = obj
                .fields
                .iter()
                .filter(|f| f.is_deprecated != Some(true))
                .map(|f| {
                    let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(f.name.as_deref().unwrap_or("")));
                    let (ts_type, default) = object_api_field_type_and_default(schema, f);
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
                    // Pre-create strings and vectors
                    for field in &obj.fields {
                        if field.is_deprecated == Some(true) {
                            continue;
                        }
                        let fname =
                            ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(field.name.as_deref().unwrap_or("")));
                        let bt = type_map::get_base_type(field.type_.as_ref());
                        match bt {
                            BaseType::BASE_TYPE_STRING => {
                                w.line(&format!(
                                    "const {fname} = (this.{fname} !== null ? builder.createString(this.{fname}!) : 0);"
                                ));
                            }
                            BaseType::BASE_TYPE_VECTOR => {
                                gen_pack_vector(w, schema, field, &fname, name);
                            }
                            _ => {}
                        }
                    }
                    w.blank();

                    // Start the table
                    w.line(&format!("{name}.start{name}(builder);"));

                    // Add each field
                    for field in &obj.fields {
                        if field.is_deprecated == Some(true) {
                            continue;
                        }
                        let fname =
                            ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(field.name.as_deref().unwrap_or("")));
                        let pascal =
                            ts_type_map::escape_ts_keyword(&ts_type_map::to_pascal_case(field.name.as_deref().unwrap_or("")));
                        let bt = type_map::get_base_type(field.type_.as_ref());

                        match bt {
                            bt if type_map::is_scalar(bt) => {
                                let is_optional = field.is_optional == Some(true);
                                if is_optional {
                                    w.line(&format!("if (this.{fname} !== null)"));
                                    w.indent();
                                    w.line(&format!(
                                        "{name}.add{pascal}(builder, this.{fname});"
                                    ));
                                    w.dedent();
                                } else {
                                    w.line(&format!(
                                        "{name}.add{pascal}(builder, this.{fname});"
                                    ));
                                }
                            }
                            BaseType::BASE_TYPE_STRING | BaseType::BASE_TYPE_VECTOR => {
                                w.line(&format!("{name}.add{pascal}(builder, {fname});"));
                            }
                            BaseType::BASE_TYPE_STRUCT => {
                                w.line(&format!(
                                    "{name}.add{pascal}(builder, (this.{fname} !== null ? this.{fname}!.pack(builder) : 0));"
                                ));
                            }
                            BaseType::BASE_TYPE_TABLE => {
                                w.line(&format!(
                                    "{name}.add{pascal}(builder, (this.{fname} !== null ? this.{fname}!.pack(builder) : 0));"
                                ));
                            }
                            BaseType::BASE_TYPE_UNION => {
                                w.line(&format!(
                                    "{name}.add{pascal}(builder, (this.{fname} !== null ? this.{fname}!.pack(builder) : 0));"
                                ));
                            }
                            _ => {}
                        }
                    }

                    w.line(&format!("return {name}.end{name}(builder);"));
                },
            );
        },
    );
}

fn gen_pack_vector(
    w: &mut CodeWriter,
    _schema: &schema::Schema,
    field: &schema::Field,
    fname: &str,
    table_name: &str,
) {
    let et = type_map::get_element_type(field.type_.as_ref());
    let pascal = ts_type_map::escape_ts_keyword(&ts_type_map::to_pascal_case(
        field.name.as_deref().unwrap_or(""),
    ));

    match et {
        et if type_map::is_scalar(et) => {
            w.line(&format!(
                "const {fname} = {table_name}.create{pascal}Vector(builder, this.{fname});"
            ));
        }
        BaseType::BASE_TYPE_STRING => {
            // Must create each string first, then create the vector
            w.line(&format!("let {fname}Offset = 0;"));
            w.block(&format!("if (this.{fname})"), |w| {
                w.line("const offsets: flatbuffers.Offset[] = [];");
                w.block(&format!("for (const s of this.{fname})"), |w| {
                    w.line("offsets.push(builder.createString(s));");
                });
                w.line(&format!(
                    "{table_name}.start{pascal}Vector(builder, offsets.length);"
                ));
                w.block("for (let i = offsets.length - 1; i >= 0; i--)", |w| {
                    w.line("builder.addOffset(offsets[i]!);");
                });
                w.line(&format!("{fname}Offset = builder.endVector();"));
            });
            // Reassign fname for later use
            w.line(&format!("const {fname} = {fname}Offset;"));
        }
        BaseType::BASE_TYPE_TABLE => {
            w.line(&format!("let {fname}Offset = 0;"));
            w.block(&format!("if (this.{fname})"), |w| {
                w.line("const offsets: flatbuffers.Offset[] = [];");
                w.block(&format!("for (const item of this.{fname})"), |w| {
                    w.line("offsets.push(item.pack(builder));");
                });
                w.line(&format!(
                    "{table_name}.start{pascal}Vector(builder, offsets.length);"
                ));
                w.block("for (let i = offsets.length - 1; i >= 0; i--)", |w| {
                    w.line("builder.addOffset(offsets[i]!);");
                });
                w.line(&format!("{fname}Offset = builder.endVector();"));
            });
            w.line(&format!("const {fname} = {fname}Offset;"));
        }
        BaseType::BASE_TYPE_STRUCT => {
            // Structs are written inline in vectors (not as offsets)
            w.line(&format!("let {fname}Offset = 0;"));
            w.block(&format!("if (this.{fname})"), |w| {
                w.line(&format!(
                    "{table_name}.start{pascal}Vector(builder, this.{fname}.length);"
                ));
                w.block(
                    &format!("for (let i = this.{fname}.length - 1; i >= 0; i--)"),
                    |w| {
                        w.line(&format!("this.{fname}[i]!.pack(builder);"));
                    },
                );
                w.line(&format!("{fname}Offset = builder.endVector();"));
            });
            w.line(&format!("const {fname} = {fname}Offset;"));
        }
        _ => {
            w.line(&format!(
                "const {fname} = 0; // TODO: unsupported vector element type"
            ));
        }
    }
}

/// Emit a JSDoc comment block if documentation is present.
fn gen_doc_comment(w: &mut CodeWriter, doc: Option<&schema::Documentation>) {
    let doc = match doc {
        Some(d) if !d.lines.is_empty() => d,
        _ => return,
    };
    w.line("/**");
    for line in &doc.lines {
        if line.is_empty() {
            w.line(" *");
        } else {
            w.line(&format!(" * {line}"));
        }
    }
    w.line(" */");
}

/// Get TypeScript type and default value for a table's Object API field.
fn object_api_field_type_and_default(
    schema: &schema::Schema,
    field: &schema::Field,
) -> (String, String) {
    let bt = type_map::get_base_type(field.type_.as_ref());
    let is_optional = field.is_optional == Some(true);

    match bt {
        bt if type_map::is_scalar(bt) => {
            let ts_type = ts_type_map::scalar_ts_type(bt);

            // Check for enum type
            let has_enum = type_map::get_index(field.type_.as_ref())
                .map(|i| i >= 0)
                .unwrap_or(false);
            let type_name = if has_enum {
                let enum_idx = field_type_index(field);
                schema.enums[enum_idx]
                    .name
                    .as_deref()
                    .unwrap_or(ts_type)
                    .to_string()
            } else {
                ts_type.to_string()
            };

            if is_optional {
                (format!("{type_name}|null"), "null".to_string())
            } else {
                let default = scalar_default_value(field, bt, schema);
                (type_name, default)
            }
        }
        BaseType::BASE_TYPE_STRING => {
            let has_default = field.default_string.is_some();
            if has_default {
                let default_str = field.default_string.as_deref().unwrap_or("");
                if default_str.is_empty() {
                    ("string".to_string(), "''".to_string())
                } else {
                    ("string".to_string(), format!("'{default_str}'"))
                }
            } else {
                ("string|null".to_string(), "null".to_string())
            }
        }
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
            let struct_name = schema.objects[idx].name.as_deref().unwrap_or("");
            (format!("{struct_name}T|null"), "null".to_string())
        }
        BaseType::BASE_TYPE_TABLE => {
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
            let table_name = schema.objects[idx].name.as_deref().unwrap_or("");
            (format!("{table_name}T|null"), "null".to_string())
        }
        BaseType::BASE_TYPE_VECTOR => {
            let et = type_map::get_element_type(field.type_.as_ref());
            let elem_type = match et {
                et if type_map::is_scalar(et) => {
                    let has_enum = type_map::get_index(field.type_.as_ref())
                        .map(|i| i >= 0)
                        .unwrap_or(false);
                    if has_enum {
                        let enum_idx = field_type_index(field);
                        schema.enums[enum_idx]
                            .name
                            .as_deref()
                            .unwrap_or("number")
                            .to_string()
                    } else {
                        ts_type_map::scalar_ts_type(et).to_string()
                    }
                }
                BaseType::BASE_TYPE_STRING => "string".to_string(),
                BaseType::BASE_TYPE_TABLE => {
                    let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
                    let name = schema.objects[idx].name.as_deref().unwrap_or("");
                    format!("{name}T")
                }
                BaseType::BASE_TYPE_STRUCT => {
                    let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
                    let name = schema.objects[idx].name.as_deref().unwrap_or("");
                    format!("{name}T")
                }
                _ => "unknown".to_string(),
            };

            (format!("({elem_type})[]"), "[]".to_string())
        }
        BaseType::BASE_TYPE_UNION => {
            // Union fields in the Object API: generate discriminated union type
            let idx = field.type_.as_ref().and_then(|t| t.index).unwrap_or(-1);
            if idx >= 0 && (idx as usize) < schema.enums.len() {
                let enum_def = &schema.enums[idx as usize];
                let variant_types: Vec<String> = enum_def
                    .values
                    .iter()
                    .filter(|v| v.name.as_deref() != Some("NONE"))
                    .filter_map(|v| {
                        let vbt = type_map::get_base_type(v.union_type.as_ref());
                        match vbt {
                            BaseType::BASE_TYPE_TABLE => {
                                let vi = v.union_type.as_ref().and_then(|t| t.index).unwrap_or(0)
                                    as usize;
                                let name = schema
                                    .objects
                                    .get(vi)
                                    .and_then(|o| o.name.as_deref())
                                    .unwrap_or("unknown");
                                Some(format!("{name}T"))
                            }
                            BaseType::BASE_TYPE_STRING => Some("string".to_string()),
                            BaseType::BASE_TYPE_STRUCT => {
                                let vi = v.union_type.as_ref().and_then(|t| t.index).unwrap_or(0)
                                    as usize;
                                let name = schema
                                    .objects
                                    .get(vi)
                                    .and_then(|o| o.name.as_deref())
                                    .unwrap_or("unknown");
                                Some(format!("{name}T"))
                            }
                            _ => None,
                        }
                    })
                    .collect();
                if variant_types.is_empty() {
                    ("null".to_string(), "null".to_string())
                } else {
                    let type_str = format!("{}|null", variant_types.join("|"));
                    (type_str, "null".to_string())
                }
            } else {
                ("null".to_string(), "null".to_string())
            }
        }
        _ => ("unknown".to_string(), "null".to_string()),
    }
}
