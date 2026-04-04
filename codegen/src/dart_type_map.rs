use flatc_rs_schema::resolved::ResolvedObject;
use flatc_rs_schema::{BaseType, Documentation};

use super::code_writer::CodeWriter;
use super::type_map;

/// Returns the Dart type name for a scalar BaseType.
///
/// # Panics
///
/// Panics if `bt` is not a scalar type.
pub fn scalar_dart_type(bt: BaseType) -> &'static str {
    match bt {
        BaseType::BASE_TYPE_BOOL => "bool",
        BaseType::BASE_TYPE_BYTE => "int",
        BaseType::BASE_TYPE_U_BYTE => "int",
        BaseType::BASE_TYPE_SHORT => "int",
        BaseType::BASE_TYPE_U_SHORT => "int",
        BaseType::BASE_TYPE_INT => "int",
        BaseType::BASE_TYPE_U_INT => "int",
        BaseType::BASE_TYPE_FLOAT => "double",
        BaseType::BASE_TYPE_DOUBLE => "double",
        BaseType::BASE_TYPE_LONG => "int",
        BaseType::BASE_TYPE_U_LONG => "int",
        BaseType::BASE_TYPE_U_TYPE => "int",
        _ => panic!("not a scalar BaseType for Dart: {bt:?}"),
    }
}

/// Returns the ByteBuffer read method name for a scalar BaseType.
pub fn bb_read_method(bt: BaseType) -> &'static str {
    match bt {
        BaseType::BASE_TYPE_BOOL | BaseType::BASE_TYPE_BYTE => "readInt8",
        BaseType::BASE_TYPE_U_BYTE | BaseType::BASE_TYPE_U_TYPE => "readUint8",
        BaseType::BASE_TYPE_SHORT => "readInt16",
        BaseType::BASE_TYPE_U_SHORT => "readUint16",
        BaseType::BASE_TYPE_INT => "readInt32",
        BaseType::BASE_TYPE_U_INT => "readUint32",
        BaseType::BASE_TYPE_LONG => "readInt64",
        BaseType::BASE_TYPE_U_LONG => "readUint64",
        BaseType::BASE_TYPE_FLOAT => "readFloat32",
        BaseType::BASE_TYPE_DOUBLE => "readFloat64",
        _ => panic!("no BB read method for BaseType: {bt:?}"),
    }
}

/// Returns the ByteBuffer write method name for a scalar BaseType.
pub fn bb_write_method(bt: BaseType) -> &'static str {
    match bt {
        BaseType::BASE_TYPE_BOOL | BaseType::BASE_TYPE_BYTE => "writeInt8",
        BaseType::BASE_TYPE_U_BYTE | BaseType::BASE_TYPE_U_TYPE => "writeUint8",
        BaseType::BASE_TYPE_SHORT => "writeInt16",
        BaseType::BASE_TYPE_U_SHORT => "writeUint16",
        BaseType::BASE_TYPE_INT => "writeInt32",
        BaseType::BASE_TYPE_U_INT => "writeInt32",
        BaseType::BASE_TYPE_LONG => "writeInt64",
        BaseType::BASE_TYPE_U_LONG => "writeInt64",
        BaseType::BASE_TYPE_FLOAT => "writeFloat32",
        BaseType::BASE_TYPE_DOUBLE => "writeFloat64",
        _ => panic!("no BB write method for BaseType: {bt:?}"),
    }
}

/// Convert a name to camelCase (first letter lowercase).
pub fn to_camel_case(name: &str) -> String {
    let snake = type_map::to_snake_case(name);
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

/// Convert a name to PascalCase (first letter uppercase).
pub fn to_pascal_case(name: &str) -> String {
    let camel = to_camel_case(name);
    let mut chars = camel.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let mut s = c.to_uppercase().collect::<String>();
            s.extend(chars);
            s
        }
    }
}

/// Returns true if the given identifier is a Dart keyword that needs escaping.
pub fn is_dart_keyword(name: &str) -> bool {
    matches!(
        name,
        "abstract"
            | "as"
            | "assert"
            | "async"
            | "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "covariant"
            | "default"
            | "deferred"
            | "do"
            | "dynamic"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "extension"
            | "external"
            | "factory"
            | "false"
            | "final"
            | "finally"
            | "for"
            | "Function"
            | "get"
            | "hide"
            | "if"
            | "implements"
            | "import"
            | "in"
            | "interface"
            | "is"
            | "late"
            | "library"
            | "mixin"
            | "new"
            | "null"
            | "on"
            | "operator"
            | "part"
            | "required"
            | "rethrow"
            | "return"
            | "set"
            | "show"
            | "static"
            | "super"
            | "switch"
            | "sync"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typedef"
            | "var"
            | "void"
            | "when"
            | "while"
            | "with"
            | "yield"
    )
}

/// Escape a Dart keyword by appending `_`. Non-keywords returned as-is.
pub fn escape_dart_keyword(name: &str) -> String {
    if is_dart_keyword(name) {
        format!("{name}_")
    } else {
        name.to_string()
    }
}

/// Format a default value for a scalar BaseType in Dart.
pub fn format_default(value: i64, bt: BaseType) -> String {
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

/// Format a default float value for Dart.
pub fn format_default_real(value: f64, _bt: BaseType) -> String {
    if value.is_nan() {
        "double.nan".to_string()
    } else if value == f64::INFINITY {
        "double.infinity".to_string()
    } else if value == f64::NEG_INFINITY {
        "double.negativeInfinity".to_string()
    } else if value == value.floor() {
        format!("{value}.0")
    } else {
        format!("{value}")
    }
}

/// Emit a documentation comment block if documentation is present.
pub fn gen_doc_comment(w: &mut CodeWriter, doc: Option<&Documentation>) {
    let doc = match doc {
        Some(d) if !d.lines.is_empty() => d,
        _ => return,
    };
    for line in &doc.lines {
        if line.is_empty() {
            w.line("///");
        } else {
            w.line(&format!("/// {line}"));
        }
    }
}

/// Build FQN like "NamespaceA.NamespaceB.TypeName".
pub fn build_fqn(obj: &ResolvedObject) -> String {
    let name = &obj.name;
    let ns = type_map::object_namespace(obj);
    if ns.is_empty() {
        name.to_string()
    } else {
        format!("{ns}.{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dart_types() {
        assert_eq!(scalar_dart_type(BaseType::BASE_TYPE_BOOL), "bool");
        assert_eq!(scalar_dart_type(BaseType::BASE_TYPE_INT), "int");
        assert_eq!(scalar_dart_type(BaseType::BASE_TYPE_FLOAT), "double");
        assert_eq!(scalar_dart_type(BaseType::BASE_TYPE_LONG), "int");
    }

    #[test]
    fn camel_case_conversions() {
        assert_eq!(to_camel_case("pos"), "pos");
        assert_eq!(to_camel_case("mana"), "mana");
        assert_eq!(to_camel_case("Monster"), "monster");
        assert_eq!(to_camel_case("my_field"), "myField");
        assert_eq!(to_camel_case("test_type"), "testType");
        assert_eq!(to_camel_case("HPMax"), "hpMax");
    }

    #[test]
    fn pascal_case_conversions() {
        assert_eq!(to_pascal_case("pos"), "Pos");
        assert_eq!(to_pascal_case("mana"), "Mana");
        assert_eq!(to_pascal_case("Monster"), "Monster");
        assert_eq!(to_pascal_case("my_field"), "MyField");
    }

    #[test]
    fn keyword_escaping() {
        assert!(is_dart_keyword("class"));
        assert!(is_dart_keyword("return"));
        assert!(!is_dart_keyword("monster"));
        assert_eq!(escape_dart_keyword("class"), "class_");
        assert_eq!(escape_dart_keyword("monster"), "monster");
    }
}
