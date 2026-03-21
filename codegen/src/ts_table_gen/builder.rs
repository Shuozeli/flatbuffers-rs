use flatc_rs_schema::resolved::{ResolvedField, ResolvedObject, ResolvedSchema};
use flatc_rs_schema::BaseType;

use crate::code_writer::CodeWriter;
use crate::ts_type_map;
use crate::type_map;
use crate::{field_id, field_type_index, obj_byte_size};

use super::helpers;

pub(super) fn gen_start_method(w: &mut CodeWriter, obj: &ResolvedObject, name: &str) {
    let field_count = obj.fields.len();
    w.block(
        &format!("static start{name}(builder:flatbuffers.Builder)"),
        |w| {
            w.line(&format!("builder.startObject({field_count});"));
        },
    );
}

pub(super) fn gen_add_method(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    _obj: &ResolvedObject,
    field: &ResolvedField,
    _table_name: &str,
) {
    let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(&field.name));
    let pascal = ts_type_map::escape_ts_keyword(&ts_type_map::to_pascal_case(&field.name));
    let bt = type_map::get_base_type(&field.type_);
    let slot = field_id(field).unwrap();

    match bt {
        bt if type_map::is_scalar(bt) => {
            let add_method = ts_type_map::builder_add_field_method(bt);
            let is_optional = field.is_optional;

            // Use enum type name if this is an enum field
            let param_type = helpers::scalar_field_ts_type(schema, field, bt);

            let default_val = if is_optional {
                "null".to_string()
            } else {
                helpers::scalar_default_value(field, bt, schema)
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

pub(super) fn gen_vector_helpers(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    field: &ResolvedField,
) {
    let pascal = ts_type_map::escape_ts_keyword(&ts_type_map::to_pascal_case(&field.name));
    let et = type_map::get_element_type(&field.type_);

    if type_map::is_scalar(et) {
        // createXxxVector for scalar types
        let elem_size = et.scalar_byte_size();
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
            let s = et.scalar_byte_size();
            (s, s)
        }
        BaseType::BASE_TYPE_STRING | BaseType::BASE_TYPE_TABLE => (4, 4),
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field_type_index(field).unwrap();
            let struct_size = obj_byte_size(&schema.objects[idx]).unwrap();
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

pub(super) fn gen_end_method(w: &mut CodeWriter, obj: &ResolvedObject, name: &str) {
    w.block(
        &format!("static end{name}(builder:flatbuffers.Builder):flatbuffers.Offset"),
        |w| {
            w.line("const offset = builder.endObject();");
            // Add required field checks
            for field in &obj.fields {
                if field.is_required {
                    let slot = field_id(field).unwrap();
                    let vt_offset = 4 + 2 * slot;
                    let fname = &field.name;
                    w.line(&format!(
                        "builder.requiredField(offset, {vt_offset}); // {fname}"
                    ));
                }
            }
            w.line("return offset;");
        },
    );
}

pub(super) fn gen_create_fn(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
) {
    // Build parameter list (skip deprecated fields)
    let params: Vec<String> = obj
        .fields
        .iter()
        .filter(|f| !f.is_deprecated)
        .map(|f| {
            let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(&f.name));
            let bt = type_map::get_base_type(&f.type_);
            let is_optional = f.is_optional;
            let param_type = helpers::create_fn_param_type(schema, f, bt);
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
                if field.is_deprecated {
                    continue;
                }
                let fname =
                    ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(&field.name));
                let pascal =
                    ts_type_map::escape_ts_keyword(&ts_type_map::to_pascal_case(&field.name));
                let bt = type_map::get_base_type(&field.type_);
                let is_optional = field.is_optional;

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
