use flatc_rs_schema::resolved::{ResolvedField, ResolvedObject, ResolvedSchema};
use flatc_rs_schema::BaseType;

use crate::code_writer::CodeWriter;
use crate::ts_type_map;
use crate::type_map;
use crate::{field_id, field_type_index, obj_byte_size};

use super::helpers;

pub(super) fn gen_get_root_as(w: &mut CodeWriter, name: &str) {
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

pub(super) fn gen_get_size_prefixed_root_as(w: &mut CodeWriter, name: &str) {
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
pub(super) fn gen_field_accessor(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    _obj: &ResolvedObject,
    field: &ResolvedField,
) {
    let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(&field.name));
    let bt = type_map::get_base_type(&field.type_);
    let slot = field_id(field).unwrap();
    let vt_offset = 4 + 2 * slot;

    match bt {
        BaseType::BASE_TYPE_BOOL => {
            gen_scalar_accessor(w, &fname, bt, field, vt_offset, schema);
        }
        bt if type_map::is_scalar(bt) => {
            // Check for enum type
            if type_map::has_enum_index(field) {
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
    field: &ResolvedField,
    vt_offset: u32,
    schema: &ResolvedSchema,
) {
    let ts_type = ts_type_map::scalar_ts_type(bt);
    let read_method = ts_type_map::bb_read_method(bt);
    let is_optional = field.is_optional;

    let default_val = if is_optional {
        "null".to_string()
    } else {
        helpers::scalar_default_value(field, bt, schema)
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
    schema: &ResolvedSchema,
    fname: &str,
    bt: BaseType,
    field: &ResolvedField,
    vt_offset: u32,
) {
    let enum_idx = field_type_index(field).unwrap();
    let enum_def = &schema.enums[enum_idx];
    let enum_name = &enum_def.name;
    let read_method = ts_type_map::bb_read_method(bt);

    let default_val = helpers::scalar_default_value(field, bt, schema);

    w.block(&format!("{fname}():{enum_name}"), |w| {
        w.line(&format!(
            "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
        ));
        w.line(&format!(
            "return offset ? this.bb!.{read_method}(this.bb_pos + offset) : {default_val};"
        ));
    });
}

fn gen_string_accessor(w: &mut CodeWriter, fname: &str, field: &ResolvedField, vt_offset: u32) {
    let has_default = field.default_string.is_some();

    if has_default {
        let default_str = field.default_string.as_deref().unwrap_or("");
        w.block(&format!("{fname}():string"), |w| {
            w.line(&format!(
                "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
            ));
            w.line(&format!(
                "return offset ? this.bb!.__string(this.bb_pos + offset)! : '{default_str}';"
            ));
        });
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
    schema: &ResolvedSchema,
    fname: &str,
    field: &ResolvedField,
    vt_offset: u32,
) {
    let idx = field_type_index(field).unwrap();
    let struct_name = &schema.objects[idx].name;

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
    schema: &ResolvedSchema,
    fname: &str,
    field: &ResolvedField,
    vt_offset: u32,
) {
    let idx = field_type_index(field).unwrap();
    let table_name = &schema.objects[idx].name;

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
    schema: &ResolvedSchema,
    fname: &str,
    field: &ResolvedField,
    vt_offset: u32,
) {
    // Check if the union has string variants - use __union_with_string in that case
    let has_string = field
        .type_
        .index
        .and_then(|idx| {
            if idx >= 0 {
                Some(&schema.enums[idx as usize])
            } else {
                None
            }
        })
        .map(|enum_def| {
            enum_def.values.iter().any(|val| {
                val.name != "NONE"
                    && val
                        .union_type
                        .as_ref()
                        .map(|t| t.base_type)
                        .unwrap_or(BaseType::BASE_TYPE_NONE)
                        == BaseType::BASE_TYPE_STRING
            })
        })
        .unwrap_or(false);

    let (signature, union_method) = if has_string {
        (
            format!("{fname}<T extends flatbuffers.Table>(obj:any|string):any|string|null"),
            "__union_with_string",
        )
    } else {
        (
            format!("{fname}<T extends flatbuffers.Table>(obj:T):T|null"),
            "__union",
        )
    };

    w.block(&signature, |w| {
        w.line(&format!(
            "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
        ));
        w.line(&format!(
            "return offset ? this.bb!.{union_method}(obj, this.bb_pos + offset) : null;"
        ));
    });
}

fn gen_vector_accessor(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    fname: &str,
    field: &ResolvedField,
    vt_offset: u32,
) {
    let et = type_map::get_element_type(&field.type_);

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
    schema: &ResolvedSchema,
    fname: &str,
    field: &ResolvedField,
    et: BaseType,
    vt_offset: u32,
) {
    let ts_type = ts_type_map::scalar_ts_type(et);
    let read_method = ts_type_map::bb_read_method(et);
    let elem_size = et.scalar_byte_size();

    // Check for enum element type
    let return_type = if type_map::has_enum_index(field) {
        let enum_idx = field_type_index(field).unwrap();
        schema.enums[enum_idx].name.clone()
    } else {
        ts_type.to_string()
    };

    // Element accessor
    let (read_prefix, default_val) = if et == BaseType::BASE_TYPE_BOOL {
        ("!!", "false")
    } else {
        ("", "0")
    };

    w.block(&format!("{fname}(index: number):{return_type}|null"), |w| {
        w.line(&format!(
            "const offset = this.bb!.__offset(this.bb_pos, {vt_offset});"
        ));
        w.line(&format!(
            "return offset ? {read_prefix}this.bb!.{read_method}(this.bb!.__vector(this.bb_pos + offset) + index * {elem_size}) : {default_val};"
        ));
    });

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
    schema: &ResolvedSchema,
    fname: &str,
    field: &ResolvedField,
    vt_offset: u32,
) {
    let idx = field_type_index(field).unwrap();
    let table_name = &schema.objects[idx].name;

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
    schema: &ResolvedSchema,
    fname: &str,
    field: &ResolvedField,
    vt_offset: u32,
) {
    let idx = field_type_index(field).unwrap();
    let struct_name = &schema.objects[idx].name;
    let struct_size = obj_byte_size(&schema.objects[idx]).unwrap();

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
pub(super) fn gen_field_mutator(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
) {
    let pascal = ts_type_map::escape_ts_keyword(&ts_type_map::to_pascal_case(&field.name));
    let bt = type_map::get_base_type(&field.type_);
    let slot = field_id(field).unwrap();
    let vt_offset = 4 + 2 * slot;

    let ts_type = helpers::scalar_field_ts_type(schema, field, bt);
    let write_method = ts_type_map::bb_write_method(bt);

    let value_expr = if bt == BaseType::BASE_TYPE_BOOL {
        "+value"
    } else {
        "value"
    };

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
            "this.bb!.{write_method}(this.bb_pos + offset, {value_expr});"
        ));
        w.line("return true;");
    });
}
