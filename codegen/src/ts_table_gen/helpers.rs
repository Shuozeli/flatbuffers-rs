use flatc_rs_schema::resolved::{ResolvedField, ResolvedSchema};
use flatc_rs_schema::BaseType;

use crate::field_type_index;
use crate::ts_type_map;
use crate::type_map;
use crate::union_variant_type_index;

/// Get the TypeScript type for a scalar field, using enum name if applicable.
pub(super) fn scalar_field_ts_type(
    schema: &ResolvedSchema,
    field: &ResolvedField,
    bt: BaseType,
) -> String {
    if type_map::has_enum_index(field) {
        let enum_idx = field_type_index(field).unwrap();
        schema.enums[enum_idx].name.clone()
    } else {
        ts_type_map::scalar_ts_type(bt).to_string()
    }
}

/// Get the TypeScript default value for a scalar field.
pub(super) fn scalar_default_value(
    field: &ResolvedField,
    bt: BaseType,
    schema: &ResolvedSchema,
) -> String {
    if type_map::is_float(bt) {
        // Parser may store integer-valued float defaults (like 42.0) in default_integer
        let val = field
            .default_real
            .unwrap_or_else(|| field.default_integer.unwrap_or(0) as f64);
        ts_type_map::format_default_real_ts(val, bt)
    } else if type_map::has_enum_index(field) {
        // Enum default: check default_string first (e.g., "Blue"), format as EnumName.ValueName
        let enum_idx = field_type_index(field).unwrap();
        let enum_def = &schema.enums[enum_idx];
        let enum_name = &enum_def.name;

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

/// Get the parameter type for the create function.
pub(super) fn create_fn_param_type(
    schema: &ResolvedSchema,
    field: &ResolvedField,
    bt: BaseType,
) -> String {
    match bt {
        bt if type_map::is_scalar(bt) => scalar_field_ts_type(schema, field, bt),
        BaseType::BASE_TYPE_STRING
        | BaseType::BASE_TYPE_TABLE
        | BaseType::BASE_TYPE_VECTOR
        | BaseType::BASE_TYPE_STRUCT
        | BaseType::BASE_TYPE_UNION => "flatbuffers.Offset".to_string(),
        _ => "number".to_string(),
    }
}

/// Get TypeScript type and default value for a table's Object API field.
pub(super) fn object_api_field_type_and_default(
    schema: &ResolvedSchema,
    field: &ResolvedField,
) -> (String, String) {
    let bt = type_map::get_base_type(&field.type_);
    let is_optional = field.is_optional;

    match bt {
        bt if type_map::is_scalar(bt) => {
            let ts_type = ts_type_map::scalar_ts_type(bt);

            // Check for enum type
            let type_name = if type_map::has_enum_index(field) {
                let enum_idx = field_type_index(field).unwrap();
                schema.enums[enum_idx].name.clone()
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
            let idx = field_type_index(field).unwrap();
            let struct_name = &schema.objects[idx].name;
            (format!("{struct_name}T|null"), "null".to_string())
        }
        BaseType::BASE_TYPE_TABLE => {
            let idx = field_type_index(field).unwrap();
            let table_name = &schema.objects[idx].name;
            (format!("{table_name}T|null"), "null".to_string())
        }
        BaseType::BASE_TYPE_VECTOR => {
            let et = type_map::get_element_type(&field.type_);
            let elem_type = match et {
                et if type_map::is_scalar(et) => {
                    if type_map::has_enum_index(field) {
                        let enum_idx = field_type_index(field).unwrap();
                        schema.enums[enum_idx].name.clone()
                    } else {
                        ts_type_map::scalar_ts_type(et).to_string()
                    }
                }
                BaseType::BASE_TYPE_STRING => "string".to_string(),
                BaseType::BASE_TYPE_TABLE => {
                    let idx = field_type_index(field).unwrap();
                    let name = &schema.objects[idx].name;
                    format!("{name}T")
                }
                BaseType::BASE_TYPE_STRUCT => {
                    let idx = field_type_index(field).unwrap();
                    let name = &schema.objects[idx].name;
                    format!("{name}T")
                }
                _ => "unknown".to_string(),
            };

            (format!("({elem_type})[]"), "[]".to_string())
        }
        BaseType::BASE_TYPE_UNION => {
            // Union fields in the Object API: generate discriminated union type
            let idx = field.type_.index.unwrap_or(-1);
            if idx >= 0 && (idx as usize) < schema.enums.len() {
                let enum_def = &schema.enums[idx as usize];
                let variant_types: Vec<String> = enum_def
                    .values
                    .iter()
                    .filter(|v| v.name != "NONE")
                    .filter_map(|v| {
                        let vbt = v.union_type.as_ref()
                            .map(|t| t.base_type)
                            .unwrap_or(BaseType::BASE_TYPE_NONE);
                        match vbt {
                            BaseType::BASE_TYPE_TABLE => {
                                let vi = union_variant_type_index(v).unwrap();
                                let name = &schema.objects[vi].name;
                                Some(format!("{name}T"))
                            }
                            BaseType::BASE_TYPE_STRING => Some("string".to_string()),
                            BaseType::BASE_TYPE_STRUCT => {
                                let vi = union_variant_type_index(v).unwrap();
                                let name = &schema.objects[vi].name;
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
