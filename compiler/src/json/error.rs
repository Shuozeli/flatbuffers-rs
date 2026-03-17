use flatc_rs_schema::buf_reader::BoundsError;
use flatc_rs_schema::BaseType;

#[derive(Debug, thiserror::Error)]
pub enum JsonError {
    // -- Decoder errors (binary -> JSON) --
    #[error("buffer too small: need {need} bytes at offset {offset}, have {buf_len}")]
    OutOfBounds {
        offset: usize,
        need: usize,
        buf_len: usize,
    },

    #[error("root type '{name}' not found in schema")]
    RootTypeNotFound { name: String },

    #[error("object index {index} out of range (have {count} objects)")]
    ObjectIndexOutOfRange { index: usize, count: usize },

    #[error("enum index {index} out of range (have {count} enums)")]
    EnumIndexOutOfRange { index: usize, count: usize },

    #[error("depth exceeded maximum of {max}")]
    MaxDepthExceeded { max: usize },

    // -- Encoder errors (JSON -> binary) --
    #[error("expected JSON object for table/struct '{type_name}', got {actual}")]
    ExpectedObject { type_name: String, actual: String },

    #[error("expected JSON array for vector field '{field_name}', got {actual}")]
    ExpectedArray { field_name: String, actual: String },

    #[error("expected JSON number for field '{field_name}' ({base_type}), got {actual}")]
    ExpectedNumber {
        field_name: String,
        base_type: String,
        actual: String,
    },

    #[error("expected JSON string for field '{field_name}', got {actual}")]
    ExpectedString { field_name: String, actual: String },

    #[error("unknown field '{field_name}' in table '{type_name}'")]
    UnknownField {
        type_name: String,
        field_name: String,
    },

    #[error("unknown enum value '{value}' for enum '{enum_name}'")]
    UnknownEnumValue { enum_name: String, value: String },

    #[error("union field '{field_name}' requires companion '{field_name}_type' field")]
    MissingUnionType { field_name: String },

    #[error("struct field '{field_name}' is missing in JSON (structs require all fields)")]
    MissingStructField { field_name: String },

    #[error("missing type index for {context}")]
    MissingTypeIndex { context: String },

    #[error("number out of range for field '{field_name}' ({base_type}): {value}")]
    NumberOutOfRange {
        field_name: String,
        base_type: String,
        value: String,
    },
}

/// Return the JSON type name for error messages.
pub fn json_type_name(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(_) => "bool".to_string(),
        serde_json::Value::Number(_) => "number".to_string(),
        serde_json::Value::String(_) => "string".to_string(),
        serde_json::Value::Array(_) => "array".to_string(),
        serde_json::Value::Object(_) => "object".to_string(),
    }
}

/// Return the byte size of a scalar BaseType.
pub fn scalar_byte_size(bt: BaseType) -> usize {
    bt.scalar_byte_size()
}

/// Return true if the BaseType is a scalar (including bool).
pub fn is_scalar(bt: BaseType) -> bool {
    bt.is_scalar()
}

impl From<BoundsError> for JsonError {
    fn from(e: BoundsError) -> Self {
        JsonError::OutOfBounds {
            offset: e.offset,
            need: e.size,
            buf_len: e.buf_len,
        }
    }
}
