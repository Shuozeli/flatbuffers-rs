use flatc_rs_schema::{BaseType, Documentation};

use super::code_writer::CodeWriter;

/// Returns the TypeScript type name for a scalar BaseType.
///
/// # Panics
///
/// Panics if `bt` is not a scalar type. The analyzer guarantees that only
/// scalar types reach codegen call sites, so this is considered unreachable.
pub fn scalar_ts_type(bt: BaseType) -> &'static str {
    match bt {
        BaseType::BASE_TYPE_BOOL => "boolean",
        BaseType::BASE_TYPE_BYTE
        | BaseType::BASE_TYPE_U_BYTE
        | BaseType::BASE_TYPE_SHORT
        | BaseType::BASE_TYPE_U_SHORT
        | BaseType::BASE_TYPE_INT
        | BaseType::BASE_TYPE_U_INT
        | BaseType::BASE_TYPE_FLOAT
        | BaseType::BASE_TYPE_DOUBLE => "number",
        BaseType::BASE_TYPE_LONG | BaseType::BASE_TYPE_U_LONG => "bigint",
        BaseType::BASE_TYPE_U_TYPE => "number",
        _ => panic!("not a scalar BaseType for TypeScript: {bt:?}"),
    }
}

/// Returns the ByteBuffer read method name for a scalar BaseType.
///
/// # Panics
/// Panics on non-scalar types (unreachable after analyzer validation).
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
///
/// # Panics
/// Panics on non-scalar types (unreachable after analyzer validation).
pub fn bb_write_method(bt: BaseType) -> &'static str {
    match bt {
        BaseType::BASE_TYPE_BOOL | BaseType::BASE_TYPE_BYTE => "writeInt8",
        BaseType::BASE_TYPE_U_BYTE | BaseType::BASE_TYPE_U_TYPE => "writeUint8",
        BaseType::BASE_TYPE_SHORT => "writeInt16",
        BaseType::BASE_TYPE_U_SHORT => "writeUint16",
        BaseType::BASE_TYPE_INT => "writeInt32",
        BaseType::BASE_TYPE_U_INT => "writeUint32",
        BaseType::BASE_TYPE_LONG => "writeInt64",
        BaseType::BASE_TYPE_U_LONG => "writeUint64",
        BaseType::BASE_TYPE_FLOAT => "writeFloat32",
        BaseType::BASE_TYPE_DOUBLE => "writeFloat64",
        _ => panic!("no BB write method for BaseType: {bt:?}"),
    }
}

/// Returns the Builder addField method name for a scalar BaseType (table fields).
///
/// # Panics
/// Panics on non-scalar types (unreachable after analyzer validation).
pub fn builder_add_field_method(bt: BaseType) -> &'static str {
    match bt {
        BaseType::BASE_TYPE_BOOL | BaseType::BASE_TYPE_BYTE | BaseType::BASE_TYPE_U_BYTE => {
            "addFieldInt8"
        }
        BaseType::BASE_TYPE_SHORT | BaseType::BASE_TYPE_U_SHORT | BaseType::BASE_TYPE_U_TYPE => {
            "addFieldInt16"
        }
        BaseType::BASE_TYPE_INT | BaseType::BASE_TYPE_U_INT => "addFieldInt32",
        BaseType::BASE_TYPE_LONG | BaseType::BASE_TYPE_U_LONG => "addFieldInt64",
        BaseType::BASE_TYPE_FLOAT => "addFieldFloat32",
        BaseType::BASE_TYPE_DOUBLE => "addFieldFloat64",
        _ => panic!("no builder addField method for BaseType: {bt:?}"),
    }
}

/// Returns the Builder inline write method name (for struct creation).
///
/// # Panics
/// Panics on non-scalar types (unreachable after analyzer validation).
pub fn builder_write_method(bt: BaseType) -> &'static str {
    match bt {
        BaseType::BASE_TYPE_BOOL | BaseType::BASE_TYPE_BYTE => "writeInt8",
        BaseType::BASE_TYPE_U_BYTE | BaseType::BASE_TYPE_U_TYPE => "writeInt8",
        BaseType::BASE_TYPE_SHORT => "writeInt16",
        BaseType::BASE_TYPE_U_SHORT => "writeInt16",
        BaseType::BASE_TYPE_INT => "writeInt32",
        BaseType::BASE_TYPE_U_INT => "writeInt32",
        BaseType::BASE_TYPE_LONG => "writeInt64",
        BaseType::BASE_TYPE_U_LONG => "writeInt64",
        BaseType::BASE_TYPE_FLOAT => "writeFloat32",
        BaseType::BASE_TYPE_DOUBLE => "writeFloat64",
        _ => panic!("no builder write method for BaseType: {bt:?}"),
    }
}

/// Returns the Builder add method name for vector element writes.
///
/// # Panics
/// Panics on non-scalar types (unreachable after analyzer validation).
pub fn builder_add_method(bt: BaseType) -> &'static str {
    match bt {
        BaseType::BASE_TYPE_BOOL | BaseType::BASE_TYPE_BYTE | BaseType::BASE_TYPE_U_BYTE => {
            "addInt8"
        }
        BaseType::BASE_TYPE_SHORT | BaseType::BASE_TYPE_U_SHORT | BaseType::BASE_TYPE_U_TYPE => {
            "addInt16"
        }
        BaseType::BASE_TYPE_INT | BaseType::BASE_TYPE_U_INT => "addInt32",
        BaseType::BASE_TYPE_LONG | BaseType::BASE_TYPE_U_LONG => "addInt64",
        BaseType::BASE_TYPE_FLOAT => "addFloat32",
        BaseType::BASE_TYPE_DOUBLE => "addFloat64",
        _ => panic!("no builder add method for BaseType: {bt:?}"),
    }
}

/// Returns the TypedArray constructor name for a scalar BaseType (for vector *Array() methods).
///
/// # Panics
/// Panics on non-scalar types (unreachable after analyzer validation).
pub fn typed_array_name(bt: BaseType) -> &'static str {
    match bt {
        BaseType::BASE_TYPE_BYTE => "Int8Array",
        BaseType::BASE_TYPE_U_BYTE | BaseType::BASE_TYPE_BOOL => "Uint8Array",
        BaseType::BASE_TYPE_SHORT => "Int16Array",
        BaseType::BASE_TYPE_U_SHORT => "Uint16Array",
        BaseType::BASE_TYPE_INT => "Int32Array",
        BaseType::BASE_TYPE_U_INT => "Uint32Array",
        BaseType::BASE_TYPE_LONG => "BigInt64Array",
        BaseType::BASE_TYPE_U_LONG => "BigUint64Array",
        BaseType::BASE_TYPE_FLOAT => "Float32Array",
        BaseType::BASE_TYPE_DOUBLE => "Float64Array",
        _ => panic!("no typed array for BaseType: {bt:?}"),
    }
}

/// Format a default value for a scalar BaseType in TypeScript.
pub fn format_default_ts(value: i64, bt: BaseType) -> String {
    match bt {
        BaseType::BASE_TYPE_BOOL => {
            if value != 0 {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        BaseType::BASE_TYPE_LONG | BaseType::BASE_TYPE_U_LONG => {
            format!("BigInt('{value}')")
        }
        _ => format!("{value}"),
    }
}

/// Format a default float value for TypeScript.
pub fn format_default_real_ts(value: f64, _bt: BaseType) -> String {
    if value.is_nan() {
        "NaN".to_string()
    } else if value == f64::INFINITY {
        "Infinity".to_string()
    } else if value == f64::NEG_INFINITY {
        "-Infinity".to_string()
    } else if value == value.floor() {
        format!("{value:.1}")
    } else {
        format!("{value}")
    }
}

// Re-export shared functions from type_map
pub use super::type_map::{to_camel_case, to_pascal_case};

/// Returns true if the given identifier is a TypeScript keyword that needs escaping.
pub fn is_ts_keyword(name: &str) -> bool {
    matches!(
        name,
        "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
            | "let"
            | "static"
            | "implements"
            | "interface"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "type"
    )
}

/// Escape a TypeScript keyword by appending `_`. Non-keywords returned as-is.
pub fn escape_ts_keyword(name: &str) -> String {
    if is_ts_keyword(name) {
        format!("{name}_")
    } else {
        name.to_string()
    }
}

/// Emit a JSDoc comment block if documentation is present.
pub fn gen_doc_comment(w: &mut CodeWriter, doc: Option<&Documentation>) {
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

// Re-export shared functions from type_map
pub use super::type_map::build_fqn;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camel_case_conversions() {
        assert_eq!(to_camel_case("pos"), "pos");
        assert_eq!(to_camel_case("mana"), "mana");
        assert_eq!(to_camel_case("hp"), "hp");
        assert_eq!(to_camel_case("Monster"), "monster");
        assert_eq!(to_camel_case("my_field"), "myField");
        assert_eq!(to_camel_case("test_type"), "testType");
        assert_eq!(to_camel_case("HPMax"), "hpMax");
        assert_eq!(to_camel_case("Vec3"), "vec3");
    }

    #[test]
    fn pascal_case_conversions() {
        assert_eq!(to_pascal_case("pos"), "Pos");
        assert_eq!(to_pascal_case("mana"), "Mana");
        assert_eq!(to_pascal_case("Monster"), "Monster");
        assert_eq!(to_pascal_case("my_field"), "MyField");
        assert_eq!(to_pascal_case("test_type"), "TestType");
    }

    #[test]
    fn default_formatting() {
        assert_eq!(format_default_ts(0, BaseType::BASE_TYPE_INT), "0");
        assert_eq!(format_default_ts(42, BaseType::BASE_TYPE_INT), "42");
        assert_eq!(format_default_ts(0, BaseType::BASE_TYPE_BOOL), "false");
        assert_eq!(format_default_ts(1, BaseType::BASE_TYPE_BOOL), "true");
        assert_eq!(
            format_default_ts(100, BaseType::BASE_TYPE_LONG),
            "BigInt('100')"
        );
    }

    #[test]
    fn ts_types() {
        assert_eq!(scalar_ts_type(BaseType::BASE_TYPE_BOOL), "boolean");
        assert_eq!(scalar_ts_type(BaseType::BASE_TYPE_INT), "number");
        assert_eq!(scalar_ts_type(BaseType::BASE_TYPE_LONG), "bigint");
        assert_eq!(scalar_ts_type(BaseType::BASE_TYPE_FLOAT), "number");
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn float_default_special_values() {
        assert_eq!(
            format_default_real_ts(f64::INFINITY, BaseType::BASE_TYPE_FLOAT),
            "Infinity"
        );
        assert_eq!(
            format_default_real_ts(f64::NEG_INFINITY, BaseType::BASE_TYPE_FLOAT),
            "-Infinity"
        );
        assert_eq!(
            format_default_real_ts(f64::NAN, BaseType::BASE_TYPE_FLOAT),
            "NaN"
        );
        assert_eq!(
            format_default_real_ts(3.14_f64, BaseType::BASE_TYPE_DOUBLE),
            "3.14"
        );
        assert_eq!(
            format_default_real_ts(42.0, BaseType::BASE_TYPE_FLOAT),
            "42.0"
        );
    }
}
