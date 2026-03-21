use std::ops::Range;

use flatc_rs_schema::BaseType;

#[derive(Debug, Clone)]
pub struct AnnotatedRegion {
    pub byte_range: Range<usize>,
    pub region_type: RegionType,
    pub label: String,
    pub field_path: Vec<String>,
    pub value_display: String,
    pub children: Vec<usize>,
    pub related_regions: Vec<usize>,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionType {
    RootOffset,
    FileIdentifier,
    VTable {
        type_name: String,
    },
    VTableSize,
    VTableTableSize,
    VTableEntry {
        field_name: String,
        field_id: u32,
    },
    TableSOffset {
        type_name: String,
    },
    ScalarField {
        field_name: String,
        base_type: BaseType,
    },
    StringOffset {
        field_name: String,
    },
    StringLength,
    StringData {
        field_name: String,
    },
    StringTerminator,
    VectorOffset {
        field_name: String,
    },
    VectorLength,
    VectorElement {
        index: usize,
    },
    StructInline {
        type_name: String,
    },
    StructField {
        field_name: String,
        base_type: BaseType,
    },
    UnionTypeField {
        field_name: String,
    },
    UnionDataOffset {
        field_name: String,
    },
    Padding,
    Unknown,

    // --- Protobuf wire format regions ---
    /// A protobuf message (top-level or nested).
    ProtoMessage {
        type_name: String,
    },
    /// A varint-encoded tag (field_number << 3 | wire_type).
    ProtoTag {
        field_number: u32,
        wire_type: u8,
    },
    /// A varint value (wire type 0).
    ProtoVarint {
        field_name: String,
    },
    /// A 64-bit fixed value (wire type 1).
    ProtoFixed64 {
        field_name: String,
    },
    /// A 32-bit fixed value (wire type 5).
    ProtoFixed32 {
        field_name: String,
    },
    /// Length-delimited data (wire type 2): string, bytes, nested message, or packed repeated.
    ProtoLengthDelimited {
        field_name: String,
    },
    /// The length prefix of a length-delimited field.
    ProtoLength,
    /// Raw bytes within a length-delimited field (when type is unknown or bytes).
    ProtoBytes {
        field_name: String,
    },
    /// String data within a length-delimited field.
    ProtoString {
        field_name: String,
    },
    /// A packed repeated element.
    ProtoPackedElement {
        index: usize,
    },
}

impl RegionType {
    pub fn color(&self) -> [u8; 3] {
        match self {
            RegionType::RootOffset | RegionType::FileIdentifier => [100, 149, 237],
            RegionType::VTable { .. }
            | RegionType::VTableSize
            | RegionType::VTableTableSize
            | RegionType::VTableEntry { .. } => [70, 130, 180],
            RegionType::TableSOffset { .. } => [30, 144, 255],
            RegionType::ScalarField { .. } => [60, 179, 113],
            RegionType::StringOffset { .. }
            | RegionType::StringLength
            | RegionType::StringData { .. }
            | RegionType::StringTerminator => [218, 165, 32],
            RegionType::VectorOffset { .. }
            | RegionType::VectorLength
            | RegionType::VectorElement { .. } => [147, 112, 219],
            RegionType::StructInline { .. } | RegionType::StructField { .. } => [255, 140, 0],
            RegionType::UnionTypeField { .. } | RegionType::UnionDataOffset { .. } => [205, 92, 92],
            RegionType::Padding => [128, 128, 128],
            RegionType::Unknown => [169, 169, 169],
            // Protobuf colors
            RegionType::ProtoMessage { .. } => [30, 144, 255],
            RegionType::ProtoTag { .. } => [70, 130, 180],
            RegionType::ProtoVarint { .. } => [60, 179, 113],
            RegionType::ProtoFixed64 { .. } | RegionType::ProtoFixed32 { .. } => [255, 140, 0],
            RegionType::ProtoLengthDelimited { .. } | RegionType::ProtoLength => [147, 112, 219],
            RegionType::ProtoBytes { .. } => [218, 165, 32],
            RegionType::ProtoString { .. } => [218, 165, 32],
            RegionType::ProtoPackedElement { .. } => [147, 112, 219],
        }
    }

    pub fn short_name(&self) -> &str {
        match self {
            RegionType::RootOffset => "root_offset",
            RegionType::FileIdentifier => "file_id",
            RegionType::VTable { .. } => "vtable",
            RegionType::VTableSize => "vtable_size",
            RegionType::VTableTableSize => "vtable_table_size",
            RegionType::VTableEntry { .. } => "vtable_entry",
            RegionType::TableSOffset { .. } => "table_soffset",
            RegionType::ScalarField { .. } => "scalar",
            RegionType::StringOffset { .. } => "string_offset",
            RegionType::StringLength => "string_length",
            RegionType::StringData { .. } => "string_data",
            RegionType::StringTerminator => "string_null",
            RegionType::VectorOffset { .. } => "vector_offset",
            RegionType::VectorLength => "vector_length",
            RegionType::VectorElement { .. } => "vector_elem",
            RegionType::StructInline { .. } => "struct",
            RegionType::StructField { .. } => "struct_field",
            RegionType::UnionTypeField { .. } => "union_type",
            RegionType::UnionDataOffset { .. } => "union_offset",
            RegionType::Padding => "padding",
            RegionType::Unknown => "unknown",
            // Protobuf
            RegionType::ProtoMessage { .. } => "proto_message",
            RegionType::ProtoTag { .. } => "proto_tag",
            RegionType::ProtoVarint { .. } => "proto_varint",
            RegionType::ProtoFixed64 { .. } => "proto_fixed64",
            RegionType::ProtoFixed32 { .. } => "proto_fixed32",
            RegionType::ProtoLengthDelimited { .. } => "proto_length_delimited",
            RegionType::ProtoLength => "proto_length",
            RegionType::ProtoBytes { .. } => "proto_bytes",
            RegionType::ProtoString { .. } => "proto_string",
            RegionType::ProtoPackedElement { .. } => "proto_packed_elem",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WalkError {
    #[error("read out of bounds: offset {offset}, size {size}, buffer length {buf_len}")]
    OutOfBounds {
        offset: usize,
        size: usize,
        buf_len: usize,
    },
    #[error("root type '{name}' not found in schema")]
    RootTypeNotFound { name: String },
    #[error("invalid offset at 0x{offset:04X}: points to 0x{target:04X} which is out of bounds")]
    InvalidOffset { offset: usize, target: usize },
    #[error("walk depth exceeded maximum of {max}")]
    MaxDepthExceeded { max: usize },
    #[error("object index {index} out of range (have {count} objects)")]
    ObjectIndexOutOfRange { index: usize, count: usize },
    #[error("enum index {index} out of range (have {count} enums)")]
    EnumIndexOutOfRange { index: usize, count: usize },
    #[error("missing type index for {context}")]
    MissingTypeIndex { context: String },
}
