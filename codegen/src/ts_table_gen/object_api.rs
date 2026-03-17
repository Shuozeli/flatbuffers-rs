use flatc_rs_schema::resolved::{ResolvedField, ResolvedObject, ResolvedSchema};
use flatc_rs_schema::BaseType;

use crate::code_writer::CodeWriter;
use crate::field_type_index;
use crate::ts_type_map;
use crate::type_map;

use super::helpers;

pub(super) fn gen_unpack(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
) {
    let non_deprecated: Vec<_> = obj.fields.iter().filter(|f| !f.is_deprecated).collect();
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

pub(super) fn gen_unpack_to(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
    name: &str,
) {
    w.block(&format!("unpackTo(_o:{name}T):void"), |w| {
        for field in &obj.fields {
            if field.is_deprecated {
                continue;
            }
            let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(&field.name));
            let expr = unpack_field_expr(schema, field);
            w.line(&format!("_o.{fname} = {expr};"));
        }
    });
}

fn unpack_field_expr(schema: &ResolvedSchema, field: &ResolvedField) -> String {
    let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(&field.name));
    let bt = type_map::get_base_type(&field.type_);

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
            let et = type_map::get_element_type(&field.type_);
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
                    let idx = field_type_index(field).unwrap();
                    let table_name = &schema.objects[idx].name;
                    format!(
                        "this.bb!.createObjList<{table_name}, {table_name}T>(this.{fname}.bind(this), this.{fname}Length())"
                    )
                }
                BaseType::BASE_TYPE_STRUCT => {
                    let idx = field_type_index(field).unwrap();
                    let struct_name = &schema.objects[idx].name;
                    format!(
                        "this.bb!.createObjList<{struct_name}, {struct_name}T>(this.{fname}.bind(this), this.{fname}Length())"
                    )
                }
                _ => format!("this.{fname}()"),
            }
        }
        BaseType::BASE_TYPE_UNION => {
            // Unpack union using unionToXxx helper for proper type dispatch
            let idx = field.type_.index.unwrap_or(-1);
            if idx >= 0 && (idx as usize) < schema.enums.len() {
                let enum_name = &schema.enums[idx as usize].name;
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

pub(super) fn gen_object_api_class(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    obj: &ResolvedObject,
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
                .filter(|f| !f.is_deprecated)
                .map(|f| {
                    let fname = ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(&f.name));
                    let (ts_type, default) = helpers::object_api_field_type_and_default(schema, f);
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
                        if field.is_deprecated {
                            continue;
                        }
                        let fname =
                            ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(&field.name));
                        let bt = type_map::get_base_type(&field.type_);
                        match bt {
                            BaseType::BASE_TYPE_STRING => {
                                w.line(&format!(
                                    "const {fname} = (this.{fname} !== null ? builder.createString(this.{fname}!) : 0);"
                                ));
                            }
                            BaseType::BASE_TYPE_VECTOR => {
                                gen_pack_vector(w, field, &fname, name);
                            }
                            _ => {}
                        }
                    }
                    w.blank();

                    // Start the table
                    w.line(&format!("{name}.start{name}(builder);"));

                    // Add each field
                    for field in &obj.fields {
                        if field.is_deprecated {
                            continue;
                        }
                        let fname =
                            ts_type_map::escape_ts_keyword(&ts_type_map::to_camel_case(&field.name));
                        let pascal =
                            ts_type_map::escape_ts_keyword(&ts_type_map::to_pascal_case(&field.name));
                        let bt = type_map::get_base_type(&field.type_);

                        match bt {
                            bt if type_map::is_scalar(bt) => {
                                let is_optional = field.is_optional;
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

fn gen_pack_vector(w: &mut CodeWriter, field: &ResolvedField, fname: &str, table_name: &str) {
    let et = type_map::get_element_type(&field.type_);
    let pascal = ts_type_map::escape_ts_keyword(&ts_type_map::to_pascal_case(&field.name));

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
