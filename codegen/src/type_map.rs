use codegen_core::CodeWriter;
use flatc_rs_schema::resolved::{ResolvedEnum, ResolvedField, ResolvedObject, ResolvedSchema};
use flatc_rs_schema::{BaseType, Documentation};

// Re-export case conversion helpers from codegen_writers
pub use codegen_writers::to_pascal_case;

/// Convert a PascalCase or camelCase identifier to snake_case.
/// Handles consecutive uppercase letters: "HPMax" -> "hp_max" (not "hpmax")
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

/// Convert a name to camelCase (first letter lowercase).
/// First converts to snake_case, then capitalizes after underscores.
/// "my_field" -> "myField", "MyField" -> "myField", "SayHello" -> "sayHello"
pub fn to_camel_case(name: &str) -> String {
    let snake = to_snake_case(name);
    let mut result = String::with_capacity(snake.len());
    let mut capitalize_next = false;
    for (i, ch) in snake.chars().enumerate() {
        if ch == '_' {
            if i > 0 {
                capitalize_next = true;
            }
        } else if capitalize_next {
            result.push(ch.to_uppercase().next().unwrap());
            capitalize_next = false;
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

/// Convert a schema identifier using the official Rust generator's
/// `Case::kSnake` rules. FlatBuffers treats ASCII digits as uppercase for word
/// boundary purposes, so `Vec3` becomes `vec_3` and `xlqy3` becomes `xlqy_3`.
pub fn to_rust_snake_case(name: &str) -> String {
    let bytes = name.as_bytes();
    let mut result = String::with_capacity(name.len() + 4);

    for (index, byte) in bytes.iter().copied().enumerate() {
        if index == 0 {
            result.push(byte.to_ascii_lowercase() as char);
        } else if byte == b'_' {
            result.push('_');
        } else if !byte.is_ascii_lowercase() {
            let previous = bytes[index - 1];
            if previous.is_ascii_lowercase()
                || (previous.is_ascii_digit() && !byte.is_ascii_digit())
            {
                result.push('_');
            }
            result.push(byte.to_ascii_lowercase() as char);
        } else {
            result.push(byte as char);
        }
    }

    result
}

/// Convert a schema identifier using the official Rust generator's
/// `Case::kScreamingSnake` rules.
pub fn to_rust_upper_snake_case(name: &str) -> String {
    let snake = to_rust_snake_case(name);
    let bytes = snake.as_bytes();
    let mut result = String::with_capacity(snake.len() + 4);

    for (index, byte) in bytes.iter().copied().enumerate() {
        if index == 0 {
            result.push(byte.to_ascii_uppercase() as char);
        } else if byte == b'_' {
            result.push('_');
        } else if !byte.is_ascii_lowercase() {
            let previous = bytes[index - 1];
            if previous.is_ascii_lowercase()
                || (previous.is_ascii_digit() && !byte.is_ascii_digit())
            {
                result.push('_');
            }
            result.push(byte.to_ascii_uppercase() as char);
        } else {
            result.push(byte.to_ascii_uppercase() as char);
        }
    }

    result
}

/// Preserve a schema field name as the official Rust generator does.
///
/// Rust historically uses `Case::kKeep` for fields and only escapes keywords.
pub fn rust_field_name(name: &str) -> String {
    escape_keyword(name)
}

/// Generate the legacy Rust vtable offset suffix used by official `flatc`.
///
/// This is `Case::kAllUpper`, not screaming snake case: `item0` becomes
/// `ITEM0`, while an existing underscore in `session_revision` is preserved.
pub fn rust_field_offset_name(name: &str) -> String {
    escape_keyword(name).to_ascii_uppercase()
}

/// Emit schema documentation exactly as the official Rust generator does.
pub fn gen_rust_doc_comment(w: &mut CodeWriter, documentation: Option<&Documentation>) {
    if let Some(documentation) = documentation {
        for line in &documentation.lines {
            w.line(&format!("///{line}"));
        }
    }
}

/// Build FQN like "MyGame.Example.Monster".
pub fn build_fqn(obj: &ResolvedObject) -> String {
    let name = &obj.name;
    let ns = object_namespace(obj);
    if ns.is_empty() {
        name.to_string()
    } else {
        format!("{ns}.{name}")
    }
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
/// Paths are relative to the current generated namespace module, matching the
/// official Rust generator's explicit `super::` qualification.
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

    let mut path = vec!["super".to_string(); current_parts.len() - common_len];
    path.extend(
        target_parts[common_len..]
            .iter()
            .map(|p| to_rust_snake_case(p)),
    );

    if path.is_empty() {
        type_name.to_string()
    } else {
        format!("{}::{type_name}", path.join("::"))
    }
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
    fn official_rust_snake_case_conversions() {
        assert_eq!(to_rust_snake_case("xlqy3"), "xlqy_3");
        assert_eq!(to_rust_snake_case("Vec3"), "vec_3");
        assert_eq!(to_rust_snake_case("EntityKind"), "entity_kind");
        assert_eq!(to_rust_snake_case("HPMax"), "hpmax");
        assert_eq!(to_rust_snake_case("URL2Value"), "url2_value");
        assert_eq!(to_rust_snake_case("already_snake"), "already_snake");
        assert_eq!(to_rust_upper_snake_case("EntityKind"), "ENTITY_KIND");
        assert_eq!(to_rust_upper_snake_case("C2sMsgType"), "C_2S_MSG_TYPE");
        assert_eq!(to_rust_upper_snake_case("AnyS2c"), "ANY_S_2C");
        assert_eq!(rust_field_name("item0"), "item0");
        assert_eq!(rust_field_name("type"), "type_");
        assert_eq!(rust_field_offset_name("item0"), "ITEM0");
        assert_eq!(
            rust_field_offset_name("session_revision"),
            "SESSION_REVISION"
        );
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
            "super::items::Item"
        );
    }

    #[test]
    fn qualified_name_ancestor_namespace() {
        assert_eq!(qualified_name("Game.Player", "Game", "Root"), "super::Root");
        assert_eq!(
            qualified_name("Game.Player", "", "GlobalTable"),
            "super::super::GlobalTable"
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
        assert_eq!(
            qualified_name("A.B.C", "A.D.E", "Stuff"),
            "super::super::d::e::Stuff"
        );
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
