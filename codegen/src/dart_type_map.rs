use flatc_rs_schema::{BaseType, Documentation};

use super::code_writer::CodeWriter;

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
/// These return constructor names (e.g., "Float32Reader"), not method names.
pub fn bb_read_method(bt: BaseType) -> &'static str {
    match bt {
        BaseType::BASE_TYPE_BOOL | BaseType::BASE_TYPE_BYTE => "Int8Reader",
        BaseType::BASE_TYPE_U_BYTE | BaseType::BASE_TYPE_U_TYPE => "Uint8Reader",
        BaseType::BASE_TYPE_SHORT => "Int16Reader",
        BaseType::BASE_TYPE_U_SHORT => "Uint16Reader",
        BaseType::BASE_TYPE_INT => "Int32Reader",
        BaseType::BASE_TYPE_U_INT => "Uint32Reader",
        BaseType::BASE_TYPE_LONG => "Int64Reader",
        BaseType::BASE_TYPE_U_LONG => "Uint64Reader",
        BaseType::BASE_TYPE_FLOAT => "Float32Reader",
        BaseType::BASE_TYPE_DOUBLE => "Float64Reader",
        _ => panic!("no BB read method for BaseType: {bt:?}"),
    }
}

/// Returns the ByteBuffer write method name for a scalar BaseType.
pub fn bb_write_method(bt: BaseType) -> &'static str {
    match bt {
        BaseType::BASE_TYPE_BOOL | BaseType::BASE_TYPE_BYTE => "putInt8",
        BaseType::BASE_TYPE_U_BYTE | BaseType::BASE_TYPE_U_TYPE => "putUint8",
        BaseType::BASE_TYPE_SHORT => "putInt16",
        BaseType::BASE_TYPE_U_SHORT => "putUint16",
        BaseType::BASE_TYPE_INT => "putInt32",
        BaseType::BASE_TYPE_U_INT => "putUint32",
        BaseType::BASE_TYPE_LONG => "putInt64",
        BaseType::BASE_TYPE_U_LONG => "putUint64",
        BaseType::BASE_TYPE_FLOAT => "putFloat32",
        BaseType::BASE_TYPE_DOUBLE => "putFloat64",
        _ => panic!("no BB write method for BaseType: {bt:?}"),
    }
}

// Re-export shared functions from type_map
pub use super::type_map::{to_camel_case, to_pascal_case};

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

// Re-export shared functions from type_map
pub use super::type_map::build_fqn;

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
