//! Serialize and deserialize compiled Schemas to/from the binary FlatBuffers schema
//! format (.bfbs).
//!
//! The .bfbs format is defined by `reflection.fbs` and uses the file identifier "BFBS".
//! Serialization builds the binary using `flatbuffers::FlatBufferBuilder` with manual
//! vtable field offsets. Deserialization uses the generated reflection reader types.
//!
//! ## Index Remapping (Serialization)
//!
//! The official `.bfbs` format requires objects and enums sorted alphabetically by
//! fully-qualified name. However, our internal `Schema` stores them in declaration
//! order (the order they appear in the `.fbs` file). This creates a mismatch:
//! `Type.index` values in our schema point to declaration-order positions, but in
//! the serialized `.bfbs` they must point to sorted positions.
//!
//! The remapping works in three steps:
//!
//! 1. **Sort**: Create sorted copies of objects and enums (by name).
//!
//! 2. **Build maps**: For each original index, record its new sorted position.
//!    `obj_index_to_sorted[orig_idx] = sorted_pos` (same for enums).
//!
//! 3. **Remap on write**: When serializing a `Type`, look at its `base_type` to
//!    decide which map to use:
//!    - TABLE/STRUCT -> use `obj_index_to_sorted`
//!    - UNION -> use `enum_index_to_sorted`
//!    - VECTOR/ARRAY -> look at `element_type` to decide (e.g., vector-of-table
//!      uses obj map, vector-of-enum uses enum map)
//!    - Scalar enum types -> use `enum_index_to_sorted`
//!
//! ## TABLE vs STRUCT Disambiguation (Deserialization)
//!
//! The wire format uses a single `Obj` (byte value 15) for both TABLE and STRUCT
//! base types. During deserialization, all `Obj` bytes are initially read as
//! `BASE_TYPE_TABLE`. A post-pass then checks each `Type.index` against the
//! `is_struct` flag on the referenced object and corrects it to `BASE_TYPE_STRUCT`
//! where needed.

use flatbuffers::{FlatBufferBuilder, TableFinishedWIPOffset, WIPOffset};
use flatc_rs_schema as schema;
use flatc_rs_schema::resolved::{
    ResolvedEnum, ResolvedEnumVal, ResolvedField, ResolvedObject, ResolvedRpcCall, ResolvedSchema,
    ResolvedService, ResolvedType,
};

use crate::reflection::reflection as refl;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during BFBS serialization or deserialization.
#[derive(Debug, thiserror::Error)]
pub enum BfbsError {
    #[error("invalid resolved schema: {0}")]
    Schema(#[from] schema::resolved::ResolveError),

    #[error("invalid BFBS buffer: {0}")]
    Invalid(String),

    #[error("unknown base type byte: {0}")]
    UnknownBaseType(u8),

    #[error("unresolved BFBS reference at {context}: '{target}'")]
    UnresolvedReference { context: String, target: String },
}

// ---------------------------------------------------------------------------
// Vtable field offset constants (slot = 4 + 2 * field_index)
// ---------------------------------------------------------------------------

// Type table fields
const TYPE_BASE_TYPE: flatbuffers::VOffsetT = 4;
const TYPE_ELEMENT: flatbuffers::VOffsetT = 6;
const TYPE_INDEX: flatbuffers::VOffsetT = 8;
const TYPE_FIXED_LENGTH: flatbuffers::VOffsetT = 10;
const TYPE_BASE_SIZE: flatbuffers::VOffsetT = 12;
const TYPE_ELEMENT_SIZE: flatbuffers::VOffsetT = 14;

// KeyValue table fields
const KV_KEY: flatbuffers::VOffsetT = 4;
const KV_VALUE: flatbuffers::VOffsetT = 6;

// EnumVal table fields
const ENUMVAL_NAME: flatbuffers::VOffsetT = 4;
const ENUMVAL_VALUE: flatbuffers::VOffsetT = 6;
const ENUMVAL_UNION_TYPE: flatbuffers::VOffsetT = 10;
const ENUMVAL_DOCUMENTATION: flatbuffers::VOffsetT = 12;
const ENUMVAL_ATTRIBUTES: flatbuffers::VOffsetT = 14;

// Enum table fields
const ENUM_NAME: flatbuffers::VOffsetT = 4;
const ENUM_VALUES: flatbuffers::VOffsetT = 6;
const ENUM_IS_UNION: flatbuffers::VOffsetT = 8;
const ENUM_UNDERLYING_TYPE: flatbuffers::VOffsetT = 10;
const ENUM_ATTRIBUTES: flatbuffers::VOffsetT = 12;
const ENUM_DOCUMENTATION: flatbuffers::VOffsetT = 14;
const ENUM_DECLARATION_FILE: flatbuffers::VOffsetT = 16;

// Field table fields
const FIELD_NAME: flatbuffers::VOffsetT = 4;
const FIELD_TYPE: flatbuffers::VOffsetT = 6;
const FIELD_ID: flatbuffers::VOffsetT = 8;
const FIELD_OFFSET: flatbuffers::VOffsetT = 10;
const FIELD_DEFAULT_INTEGER: flatbuffers::VOffsetT = 12;
const FIELD_DEFAULT_REAL: flatbuffers::VOffsetT = 14;
const FIELD_DEPRECATED: flatbuffers::VOffsetT = 16;
const FIELD_REQUIRED: flatbuffers::VOffsetT = 18;
const FIELD_KEY: flatbuffers::VOffsetT = 20;
const FIELD_ATTRIBUTES: flatbuffers::VOffsetT = 22;
const FIELD_DOCUMENTATION: flatbuffers::VOffsetT = 24;
const FIELD_OPTIONAL: flatbuffers::VOffsetT = 26;
const FIELD_PADDING: flatbuffers::VOffsetT = 28;
const FIELD_OFFSET64: flatbuffers::VOffsetT = 30;

// Object table fields
const OBJECT_NAME: flatbuffers::VOffsetT = 4;
const OBJECT_FIELDS: flatbuffers::VOffsetT = 6;
const OBJECT_IS_STRUCT: flatbuffers::VOffsetT = 8;
const OBJECT_MINALIGN: flatbuffers::VOffsetT = 10;
const OBJECT_BYTESIZE: flatbuffers::VOffsetT = 12;
const OBJECT_ATTRIBUTES: flatbuffers::VOffsetT = 14;
const OBJECT_DOCUMENTATION: flatbuffers::VOffsetT = 16;
const OBJECT_DECLARATION_FILE: flatbuffers::VOffsetT = 18;

// RPCCall table fields
const RPCCALL_NAME: flatbuffers::VOffsetT = 4;
const RPCCALL_REQUEST: flatbuffers::VOffsetT = 6;
const RPCCALL_RESPONSE: flatbuffers::VOffsetT = 8;
const RPCCALL_ATTRIBUTES: flatbuffers::VOffsetT = 10;
const RPCCALL_DOCUMENTATION: flatbuffers::VOffsetT = 12;

// Service table fields
const SERVICE_NAME: flatbuffers::VOffsetT = 4;
const SERVICE_CALLS: flatbuffers::VOffsetT = 6;
const SERVICE_ATTRIBUTES: flatbuffers::VOffsetT = 8;
const SERVICE_DOCUMENTATION: flatbuffers::VOffsetT = 10;
const SERVICE_DECLARATION_FILE: flatbuffers::VOffsetT = 12;

// SchemaFile table fields
const SCHEMAFILE_FILENAME: flatbuffers::VOffsetT = 4;
const SCHEMAFILE_INCLUDED_FILENAMES: flatbuffers::VOffsetT = 6;

// Schema table fields
const SCHEMA_OBJECTS: flatbuffers::VOffsetT = 4;
const SCHEMA_ENUMS: flatbuffers::VOffsetT = 6;
const SCHEMA_FILE_IDENT: flatbuffers::VOffsetT = 8;
const SCHEMA_FILE_EXT: flatbuffers::VOffsetT = 10;
const SCHEMA_ROOT_TABLE: flatbuffers::VOffsetT = 12;
const SCHEMA_SERVICES: flatbuffers::VOffsetT = 14;
const SCHEMA_ADVANCED_FEATURES: flatbuffers::VOffsetT = 16;
const SCHEMA_FBS_FILES: flatbuffers::VOffsetT = 18;

/// Finished table offset alias.
type TOff = WIPOffset<TableFinishedWIPOffset>;

/// Index remapping tables for serialization.
/// BFBS requires objects and enums sorted by name; Type.index values must
/// reference sorted positions, not original declaration-order positions.
struct IndexMaps<'a> {
    obj_index_to_sorted: &'a [usize],
    enum_index_to_sorted: &'a [usize],
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Serialize a compiled Schema into the .bfbs binary format.
///
/// The output is a valid FlatBuffer with file identifier "BFBS" conforming to
/// the official `reflection.fbs` schema.
pub fn serialize_schema(schema: &ResolvedSchema) -> Result<Vec<u8>, BfbsError> {
    schema.validate()?;
    let mut b = FlatBufferBuilder::with_capacity(4096);

    // Sort objects and enums by name (reflection.fbs requires sorted vectors).
    let mut sorted_objects: Vec<(usize, &ResolvedObject)> =
        schema.objects.iter().enumerate().collect();
    sorted_objects.sort_by(|a, b| resolved_obj_name(a.1).cmp(resolved_obj_name(b.1)));

    let mut sorted_enums: Vec<(usize, &ResolvedEnum)> = schema.enums.iter().enumerate().collect();
    sorted_enums.sort_by(|a, b| resolved_enum_name(a.1).cmp(resolved_enum_name(b.1)));

    // Mapping from original index -> sorted position (for Type.index remapping).
    let mut obj_index_to_sorted: Vec<usize> = vec![0; schema.objects.len()];
    for (sorted_pos, &(orig_idx, _)) in sorted_objects.iter().enumerate() {
        obj_index_to_sorted[orig_idx] = sorted_pos;
    }
    let mut enum_index_to_sorted: Vec<usize> = vec![0; schema.enums.len()];
    for (sorted_pos, &(orig_idx, _)) in sorted_enums.iter().enumerate() {
        enum_index_to_sorted[orig_idx] = sorted_pos;
    }
    let index_maps = IndexMaps {
        obj_index_to_sorted: &obj_index_to_sorted,
        enum_index_to_sorted: &enum_index_to_sorted,
    };

    // --- Serialize objects ---
    let obj_offs: Vec<TOff> = sorted_objects
        .iter()
        .map(|(_, obj)| write_resolved_object(&mut b, obj, &index_maps))
        .collect();
    let objects_vec = b.create_vector(&obj_offs);

    // --- Serialize enums ---
    let enum_offs: Vec<TOff> = sorted_enums
        .iter()
        .map(|(_, e)| write_resolved_enum(&mut b, e, &index_maps))
        .collect();
    let enums_vec = b.create_vector(&enum_offs);

    // --- Serialize services ---
    let svc_offs: Vec<TOff> = schema
        .services
        .iter()
        .map(|svc| write_resolved_service(&mut b, svc, &schema.objects, &index_maps))
        .collect();
    let services_vec = if svc_offs.is_empty() {
        None
    } else {
        Some(b.create_vector(&svc_offs))
    };

    // --- Serialize fbs_files ---
    let fbs_offs: Vec<TOff> = schema
        .fbs_files
        .iter()
        .map(|f| write_schema_file(&mut b, f))
        .collect();
    let fbs_vec = if fbs_offs.is_empty() {
        None
    } else {
        Some(b.create_vector(&fbs_offs))
    };

    // --- Serialize string fields ---
    let file_ident = schema.file_ident.as_deref().map(|s| b.create_string(s));
    let file_ext = schema.file_ext.as_deref().map(|s| b.create_string(s));

    // --- Find root_table offset ---
    let root_table_off: Option<TOff> = schema
        .root_table_index
        .map(|idx| obj_offs[obj_index_to_sorted[idx]]);

    // --- Build Schema table ---
    let start = b.start_table();
    b.push_slot_always(SCHEMA_OBJECTS, objects_vec);
    b.push_slot_always(SCHEMA_ENUMS, enums_vec);
    if let Some(fi) = file_ident {
        b.push_slot_always(SCHEMA_FILE_IDENT, fi);
    }
    if let Some(fe) = file_ext {
        b.push_slot_always(SCHEMA_FILE_EXT, fe);
    }
    if let Some(rt) = root_table_off {
        b.push_slot_always(SCHEMA_ROOT_TABLE, rt);
    }
    if let Some(svcs) = services_vec {
        b.push_slot_always(SCHEMA_SERVICES, svcs);
    }
    if schema.advanced_features.0 != 0 {
        b.push_slot::<u64>(SCHEMA_ADVANCED_FEATURES, schema.advanced_features.0, 0);
    }
    if let Some(fbs) = fbs_vec {
        b.push_slot_always(SCHEMA_FBS_FILES, fbs);
    }
    let schema_offset = b.end_table(start);

    b.finish(schema_offset, Some("BFBS"));
    Ok(b.finished_data().to_vec())
}

// ---------------------------------------------------------------------------
// Table writers -- each returns a finished table offset (TOff)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Resolved type writers (for serialization from ResolvedSchema)
// ---------------------------------------------------------------------------

fn write_resolved_type(
    b: &mut FlatBufferBuilder<'_>,
    ty: &ResolvedType,
    maps: &IndexMaps<'_>,
) -> TOff {
    let bt = ty.base_type;
    let et = ty.element_type.unwrap_or(schema::BaseType::BASE_TYPE_NONE);

    // Remap Type.index from declaration order to sorted order.
    let index = match ty.index {
        Some(idx) if idx >= 0 => {
            let i = idx as usize;
            match bt {
                schema::BaseType::BASE_TYPE_TABLE | schema::BaseType::BASE_TYPE_STRUCT => {
                    if i < maps.obj_index_to_sorted.len() {
                        maps.obj_index_to_sorted[i] as i32
                    } else {
                        idx
                    }
                }
                schema::BaseType::BASE_TYPE_UNION => {
                    if i < maps.enum_index_to_sorted.len() {
                        maps.enum_index_to_sorted[i] as i32
                    } else {
                        idx
                    }
                }
                schema::BaseType::BASE_TYPE_VECTOR
                | schema::BaseType::BASE_TYPE_ARRAY
                | schema::BaseType::BASE_TYPE_VECTOR64 => match et {
                    schema::BaseType::BASE_TYPE_TABLE | schema::BaseType::BASE_TYPE_STRUCT => {
                        if i < maps.obj_index_to_sorted.len() {
                            maps.obj_index_to_sorted[i] as i32
                        } else {
                            idx
                        }
                    }
                    schema::BaseType::BASE_TYPE_UNION => {
                        if i < maps.enum_index_to_sorted.len() {
                            maps.enum_index_to_sorted[i] as i32
                        } else {
                            idx
                        }
                    }
                    _ => {
                        if i < maps.enum_index_to_sorted.len() {
                            maps.enum_index_to_sorted[i] as i32
                        } else {
                            idx
                        }
                    }
                },
                _ => {
                    if i < maps.enum_index_to_sorted.len() {
                        maps.enum_index_to_sorted[i] as i32
                    } else {
                        idx
                    }
                }
            }
        }
        Some(idx) => idx,
        None => -1,
    };

    let start = b.start_table();
    b.push_slot::<u8>(TYPE_BASE_TYPE, bt.to_reflection_byte(), 0);
    b.push_slot::<u8>(TYPE_ELEMENT, et.to_reflection_byte(), 0);
    b.push_slot::<i32>(TYPE_INDEX, index, -1);
    b.push_slot::<u16>(TYPE_FIXED_LENGTH, ty.fixed_length.unwrap_or(0) as u16, 0);
    b.push_slot::<u32>(TYPE_BASE_SIZE, ty.base_size.unwrap_or(4), 4);
    b.push_slot::<u32>(TYPE_ELEMENT_SIZE, ty.element_size.unwrap_or(0), 0);
    b.end_table(start)
}

fn write_resolved_enum_val(
    b: &mut FlatBufferBuilder<'_>,
    ev: &ResolvedEnumVal,
    maps: &IndexMaps<'_>,
) -> TOff {
    let name = b.create_string(&ev.name);
    let union_type = ev
        .union_type
        .as_ref()
        .map(|t| write_resolved_type(b, t, maps));
    let doc = ev.documentation.as_ref().and_then(|d| write_doc_vec(b, d));
    let attrs = ev.attributes.as_ref().and_then(|a| write_attrs_vec(b, a));

    let start = b.start_table();
    b.push_slot_always(ENUMVAL_NAME, name);
    b.push_slot::<i64>(ENUMVAL_VALUE, ev.value, 0);
    if let Some(ut) = union_type {
        b.push_slot_always(ENUMVAL_UNION_TYPE, ut);
    }
    if let Some(d) = doc {
        b.push_slot_always(ENUMVAL_DOCUMENTATION, d);
    }
    if let Some(a) = attrs {
        b.push_slot_always(ENUMVAL_ATTRIBUTES, a);
    }
    b.end_table(start)
}

fn write_resolved_enum(
    b: &mut FlatBufferBuilder<'_>,
    e: &ResolvedEnum,
    maps: &IndexMaps<'_>,
) -> TOff {
    let fq_name = fq_resolved_enum_name(e);
    let name = b.create_string(&fq_name);

    let mut sorted_vals: Vec<&ResolvedEnumVal> = e.values.iter().collect();
    sorted_vals.sort_by_key(|v| v.value);
    let val_offs: Vec<TOff> = sorted_vals
        .iter()
        .map(|v| write_resolved_enum_val(b, v, maps))
        .collect();
    let values_vec = b.create_vector(&val_offs);

    let underlying = write_resolved_type(b, &e.underlying_type, maps);
    let attrs = e.attributes.as_ref().and_then(|a| write_attrs_vec(b, a));
    let doc = e.documentation.as_ref().and_then(|d| write_doc_vec(b, d));
    let decl_file = e.declaration_file.as_deref().map(|s| b.create_string(s));

    let start = b.start_table();
    b.push_slot_always(ENUM_NAME, name);
    b.push_slot_always(ENUM_VALUES, values_vec);
    b.push_slot::<bool>(ENUM_IS_UNION, e.is_union, false);
    b.push_slot_always(ENUM_UNDERLYING_TYPE, underlying);
    if let Some(a) = attrs {
        b.push_slot_always(ENUM_ATTRIBUTES, a);
    }
    if let Some(d) = doc {
        b.push_slot_always(ENUM_DOCUMENTATION, d);
    }
    if let Some(df) = decl_file {
        b.push_slot_always(ENUM_DECLARATION_FILE, df);
    }
    b.end_table(start)
}

fn write_resolved_field(
    b: &mut FlatBufferBuilder<'_>,
    field: &ResolvedField,
    maps: &IndexMaps<'_>,
) -> TOff {
    let name = b.create_string(&field.name);
    let field_type = write_resolved_type(b, &field.type_, maps);
    let attrs = field
        .attributes
        .as_ref()
        .and_then(|a| write_attrs_vec(b, a));
    let doc = field
        .documentation
        .as_ref()
        .and_then(|d| write_doc_vec(b, d));

    let start = b.start_table();
    b.push_slot_always(FIELD_NAME, name);
    b.push_slot_always(FIELD_TYPE, field_type);
    b.push_slot::<u16>(FIELD_ID, field.id.unwrap_or(0) as u16, 0);
    b.push_slot::<u16>(FIELD_OFFSET, field.offset.unwrap_or(0) as u16, 0);
    b.push_slot::<i64>(FIELD_DEFAULT_INTEGER, field.default_integer.unwrap_or(0), 0);
    b.push_slot::<f64>(FIELD_DEFAULT_REAL, field.default_real.unwrap_or(0.0), 0.0);
    b.push_slot::<bool>(FIELD_DEPRECATED, field.is_deprecated, false);
    b.push_slot::<bool>(FIELD_REQUIRED, field.is_required, false);
    b.push_slot::<bool>(FIELD_KEY, field.is_key, false);
    if let Some(a) = attrs {
        b.push_slot_always(FIELD_ATTRIBUTES, a);
    }
    if let Some(d) = doc {
        b.push_slot_always(FIELD_DOCUMENTATION, d);
    }
    b.push_slot::<bool>(FIELD_OPTIONAL, field.is_optional, false);
    b.push_slot::<u16>(FIELD_PADDING, field.padding.unwrap_or(0) as u16, 0);
    b.push_slot::<bool>(FIELD_OFFSET64, field.is_offset_64, false);
    b.end_table(start)
}

fn write_resolved_object(
    b: &mut FlatBufferBuilder<'_>,
    obj: &ResolvedObject,
    maps: &IndexMaps<'_>,
) -> TOff {
    let fq_name = fq_resolved_obj_name(obj);
    let name = b.create_string(&fq_name);

    // Fields sorted by name (reflection.fbs requires sorted fields)
    let mut sorted_fields: Vec<&ResolvedField> = obj.fields.iter().collect();
    sorted_fields.sort_by(|a, b| a.name.cmp(&b.name));
    let field_offs: Vec<TOff> = sorted_fields
        .iter()
        .map(|f| write_resolved_field(b, f, maps))
        .collect();
    let fields_vec = b.create_vector(&field_offs);

    let attrs = obj.attributes.as_ref().and_then(|a| write_attrs_vec(b, a));
    let doc = obj.documentation.as_ref().and_then(|d| write_doc_vec(b, d));
    let decl_file = obj.declaration_file.as_deref().map(|s| b.create_string(s));

    let start = b.start_table();
    b.push_slot_always(OBJECT_NAME, name);
    b.push_slot_always(OBJECT_FIELDS, fields_vec);
    b.push_slot::<bool>(OBJECT_IS_STRUCT, obj.is_struct, false);
    b.push_slot::<i32>(OBJECT_MINALIGN, obj.min_align.unwrap_or(0), 0);
    b.push_slot::<i32>(OBJECT_BYTESIZE, obj.byte_size.unwrap_or(0), 0);
    if let Some(a) = attrs {
        b.push_slot_always(OBJECT_ATTRIBUTES, a);
    }
    if let Some(d) = doc {
        b.push_slot_always(OBJECT_DOCUMENTATION, d);
    }
    if let Some(df) = decl_file {
        b.push_slot_always(OBJECT_DECLARATION_FILE, df);
    }
    b.end_table(start)
}

fn write_resolved_rpc_call(
    b: &mut FlatBufferBuilder<'_>,
    call: &ResolvedRpcCall,
    objects: &[ResolvedObject],
    maps: &IndexMaps<'_>,
) -> TOff {
    let name = b.create_string(&call.name);
    let request = write_resolved_object(b, &objects[call.request_index], maps);
    let response = write_resolved_object(b, &objects[call.response_index], maps);
    let attrs = call.attributes.as_ref().and_then(|a| write_attrs_vec(b, a));
    let doc = call
        .documentation
        .as_ref()
        .and_then(|d| write_doc_vec(b, d));

    let start = b.start_table();
    b.push_slot_always(RPCCALL_NAME, name);
    b.push_slot_always(RPCCALL_REQUEST, request);
    b.push_slot_always(RPCCALL_RESPONSE, response);
    if let Some(a) = attrs {
        b.push_slot_always(RPCCALL_ATTRIBUTES, a);
    }
    if let Some(d) = doc {
        b.push_slot_always(RPCCALL_DOCUMENTATION, d);
    }
    b.end_table(start)
}

fn write_resolved_service(
    b: &mut FlatBufferBuilder<'_>,
    svc: &ResolvedService,
    objects: &[ResolvedObject],
    maps: &IndexMaps<'_>,
) -> TOff {
    let fq_name = fq_resolved_svc_name(svc);
    let name = b.create_string(&fq_name);

    let call_offs: Vec<TOff> = svc
        .calls
        .iter()
        .map(|c| write_resolved_rpc_call(b, c, objects, maps))
        .collect();
    let calls_vec = if call_offs.is_empty() {
        None
    } else {
        Some(b.create_vector(&call_offs))
    };
    let attrs = svc.attributes.as_ref().and_then(|a| write_attrs_vec(b, a));
    let doc = svc.documentation.as_ref().and_then(|d| write_doc_vec(b, d));
    let decl_file = svc.declaration_file.as_deref().map(|s| b.create_string(s));

    let start = b.start_table();
    b.push_slot_always(SERVICE_NAME, name);
    if let Some(c) = calls_vec {
        b.push_slot_always(SERVICE_CALLS, c);
    }
    if let Some(a) = attrs {
        b.push_slot_always(SERVICE_ATTRIBUTES, a);
    }
    if let Some(d) = doc {
        b.push_slot_always(SERVICE_DOCUMENTATION, d);
    }
    if let Some(df) = decl_file {
        b.push_slot_always(SERVICE_DECLARATION_FILE, df);
    }
    b.end_table(start)
}

fn write_schema_file(b: &mut FlatBufferBuilder<'_>, sf: &schema::SchemaFile) -> TOff {
    let filename = sf.filename.as_deref().map(|s| b.create_string(s));
    let included = if sf.included_filenames.is_empty() {
        None
    } else {
        let strs: Vec<_> = sf
            .included_filenames
            .iter()
            .map(|s| b.create_string(s))
            .collect();
        Some(b.create_vector(&strs))
    };

    let start = b.start_table();
    if let Some(f) = filename {
        b.push_slot_always(SCHEMAFILE_FILENAME, f);
    }
    if let Some(inc) = included {
        b.push_slot_always(SCHEMAFILE_INCLUDED_FILENAMES, inc);
    }
    b.end_table(start)
}

// ---------------------------------------------------------------------------
// Vector helpers for attributes and documentation
// ---------------------------------------------------------------------------

fn write_key_value(b: &mut FlatBufferBuilder<'_>, kv: &schema::KeyValue) -> TOff {
    let key = kv.key.as_deref().map(|s| b.create_string(s));
    let value = kv.value.as_deref().map(|s| b.create_string(s));

    let start = b.start_table();
    if let Some(k) = key {
        b.push_slot_always(KV_KEY, k);
    }
    if let Some(v) = value {
        b.push_slot_always(KV_VALUE, v);
    }
    b.end_table(start)
}

/// Serialize an Attributes list into a vector of KeyValue tables.
/// Returns None if there are no entries.
fn write_attrs_vec<'a>(
    b: &mut FlatBufferBuilder<'a>,
    attrs: &schema::Attributes,
) -> Option<WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<TableFinishedWIPOffset>>>>
{
    if attrs.entries.is_empty() {
        return None;
    }
    let offs: Vec<TOff> = attrs
        .entries
        .iter()
        .map(|kv| write_key_value(b, kv))
        .collect();
    Some(b.create_vector(&offs))
}

/// Serialize a Documentation list into a vector of strings.
/// Returns None if there are no lines.
fn write_doc_vec<'a>(
    b: &mut FlatBufferBuilder<'a>,
    doc: &schema::Documentation,
) -> Option<WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<&'a str>>>> {
    if doc.lines.is_empty() {
        return None;
    }
    let offs: Vec<_> = doc.lines.iter().map(|s| b.create_string(s)).collect();
    Some(b.create_vector(&offs))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn resolved_obj_name(obj: &ResolvedObject) -> &str {
    &obj.name
}

fn resolved_enum_name(e: &ResolvedEnum) -> &str {
    &e.name
}

fn fq_resolved_obj_name(obj: &ResolvedObject) -> String {
    let name = &obj.name;
    match &obj.namespace {
        Some(ns) if !ns.namespace.as_deref().unwrap_or("").is_empty() => {
            format!("{}.{}", ns.namespace.as_deref().unwrap_or(""), name)
        }
        _ => name.to_string(),
    }
}

fn fq_resolved_enum_name(e: &ResolvedEnum) -> String {
    let name = &e.name;
    match &e.namespace {
        Some(ns) if !ns.namespace.as_deref().unwrap_or("").is_empty() => {
            format!("{}.{}", ns.namespace.as_deref().unwrap_or(""), name)
        }
        _ => name.to_string(),
    }
}

fn fq_resolved_svc_name(svc: &ResolvedService) -> String {
    let name = &svc.name;
    match &svc.namespace {
        Some(ns) if !ns.namespace.as_deref().unwrap_or("").is_empty() => {
            format!("{}.{}", ns.namespace.as_deref().unwrap_or(""), name)
        }
        _ => name.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Legacy helpers (used by deserialization which builds parsed Schema types)
// ---------------------------------------------------------------------------

fn fully_qualified_obj_name(obj: &schema::Object) -> String {
    let name = obj.name.as_deref().unwrap_or("");
    match &obj.namespace {
        Some(ns) if !ns.namespace.as_deref().unwrap_or("").is_empty() => {
            format!("{}.{}", ns.namespace.as_deref().unwrap_or(""), name)
        }
        _ => name.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Deserialization: .bfbs -> Schema
// ---------------------------------------------------------------------------

/// Deserialize a .bfbs binary buffer into an owned, validated `ResolvedSchema`.
///
/// The buffer must be a valid FlatBuffer with the "BFBS" file identifier.
/// After initial conversion, a post-pass disambiguates TABLE vs STRUCT references
/// using the `is_struct` flag on each object.
pub fn deserialize_resolved_schema(buf: &[u8]) -> Result<ResolvedSchema, BfbsError> {
    if buf.len() < 8 {
        return Err(BfbsError::Invalid("buffer too small".into()));
    }

    // Verify file identifier
    if !refl::schema_buffer_has_identifier(buf) {
        return Err(BfbsError::Invalid("missing BFBS file identifier".into()));
    }

    let root = refl::root_as_schema(buf)
        .map_err(|e| BfbsError::Invalid(format!("flatbuffer verification failed: {e}")))?;

    // Build object struct-ness lookup (indexed by position in the objects vector).
    let objects_vec = root.objects();
    let mut is_struct_flags: Vec<bool> = Vec::with_capacity(objects_vec.len());
    for i in 0..objects_vec.len() {
        is_struct_flags.push(objects_vec.get(i).is_struct());
    }

    // Convert objects
    let mut out_objects: Vec<schema::Object> = Vec::with_capacity(objects_vec.len());
    for i in 0..objects_vec.len() {
        out_objects.push(read_object(&objects_vec.get(i), &is_struct_flags)?);
    }

    // Convert enums
    let enums_vec = root.enums();
    let mut out_enums: Vec<schema::Enum> = Vec::with_capacity(enums_vec.len());
    for i in 0..enums_vec.len() {
        out_enums.push(read_enum(&enums_vec.get(i), &is_struct_flags)?);
    }

    // Convert services
    let mut out_services: Vec<schema::Service> = Vec::new();
    if let Some(svcs) = root.services() {
        for i in 0..svcs.len() {
            out_services.push(read_service(&svcs.get(i), &is_struct_flags)?);
        }
    }

    // Resolve RPC request/response indices by matching inline Object names
    // against the deserialized objects list.
    resolve_rpc_indices(&mut out_services, &out_objects)?;

    // Convert fbs_files
    let mut out_fbs_files: Vec<schema::SchemaFile> = Vec::new();
    if let Some(files) = root.fbs_files() {
        for i in 0..files.len() {
            out_fbs_files.push(read_schema_file(&files.get(i)));
        }
    }

    // root_table: find by name in out_objects, and record the index
    let (root_table, root_table_index) = match root.root_table() {
        Some(rt) => {
            let rt_name = rt.name();
            let found = out_objects.iter().enumerate().find(|(_, obj)| {
                let fq = fully_qualified_obj_name(obj);
                fq == rt_name
            });
            match found {
                Some((idx, obj)) => (Some(obj.clone()), Some(idx)),
                None => {
                    return Err(BfbsError::UnresolvedReference {
                        context: "schema.root_table".to_string(),
                        target: rt_name.to_string(),
                    });
                }
            }
        }
        None => (None, None),
    };

    let advanced_features = schema::AdvancedFeatures(root.advanced_features().bits());

    let parsed = schema::Schema {
        objects: out_objects,
        enums: out_enums,
        file_ident: root.file_ident().map(|s| s.to_string()),
        file_ext: root.file_ext().map(|s| s.to_string()),
        root_table,
        root_table_index,
        services: out_services,
        advanced_features,
        fbs_files: out_fbs_files,
    };
    ResolvedSchema::try_from_parsed(&parsed).map_err(BfbsError::Schema)
}

/// Deserialize a .bfbs buffer into the legacy parsed schema representation.
///
/// This compatibility API uses the strict resolved path and therefore never
/// returns partial root or RPC references.
pub fn deserialize_schema(buf: &[u8]) -> Result<schema::Schema, BfbsError> {
    let resolved = deserialize_resolved_schema(buf)?;
    let mut legacy = resolved.as_legacy()?;
    for (legacy_service, resolved_service) in legacy.services.iter_mut().zip(&resolved.services) {
        for (legacy_call, resolved_call) in
            legacy_service.calls.iter_mut().zip(&resolved_service.calls)
        {
            legacy_call.request = Some(legacy.objects[resolved_call.request_index].clone());
            legacy_call.response = Some(legacy.objects[resolved_call.response_index].clone());
        }
    }
    Ok(legacy)
}

/// Resolve `request_index` and `response_index` on RpcCalls by matching
/// the inline Object's fully-qualified name against the objects list.
fn resolve_rpc_indices(
    services: &mut [schema::Service],
    objects: &[schema::Object],
) -> Result<(), BfbsError> {
    for service in services.iter_mut() {
        let service_name = service.name.as_deref().unwrap_or("<unnamed>");
        for call in service.calls.iter_mut() {
            let call_name = call.name.as_deref().unwrap_or("<unnamed>");
            if let Some(ref req) = call.request {
                let fq = fully_qualified_obj_name(req);
                let idx = objects
                    .iter()
                    .position(|o| fully_qualified_obj_name(o) == fq)
                    .ok_or_else(|| BfbsError::UnresolvedReference {
                        context: format!("service '{service_name}' call '{call_name}' request"),
                        target: fq,
                    })?;
                call.request_index = Some(i32::try_from(idx).map_err(|_| {
                    BfbsError::Invalid(format!(
                        "service '{service_name}' call '{call_name}' request index exceeds i32"
                    ))
                })?);
            }
            if let Some(ref resp) = call.response {
                let fq = fully_qualified_obj_name(resp);
                let idx = objects
                    .iter()
                    .position(|o| fully_qualified_obj_name(o) == fq)
                    .ok_or_else(|| BfbsError::UnresolvedReference {
                        context: format!("service '{service_name}' call '{call_name}' response"),
                        target: fq,
                    })?;
                call.response_index = Some(i32::try_from(idx).map_err(|_| {
                    BfbsError::Invalid(format!(
                        "service '{service_name}' call '{call_name}' response index exceeds i32"
                    ))
                })?);
            }
        }
    }
    Ok(())
}

/// Split a fully-qualified name like "MyGame.Example.Monster" into
/// (Some("MyGame.Example"), "Monster"). If there is no dot, returns (None, name).
fn split_fq_name(fq: &str) -> (Option<&str>, &str) {
    match fq.rfind('.') {
        Some(pos) => (Some(&fq[..pos]), &fq[pos + 1..]),
        None => (None, fq),
    }
}

fn read_base_type(
    b: u8,
    is_struct_flags: &[bool],
    index: i32,
) -> Result<schema::BaseType, BfbsError> {
    let bt = schema::BaseType::from_reflection_byte(b).ok_or(BfbsError::UnknownBaseType(b))?;

    // Disambiguate Obj (15) -> TABLE or STRUCT based on the referenced object
    if bt == schema::BaseType::BASE_TYPE_TABLE
        && index >= 0
        && (index as usize) < is_struct_flags.len()
        && is_struct_flags[index as usize]
    {
        return Ok(schema::BaseType::BASE_TYPE_STRUCT);
    }
    Ok(bt)
}

fn read_type(ty: &refl::Type<'_>, is_struct_flags: &[bool]) -> Result<schema::Type, BfbsError> {
    let index = ty.index();
    let base_type = read_base_type(ty.base_type().0 as u8, is_struct_flags, index)?;
    let element_byte = ty.element().0 as u8;
    let element_type = if element_byte != 0 {
        Some(read_base_type(element_byte, is_struct_flags, index)?)
    } else {
        None
    };

    let fixed_length = ty.fixed_length();
    let base_size = ty.base_size();
    let element_size = ty.element_size();

    Ok(schema::Type {
        base_type: Some(base_type),
        base_size: Some(base_size),
        element_size: if element_size != 0 {
            Some(element_size)
        } else {
            None
        },
        element_type,
        index: if index != -1 { Some(index) } else { None },
        fixed_length: if fixed_length != 0 {
            Some(fixed_length as u32)
        } else {
            None
        },
        unresolved_name: None,
        span: None,
    })
}

fn read_key_value(kv: &refl::KeyValue<'_>) -> schema::KeyValue {
    schema::KeyValue {
        key: Some(kv.key().to_string()),
        value: kv.value().map(|s| s.to_string()),
    }
}

fn read_attributes(
    attrs: Option<flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<refl::KeyValue<'_>>>>,
) -> Option<schema::Attributes> {
    let vec = attrs?;
    if vec.is_empty() {
        return None;
    }
    let entries: Vec<schema::KeyValue> = (0..vec.len())
        .map(|i| read_key_value(&vec.get(i)))
        .collect();
    Some(schema::Attributes { entries })
}

fn read_documentation(
    doc: Option<flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<&str>>>,
) -> Option<schema::Documentation> {
    let vec = doc?;
    if vec.is_empty() {
        return None;
    }
    let lines: Vec<String> = (0..vec.len()).map(|i| vec.get(i).to_string()).collect();
    Some(schema::Documentation { lines })
}

fn read_field(
    field: &refl::Field<'_>,
    is_struct_flags: &[bool],
    parent_is_struct: bool,
) -> Result<schema::Field, BfbsError> {
    let ty = read_type(&field.type_(), is_struct_flags)?;
    let id = field.id();
    let offset = field.offset();
    let default_integer = field.default_integer();
    let default_real = field.default_real();

    Ok(schema::Field {
        name: Some(field.name().to_string()),
        type_: Some(ty),
        id: Some(id as u32),
        offset: if parent_is_struct || offset != 0 {
            Some(offset as u32)
        } else {
            None
        },
        default_integer: if default_integer != 0 {
            Some(default_integer)
        } else {
            None
        },
        default_real: if default_real != 0.0 {
            Some(default_real)
        } else {
            None
        },
        default_string: None,
        is_deprecated: field.deprecated(),
        is_required: field.required(),
        is_key: field.key(),
        is_optional: field.optional(),
        attributes: read_attributes(field.attributes()),
        documentation: read_documentation(field.documentation()),
        padding: if field.padding() != 0 {
            Some(field.padding() as u32)
        } else {
            None
        },
        is_offset_64: field.offset64(),
        span: None,
    })
}

fn read_object(
    obj: &refl::Object<'_>,
    is_struct_flags: &[bool],
) -> Result<schema::Object, BfbsError> {
    let fq_name = obj.name();
    let (ns, short_name) = split_fq_name(fq_name);

    let fields_vec = obj.fields();
    let mut fields: Vec<schema::Field> = Vec::with_capacity(fields_vec.len());
    for i in 0..fields_vec.len() {
        fields.push(read_field(
            &fields_vec.get(i),
            is_struct_flags,
            obj.is_struct(),
        )?);
    }
    // Sort fields by id to restore original declaration order
    fields.sort_by_key(|f| f.id.unwrap_or(0));

    let minalign = obj.minalign();
    let bytesize = obj.bytesize();

    Ok(schema::Object {
        name: Some(short_name.to_string()),
        fields,
        is_struct: obj.is_struct(),
        min_align: if minalign != 0 { Some(minalign) } else { None },
        byte_size: if bytesize != 0 { Some(bytesize) } else { None },
        attributes: read_attributes(obj.attributes()),
        documentation: read_documentation(obj.documentation()),
        declaration_file: obj.declaration_file().map(|s| s.to_string()),
        namespace: ns.map(|s| schema::Namespace {
            namespace: Some(s.to_string()),
        }),
        span: None,
    })
}

fn read_enum_val(
    ev: &refl::EnumVal<'_>,
    is_struct_flags: &[bool],
    is_union: bool,
) -> Result<schema::EnumVal, BfbsError> {
    let union_type = match (is_union, ev.union_type()) {
        (true, Some(ut)) => {
            let ty = read_type(&ut, is_struct_flags)?;
            (ty.base_type != Some(schema::BaseType::BASE_TYPE_NONE)).then_some(ty)
        }
        _ => None,
    };

    Ok(schema::EnumVal {
        name: Some(ev.name().to_string()),
        value: Some(ev.value()),
        union_type,
        documentation: read_documentation(ev.documentation()),
        attributes: read_attributes(ev.attributes()),
        span: None,
    })
}

fn read_enum(e: &refl::Enum<'_>, is_struct_flags: &[bool]) -> Result<schema::Enum, BfbsError> {
    let fq_name = e.name();
    let (ns, short_name) = split_fq_name(fq_name);

    let values_vec = e.values();
    let mut values: Vec<schema::EnumVal> = Vec::with_capacity(values_vec.len());
    for i in 0..values_vec.len() {
        values.push(read_enum_val(
            &values_vec.get(i),
            is_struct_flags,
            e.is_union(),
        )?);
    }

    let underlying = read_type(&e.underlying_type(), is_struct_flags)?;

    Ok(schema::Enum {
        name: Some(short_name.to_string()),
        values,
        is_union: e.is_union(),
        underlying_type: Some(underlying),
        attributes: read_attributes(e.attributes()),
        documentation: read_documentation(e.documentation()),
        declaration_file: e.declaration_file().map(|s| s.to_string()),
        namespace: ns.map(|s| schema::Namespace {
            namespace: Some(s.to_string()),
        }),
        span: None,
    })
}

fn read_service(
    svc: &refl::Service<'_>,
    is_struct_flags: &[bool],
) -> Result<schema::Service, BfbsError> {
    let fq_name = svc.name();
    let (ns, short_name) = split_fq_name(fq_name);

    let mut calls: Vec<schema::RpcCall> = Vec::new();
    if let Some(calls_vec) = svc.calls() {
        for i in 0..calls_vec.len() {
            calls.push(read_rpc_call(&calls_vec.get(i), is_struct_flags)?);
        }
    }

    Ok(schema::Service {
        name: Some(short_name.to_string()),
        calls,
        attributes: read_attributes(svc.attributes()),
        documentation: read_documentation(svc.documentation()),
        declaration_file: svc.declaration_file().map(|s| s.to_string()),
        namespace: ns.map(|s| schema::Namespace {
            namespace: Some(s.to_string()),
        }),
        span: None,
    })
}

fn read_rpc_call(
    call: &refl::RPCCall<'_>,
    is_struct_flags: &[bool],
) -> Result<schema::RpcCall, BfbsError> {
    // Read inline Objects to extract names; indices are resolved in a post-pass.
    let request = read_object(&call.request(), is_struct_flags)?;
    let response = read_object(&call.response(), is_struct_flags)?;
    Ok(schema::RpcCall {
        name: Some(call.name().to_string()),
        request_index: None, // resolved in resolve_rpc_indices()
        response_index: None,
        request: Some(request),
        response: Some(response),
        attributes: read_attributes(call.attributes()),
        documentation: read_documentation(call.documentation()),
        span: None,
    })
}

fn read_schema_file(sf: &refl::SchemaFile<'_>) -> schema::SchemaFile {
    let included = match sf.included_filenames() {
        Some(vec) => (0..vec.len()).map(|i| vec.get(i).to_string()).collect(),
        None => Vec::new(),
    };

    schema::SchemaFile {
        filename: Some(sf.filename().to_string()),
        included_filenames: included,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_to_bfbs(source: &str) -> Vec<u8> {
        let schema = crate::compile_single(source)
            .expect("compile BFBS fixture")
            .schema;
        serialize_schema(&schema).expect("serialize BFBS fixture")
    }

    fn string_range(buf: &[u8], value: &str) -> std::ops::Range<usize> {
        let start = value.as_ptr() as usize - buf.as_ptr() as usize;
        start..start + value.len()
    }

    fn table_field_location(
        buf: &[u8],
        table_location: usize,
        slot: flatbuffers::VOffsetT,
    ) -> Option<usize> {
        let soff = i32::from_le_bytes(
            buf.get(table_location..table_location + 4)?
                .try_into()
                .ok()?,
        );
        let vtable_location = if soff >= 0 {
            table_location.checked_sub(soff as usize)?
        } else {
            table_location.checked_add(soff.unsigned_abs() as usize)?
        };
        let vtable_length = u16::from_le_bytes(
            buf.get(vtable_location..vtable_location + 2)?
                .try_into()
                .ok()?,
        );
        if slot >= vtable_length {
            return None;
        }
        let field_offset = u16::from_le_bytes(
            buf.get(vtable_location + slot as usize..vtable_location + slot as usize + 2)?
                .try_into()
                .ok()?,
        ) as usize;
        (field_offset != 0).then(|| table_location + field_offset)
    }

    fn set_first_field_type_index(buf: &mut [u8], object_name: &str, index: i32) {
        let root = refl::root_as_schema(buf).expect("verified BFBS fixture");
        let objects = root.objects();
        let object = (0..objects.len())
            .map(|position| objects.get(position))
            .find(|object| object.name() == object_name)
            .expect("fixture object");
        let type_location = object.fields().get(0).type_()._tab.loc();
        let index_location = table_field_location(buf, type_location, refl::Type::VT_INDEX)
            .expect("serialized type index");
        buf[index_location..index_location + 4].copy_from_slice(&index.to_le_bytes());
    }

    #[test]
    fn test_serialize_empty_schema() {
        let schema = ResolvedSchema {
            objects: vec![],
            enums: vec![],
            file_ident: None,
            file_ext: None,
            root_table_index: None,
            services: vec![],
            advanced_features: schema::AdvancedFeatures(0),
            fbs_files: vec![],
        };
        let buf = serialize_schema(&schema).expect("serialize schema");
        assert!(buf.len() >= 8, "buffer too small: {} bytes", buf.len());
        assert_eq!(&buf[4..8], b"BFBS", "missing BFBS file identifier");
    }

    #[test]
    fn test_serialize_minimal_schema_with_table() {
        let obj = ResolvedObject {
            name: "Monster".to_string(),
            fields: vec![ResolvedField {
                name: "hp".to_string(),
                type_: ResolvedType {
                    base_type: schema::BaseType::BASE_TYPE_SHORT,
                    base_size: Some(2),
                    element_size: None,
                    element_type: None,
                    index: None,
                    fixed_length: None,
                },
                id: Some(0),
                offset: None,
                default_integer: None,
                default_real: None,
                default_string: None,
                is_deprecated: false,
                is_required: false,
                is_key: false,
                is_optional: false,
                attributes: None,
                documentation: None,
                padding: None,
                is_offset_64: false,
                span: None,
            }],
            is_struct: false,
            min_align: None,
            byte_size: None,
            attributes: None,
            documentation: None,
            declaration_file: None,
            namespace: None,
            span: None,
        };

        let schema = ResolvedSchema {
            objects: vec![obj],
            enums: vec![],
            file_ident: None,
            file_ext: None,
            root_table_index: Some(0),
            services: vec![],
            advanced_features: schema::AdvancedFeatures(0),
            fbs_files: vec![],
        };

        let buf = serialize_schema(&schema).expect("serialize schema");
        assert_eq!(&buf[4..8], b"BFBS");
        assert!(buf.len() > 20, "buffer suspiciously small");
    }

    #[test]
    fn test_base_type_reflection_byte() {
        assert_eq!(schema::BaseType::BASE_TYPE_NONE.to_reflection_byte(), 0);
        assert_eq!(schema::BaseType::BASE_TYPE_BOOL.to_reflection_byte(), 2);
        assert_eq!(schema::BaseType::BASE_TYPE_TABLE.to_reflection_byte(), 15);
        assert_eq!(schema::BaseType::BASE_TYPE_STRUCT.to_reflection_byte(), 15);
        assert_eq!(schema::BaseType::BASE_TYPE_UNION.to_reflection_byte(), 16);
        assert_eq!(
            schema::BaseType::BASE_TYPE_VECTOR64.to_reflection_byte(),
            18
        );
    }

    #[test]
    fn strict_deserializer_rejects_root_missing_from_objects() {
        // Arrange
        let mut buf = compile_to_bfbs("table RootA {} root_type RootA;");
        let root = refl::root_as_schema(&buf).expect("verified BFBS fixture");
        let objects_field = table_field_location(&buf, root._tab.loc(), refl::Schema::VT_OBJECTS)
            .expect("objects field");
        let objects_location = objects_field
            + u32::from_le_bytes(buf[objects_field..objects_field + 4].try_into().unwrap())
                as usize;
        buf[objects_location..objects_location + 4].copy_from_slice(&0_u32.to_le_bytes());

        // Act
        let error = deserialize_resolved_schema(&buf).unwrap_err();

        // Assert
        assert!(error.to_string().contains("schema.root_table"));
        assert!(error.to_string().contains("RootA"));
    }

    #[test]
    fn strict_deserializer_rejects_missing_rpc_request_and_response() {
        // Arrange
        let source =
            "table Request {} table Response {} rpc_service Api { Get(Request):Response; }";
        let mutate_rpc_type = |member: &str, replacement: &[u8]| {
            let mut buf = compile_to_bfbs(source);
            let root = refl::root_as_schema(&buf).expect("verified BFBS fixture");
            let call = root
                .services()
                .expect("services")
                .get(0)
                .calls()
                .expect("calls")
                .get(0);
            let object = match member {
                "request" => call.request(),
                "response" => call.response(),
                _ => unreachable!("known RPC member"),
            };
            let range = string_range(&buf, object.name());
            buf[range].copy_from_slice(replacement);
            buf
        };

        // Act
        let request_error =
            deserialize_resolved_schema(&mutate_rpc_type("request", b"Missing")).unwrap_err();
        let response_error =
            deserialize_resolved_schema(&mutate_rpc_type("response", b"Absent__")).unwrap_err();

        // Assert
        assert!(request_error.to_string().contains("call 'Get' request"));
        assert!(request_error.to_string().contains("Missing"));
        assert!(response_error.to_string().contains("call 'Get' response"));
        assert!(response_error.to_string().contains("Absent__"));
    }

    #[test]
    fn strict_deserializer_rejects_negative_and_out_of_bounds_object_indices() {
        for index in [-2, 99] {
            // Arrange
            let mut buf =
                compile_to_bfbs("table Child {} table Root { child:Child; } root_type Root;");
            set_first_field_type_index(&mut buf, "Root", index);

            // Act
            let error = deserialize_resolved_schema(&buf).unwrap_err();

            // Assert
            assert!(
                error.to_string().contains(&format!("index {index}")),
                "{error}"
            );
        }
    }

    #[test]
    fn strict_deserializer_rejects_negative_and_out_of_bounds_enum_indices() {
        for index in [-2, 99] {
            // Arrange
            let mut buf = compile_to_bfbs(
                "enum Color:byte { Red } table Root { color:Color; } root_type Root;",
            );
            set_first_field_type_index(&mut buf, "Root", index);

            // Act
            let error = deserialize_resolved_schema(&buf).unwrap_err();

            // Assert
            assert!(
                error.to_string().contains(&format!("index {index}")),
                "{error}"
            );
        }
    }

    #[test]
    fn strict_deserializer_disambiguates_tables_and_structs() {
        // Arrange
        let buf = compile_to_bfbs(
            "struct Point { x:int; } table Child {} table Root { point:Point; child:Child; } root_type Root;",
        );

        // Act
        let schema = deserialize_resolved_schema(&buf).expect("strict BFBS schema");

        // Assert
        let root = schema
            .objects
            .iter()
            .find(|object| object.name == "Root")
            .unwrap();
        assert_eq!(
            root.fields[0].type_.base_type,
            schema::BaseType::BASE_TYPE_STRUCT
        );
        assert_eq!(
            root.fields[1].type_.base_type,
            schema::BaseType::BASE_TYPE_TABLE
        );
        let point = schema
            .objects
            .iter()
            .find(|object| object.name == "Point")
            .unwrap();
        assert_eq!(point.fields[0].offset, Some(0));
    }

    #[test]
    fn valid_bfbs_is_codegen_ready_and_round_trips() {
        // Arrange
        let buf = compile_to_bfbs(
            "struct Point { x:int; } table Root { point:Point; values:[int]; } root_type Root;",
        );

        // Act
        let schema = deserialize_resolved_schema(&buf).expect("strict BFBS schema");
        let generated =
            flatc_rs_codegen::generate_rust(&schema, &flatc_rs_codegen::CodeGenOptions::default())
                .expect("codegen from BFBS");
        let round_trip = serialize_schema(&schema).expect("round-trip BFBS");

        // Assert
        assert!(generated.contains("pub struct Root"));
        assert_eq!(deserialize_resolved_schema(&round_trip).unwrap(), schema);
    }

    #[test]
    fn official_cpp_bfbs_fixture_is_codegen_ready_and_round_trips() {
        // Arrange
        let buf = include_bytes!("../testdata/official_bfbs/strict_roundtrip.bfbs");

        // Act
        let schema = deserialize_resolved_schema(buf).expect("official C++ BFBS fixture");
        let generated = flatc_rs_codegen::generate_rust(
            &schema,
            &flatc_rs_codegen::CodeGenOptions {
                gen_object_api: true,
                ..Default::default()
            },
        )
        .expect("codegen from official BFBS");
        let round_trip = serialize_schema(&schema).expect("serialize official schema");

        // Assert
        assert!(generated.contains("pub struct Root"));
        assert_eq!(schema.root_table_index, Some(2));
        assert_eq!(schema.services[0].calls[0].request_index, 0);
        assert_eq!(schema.services[0].calls[0].response_index, 1);
        assert!(schema.enums[0]
            .values
            .iter()
            .all(|value| value.union_type.is_none()));
        assert!(schema.enums[1].values[0].union_type.is_none());
        assert_eq!(deserialize_resolved_schema(&round_trip).unwrap(), schema);
    }
}
