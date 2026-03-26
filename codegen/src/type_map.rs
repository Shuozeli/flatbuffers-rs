use flatc_rs_schema::resolved::{ResolvedEnum, ResolvedField, ResolvedObject, ResolvedSchema};
use flatc_rs_schema::BaseType;

/// Returns true if a field's type has an index, indicating it references
/// a user-defined type (enum, union, table, or struct) in the schema.
pub fn has_type_index(field: &ResolvedField) -> bool {
    field.type_.index.is_some()
}

/// Returns the Rust type name for a scalar BaseType.
///
/// # Panics
///
/// Panics if `bt` is not a scalar type. The analyzer guarantees that only
/// scalar types reach codegen call sites, so this is considered unreachable.
pub fn scalar_rust_type(bt: BaseType) -> &'static str {
    match bt {
        BaseType::BASE_TYPE_BOOL => "bool",
        BaseType::BASE_TYPE_BYTE => "i8",
        BaseType::BASE_TYPE_U_BYTE => "u8",
        BaseType::BASE_TYPE_SHORT => "i16",
        BaseType::BASE_TYPE_U_SHORT => "u16",
        BaseType::BASE_TYPE_INT => "i32",
        BaseType::BASE_TYPE_U_INT => "u32",
        BaseType::BASE_TYPE_LONG => "i64",
        BaseType::BASE_TYPE_U_LONG => "u64",
        BaseType::BASE_TYPE_FLOAT => "f32",
        BaseType::BASE_TYPE_DOUBLE => "f64",
        BaseType::BASE_TYPE_U_TYPE => "u8",
        _ => panic!("not a scalar BaseType: {bt:?}"),
    }
}

/// Returns true if the BaseType is a scalar (including bool).
pub fn is_scalar(bt: BaseType) -> bool {
    bt.is_scalar()
}

/// Returns true if the BaseType is a floating-point type.
pub fn is_float(bt: BaseType) -> bool {
    matches!(bt, BaseType::BASE_TYPE_FLOAT | BaseType::BASE_TYPE_DOUBLE)
}

/// Convert a PascalCase or camelCase identifier to snake_case.
pub fn to_snake_case(name: &str) -> String {
    let mut result = String::with_capacity(name.len() + 4);
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                // Don't insert underscore between consecutive uppercase letters
                // e.g., "HPMax" -> "hp_max" not "h_p_max"
                let prev = name.as_bytes()[i - 1];
                if prev.is_ascii_lowercase() || prev.is_ascii_digit() {
                    result.push('_');
                } else if prev != b'_' && i + 1 < name.len() {
                    let next = name.as_bytes()[i + 1];
                    if next.is_ascii_lowercase() {
                        result.push('_');
                    }
                }
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Convert a name to UPPER_SNAKE_CASE.
pub fn to_upper_snake_case(name: &str) -> String {
    to_snake_case(name).to_uppercase()
}

/// Format a default integer value for a given BaseType.
pub fn format_default_integer(value: i64, bt: BaseType) -> String {
    match bt {
        BaseType::BASE_TYPE_BOOL => {
            if value != 0 {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        _ => format!("{value}"),
    }
}

/// Format a default float value for a given BaseType.
pub fn format_default_real(value: f64, bt: BaseType) -> String {
    let type_suffix = match bt {
        BaseType::BASE_TYPE_FLOAT => "f32",
        BaseType::BASE_TYPE_DOUBLE => "f64",
        _ => "f64",
    };

    if value.is_nan() {
        return format!("{type_suffix}::NAN");
    }
    if value == f64::INFINITY {
        return format!("{type_suffix}::INFINITY");
    }
    if value == f64::NEG_INFINITY {
        return format!("{type_suffix}::NEG_INFINITY");
    }

    let s = if value == value.floor() {
        format!("{value:.1}")
    } else {
        format!("{value}")
    };
    // C++ flatc does not add type suffix - Rust infers type from context
    s
}

/// Sanitize a union variant name for use as a Rust enum constant.
/// Converts FQN dots to underscores: "MyGame.Example2.Monster" -> "MyGame_Example2_Monster"
pub fn sanitize_union_const_name(name: &str) -> String {
    name.replace('.', "_")
}

/// Convert a union variant FQN to PascalCase for Object API T enum variants.
/// Removes dots: "MyGame.Example2.Monster" -> "MyGameExample2Monster"
pub fn fqn_to_pascal(name: &str) -> String {
    name.replace('.', "")
}

/// Returns true if the given identifier is a Rust keyword that needs escaping.
pub fn is_rust_keyword(name: &str) -> bool {
    matches!(
        name,
        // Strict keywords
        "as" | "async" | "await" | "break" | "const" | "continue" | "crate" | "dyn"
        | "else" | "enum" | "extern" | "false" | "fn" | "for" | "if" | "impl" | "in"
        | "let" | "loop" | "match" | "mod" | "move" | "mut" | "pub" | "ref" | "return"
        | "self" | "Self" | "static" | "struct" | "super" | "trait" | "true" | "type"
        | "unsafe" | "use" | "where" | "while"
        // Reserved keywords
        | "abstract" | "become" | "box" | "do" | "final" | "macro" | "override"
        | "priv" | "try" | "typeof" | "unsized" | "virtual" | "yield"
    )
}

/// Escape a Rust keyword by appending `_`. Non-keywords are returned as-is.
pub fn escape_keyword(name: &str) -> String {
    if is_rust_keyword(name) {
        format!("{name}_")
    } else {
        name.to_string()
    }
}

/// Compute the qualified Rust module path for a type in `target_ns` when
/// referenced from `current_ns`. Both use dot-separated format (e.g., "Game.Items").
///
/// With the `use super::*;` chain in each generated module, all ancestor
/// namespace items are visible. So we only need to add a module path prefix
/// when the target namespace diverges from the current one (i.e., is not
/// an ancestor of the current namespace).
pub fn qualified_name(current_ns: &str, target_ns: &str, type_name: &str) -> String {
    if current_ns == target_ns {
        return type_name.to_string();
    }

    let current_parts: Vec<&str> = if current_ns.is_empty() {
        vec![]
    } else {
        current_ns.split('.').collect()
    };
    let target_parts: Vec<&str> = if target_ns.is_empty() {
        vec![]
    } else {
        target_ns.split('.').collect()
    };

    // Find length of common prefix
    let common_len = current_parts
        .iter()
        .zip(target_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();

    // If target is a prefix of (or equal to) current, then the target's items
    // are visible via the `use super::*;` chain -- no qualification needed.
    if common_len == target_parts.len() {
        return type_name.to_string();
    }

    // Otherwise, build the module path from the divergence point.
    let suffix: Vec<String> = target_parts[common_len..]
        .iter()
        .map(|p| to_snake_case(p))
        .collect();

    format!("{}::{}", suffix.join("::"), type_name)
}

/// Extract the dot-separated namespace string from a resolved object.
pub fn object_namespace(obj: &ResolvedObject) -> &str {
    obj.namespace
        .as_ref()
        .and_then(|n| n.namespace.as_deref())
        .unwrap_or("")
}

/// Extract the dot-separated namespace string from a resolved enum.
pub fn enum_namespace(e: &ResolvedEnum) -> &str {
    e.namespace
        .as_ref()
        .and_then(|n| n.namespace.as_deref())
        .unwrap_or("")
}

/// Resolve a qualified object (table/struct) name relative to the current namespace.
pub fn resolve_object_name(schema: &ResolvedSchema, current_ns: &str, obj_idx: usize) -> String {
    let obj = &schema.objects[obj_idx];
    let name = &obj.name;
    let target_ns = object_namespace(obj);
    qualified_name(current_ns, target_ns, name)
}

/// Check if an enum at a given index is a bitflags enum.
pub fn is_bitflags_enum(schema: &ResolvedSchema, enum_idx: usize) -> bool {
    let e = &schema.enums[enum_idx];
    e.attributes
        .as_ref()
        .is_some_and(|attrs| attrs.has("bit_flags"))
}

/// Resolve a qualified enum name relative to the current namespace.
pub fn resolve_enum_name(schema: &ResolvedSchema, current_ns: &str, enum_idx: usize) -> String {
    let e = &schema.enums[enum_idx];
    let name = &e.name;
    let target_ns = enum_namespace(e);
    qualified_name(current_ns, target_ns, name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case_conversions() {
        assert_eq!(to_snake_case("Monster"), "monster");
        assert_eq!(to_snake_case("myField"), "my_field");
        assert_eq!(to_snake_case("HPMax"), "hp_max");
        assert_eq!(to_snake_case("testURL"), "test_url");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
        assert_eq!(to_snake_case("Vec3"), "vec3");
    }

    #[test]
    fn upper_snake_case() {
        assert_eq!(to_upper_snake_case("Monster"), "MONSTER");
        assert_eq!(to_upper_snake_case("myField"), "MY_FIELD");
    }

    #[test]
    fn scalar_types() {
        assert_eq!(scalar_rust_type(BaseType::BASE_TYPE_INT), "i32");
        assert_eq!(scalar_rust_type(BaseType::BASE_TYPE_BOOL), "bool");
        assert_eq!(scalar_rust_type(BaseType::BASE_TYPE_FLOAT), "f32");
    }

    #[test]
    fn qualified_name_same_namespace() {
        assert_eq!(qualified_name("Game.Items", "Game.Items", "Item"), "Item");
        assert_eq!(qualified_name("", "", "Root"), "Root");
    }

    #[test]
    fn qualified_name_sibling_namespace() {
        assert_eq!(
            qualified_name("Game.Player", "Game.Items", "Item"),
            "items::Item"
        );
    }

    #[test]
    fn qualified_name_ancestor_namespace() {
        // Target is ancestor of current - visible via use super::*
        assert_eq!(qualified_name("Game.Player", "Game", "Root"), "Root");
        assert_eq!(
            qualified_name("Game.Player", "", "GlobalTable"),
            "GlobalTable"
        );
    }

    #[test]
    fn qualified_name_descendant_namespace() {
        // Target is deeper than current
        assert_eq!(qualified_name("Game", "Game.Items", "Item"), "items::Item");
        assert_eq!(
            qualified_name("", "Game.Items", "Item"),
            "game::items::Item"
        );
    }

    #[test]
    fn qualified_name_distant_namespace() {
        assert_eq!(qualified_name("A.B.C", "A.D.E", "Stuff"), "d::e::Stuff");
    }

    #[test]
    fn snake_case_with_underscores() {
        // Should not produce double underscores when input already has underscores
        assert_eq!(
            to_snake_case("MyGame_Example2_Monster"),
            "my_game_example2_monster"
        );
    }

    #[test]
    fn sanitize_fqn_names() {
        assert_eq!(
            sanitize_union_const_name("MyGame.Example2.Monster"),
            "MyGame_Example2_Monster"
        );
        assert_eq!(sanitize_union_const_name("Monster"), "Monster");
        assert_eq!(
            fqn_to_pascal("MyGame.Example2.Monster"),
            "MyGameExample2Monster"
        );
        assert_eq!(fqn_to_pascal("Monster"), "Monster");
    }
}
