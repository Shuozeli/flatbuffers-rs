use flatc_rs_schema::{self as schema, BaseType};

use super::code_writer::CodeWriter;
use super::type_map;

/// Generate TypeScript code for the enum at `schema.enums[index]`.
pub fn generate(w: &mut CodeWriter, schema: &schema::Schema, index: usize, gen_object_api: bool) {
    let enum_def = &schema.enums[index];
    let is_union = enum_def.is_union == Some(true);

    if is_union {
        generate_union_enum(w, schema, index, gen_object_api);
    } else {
        generate_regular_enum(w, enum_def);
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

/// Generate a regular TypeScript enum.
fn generate_regular_enum(w: &mut CodeWriter, enum_def: &schema::Enum) {
    let name = enum_def.name.as_deref().unwrap_or("");

    gen_doc_comment(w, enum_def.documentation.as_ref());
    w.block(&format!("export enum {name}"), |w| {
        for (i, val) in enum_def.values.iter().enumerate() {
            let vname = val.name.as_deref().unwrap_or("");
            let vval = val.value.unwrap_or(0);
            let comma = if i < enum_def.values.len() - 1 {
                ","
            } else {
                ""
            };
            gen_doc_comment(w, val.documentation.as_ref());
            w.line(&format!("{vname} = {vval}{comma}"));
        }
    });
}

/// Generate a union enum plus helper functions.
fn generate_union_enum(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    index: usize,
    gen_object_api: bool,
) {
    let enum_def = &schema.enums[index];
    let name = enum_def.name.as_deref().unwrap_or("");

    // Generate the union discriminator enum
    gen_doc_comment(w, enum_def.documentation.as_ref());
    w.block(&format!("export enum {name}"), |w| {
        for (i, val) in enum_def.values.iter().enumerate() {
            let vname = val.name.as_deref().unwrap_or("");
            let vval = val.value.unwrap_or(0);
            let comma = if i < enum_def.values.len() - 1 {
                ","
            } else {
                ""
            };
            gen_doc_comment(w, val.documentation.as_ref());
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
fn union_variant_types(schema: &schema::Schema, enum_def: &schema::Enum) -> Vec<String> {
    let mut types = Vec::new();
    for val in &enum_def.values {
        let vname = val.name.as_deref().unwrap_or("");
        if vname == "NONE" {
            continue;
        }
        let variant_bt = type_map::get_base_type(val.union_type.as_ref());
        match variant_bt {
            BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => {
                let idx = val
                    .union_type
                    .as_ref()
                    .and_then(|t| t.index)
                    .unwrap_or(0) as usize;
                let obj_name = schema.objects[idx].name.as_deref().unwrap_or("");
                if !types.contains(&obj_name.to_string()) {
                    types.push(obj_name.to_string());
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

/// Check if any union variant is a string type.
fn union_has_string_variant(enum_def: &schema::Enum) -> bool {
    enum_def.values.iter().any(|val| {
        let vname = val.name.as_deref().unwrap_or("");
        if vname == "NONE" {
            return false;
        }
        type_map::get_base_type(val.union_type.as_ref()) == BaseType::BASE_TYPE_STRING
    })
}

/// Get the TypeScript union type string for the Object API.
pub fn get_union_t_type(schema: &schema::Schema, enum_idx: usize) -> String {
    let enum_def = &schema.enums[enum_idx];
    let mut variant_t_types: Vec<String> = Vec::new();
    for val in &enum_def.values {
        let vname = val.name.as_deref().unwrap_or("");
        if vname == "NONE" {
            continue;
        }
        let variant_bt = type_map::get_base_type(val.union_type.as_ref());
        let t_type = match variant_bt {
            BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => {
                let idx = val
                    .union_type
                    .as_ref()
                    .and_then(|t| t.index)
                    .unwrap_or(0) as usize;
                let obj_name = schema.objects[idx].name.as_deref().unwrap_or("");
                format!("{obj_name}T")
            }
            BaseType::BASE_TYPE_STRING => "string".to_string(),
            _ => continue,
        };
        if !variant_t_types.contains(&t_type) {
            variant_t_types.push(t_type);
        }
    }
    if variant_t_types.is_empty() {
        "null".to_string()
    } else {
        format!("{}|null", variant_t_types.join("|"))
    }
}

/// Generate `unionToXxx(type, accessor)` helper function.
fn gen_union_to_helper(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    enum_def: &schema::Enum,
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
                    let vname = val.name.as_deref().unwrap_or("");
                    if vname == "NONE" {
                        continue;
                    }
                    let variant_bt = type_map::get_base_type(val.union_type.as_ref());
                    match variant_bt {
                        BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => {
                            let idx = val
                                .union_type
                                .as_ref()
                                .and_then(|t| t.index)
                                .unwrap_or(0) as usize;
                            let obj_name = schema.objects[idx].name.as_deref().unwrap_or("");
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
    schema: &schema::Schema,
    enum_def: &schema::Enum,
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
                    let vname = val.name.as_deref().unwrap_or("");
                    if vname == "NONE" {
                        continue;
                    }
                    let variant_bt = type_map::get_base_type(val.union_type.as_ref());
                    match variant_bt {
                        BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => {
                            let idx = val
                                .union_type
                                .as_ref()
                                .and_then(|t| t.index)
                                .unwrap_or(0) as usize;
                            let obj_name = schema.objects[idx].name.as_deref().unwrap_or("");
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
fn gen_union_object_api(w: &mut CodeWriter, schema: &schema::Schema, index: usize) {
    let enum_def = &schema.enums[index];
    let name = enum_def.name.as_deref().unwrap_or("");
    let t_name = format!("{name}T");

    // Collect variant info: (variant_name, type_name, base_type)
    let mut variants: Vec<(&str, String, BaseType)> = Vec::new();
    for val in &enum_def.values {
        let vname = val.name.as_deref().unwrap_or("");
        if vname == "NONE" {
            continue;
        }
        let variant_bt = type_map::get_base_type(val.union_type.as_ref());
        match variant_bt {
            BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => {
                let idx = val
                    .union_type
                    .as_ref()
                    .and_then(|t| t.index)
                    .unwrap_or(0) as usize;
                let obj_name = schema.objects[idx].name.as_deref().unwrap_or("");
                variants.push((vname, obj_name.to_string(), variant_bt));
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
