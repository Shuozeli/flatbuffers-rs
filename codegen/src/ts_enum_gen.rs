use flatc_rs_schema::resolved::{ResolvedEnum, ResolvedSchema};
use flatc_rs_schema::BaseType;

use super::code_writer::CodeWriter;
use super::ts_type_map;
use super::union_variant_type_index;

/// Generate TypeScript code for the enum at `schema.enums[index]`.
pub fn generate(w: &mut CodeWriter, schema: &ResolvedSchema, index: usize, gen_object_api: bool) {
    let enum_def = &schema.enums[index];
    let is_union = enum_def.is_union;

    if is_union {
        generate_union_enum(w, schema, index, gen_object_api);
    } else {
        generate_regular_enum(w, enum_def);
    }
}

/// Generate a regular TypeScript enum.
fn generate_regular_enum(w: &mut CodeWriter, enum_def: &ResolvedEnum) {
    let name = &enum_def.name;

    ts_type_map::gen_doc_comment(w, enum_def.documentation.as_ref());
    w.block(&format!("export enum {name}"), |w| {
        for (i, val) in enum_def.values.iter().enumerate() {
            let vname = &val.name;
            let vval = val.value;
            let comma = if i < enum_def.values.len() - 1 {
                ","
            } else {
                ""
            };
            ts_type_map::gen_doc_comment(w, val.documentation.as_ref());
            w.line(&format!("{vname} = {vval}{comma}"));
        }
    });
}

/// Generate a union enum plus helper functions.
fn generate_union_enum(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    index: usize,
    gen_object_api: bool,
) {
    let enum_def = &schema.enums[index];
    let name = &enum_def.name;

    // Generate the union discriminator enum
    ts_type_map::gen_doc_comment(w, enum_def.documentation.as_ref());
    w.block(&format!("export enum {name}"), |w| {
        for (i, val) in enum_def.values.iter().enumerate() {
            let vname = &val.name;
            let vval = val.value;
            let comma = if i < enum_def.values.len() - 1 {
                ","
            } else {
                ""
            };
            ts_type_map::gen_doc_comment(w, val.documentation.as_ref());
            w.line(&format!("{vname} = {vval}{comma}"));
        }
    });
    w.blank();

    // Generate unionToXxx helper function
    gen_union_to_helper(w, schema, enum_def, name);
    w.blank();

    // Generate unionListToXxx helper function
    gen_union_list_to_helper(w, schema, enum_def, name);

    // Object API: union T type
    if gen_object_api {
        w.blank();
        gen_union_object_api(w, schema, index);
    }
}

/// Collect the union variant type names for use in type signatures.
fn union_variant_types(schema: &ResolvedSchema, enum_def: &ResolvedEnum) -> Vec<String> {
    let mut types = Vec::new();
    for val in &enum_def.values {
        let vname = &val.name;
        if vname == "NONE" {
            continue;
        }
        let variant_bt = val.union_type.as_ref()
            .map(|t| t.base_type)
            .unwrap_or(BaseType::BASE_TYPE_NONE);
        match variant_bt {
            BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => {
                let idx = union_variant_type_index(val).unwrap();
                let obj_name = &schema.objects[idx].name;
                if !types.contains(obj_name) {
                    types.push(obj_name.clone());
                }
            }
            BaseType::BASE_TYPE_STRING => {
                if !types.contains(&"string".to_string()) {
                    types.push("string".to_string());
                }
            }
            _ => {}
        }
    }
    types
}

/// Generate `unionToXxx(type, accessor)` helper function.
fn gen_union_to_helper(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    enum_def: &ResolvedEnum,
    name: &str,
) {
    let variant_types = union_variant_types(schema, enum_def);
    if variant_types.is_empty() {
        return;
    }

    let union_type = variant_types.join("|");

    w.block(
        &format!(
            "export function unionTo{name}(\n  type: {name},\n  accessor: (obj:{union_type}) => {union_type}|null\n): {union_type}|null"
        ),
        |w| {
            w.block(&format!("switch({name}[type])"), |w| {
                w.line("case 'NONE': return null;");
                for val in &enum_def.values {
                    let vname = &val.name;
                    if vname == "NONE" {
                        continue;
                    }
                    let variant_bt = val.union_type.as_ref()
                        .map(|t| t.base_type)
                        .unwrap_or(BaseType::BASE_TYPE_NONE);
                    match variant_bt {
                        BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => {
                            let idx = union_variant_type_index(val).unwrap();
                            let obj_name = &schema.objects[idx].name;
                            w.line(&format!(
                                "case '{vname}': return accessor(new {obj_name}())! as {obj_name};"
                            ));
                        }
                        BaseType::BASE_TYPE_STRING => {
                            w.line(&format!(
                                "case '{vname}': return accessor('') as string;"
                            ));
                        }
                        _ => {}
                    }
                }
                w.line("default: return null;");
            });
        },
    );
}

/// Generate `unionListToXxx(type, accessor, index)` helper function.
fn gen_union_list_to_helper(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    enum_def: &ResolvedEnum,
    name: &str,
) {
    let variant_types = union_variant_types(schema, enum_def);
    if variant_types.is_empty() {
        return;
    }

    let union_type = variant_types.join("|");

    w.block(
        &format!(
            "export function unionListTo{name}(\n  type: {name},\n  accessor: (index: number, obj:{union_type}) => {union_type}|null,\n  index: number\n): {union_type}|null"
        ),
        |w| {
            w.block(&format!("switch({name}[type])"), |w| {
                w.line("case 'NONE': return null;");
                for val in &enum_def.values {
                    let vname = &val.name;
                    if vname == "NONE" {
                        continue;
                    }
                    let variant_bt = val.union_type.as_ref()
                        .map(|t| t.base_type)
                        .unwrap_or(BaseType::BASE_TYPE_NONE);
                    match variant_bt {
                        BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => {
                            let idx = union_variant_type_index(val).unwrap();
                            let obj_name = &schema.objects[idx].name;
                            w.line(&format!(
                                "case '{vname}': return accessor(index, new {obj_name}())! as {obj_name};"
                            ));
                        }
                        BaseType::BASE_TYPE_STRING => {
                            w.line(&format!(
                                "case '{vname}': return accessor(index, '') as string;"
                            ));
                        }
                        _ => {}
                    }
                }
                w.line("default: return null;");
            });
        },
    );
}

/// Generate the Object API union T class.
fn gen_union_object_api(w: &mut CodeWriter, schema: &ResolvedSchema, index: usize) {
    let enum_def = &schema.enums[index];
    let name = &enum_def.name;
    let t_name = format!("{name}T");

    // Collect variant info: (variant_name, type_name, base_type)
    let mut variants: Vec<(&str, String, BaseType)> = Vec::new();
    for val in &enum_def.values {
        let vname = val.name.as_str();
        if vname == "NONE" {
            continue;
        }
        let variant_bt = val.union_type.as_ref()
            .map(|t| t.base_type)
            .unwrap_or(BaseType::BASE_TYPE_NONE);
        match variant_bt {
            BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => {
                let idx = union_variant_type_index(val).unwrap();
                let obj_name = schema.objects[idx].name.clone();
                variants.push((vname, obj_name, variant_bt));
            }
            BaseType::BASE_TYPE_STRING => {
                variants.push((vname, "string".to_string(), variant_bt));
            }
            _ => {}
        }
    }

    // Generate the T class
    let mut variant_t_types: Vec<String> = Vec::new();
    for (_, tname, bt) in &variants {
        let t_type = if *bt == BaseType::BASE_TYPE_STRING {
            "string".to_string()
        } else {
            format!("{tname}T")
        };
        if !variant_t_types.contains(&t_type) {
            variant_t_types.push(t_type);
        }
    }
    let value_type = if variant_t_types.is_empty() {
        "null".to_string()
    } else {
        format!("{}|null", variant_t_types.join("|"))
    };

    w.block(
        &format!("export class {t_name} implements flatbuffers.IGeneratedObject"),
        |w| {
            w.line(&format!(
                "constructor(\n  public type: {name} = {name}.NONE,\n  public value: {value_type} = null\n) {{}}"
            ));
            w.blank();

            // pack method
            w.block(
                "pack(builder:flatbuffers.Builder): flatbuffers.Offset",
                |w| {
                    if variants.is_empty() {
                        w.line("return 0;");
                        return;
                    }
                    w.block("switch(this.type)", |w| {
                        w.line(&format!("case {name}.NONE: return 0;"));
                        for (vname, tname, bt) in &variants {
                            if *bt == BaseType::BASE_TYPE_STRING {
                                w.line(&format!(
                                    "case {name}.{vname}: return builder.createString(this.value as string);"
                                ));
                            } else {
                                w.line(&format!(
                                    "case {name}.{vname}: return (this.value as {tname}T).pack(builder);"
                                ));
                            }
                        }
                        w.line("default: return 0;");
                    });
                },
            );
        },
    );
}
