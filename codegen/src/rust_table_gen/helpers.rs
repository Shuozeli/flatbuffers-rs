use crate::field_type_index;
use crate::type_map::has_type_index;
use flatc_rs_schema::resolved::{ResolvedField, ResolvedSchema};
use flatc_rs_schema::BaseType;

use crate::type_map;
use crate::CodeGenError;

/// Returns the alignment/size in bytes of a scalar type.
/// Used to sort fields for optimal vtable packing (matching C++ flatc ordering).
///
/// Delegates to [`BaseType::scalar_byte_size()`] -- the canonical source.
pub(super) fn scalar_alignment_size(bt: BaseType) -> u32 {
    bt.scalar_byte_size() as u32
}

/// Get the Rust type string for a vector element.
pub(super) fn vector_element_type(
    schema: &ResolvedSchema,
    field: &ResolvedField,
    element_bt: BaseType,
    lifetime: &str,
    current_ns: &str,
) -> Result<String, CodeGenError> {
    match element_bt {
        bt if type_map::is_scalar(bt) => {
            // Check if vector of enum
            if has_type_index(field) {
                let enum_idx = field_type_index(field)?;
                if enum_idx < schema.enums.len() {
                    return Ok(type_map::resolve_enum_name(schema, current_ns, enum_idx));
                }
            }
            Ok(type_map::scalar_rust_type(bt).to_string())
        }
        BaseType::BASE_TYPE_STRING => {
            // C++ uses double space for &'b  str (builder lifetime) but single space for &'a str
            let space = if lifetime == "'b" { "  " } else { " " };
            Ok(format!(
                "::flatbuffers::ForwardsUOffset<&{lifetime}{space}str>"
            ))
        }
        BaseType::BASE_TYPE_TABLE => {
            let idx = field_type_index(field)?;
            let tname = type_map::resolve_object_name(schema, current_ns, idx);
            Ok(format!(
                "::flatbuffers::ForwardsUOffset<{tname}<{lifetime}>>"
            ))
        }
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field_type_index(field)?;
            Ok(type_map::resolve_object_name(schema, current_ns, idx))
        }
        _ => Err(CodeGenError::Internal(format!(
            "unhandled vector element type {element_bt:?}"
        ))),
    }
}

/// Get the type string for verifier field visitation.
pub(super) fn verifier_type_str(
    schema: &ResolvedSchema,
    field: &ResolvedField,
    current_ns: &str,
) -> Result<String, CodeGenError> {
    let bt = field.type_.base_type;

    match bt {
        bt if type_map::is_scalar(bt) => {
            if has_type_index(field) {
                let idx = field_type_index(field)?;
                Ok(type_map::resolve_enum_name(schema, current_ns, idx))
            } else {
                Ok(type_map::scalar_rust_type(bt).to_string())
            }
        }
        BaseType::BASE_TYPE_STRING => Ok("::flatbuffers::ForwardsUOffset<&str>".to_string()),
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field_type_index(field)?;
            Ok(type_map::resolve_object_name(schema, current_ns, idx))
        }
        BaseType::BASE_TYPE_TABLE => {
            let idx = field_type_index(field)?;
            let tname = type_map::resolve_object_name(schema, current_ns, idx);
            Ok(format!("::flatbuffers::ForwardsUOffset<{tname}>"))
        }
        BaseType::BASE_TYPE_VECTOR => {
            let element_bt = field.type_.element_type_or_none();
            let inner = vector_element_type(schema, field, element_bt, "'_", current_ns)?;
            Ok(format!(
                "::flatbuffers::ForwardsUOffset<::flatbuffers::Vector<'_, {inner}>>"
            ))
        }
        BaseType::BASE_TYPE_UNION => {
            Ok("::flatbuffers::ForwardsUOffset<::flatbuffers::Table<'_>>".to_string())
        }
        _ => Ok("u8".to_string()),
    }
}

/// Get the scalar default value as a Rust expression.
pub(super) fn scalar_default(field: &ResolvedField, bt: BaseType) -> String {
    if let Some(ref ds) = field.default_string {
        // Enum default handled elsewhere
        return ds.clone();
    }

    if type_map::is_float(bt) {
        let val = field.default_real.unwrap_or(0.0);
        type_map::format_default_real(val, bt)
    } else if bt == BaseType::BASE_TYPE_BOOL {
        let val = field.default_integer.unwrap_or(0);
        type_map::format_default_integer(val, bt)
    } else {
        let val = field.default_integer.unwrap_or(0);
        format!("{val}")
    }
}

/// Get the Rust type for a scalar builder parameter.
/// Returns (type_str, use_push_slot_with_default).
pub(super) fn scalar_builder_type(
    schema: &ResolvedSchema,
    field: &ResolvedField,
    bt: BaseType,
    current_ns: &str,
) -> Result<(String, bool), CodeGenError> {
    let is_optional = field.is_optional;

    if has_type_index(field) {
        let idx = field_type_index(field)?;
        let enum_name = type_map::resolve_enum_name(schema, current_ns, idx);
        Ok((enum_name, !is_optional))
    } else {
        Ok((type_map::scalar_rust_type(bt).to_string(), !is_optional))
    }
}

/// Get the default value string for a scalar builder push_slot call.
pub(super) fn scalar_builder_default(
    schema: &ResolvedSchema,
    field: &ResolvedField,
    bt: BaseType,
    current_ns: &str,
) -> Result<String, CodeGenError> {
    if has_type_index(field) {
        let idx = field_type_index(field)?;
        let enum_name = type_map::resolve_enum_name(schema, current_ns, idx);
        let is_bitflags = type_map::is_bitflags_enum(schema, idx);
        if let Some(ref ds) = field.default_string {
            Ok(format!("{enum_name}::{ds}"))
        } else {
            let dv = field.default_integer.unwrap_or(0);
            if is_bitflags {
                Ok(format!("{enum_name}::from_bits_retain({dv})"))
            } else {
                Ok(format!("{enum_name}({dv})"))
            }
        }
    } else {
        Ok(scalar_default(field, bt))
    }
}

/// Get the Rust type for an Args struct field.
pub(super) fn args_field_type(
    schema: &ResolvedSchema,
    field: &ResolvedField,
    current_ns: &str,
) -> Result<String, CodeGenError> {
    let bt = field.type_.base_type;
    let is_optional = field.is_optional;

    match bt {
        bt if type_map::is_scalar(bt) => {
            let base = if has_type_index(field) {
                let idx = field_type_index(field)?;
                type_map::resolve_enum_name(schema, current_ns, idx)
            } else {
                type_map::scalar_rust_type(bt).to_string()
            };
            if is_optional {
                Ok(format!("Option<{base}>"))
            } else {
                Ok(base)
            }
        }
        BaseType::BASE_TYPE_STRING => Ok("Option<::flatbuffers::WIPOffset<&'a str>>".to_string()),
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field_type_index(field)?;
            let sname = type_map::resolve_object_name(schema, current_ns, idx);
            Ok(format!("Option<&'a {sname}>"))
        }
        BaseType::BASE_TYPE_TABLE => {
            let idx = field_type_index(field)?;
            let tname = type_map::resolve_object_name(schema, current_ns, idx);
            Ok(format!("Option<::flatbuffers::WIPOffset<{tname}<'a>>>"))
        }
        BaseType::BASE_TYPE_VECTOR => {
            let element_bt = field.type_.element_type_or_none();
            let inner = vector_element_type(schema, field, element_bt, "'a", current_ns)?;
            Ok(format!(
                "Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, {inner}>>>"
            ))
        }
        BaseType::BASE_TYPE_UNION => {
            Ok("Option<::flatbuffers::WIPOffset<::flatbuffers::UnionWIPOffset>>".to_string())
        }
        _ => Ok("u8".to_string()),
    }
}

/// Get the default value for an Args struct field.
pub(super) fn args_field_default(
    schema: &ResolvedSchema,
    field: &ResolvedField,
    current_ns: &str,
) -> Result<String, CodeGenError> {
    let bt = field.type_.base_type;

    match bt {
        bt if type_map::is_scalar(bt) => {
            if field.is_optional {
                Ok("None".to_string())
            } else {
                scalar_builder_default(schema, field, bt, current_ns)
            }
        }
        _ => Ok("None".to_string()),
    }
}

/// Extract the `nested_flatbuffer` attribute value from a field, if present.
/// The value is the type name of the nested table (e.g., "Monster").
pub(super) fn get_nested_flatbuffer_attr(field: &ResolvedField) -> Option<String> {
    field
        .attributes
        .as_ref()
        .and_then(|attrs| attrs.get("nested_flatbuffer"))
        .map(|v| {
            // Strip surrounding quotes if present (parser preserves them)
            v.trim_matches('"').to_string()
        })
}

/// Find a table in the schema by its short name (not FQN).
pub(super) fn find_table_by_name(schema: &ResolvedSchema, name: &str) -> Option<usize> {
    schema
        .objects
        .iter()
        .position(|obj| !obj.is_struct && obj.name == name)
}

/// Check if a field has the `key` attribute.
pub(crate) fn has_key_attribute(field: &ResolvedField) -> bool {
    field
        .attributes
        .as_ref()
        .is_some_and(|attrs| attrs.has("key"))
}

/// Check if a field's type is a union.
pub(super) fn is_union_field(field: &ResolvedField) -> bool {
    field.type_.base_type == BaseType::BASE_TYPE_UNION
}

/// Check if a field is a union type discriminator (the `_type` field for a union).
/// Returns false if the field type index cannot be resolved (should not happen
/// after analyzer validation).
pub(super) fn is_union_type_field(schema: &ResolvedSchema, field: &ResolvedField) -> bool {
    let bt = field.type_.base_type;
    if !type_map::is_scalar(bt) {
        return false;
    }
    if !has_type_index(field) {
        return false;
    }
    let idx = match field_type_index(field) {
        Ok(idx) => idx,
        Err(_) => return false,
    };
    if idx >= schema.enums.len() {
        return false;
    }
    schema.enums[idx].is_union
}

/// Check if a field is required (explicitly or implicitly as a string key).
pub(super) fn is_field_required(field: &ResolvedField) -> bool {
    if field.is_required {
        return true;
    }
    // String key fields are implicitly required (C++ flatc behavior)
    if field.is_key {
        let bt = field.type_.base_type;
        if bt == BaseType::BASE_TYPE_STRING {
            return true;
        }
    }
    false
}
