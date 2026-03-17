pub mod buf_reader;
pub mod resolved;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// BaseType enum
// ---------------------------------------------------------------------------

/// FlatBuffers base types matching the official `reflection.fbs` enum.
///
/// Discriminant values match the official spec exactly (None=0 through Vector64=18).
/// `TABLE` and `STRUCT` are internal-only variants (both map to `Obj=15` in the
/// binary schema); use `to_reflection_byte()` for wire-format serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum BaseType {
    #[default]
    BASE_TYPE_NONE = 0,
    BASE_TYPE_U_TYPE = 1,
    BASE_TYPE_BOOL = 2,
    BASE_TYPE_BYTE = 3,
    BASE_TYPE_U_BYTE = 4,
    BASE_TYPE_SHORT = 5,
    BASE_TYPE_U_SHORT = 6,
    BASE_TYPE_INT = 7,
    BASE_TYPE_U_INT = 8,
    BASE_TYPE_LONG = 9,
    BASE_TYPE_U_LONG = 10,
    BASE_TYPE_FLOAT = 11,
    BASE_TYPE_DOUBLE = 12,
    BASE_TYPE_STRING = 13,
    BASE_TYPE_VECTOR = 14,
    /// Tables in the internal representation. Maps to `Obj` (15) in reflection.fbs.
    BASE_TYPE_TABLE = 15,
    BASE_TYPE_UNION = 16,
    BASE_TYPE_ARRAY = 17,
    BASE_TYPE_VECTOR64 = 18,
    /// Structs in the internal representation. Maps to `Obj` (15) in reflection.fbs.
    /// Uses an internal-only discriminant value (not part of the official spec).
    BASE_TYPE_STRUCT = 100,
}

impl BaseType {
    /// Convert to the official reflection.fbs byte value.
    /// Maps both `TABLE` and `STRUCT` to `Obj` (15).
    pub fn to_reflection_byte(self) -> u8 {
        match self {
            BaseType::BASE_TYPE_STRUCT => 15, // Obj
            _ => self as u8,
        }
    }

    /// Returns true if this is a scalar type (including bool and u_type).
    pub fn is_scalar(self) -> bool {
        matches!(
            self,
            Self::BASE_TYPE_BOOL
                | Self::BASE_TYPE_BYTE
                | Self::BASE_TYPE_U_BYTE
                | Self::BASE_TYPE_SHORT
                | Self::BASE_TYPE_U_SHORT
                | Self::BASE_TYPE_INT
                | Self::BASE_TYPE_U_INT
                | Self::BASE_TYPE_LONG
                | Self::BASE_TYPE_U_LONG
                | Self::BASE_TYPE_FLOAT
                | Self::BASE_TYPE_DOUBLE
                | Self::BASE_TYPE_U_TYPE
        )
    }

    /// Returns the byte size of a scalar type, or 0 for non-scalar types.
    pub fn scalar_byte_size(self) -> usize {
        match self {
            Self::BASE_TYPE_BOOL
            | Self::BASE_TYPE_BYTE
            | Self::BASE_TYPE_U_BYTE
            | Self::BASE_TYPE_U_TYPE => 1,
            Self::BASE_TYPE_SHORT | Self::BASE_TYPE_U_SHORT => 2,
            Self::BASE_TYPE_INT | Self::BASE_TYPE_U_INT | Self::BASE_TYPE_FLOAT => 4,
            Self::BASE_TYPE_LONG | Self::BASE_TYPE_U_LONG | Self::BASE_TYPE_DOUBLE => 8,
            _ => 0,
        }
    }

    /// Convert from the official reflection.fbs byte value.
    /// Maps `Obj` (15) to `BASE_TYPE_TABLE`; the caller must disambiguate
    /// TABLE vs STRUCT using the referenced object's `is_struct` flag.
    pub fn from_reflection_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::BASE_TYPE_NONE),
            1 => Some(Self::BASE_TYPE_U_TYPE),
            2 => Some(Self::BASE_TYPE_BOOL),
            3 => Some(Self::BASE_TYPE_BYTE),
            4 => Some(Self::BASE_TYPE_U_BYTE),
            5 => Some(Self::BASE_TYPE_SHORT),
            6 => Some(Self::BASE_TYPE_U_SHORT),
            7 => Some(Self::BASE_TYPE_INT),
            8 => Some(Self::BASE_TYPE_U_INT),
            9 => Some(Self::BASE_TYPE_LONG),
            10 => Some(Self::BASE_TYPE_U_LONG),
            11 => Some(Self::BASE_TYPE_FLOAT),
            12 => Some(Self::BASE_TYPE_DOUBLE),
            13 => Some(Self::BASE_TYPE_STRING),
            14 => Some(Self::BASE_TYPE_VECTOR),
            15 => Some(Self::BASE_TYPE_TABLE), // Obj; caller resolves TABLE vs STRUCT
            16 => Some(Self::BASE_TYPE_UNION),
            17 => Some(Self::BASE_TYPE_ARRAY),
            18 => Some(Self::BASE_TYPE_VECTOR64),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Span (source location)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Span {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    pub line: u32,
    pub col: u32,
}

impl Span {
    pub fn new(file: Option<String>, line: u32, col: u32) -> Self {
        Self { file, line, col }
    }
}

// ---------------------------------------------------------------------------
// Namespace
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Namespace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

impl Namespace {
    pub fn new() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// Type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Type {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_type: Option<BaseType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_size: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_size: Option<u32>,

    #[serde(rename = "element", skip_serializing_if = "Option::is_none")]
    pub element_type: Option<BaseType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_length: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub unresolved_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

impl Type {
    pub fn new() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// KeyValue (was attributes::Entry in proto)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct KeyValue {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl KeyValue {
    pub fn new() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// Attributes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Attributes {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<KeyValue>,
}

impl Attributes {
    pub fn new() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// Documentation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Documentation {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<String>,
}

impl Documentation {
    pub fn new() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// EnumVal
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct EnumVal {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub union_type: Option<Type>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<Documentation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Attributes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

impl EnumVal {
    pub fn new() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// Enum (covers both enums and unions)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Enum {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<EnumVal>,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_union: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlying_type: Option<Type>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Attributes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<Documentation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration_file: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<Namespace>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

impl Enum {
    pub fn new() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// Field
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Field {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<Type>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_integer: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_real: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_string: Option<String>,

    #[serde(
        rename = "deprecated",
        default,
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub is_deprecated: bool,

    #[serde(
        rename = "required",
        default,
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub is_required: bool,

    #[serde(rename = "key", default, skip_serializing_if = "std::ops::Not::not")]
    pub is_key: bool,

    #[serde(
        rename = "optional",
        default,
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub is_optional: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Attributes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<Documentation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<u32>,

    #[serde(
        rename = "offset64",
        default,
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub is_offset_64: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

impl Field {
    pub fn new() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// Object (tables and structs)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Object {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<Field>,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_struct: bool,

    #[serde(rename = "minalign", skip_serializing_if = "Option::is_none")]
    pub min_align: Option<i32>,

    #[serde(rename = "bytesize", skip_serializing_if = "Option::is_none")]
    pub byte_size: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Attributes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<Documentation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration_file: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<Namespace>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

impl Object {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_namespace(&mut self, ns: Namespace) {
        self.namespace = Some(ns);
    }
}

// ---------------------------------------------------------------------------
// RpcCall
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RpcCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Index into `Schema.objects` for the request type (table).
    /// Set by the analyzer after resolving the type name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_index: Option<i32>,

    /// Index into `Schema.objects` for the response type (table).
    /// Set by the analyzer after resolving the type name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_index: Option<i32>,

    /// Stub Object used only during parsing to carry the unresolved type name.
    /// After analysis, use `request_index` to look up the actual Object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<Object>,

    /// Stub Object used only during parsing to carry the unresolved type name.
    /// After analysis, use `response_index` to look up the actual Object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Object>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Attributes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<Documentation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

impl RpcCall {
    pub fn new() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Service {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub calls: Vec<RpcCall>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Attributes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<Documentation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration_file: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<Namespace>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

impl Service {
    pub fn new() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// AdvancedFeatures
// ---------------------------------------------------------------------------

/// Bit flags indicating which advanced schema features are used.
/// Matches the official `reflection.fbs` definition:
/// `enum AdvancedFeatures : ulong (bit_flags)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AdvancedFeatures(pub u64);

impl AdvancedFeatures {
    pub const ADVANCED_ARRAY_FEATURES: u64 = 1;
    pub const ADVANCED_UNION_FEATURES: u64 = 2;
    pub const OPTIONAL_SCALARS: u64 = 4;
    pub const DEFAULT_VECTORS_AND_STRINGS: u64 = 8;

    pub fn new() -> Self {
        Self(0)
    }

    pub fn set(&mut self, flag: u64) {
        self.0 |= flag;
    }

    pub fn has(self, flag: u64) -> bool {
        self.0 & flag != 0
    }
}

// ---------------------------------------------------------------------------
// SchemaFile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SchemaFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub included_filenames: Vec<String>,
}

impl SchemaFile {
    pub fn new() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// Schema (top-level)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Schema {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub objects: Vec<Object>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enums: Vec<Enum>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ident: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ext: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_table: Option<Object>,

    /// Index into `objects` for the root table. Preferred over `root_table`
    /// because `root_table` is a deep clone that can go stale after layout
    /// computation mutates the objects in-place.
    #[serde(skip)]
    pub root_table_index: Option<usize>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<Service>,

    #[serde(default)]
    pub advanced_features: AdvancedFeatures,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fbs_files: Vec<SchemaFile>,
}

impl Schema {
    pub fn new() -> Self {
        Self::default()
    }
}
