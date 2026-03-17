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

use crate::reflection::reflection as refl;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during BFBS deserialization.
#[derive(Debug, thiserror::Error)]
pub enum BfbsError {
    #[error("invalid BFBS buffer: {0}")]
    Invalid(String),

    #[error("unknown base type byte: {0}")]
    UnknownBaseType(u8),
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
pub fn serialize_schema(schema: &schema::Schema) -> Vec<u8> {
    let mut b = FlatBufferBuilder::with_capacity(4096);

    // Sort objects and enums by name (reflection.fbs requires sorted vectors).
    let mut sorted_objects: Vec<(usize, &schema::Object)> =
        schema.objects.iter().enumerate().collect();
    sorted_objects.sort_by(|a, b| obj_name(a.1).cmp(obj_name(b.1)));

    let mut sorted_enums: Vec<(usize, &schema::Enum)> = schema.enums.iter().enumerate().collect();
    sorted_enums.sort_by(|a, b| enum_name(a.1).cmp(enum_name(b.1)));

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
        .map(|(_, obj)| write_object(&mut b, obj, &index_maps))
        .collect();
    let objects_vec = b.create_vector(&obj_offs);

    // --- Serialize enums ---
    let enum_offs: Vec<TOff> = sorted_enums
        .iter()
        .map(|(_, e)| write_enum(&mut b, e, &index_maps))
        .collect();
    let enums_vec = b.create_vector(&enum_offs);

    // --- Serialize services ---
    let svc_offs: Vec<TOff> = schema
        .services
        .iter()
        .map(|svc| write_service(&mut b, svc, &schema.objects, &index_maps))
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
        .map(|idx| obj_offs[obj_index_to_sorted[idx]])
        .or_else(|| {
            // Fallback: match by name if root_table_index is not set
            schema.root_table.as_ref().and_then(|rt| {
                let rt_name = rt.name.as_deref().unwrap_or("");
                schema
                    .objects
                    .iter()
                    .enumerate()
                    .find(|(_, obj)| obj.name.as_deref() == Some(rt_name))
                    .map(|(orig_idx, _)| obj_offs[obj_index_to_sorted[orig_idx]])
            })
        });

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
    b.finished_data().to_vec()
}

// ---------------------------------------------------------------------------
// Table writers -- each returns a finished table offset (TOff)
// ---------------------------------------------------------------------------

fn write_type(b: &mut FlatBufferBuilder<'_>, ty: &schema::Type, maps: &IndexMaps<'_>) -> TOff {
    let bt = ty.base_type.unwrap_or(schema::BaseType::BASE_TYPE_NONE);
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
                // For vector/array of objects, the element type determines the index target
                schema::BaseType::BASE_TYPE_VECTOR
                | schema::BaseType::BASE_TYPE_ARRAY
                | schema::BaseType::BASE_TYPE_VECTOR64 => {
                    match et {
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
                            // Enum element types also need remapping
                            if i < maps.enum_index_to_sorted.len() {
                                maps.enum_index_to_sorted[i] as i32
                            } else {
                                idx
                            }
                        }
                    }
                }
                _ => {
                    // Scalar enum types: index references enums
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

fn write_enum_val(
    b: &mut FlatBufferBuilder<'_>,
    ev: &schema::EnumVal,
    maps: &IndexMaps<'_>,
) -> TOff {
    let name = ev.name.as_deref().map(|s| b.create_string(s));
    let union_type = ev.union_type.as_ref().map(|t| write_type(b, t, maps));
    let doc = ev.documentation.as_ref().and_then(|d| write_doc_vec(b, d));
    let attrs = ev.attributes.as_ref().and_then(|a| write_attrs_vec(b, a));

    let start = b.start_table();
    if let Some(n) = name {
        b.push_slot_always(ENUMVAL_NAME, n);
    }
    b.push_slot::<i64>(ENUMVAL_VALUE, ev.value.unwrap_or(0), 0);
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

fn write_enum(b: &mut FlatBufferBuilder<'_>, e: &schema::Enum, maps: &IndexMaps<'_>) -> TOff {
    let fq_name = fully_qualified_enum_name(e);
    let name = b.create_string(&fq_name);

    let mut sorted_vals: Vec<&schema::EnumVal> = e.values.iter().collect();
    sorted_vals.sort_by_key(|v| v.value.unwrap_or(0));
    let val_offs: Vec<TOff> = sorted_vals
        .iter()
        .map(|v| write_enum_val(b, v, maps))
        .collect();
    let values_vec = b.create_vector(&val_offs);

    let underlying = e.underlying_type.as_ref().map(|t| write_type(b, t, maps));
    let attrs = e.attributes.as_ref().and_then(|a| write_attrs_vec(b, a));
    let doc = e.documentation.as_ref().and_then(|d| write_doc_vec(b, d));
    let decl_file = e.declaration_file.as_deref().map(|s| b.create_string(s));

    let start = b.start_table();
    b.push_slot_always(ENUM_NAME, name);
    b.push_slot_always(ENUM_VALUES, values_vec);
    b.push_slot::<bool>(ENUM_IS_UNION, e.is_union, false);
    if let Some(ut) = underlying {
        b.push_slot_always(ENUM_UNDERLYING_TYPE, ut);
    }
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

fn write_field(b: &mut FlatBufferBuilder<'_>, field: &schema::Field, maps: &IndexMaps<'_>) -> TOff {
    let name = field.name.as_deref().map(|s| b.create_string(s));
    let field_type = field.type_.as_ref().map(|t| write_type(b, t, maps));
    let attrs = field
        .attributes
        .as_ref()
        .and_then(|a| write_attrs_vec(b, a));
    let doc = field
        .documentation
        .as_ref()
        .and_then(|d| write_doc_vec(b, d));

    let start = b.start_table();
    if let Some(n) = name {
        b.push_slot_always(FIELD_NAME, n);
    }
    if let Some(t) = field_type {
        b.push_slot_always(FIELD_TYPE, t);
    }
    b.push_slot::<u16>(FIELD_ID, field.id.unwrap_or(0) as u16, 0);
    b.push_slot::<u16>(FIELD_OFFSET, field.offset.unwrap_or(0) as u16, 0);
    b.push_slot::<i64>(FIELD_DEFAULT_INTEGER, field.default_integer.unwrap_or(0), 0);
    b.push_slot::<f64>(FIELD_DEFAULT_REAL, field.default_real.unwrap_or(0.0), 0.0);
    b.push_slot::<bool>(
        FIELD_DEPRECATED,
        field.is_deprecated,
        false,
    );
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

fn write_object(b: &mut FlatBufferBuilder<'_>, obj: &schema::Object, maps: &IndexMaps<'_>) -> TOff {
    let fq_name = fully_qualified_obj_name(obj);
    let name = b.create_string(&fq_name);

    // Fields sorted by name (reflection.fbs requires sorted fields)
    let mut sorted_fields: Vec<&schema::Field> = obj.fields.iter().collect();
    sorted_fields.sort_by(|a, b| {
        a.name
            .as_deref()
            .unwrap_or("")
            .cmp(b.name.as_deref().unwrap_or(""))
    });
    let field_offs: Vec<TOff> = sorted_fields
        .iter()
        .map(|f| write_field(b, f, maps))
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

fn write_rpc_call(
    b: &mut FlatBufferBuilder<'_>,
    call: &schema::RpcCall,
    objects: &[schema::Object],
    maps: &IndexMaps<'_>,
) -> TOff {
    let name = call.name.as_deref().map(|s| b.create_string(s));
    let request = call
        .request_index
        .map(|idx| write_object(b, &objects[idx as usize], maps));
    let response = call
        .response_index
        .map(|idx| write_object(b, &objects[idx as usize], maps));
    let attrs = call.attributes.as_ref().and_then(|a| write_attrs_vec(b, a));
    let doc = call
        .documentation
        .as_ref()
        .and_then(|d| write_doc_vec(b, d));

    let start = b.start_table();
    if let Some(n) = name {
        b.push_slot_always(RPCCALL_NAME, n);
    }
    if let Some(r) = request {
        b.push_slot_always(RPCCALL_REQUEST, r);
    }
    if let Some(r) = response {
        b.push_slot_always(RPCCALL_RESPONSE, r);
    }
    if let Some(a) = attrs {
        b.push_slot_always(RPCCALL_ATTRIBUTES, a);
    }
    if let Some(d) = doc {
        b.push_slot_always(RPCCALL_DOCUMENTATION, d);
    }
    b.end_table(start)
}

fn write_service(
    b: &mut FlatBufferBuilder<'_>,
    svc: &schema::Service,
    objects: &[schema::Object],
    maps: &IndexMaps<'_>,
) -> TOff {
    let fq_name = fully_qualified_svc_name(svc);
    let name = b.create_string(&fq_name);

    let call_offs: Vec<TOff> = svc
        .calls
        .iter()
        .map(|c| write_rpc_call(b, c, objects, maps))
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

fn obj_name(obj: &schema::Object) -> &str {
    obj.name.as_deref().unwrap_or("")
}

fn enum_name(e: &schema::Enum) -> &str {
    e.name.as_deref().unwrap_or("")
}

fn fully_qualified_obj_name(obj: &schema::Object) -> String {
    let name = obj.name.as_deref().unwrap_or("");
    match &obj.namespace {
        Some(ns) if !ns.namespace.as_deref().unwrap_or("").is_empty() => {
            format!("{}.{}", ns.namespace.as_deref().unwrap_or(""), name)
        }
        _ => name.to_string(),
    }
}

fn fully_qualified_enum_name(e: &schema::Enum) -> String {
    let name = e.name.as_deref().unwrap_or("");
    match &e.namespace {
        Some(ns) if !ns.namespace.as_deref().unwrap_or("").is_empty() => {
            format!("{}.{}", ns.namespace.as_deref().unwrap_or(""), name)
        }
        _ => name.to_string(),
    }
}

fn fully_qualified_svc_name(svc: &schema::Service) -> String {
    let name = svc.name.as_deref().unwrap_or("");
    match &svc.namespace {
        Some(ns) if !ns.namespace.as_deref().unwrap_or("").is_empty() => {
            format!("{}.{}", ns.namespace.as_deref().unwrap_or(""), name)
        }
        _ => name.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Deserialization: .bfbs -> Schema
// ---------------------------------------------------------------------------

/// Deserialize a .bfbs binary buffer into an owned `Schema`.
///
/// The buffer must be a valid FlatBuffer with the "BFBS" file identifier.
/// After initial conversion, a post-pass disambiguates TABLE vs STRUCT references
/// using the `is_struct` flag on each object.
pub fn deserialize_schema(buf: &[u8]) -> Result<schema::Schema, BfbsError> {
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
    resolve_rpc_indices(&mut out_services, &out_objects);

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
                    // Fallback: create a minimal Object with just the name
                    let (ns, short) = split_fq_name(rt_name);
                    let mut obj = schema::Object::new();
                    obj.name = Some(short.to_string());
                    if let Some(ns_str) = ns {
                        obj.namespace = Some(schema::Namespace {
                            namespace: Some(ns_str.to_string()),
                        });
                    }
                    (Some(obj), None)
                }
            }
        }
        None => (None, None),
    };

    let advanced_features = schema::AdvancedFeatures(root.advanced_features().bits());

    Ok(schema::Schema {
        objects: out_objects,
        enums: out_enums,
        file_ident: root.file_ident().map(|s| s.to_string()),
        file_ext: root.file_ext().map(|s| s.to_string()),
        root_table,
        root_table_index,
        services: out_services,
        advanced_features,
        fbs_files: out_fbs_files,
    })
}

/// Resolve `request_index` and `response_index` on RpcCalls by matching
/// the inline Object's fully-qualified name against the objects list.
fn resolve_rpc_indices(services: &mut [schema::Service], objects: &[schema::Object]) {
    for svc in services.iter_mut() {
        for call in svc.calls.iter_mut() {
            if let Some(ref req) = call.request {
                let fq = fully_qualified_obj_name(req);
                if let Some(idx) = objects
                    .iter()
                    .position(|o| fully_qualified_obj_name(o) == fq)
                {
                    call.request_index = Some(idx as i32);
                }
            }
            if let Some(ref resp) = call.response {
                let fq = fully_qualified_obj_name(resp);
                if let Some(idx) = objects
                    .iter()
                    .position(|o| fully_qualified_obj_name(o) == fq)
                {
                    call.response_index = Some(idx as i32);
                }
            }
        }
    }
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
        offset: if offset != 0 {
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
        fields.push(read_field(&fields_vec.get(i), is_struct_flags)?);
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
) -> Result<schema::EnumVal, BfbsError> {
    let union_type = match ev.union_type() {
        Some(ut) => Some(read_type(&ut, is_struct_flags)?),
        None => None,
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
        values.push(read_enum_val(&values_vec.get(i), is_struct_flags)?);
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

    #[test]
    fn test_serialize_empty_schema() {
        let schema = schema::Schema::new();
        let buf = serialize_schema(&schema);
        assert!(buf.len() >= 8, "buffer too small: {} bytes", buf.len());
        assert_eq!(&buf[4..8], b"BFBS", "missing BFBS file identifier");
    }

    #[test]
    fn test_serialize_minimal_schema_with_table() {
        let mut schema = schema::Schema::new();
        let mut obj = schema::Object::new();
        obj.name = Some("Monster".to_string());

        let mut field = schema::Field::new();
        field.name = Some("hp".to_string());
        field.id = Some(0);
        field.type_ = Some(schema::Type {
            base_type: Some(schema::BaseType::BASE_TYPE_SHORT),
            base_size: Some(2),
            ..Default::default()
        });
        obj.fields.push(field);

        schema.objects.push(obj.clone());
        schema.root_table = Some(obj);
        schema.root_table_index = Some(0);

        let buf = serialize_schema(&schema);
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
}
