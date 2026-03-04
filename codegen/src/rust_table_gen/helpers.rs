use crate::field_type_index;
use crate::type_map::{get_base_type, get_element_type, has_enum_index};
use flatc_rs_schema::{self as schema, BaseType};

use crate::type_map;

/// Returns the alignment/size in bytes of a scalar type.
/// Used to sort fields for optimal vtable packing (matching C++ flatc ordering).
pub(super) fn scalar_alignment_size(bt: BaseType) -> u32 {
    match bt {
        BaseType::BASE_TYPE_BOOL
        | BaseType::BASE_TYPE_BYTE
        | BaseType::BASE_TYPE_U_BYTE
        | BaseType::BASE_TYPE_U_TYPE => 1,
        BaseType::BASE_TYPE_SHORT | BaseType::BASE_TYPE_U_SHORT => 2,
        BaseType::BASE_TYPE_INT | BaseType::BASE_TYPE_U_INT | BaseType::BASE_TYPE_FLOAT => 4,
        BaseType::BASE_TYPE_LONG | BaseType::BASE_TYPE_U_LONG | BaseType::BASE_TYPE_DOUBLE => 8,
        _ => 0,
    }
}

/// Get the Rust type string for a vector element.
pub(super) fn vector_element_type(
    schema: &schema::Schema,
    field: &schema::Field,
    element_bt: BaseType,
    lifetime: &str,
    current_ns: &str,
) -> String {
    match element_bt {
        bt if type_map::is_scalar(bt) => {
            // Check if vector of enum
            if has_enum_index(field) {
                let enum_idx = field_type_index(field);
                if enum_idx < schema.enums.len() {
                    return type_map::resolve_enum_name(schema, current_ns, enum_idx);
                }
            }
            type_map::scalar_rust_type(bt).to_string()
        }
        BaseType::BASE_TYPE_STRING => {
            // C++ uses double space for &'b  str (builder lifetime) but single space for &'a str
            let space = if lifetime == "'b" { "  " } else { " " };
            format!("::flatbuffers::ForwardsUOffset<&{lifetime}{space}str>")
        }
        BaseType::BASE_TYPE_TABLE => {
            let idx = field_type_index(field);
            let tname = type_map::resolve_object_name(schema, current_ns, idx);
            format!("::flatbuffers::ForwardsUOffset<{tname}<{lifetime}>>")
        }
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field_type_index(field);
            type_map::resolve_object_name(schema, current_ns, idx)
        }
        _ => {
            panic!("BUG: unhandled vector element type {element_bt:?} (schema should have been validated by analyzer)");
        }
    }
}

/// Get the type string for verifier field visitation.
pub(super) fn verifier_type_str(
    schema: &schema::Schema,
    field: &schema::Field,
    current_ns: &str,
) -> String {
    let bt = get_base_type(field.type_.as_ref());

    match bt {
        bt if type_map::is_scalar(bt) => {
            if has_enum_index(field) {
                let idx = field_type_index(field);
                type_map::resolve_enum_name(schema, current_ns, idx)
            } else {
                type_map::scalar_rust_type(bt).to_string()
            }
        }
        BaseType::BASE_TYPE_STRING => "::flatbuffers::ForwardsUOffset<&str>".to_string(),
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field_type_index(field);
            type_map::resolve_object_name(schema, current_ns, idx)
        }
        BaseType::BASE_TYPE_TABLE => {
            let idx = field_type_index(field);
            let tname = type_map::resolve_object_name(schema, current_ns, idx);
            format!("::flatbuffers::ForwardsUOffset<{tname}>")
        }
        BaseType::BASE_TYPE_VECTOR => {
            let element_bt = get_element_type(field.type_.as_ref());
            let inner = vector_element_type(schema, field, element_bt, "'_", current_ns);
            format!("::flatbuffers::ForwardsUOffset<::flatbuffers::Vector<'_, {inner}>>")
        }
        BaseType::BASE_TYPE_UNION => {
            "::flatbuffers::ForwardsUOffset<::flatbuffers::Table<'_>>".to_string()
        }
        _ => "u8".to_string(),
    }
}

/// Get the scalar default value as a Rust expression.
pub(super) fn scalar_default(field: &schema::Field, bt: BaseType) -> String {
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
    schema: &schema::Schema,
    field: &schema::Field,
    bt: BaseType,
    current_ns: &str,
) -> (String, bool) {
    let is_optional = field.is_optional == Some(true);

    if has_enum_index(field) {
        let idx = field_type_index(field);
        let enum_name = type_map::resolve_enum_name(schema, current_ns, idx);
        (enum_name, !is_optional)
    } else {
        (type_map::scalar_rust_type(bt).to_string(), !is_optional)
    }
}

/// Get the default value string for a scalar builder push_slot call.
pub(super) fn scalar_builder_default(
    schema: &schema::Schema,
    field: &schema::Field,
    bt: BaseType,
    current_ns: &str,
) -> String {
    if has_enum_index(field) {
        let idx = field_type_index(field);
        let enum_name = type_map::resolve_enum_name(schema, current_ns, idx);
        let is_bitflags = type_map::is_bitflags_enum(schema, idx);
        if let Some(ref ds) = field.default_string {
            format!("{enum_name}::{ds}")
        } else {
            let dv = field.default_integer.unwrap_or(0);
            if is_bitflags {
                format!("{enum_name}::from_bits_retain({dv})")
            } else {
                format!("{enum_name}({dv})")
            }
        }
    } else {
        scalar_default(field, bt)
    }
}

/// Get the Rust type for an Args struct field.
pub(super) fn args_field_type(
    schema: &schema::Schema,
    field: &schema::Field,
    current_ns: &str,
) -> String {
    let bt = get_base_type(field.type_.as_ref());
    let is_optional = field.is_optional == Some(true);

    match bt {
        bt if type_map::is_scalar(bt) => {
            let base = if has_enum_index(field) {
                let idx = field_type_index(field);
                type_map::resolve_enum_name(schema, current_ns, idx)
            } else {
                type_map::scalar_rust_type(bt).to_string()
            };
            if is_optional {
                format!("Option<{base}>")
            } else {
                base
            }
        }
        BaseType::BASE_TYPE_STRING => "Option<::flatbuffers::WIPOffset<&'a str>>".to_string(),
        BaseType::BASE_TYPE_STRUCT => {
            let idx = field_type_index(field);
            let sname = type_map::resolve_object_name(schema, current_ns, idx);
            format!("Option<&'a {sname}>")
        }
        BaseType::BASE_TYPE_TABLE => {
            let idx = field_type_index(field);
            let tname = type_map::resolve_object_name(schema, current_ns, idx);
            format!("Option<::flatbuffers::WIPOffset<{tname}<'a>>>")
        }
        BaseType::BASE_TYPE_VECTOR => {
            let element_bt = get_element_type(field.type_.as_ref());
            let inner = vector_element_type(schema, field, element_bt, "'a", current_ns);
            format!("Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, {inner}>>>")
        }
        BaseType::BASE_TYPE_UNION => {
            "Option<::flatbuffers::WIPOffset<::flatbuffers::UnionWIPOffset>>".to_string()
        }
        _ => "u8".to_string(),
    }
}

/// Get the default value for an Args struct field.
pub(super) fn args_field_default(
    schema: &schema::Schema,
    field: &schema::Field,
    current_ns: &str,
) -> String {
    let bt = get_base_type(field.type_.as_ref());

    match bt {
        bt if type_map::is_scalar(bt) => {
            if field.is_optional == Some(true) {
                "None".to_string()
            } else {
                scalar_builder_default(schema, field, bt, current_ns)
            }
        }
        _ => "None".to_string(),
    }
}

/// Extract the `nested_flatbuffer` attribute value from a field, if present.
/// The value is the type name of the nested table (e.g., "Monster").
pub(super) fn get_nested_flatbuffer_attr(field: &schema::Field) -> Option<String> {
    field.attributes.as_ref().and_then(|attrs| {
        attrs.entries.iter().find_map(|e| {
            if e.key.as_deref() == Some("nested_flatbuffer") {
                e.value.as_ref().map(|v| {
                    // Strip surrounding quotes if present (parser preserves them)
                    v.trim_matches('"').to_string()
                })
            } else {
                None
            }
        })
    })
}

/// Find a table in the schema by its short name (not FQN).
pub(super) fn find_table_by_name(schema: &schema::Schema, name: &str) -> Option<usize> {
    schema
        .objects
        .iter()
        .position(|obj| obj.is_struct != Some(true) && obj.name.as_deref() == Some(name))
}

/// Check if a field has the `key` attribute.
pub(super) fn has_key_attribute(field: &schema::Field) -> bool {
    field.attributes.as_ref().is_some_and(|attrs| {
        attrs
            .entries
            .iter()
            .any(|e| e.key.as_deref() == Some("key"))
    })
}

/// Check if a field's type is a union.
pub(super) fn is_union_field(field: &schema::Field) -> bool {
    get_base_type(field.type_.as_ref()) == BaseType::BASE_TYPE_UNION
}

/// Check if a field is a union type discriminator (the `_type` field for a union).
pub(super) fn is_union_type_field(schema: &schema::Schema, field: &schema::Field) -> bool {
    let bt = get_base_type(field.type_.as_ref());
    if !type_map::is_scalar(bt) {
        return false;
    }
    if !has_enum_index(field) {
        return false;
    }
    let idx = field_type_index(field);
    if idx >= schema.enums.len() {
        return false;
    }
    schema.enums[idx].is_union == Some(true)
}

/// Check if a field is required (explicitly or implicitly as a string key).
pub(super) fn is_field_required(field: &schema::Field) -> bool {
    if field.is_required == Some(true) {
        return true;
    }
    // String key fields are implicitly required (C++ flatc behavior)
    if field.is_key == Some(true) {
        let bt = get_base_type(field.type_.as_ref());
        if bt == BaseType::BASE_TYPE_STRING {
            return true;
        }
    }
    false
}
