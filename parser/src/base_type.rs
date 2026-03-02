use flatc_rs_schema::BaseType;

/// Map a FlatBuffers type keyword to its BaseType.
/// Returns None for user-defined types.
pub fn lookup_base_type(name: &str) -> Option<BaseType> {
    match name {
        "bool" => Some(BaseType::BASE_TYPE_BOOL),
        "byte" | "int8" => Some(BaseType::BASE_TYPE_BYTE),
        "ubyte" | "uint8" => Some(BaseType::BASE_TYPE_U_BYTE),
        "short" | "int16" => Some(BaseType::BASE_TYPE_SHORT),
        "ushort" | "uint16" => Some(BaseType::BASE_TYPE_U_SHORT),
        "int" | "int32" => Some(BaseType::BASE_TYPE_INT),
        "uint" | "uint32" => Some(BaseType::BASE_TYPE_U_INT),
        "long" | "int64" => Some(BaseType::BASE_TYPE_LONG),
        "ulong" | "uint64" => Some(BaseType::BASE_TYPE_U_LONG),
        "float" | "float32" => Some(BaseType::BASE_TYPE_FLOAT),
        "double" | "float64" => Some(BaseType::BASE_TYPE_DOUBLE),
        "string" => Some(BaseType::BASE_TYPE_STRING),
        _ => None,
    }
}

/// Size in bytes for a base type. Returns None for types whose size depends on
/// context (TABLE, ARRAY).
pub fn base_type_size(bt: BaseType) -> Option<u32> {
    match bt {
        BaseType::BASE_TYPE_BOOL
        | BaseType::BASE_TYPE_BYTE
        | BaseType::BASE_TYPE_U_BYTE
        | BaseType::BASE_TYPE_NONE
        | BaseType::BASE_TYPE_U_TYPE => Some(1),

        BaseType::BASE_TYPE_SHORT | BaseType::BASE_TYPE_U_SHORT => Some(2),

        BaseType::BASE_TYPE_INT | BaseType::BASE_TYPE_U_INT | BaseType::BASE_TYPE_FLOAT => Some(4),

        BaseType::BASE_TYPE_LONG | BaseType::BASE_TYPE_U_LONG | BaseType::BASE_TYPE_DOUBLE => {
            Some(8)
        }

        // Offset types: 4 bytes (uoffset_t)
        BaseType::BASE_TYPE_STRING
        | BaseType::BASE_TYPE_VECTOR
        | BaseType::BASE_TYPE_STRUCT
        | BaseType::BASE_TYPE_UNION => Some(4),

        BaseType::BASE_TYPE_VECTOR64 => Some(8),

        // Context-dependent
        BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_ARRAY => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_scalar_keywords() {
        assert_eq!(lookup_base_type("bool"), Some(BaseType::BASE_TYPE_BOOL));
        assert_eq!(lookup_base_type("byte"), Some(BaseType::BASE_TYPE_BYTE));
        assert_eq!(lookup_base_type("ubyte"), Some(BaseType::BASE_TYPE_U_BYTE));
        assert_eq!(lookup_base_type("short"), Some(BaseType::BASE_TYPE_SHORT));
        assert_eq!(
            lookup_base_type("ushort"),
            Some(BaseType::BASE_TYPE_U_SHORT)
        );
        assert_eq!(lookup_base_type("int"), Some(BaseType::BASE_TYPE_INT));
        assert_eq!(lookup_base_type("uint"), Some(BaseType::BASE_TYPE_U_INT));
        assert_eq!(lookup_base_type("long"), Some(BaseType::BASE_TYPE_LONG));
        assert_eq!(lookup_base_type("ulong"), Some(BaseType::BASE_TYPE_U_LONG));
        assert_eq!(lookup_base_type("float"), Some(BaseType::BASE_TYPE_FLOAT));
        assert_eq!(lookup_base_type("double"), Some(BaseType::BASE_TYPE_DOUBLE));
        assert_eq!(lookup_base_type("string"), Some(BaseType::BASE_TYPE_STRING));
    }

    #[test]
    fn test_type_aliases() {
        assert_eq!(lookup_base_type("int8"), Some(BaseType::BASE_TYPE_BYTE));
        assert_eq!(lookup_base_type("uint8"), Some(BaseType::BASE_TYPE_U_BYTE));
        assert_eq!(lookup_base_type("int16"), Some(BaseType::BASE_TYPE_SHORT));
        assert_eq!(
            lookup_base_type("uint16"),
            Some(BaseType::BASE_TYPE_U_SHORT)
        );
        assert_eq!(lookup_base_type("int32"), Some(BaseType::BASE_TYPE_INT));
        assert_eq!(lookup_base_type("uint32"), Some(BaseType::BASE_TYPE_U_INT));
        assert_eq!(lookup_base_type("int64"), Some(BaseType::BASE_TYPE_LONG));
        assert_eq!(lookup_base_type("uint64"), Some(BaseType::BASE_TYPE_U_LONG));
        assert_eq!(lookup_base_type("float32"), Some(BaseType::BASE_TYPE_FLOAT));
        assert_eq!(
            lookup_base_type("float64"),
            Some(BaseType::BASE_TYPE_DOUBLE)
        );
    }

    #[test]
    fn test_unknown_type() {
        assert_eq!(lookup_base_type("Monster"), None);
        assert_eq!(lookup_base_type("Vec3"), None);
        assert_eq!(lookup_base_type(""), None);
    }

    #[test]
    fn test_base_type_sizes() {
        assert_eq!(base_type_size(BaseType::BASE_TYPE_BOOL), Some(1));
        assert_eq!(base_type_size(BaseType::BASE_TYPE_BYTE), Some(1));
        assert_eq!(base_type_size(BaseType::BASE_TYPE_SHORT), Some(2));
        assert_eq!(base_type_size(BaseType::BASE_TYPE_INT), Some(4));
        assert_eq!(base_type_size(BaseType::BASE_TYPE_LONG), Some(8));
        assert_eq!(base_type_size(BaseType::BASE_TYPE_FLOAT), Some(4));
        assert_eq!(base_type_size(BaseType::BASE_TYPE_DOUBLE), Some(8));
        assert_eq!(base_type_size(BaseType::BASE_TYPE_STRING), Some(4));
        assert_eq!(base_type_size(BaseType::BASE_TYPE_VECTOR), Some(4));
        assert_eq!(base_type_size(BaseType::BASE_TYPE_TABLE), None);
        assert_eq!(base_type_size(BaseType::BASE_TYPE_ARRAY), None);
    }
}
