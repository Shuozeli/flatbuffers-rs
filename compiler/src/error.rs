use flatc_rs_schema::Span;

#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    #[error("{span:?}: unresolved type '{name}' in {context}")]
    UnresolvedType {
        name: String,
        context: String,
        span: Option<Span>,
    },

    #[error("{span:?}: duplicate type name: {name}")]
    DuplicateName { name: String, span: Option<Span> },

    #[error("{span:?}: duplicate enum value {value} in enum '{enum_name}'")]
    DuplicateEnumValue {
        enum_name: String,
        value: i64,
        span: Option<Span>,
    },

    #[error("{span:?}: invalid struct field '{field_name}' in struct '{struct_name}': {reason}")]
    InvalidStructField {
        struct_name: String,
        field_name: String,
        reason: String,
        span: Option<Span>,
    },

    #[error("circular struct dependency: {}", format_cycle(.0))]
    CircularStruct(Vec<String>),

    #[error("struct nesting depth limit exceeded ({depth} levels) while computing layout for '{type_name}'")]
    StructDepthLimitExceeded { depth: usize, type_name: String },

    #[error("{span:?}: root_type '{name}' must be a table, not a struct or enum")]
    RootTypeMustBeTable { name: String, span: Option<Span> },

    #[error("{span:?}: file identifier must be exactly 4 bytes, got {len}: \"{ident}\"")]
    InvalidFileIdentifier {
        ident: String,
        len: usize,
        span: Option<Span>,
    },

    #[error("{span:?}: mixed id assignment in table '{table_name}': either all fields must have id or none")]
    MixedIdAssignment {
        table_name: String,
        span: Option<Span>,
    },

    #[error("{span:?}: duplicate field id {id} in table '{table_name}' (fields '{field_a}' and '{field_b}')")]
    DuplicateFieldId {
        table_name: String,
        id: u32,
        field_a: String,
        field_b: String,
        span: Option<Span>,
    },

    #[error("{span:?}: non-contiguous field ids in table '{table_name}': expected ids 0..{expected}, got gaps")]
    NonContiguousFieldIds {
        table_name: String,
        expected: usize,
        span: Option<Span>,
    },

    #[error("{span:?}: field id {id} out of range in table '{table_name}' (field '{field_name}'): must be 0..=65535")]
    FieldIdOutOfRange {
        table_name: String,
        field_name: String,
        id: u32,
        span: Option<Span>,
    },

    #[error("{span:?}: invalid RPC type '{type_name}' in {service}.{method}: must be a table")]
    InvalidRpcType {
        service: String,
        method: String,
        type_name: String,
        span: Option<Span>,
    },

    #[error("{span:?}: invalid union variant '{variant}' in union '{union_name}': {reason}")]
    InvalidUnionVariant {
        union_name: String,
        variant: String,
        reason: String,
        span: Option<Span>,
    },

    #[error("{span:?}: duplicate field name '{field_name}' in '{type_name}'")]
    DuplicateFieldName {
        type_name: String,
        field_name: String,
        span: Option<Span>,
    },

    #[error("{span:?}: default values are not allowed on struct field '{field_name}' in struct '{struct_name}'")]
    StructDefaultValue {
        struct_name: String,
        field_name: String,
        span: Option<Span>,
    },

    #[error("{span:?}: deprecated attribute is not allowed on struct field '{field_name}' in struct '{struct_name}'")]
    StructDeprecatedField {
        struct_name: String,
        field_name: String,
        span: Option<Span>,
    },

    #[error("{span:?}: invalid underlying type for enum '{enum_name}': must be an integer type")]
    InvalidEnumUnderlyingType {
        enum_name: String,
        span: Option<Span>,
    },

    #[error("{span:?}: duplicate enum value name '{value_name}' in enum '{enum_name}'")]
    DuplicateEnumValueName {
        enum_name: String,
        value_name: String,
        span: Option<Span>,
    },

    #[error("{span:?}: force_align may only be applied to structs, not '{name}'")]
    ForceAlignOnNonStruct { name: String, span: Option<Span> },

    #[error("{span:?}: force_align must be a power of 2, got {value} on '{name}'")]
    ForceAlignNotPowerOf2 {
        name: String,
        value: i64,
        span: Option<Span>,
    },

    #[error("{span:?}: struct '{name}' has no fields")]
    EmptyStruct { name: String, span: Option<Span> },

    #[error("{span:?}: enum value {value} does not fit in {type_name} for enum '{enum_name}'")]
    EnumValueOutOfRange {
        enum_name: String,
        value: i64,
        type_name: String,
        span: Option<Span>,
    },

    #[error("{span:?}: default value {value} does not fit in {type_name} for field '{field_name}' in table '{table_name}'")]
    DefaultValueOutOfRange {
        table_name: String,
        field_name: String,
        value: i64,
        type_name: String,
        span: Option<Span>,
    },

    #[error("enum value overflow in enum '{enum_name}': auto-assigned value after {last_value} exceeds i64 range")]
    EnumValueOverflow { enum_name: String, last_value: i64 },

    #[error(
        "invalid type index {index} in struct '{struct_name}': out of range (max {max_index})"
    )]
    InvalidTypeIndex {
        struct_name: String,
        index: i32,
        max_index: usize,
    },

    #[error("bit_flags value {value} out of range in enum '{enum_name}': must be 0..{max_bits} for {type_name}")]
    BitFlagsValueOutOfRange {
        enum_name: String,
        value: i64,
        max_bits: u32,
        type_name: String,
    },

    #[error("{span:?}: union field '{field_name}' in table '{table_name}' has id: 0, but union fields require a companion _type field at id-1; union field ids must be >= 1")]
    UnionFieldIdZero {
        table_name: String,
        field_name: String,
        span: Option<Span>,
    },

    #[error("{span:?}: union NONE variant in '{union_name}' {reason}")]
    InvalidUnionNone {
        union_name: String,
        reason: String,
        span: Option<Span>,
    },

    #[error("{span:?}: multiple key fields in table '{table_name}': '{field_a}' and '{field_b}'")]
    MultipleKeys {
        table_name: String,
        field_a: String,
        field_b: String,
        span: Option<Span>,
    },

    #[error("{span:?}: key field '{field_name}' in table '{table_name}' must be a scalar or string type, got {actual_type}")]
    InvalidKeyFieldType {
        table_name: String,
        field_name: String,
        actual_type: String,
        span: Option<Span>,
    },

    #[error("{span:?}: key field '{field_name}' in table '{table_name}' cannot be deprecated")]
    DeprecatedKeyField {
        table_name: String,
        field_name: String,
        span: Option<Span>,
    },

    #[error(
        "{span:?}: invalid force_align value '{value}' on '{name}': must be a positive integer"
    )]
    InvalidForceAlignValue {
        name: String,
        value: String,
        span: Option<Span>,
    },

    #[error("Leaking private implementation, verify all objects have similar annotations")]
    PrivateLeak {
        public_type: String,
        private_type: String,
    },

    #[error("schema size limit exceeded: {detail}")]
    SchemaSizeLimitExceeded { detail: String },
}

fn format_cycle(names: &[String]) -> String {
    names.join(" -> ")
}

pub type Result<T> = std::result::Result<T, AnalyzeError>;
