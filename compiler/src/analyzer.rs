//! Semantic analysis pipeline for parsed FlatBuffers schemas.
//!
//! The analyzer transforms an unresolved `Schema` (produced by the parser, where
//! user-defined types are just name strings) into a fully resolved schema ready
//! for code generation. It runs 8 sequential steps, each depending on the
//! results of the previous:
//!
//! 1. **Build type index** -- Collect all type names into a lookup table.
//!    Detects duplicate names. After this step, every type name can be mapped
//!    to its position in `schema.objects` or `schema.enums`.
//!
//! 2. **Resolve field types** -- Walk every field in every object and resolve
//!    `unresolved_name` strings to concrete `BaseType` + `index` pairs using
//!    the type index. After this step, all `Type.index` values point to the
//!    correct object or enum.
//!
//! 2b. **Insert union type fields** -- For each union-typed field in a table,
//!     insert a companion `_type` discriminator field (an enum of the union's
//!     underlying type). Processed back-to-front to avoid index invalidation.
//!
//! 3. **Assign enum values** -- Walk every enum and assign sequential integer
//!    values to variants that lack explicit values. Validates ranges against
//!    the underlying type (e.g., byte values must fit in 0..255).
//!
//! 4. **Resolve union variant types** -- For each union, resolve variant type
//!    names to concrete object references.
//!
//! 5. **Resolve root type** -- If `root_type` was declared, find the matching
//!    table in the schema and set `schema.root_table`.
//!
//! 6. **Resolve RPC types** -- For each RPC service call, resolve the request
//!    and response type names to concrete table references.
//!
//! 7. **Validate schema** -- Run constraint checks: field ID validity, key
//!    attributes, struct field restrictions, bitflags ranges, force_align
//!    validity, etc. This is where most user-facing errors are reported.
//!
//! 8. **Compute struct layouts** -- Topologically sort structs (leaf-first),
//!    then compute byte_size, min_align, field offsets, and padding for each.
//!    Detects circular struct dependencies.

use crate::error::{AnalyzeError, Result};
use crate::struct_layout;
use crate::type_index::{TypeIndex, TypeRef};
use flatc_rs_parser::{ParseOutput, ParserState};
use flatc_rs_schema::{self as schema, Attributes, BaseType};

/// Analyze a parsed schema, resolving type references, assigning enum values,
/// computing struct layouts, and validating correctness.
///
/// Returns a fully resolved `Schema` ready for code generation.
///
/// # Errors
///
/// Returns `AnalyzeError` if any validation step fails. Errors carry `Span`
/// information (file, line, column) whenever available.
pub fn analyze(output: ParseOutput) -> Result<schema::Schema> {
    let ParseOutput { mut schema, state } = output;

    // G3.20: Validate schema size limits to prevent OOM on malicious input
    validate_schema_size_limits(&schema)?;

    // 1. Build type index (also checks for duplicate names)
    let index = TypeIndex::build(&schema)?;

    // 2. Resolve type references in object fields
    resolve_field_types(&mut schema, &index)?;

    // 2b. Insert companion _type fields for union fields in tables
    insert_union_type_fields(&mut schema);

    // 3. Assign sequential enum values
    assign_enum_values(&mut schema)?;

    // 4. Resolve union variant types
    resolve_union_types(&mut schema, &index)?;

    // 5. Resolve root type
    resolve_root_type(&mut schema, &state, &index)?;

    // 6. Resolve RPC request/response types
    resolve_rpc_types(&mut schema, &index)?;

    // 7. Validate schema constraints
    validate_schema(&schema, &state)?;

    // 8. Compute struct layouts (includes circular struct detection via topological sort)
    struct_layout::compute_struct_layouts(&mut schema)?;

    // 9. Refresh root_table from the indexed object so it reflects post-layout
    //    offsets (the earlier clone was taken before layout computation).
    if let Some(idx) = schema.root_table_index {
        schema.root_table = Some(schema.objects[idx].clone());
    }

    Ok(schema)
}

// ---------------------------------------------------------------------------
// Schema size limits (G3.20)
// ---------------------------------------------------------------------------

const MAX_OBJECTS: usize = 10_000;
const MAX_ENUMS: usize = 10_000;
const MAX_TOTAL_FIELDS: usize = 100_000;
const MAX_TOTAL_ENUM_VALUES: usize = 100_000;

fn validate_schema_size_limits(schema: &schema::Schema) -> Result<()> {
    if schema.objects.len() > MAX_OBJECTS {
        return Err(AnalyzeError::SchemaSizeLimitExceeded {
            detail: format!(
                "{} objects exceeds limit of {MAX_OBJECTS}",
                schema.objects.len()
            ),
        });
    }

    if schema.enums.len() > MAX_ENUMS {
        return Err(AnalyzeError::SchemaSizeLimitExceeded {
            detail: format!(
                "{} enums exceeds limit of {MAX_ENUMS}",
                schema.enums.len()
            ),
        });
    }

    let total_fields: usize = schema.objects.iter().map(|o| o.fields.len()).sum();
    if total_fields > MAX_TOTAL_FIELDS {
        return Err(AnalyzeError::SchemaSizeLimitExceeded {
            detail: format!(
                "{total_fields} total fields exceeds limit of {MAX_TOTAL_FIELDS}"
            ),
        });
    }

    let total_enum_values: usize = schema.enums.iter().map(|e| e.values.len()).sum();
    if total_enum_values > MAX_TOTAL_ENUM_VALUES {
        return Err(AnalyzeError::SchemaSizeLimitExceeded {
            detail: format!(
                "{total_enum_values} total enum values exceeds limit of {MAX_TOTAL_ENUM_VALUES}"
            ),
        });
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Pre-computed type metadata (avoids borrow issues during mutation)
// ---------------------------------------------------------------------------

struct TypeMetadata {
    obj_is_struct: Vec<bool>,
    enum_is_union: Vec<bool>,
    enum_underlying: Vec<Option<BaseType>>,
}

impl TypeMetadata {
    fn from_schema(schema: &schema::Schema) -> Self {
        Self {
            obj_is_struct: schema
                .objects
                .iter()
                .map(|o| o.is_struct)
                .collect(),
            enum_is_union: schema
                .enums
                .iter()
                .map(|e| e.is_union)
                .collect(),
            enum_underlying: schema
                .enums
                .iter()
                .map(|e| e.underlying_type.as_ref().and_then(|t| t.base_type))
                .collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// Type resolution
// ---------------------------------------------------------------------------

/// Resolve unresolved type references in all object fields, setting both
/// the correct `index` and `base_type`.
fn resolve_field_types(schema: &mut schema::Schema, index: &TypeIndex) -> Result<()> {
    let meta = TypeMetadata::from_schema(schema);

    for obj_idx in 0..schema.objects.len() {
        let current_ns = schema.objects[obj_idx]
            .namespace
            .as_ref()
            .and_then(|n| n.namespace.as_deref())
            .map(|s| s.to_string());
        let obj_name = schema.objects[obj_idx]
            .name
            .as_deref()
            .unwrap_or("")
            .to_string();

        for field_idx in 0..schema.objects[obj_idx].fields.len() {
            let field_name = schema.objects[obj_idx].fields[field_idx]
                .name
                .as_deref()
                .unwrap_or("")
                .to_string();

            if let Some(ty) = schema.objects[obj_idx].fields[field_idx].type_.as_mut() {
                resolve_type(
                    ty,
                    current_ns.as_deref(),
                    index,
                    &meta,
                    &obj_name,
                    &field_name,
                )?;
            }
        }
    }
    Ok(())
}

/// Insert companion `_type` fields for union fields in tables.
///
/// In FlatBuffers, each union field in a table implicitly creates a companion
/// discriminant field with a `_type` suffix. For example, `equipped: Equipment`
/// generates both `equipped_type` (u8 discriminant) and `equipped` (table offset).
///
/// This function inserts these companion fields after type resolution, adjusting
/// field IDs so the discriminant precedes its union value in the vtable.
fn insert_union_type_fields(schema: &mut schema::Schema) {
    for obj_idx in 0..schema.objects.len() {
        if schema.objects[obj_idx].is_struct {
            continue; // structs can't have union fields
        }

        // Collect existing field names to avoid collisions.
        let existing_names: std::collections::HashSet<String> = schema.objects[obj_idx]
            .fields
            .iter()
            .filter_map(|f| f.name.clone())
            .collect();

        // Collect positions where we need to insert companion fields.
        // We process from back to front to avoid invalidating indices.
        let mut insertions: Vec<(usize, schema::Field)> = Vec::new();

        for field_idx in 0..schema.objects[obj_idx].fields.len() {
            let bt = schema.objects[obj_idx].fields[field_idx]
                .type_
                .as_ref()
                .and_then(|t| t.base_type)
                .unwrap_or(BaseType::BASE_TYPE_NONE);

            if bt == BaseType::BASE_TYPE_UNION {
                let union_field = &schema.objects[obj_idx].fields[field_idx];
                let union_name = union_field.name.as_deref().unwrap_or("");
                let union_field_id = union_field.id;
                let enum_index = union_field.type_.as_ref().and_then(|t| t.index);

                // Determine underlying type of the union enum
                let underlying_bt = enum_index
                    .and_then(|idx| {
                        let idx = idx as usize;
                        if idx < schema.enums.len() {
                            schema.enums[idx]
                                .underlying_type
                                .as_ref()
                                .and_then(|t| t.base_type)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(BaseType::BASE_TYPE_U_BYTE);

                let companion_name = format!("{union_name}_type");
                // Skip if a field with this name already exists
                if existing_names.contains(&companion_name) {
                    continue;
                }

                let mut type_field = schema::Field::new();
                type_field.name = Some(companion_name);
                type_field.type_ = Some(schema::Type {
                    base_type: Some(underlying_bt),
                    base_size: Some(1),
                    index: enum_index,
                    ..schema::Type::new()
                });
                // For tables with explicit IDs, the companion type field
                // gets the ID immediately before the union value field.
                if let Some(uid) = union_field_id {
                    type_field.id = Some(uid.saturating_sub(1));
                }

                insertions.push((field_idx, type_field));
            }
        }

        // Insert from back to front to preserve indices
        for (pos, type_field) in insertions.into_iter().rev() {
            schema.objects[obj_idx].fields.insert(pos, type_field);
        }

        // Re-assign field IDs sequentially for tables (unless they have explicit IDs)
        let has_explicit_ids = schema.objects[obj_idx]
            .fields
            .iter()
            .any(|f| f.id.is_some());
        if !has_explicit_ids {
            for (i, field) in schema.objects[obj_idx].fields.iter_mut().enumerate() {
                field.id = Some(i as u32);
            }
        }
    }
}

/// Fully resolve a single Type: lookup by name, set index, and correct base_type.
fn resolve_type(
    ty: &mut schema::Type,
    current_ns: Option<&str>,
    index: &TypeIndex,
    meta: &TypeMetadata,
    obj_name: &str,
    field_name: &str,
) -> Result<()> {
    let bt = ty.base_type.unwrap_or(BaseType::BASE_TYPE_NONE);

    // Vector/array with user-defined element type
    if bt == BaseType::BASE_TYPE_VECTOR || bt == BaseType::BASE_TYPE_ARRAY {
        if let Some(name) = ty.unresolved_name.take() {
            let type_ref =
                index
                    .resolve(&name, current_ns)
                    .ok_or_else(|| AnalyzeError::UnresolvedType {
                        name: name.clone(),
                        context: format!("{obj_name}.{field_name} (element type)"),
                        span: ty.span.clone(),
                    })?;

            match type_ref {
                TypeRef::Object(idx) => {
                    ty.index = Some(idx as i32);
                    if meta.obj_is_struct[idx] {
                        ty.element_type = Some(BaseType::BASE_TYPE_STRUCT);
                    } else {
                        ty.element_type = Some(BaseType::BASE_TYPE_TABLE);
                    }
                    ty.element_size = Some(4);
                }
                TypeRef::Enum(idx) => {
                    ty.index = Some(idx as i32);
                    if meta.enum_is_union[idx] {
                        ty.element_type = Some(BaseType::BASE_TYPE_UNION);
                    } else if let Some(ubt) = meta.enum_underlying[idx] {
                        ty.element_type = Some(ubt);
                        if let Some(size) = scalar_size(ubt) {
                            ty.element_size = Some(size);
                        }
                    }
                }
            }
        }
        return Ok(());
    }

    // Direct user-defined type reference
    if let Some(name) = ty.unresolved_name.take() {
        let type_ref =
            index
                .resolve(&name, current_ns)
                .ok_or_else(|| AnalyzeError::UnresolvedType {
                    name: name.clone(),
                    context: format!("{obj_name}.{field_name}"),
                    span: ty.span.clone(),
                })?;

        match type_ref {
            TypeRef::Object(idx) => {
                ty.index = Some(idx as i32);
                if meta.obj_is_struct[idx] {
                    ty.base_type = Some(BaseType::BASE_TYPE_STRUCT);
                }
                // else TABLE is already correct
            }
            TypeRef::Enum(idx) => {
                ty.index = Some(idx as i32);
                if meta.enum_is_union[idx] {
                    ty.base_type = Some(BaseType::BASE_TYPE_UNION);
                } else if let Some(ubt) = meta.enum_underlying[idx] {
                    // Enum-typed field: use underlying base type
                    ty.base_type = Some(ubt);
                    if let Some(size) = scalar_size(ubt) {
                        ty.base_size = Some(size);
                    }
                }
            }
        }
    }

    Ok(())
}

fn scalar_size(bt: BaseType) -> Option<u32> {
    match bt {
        BaseType::BASE_TYPE_BOOL | BaseType::BASE_TYPE_BYTE | BaseType::BASE_TYPE_U_BYTE => Some(1),
        BaseType::BASE_TYPE_SHORT | BaseType::BASE_TYPE_U_SHORT => Some(2),
        BaseType::BASE_TYPE_INT | BaseType::BASE_TYPE_U_INT | BaseType::BASE_TYPE_FLOAT => Some(4),
        BaseType::BASE_TYPE_LONG | BaseType::BASE_TYPE_U_LONG | BaseType::BASE_TYPE_DOUBLE => {
            Some(8)
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Enum value assignment
// ---------------------------------------------------------------------------

/// Assign sequential values to enum variants that don't have explicit values.
fn assign_enum_values(schema: &mut schema::Schema) -> Result<()> {
    for enum_decl in &mut schema.enums {
        if enum_decl.is_union {
            continue; // union values are already assigned during parsing
        }

        let enum_name = enum_decl.name.as_deref().unwrap_or("").to_string();
        let mut next_value: i64 = 0;
        let mut seen_values = std::collections::HashSet::new();

        for eval in &mut enum_decl.values {
            if let Some(explicit) = eval.value {
                if !seen_values.insert(explicit) {
                    return Err(AnalyzeError::DuplicateEnumValue {
                        enum_name,
                        value: explicit,
                        span: eval.span.clone(),
                    });
                }
                next_value =
                    explicit
                        .checked_add(1)
                        .ok_or_else(|| AnalyzeError::EnumValueOverflow {
                            enum_name: enum_name.clone(),
                            last_value: explicit,
                        })?;
            } else {
                eval.value = Some(next_value);
                if !seen_values.insert(next_value) {
                    return Err(AnalyzeError::DuplicateEnumValue {
                        enum_name,
                        value: next_value,
                        span: eval.span.clone(),
                    });
                }
                next_value =
                    next_value
                        .checked_add(1)
                        .ok_or_else(|| AnalyzeError::EnumValueOverflow {
                            enum_name: enum_name.clone(),
                            last_value: next_value,
                        })?;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Union type resolution
// ---------------------------------------------------------------------------

fn resolve_union_types(schema: &mut schema::Schema, index: &TypeIndex) -> Result<()> {
    let meta = TypeMetadata::from_schema(schema);

    for enum_idx in 0..schema.enums.len() {
        if !schema.enums[enum_idx].is_union {
            continue;
        }

        let current_ns = schema.enums[enum_idx]
            .namespace
            .as_ref()
            .and_then(|n| n.namespace.as_deref())
            .map(|s| s.to_string());
        let union_name = schema.enums[enum_idx]
            .name
            .as_deref()
            .unwrap_or("")
            .to_string();

        for val_idx in 0..schema.enums[enum_idx].values.len() {
            if schema.enums[enum_idx].values[val_idx].name.as_deref() == Some("NONE") {
                continue;
            }

            let variant_name = schema.enums[enum_idx].values[val_idx]
                .name
                .as_deref()
                .unwrap_or("")
                .to_string();

            if let Some(union_type) = schema.enums[enum_idx].values[val_idx].union_type.as_mut() {
                if let Some(name) = union_type.unresolved_name.take() {
                    let type_ref =
                        index.resolve(&name, current_ns.as_deref()).ok_or_else(|| {
                            AnalyzeError::UnresolvedType {
                                name: name.clone(),
                                context: format!("union {union_name} variant {variant_name}"),
                                span: union_type.span.clone(),
                            }
                        })?;

                    match type_ref {
                        TypeRef::Object(idx) => {
                            union_type.index = Some(idx as i32);
                            if meta.obj_is_struct[idx] {
                                union_type.base_type = Some(BaseType::BASE_TYPE_STRUCT);
                            } else {
                                union_type.base_type = Some(BaseType::BASE_TYPE_TABLE);
                            }
                        }
                        TypeRef::Enum(_) => {
                            return Err(AnalyzeError::InvalidUnionVariant {
                                union_name,
                                variant: variant_name,
                                reason: "union variants must be tables or structs, not enums"
                                    .into(),
                                span: schema.enums[enum_idx].values[val_idx].span.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Root type resolution
// ---------------------------------------------------------------------------

fn resolve_root_type(
    schema: &mut schema::Schema,
    state: &ParserState,
    index: &TypeIndex,
) -> Result<()> {
    let root_name = match &state.root_type_name {
        Some(name) => name.clone(),
        None => return Ok(()),
    };

    let type_ref = index
        .resolve(&root_name, state.root_type_namespace.as_deref())
        .ok_or_else(|| AnalyzeError::UnresolvedType {
            name: root_name.clone(),
            context: "root_type declaration".into(),
            span: state.root_type_span.clone(),
        })?;

    match type_ref {
        TypeRef::Object(idx) => {
            let obj = &schema.objects[idx];
            if obj.is_struct {
                return Err(AnalyzeError::RootTypeMustBeTable {
                    name: root_name,
                    span: state.root_type_span.clone(),
                });
            }
            schema.root_table_index = Some(idx);
            schema.root_table = Some(obj.clone());
        }
        TypeRef::Enum(_) => {
            return Err(AnalyzeError::RootTypeMustBeTable {
                name: root_name,
                span: state.root_type_span.clone(),
            });
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// RPC type resolution
// ---------------------------------------------------------------------------

fn resolve_rpc_types(schema: &mut schema::Schema, index: &TypeIndex) -> Result<()> {
    for svc_idx in 0..schema.services.len() {
        let current_ns = schema.services[svc_idx]
            .namespace
            .as_ref()
            .and_then(|n| n.namespace.as_deref())
            .map(|s| s.to_string());
        let svc_name = schema.services[svc_idx]
            .name
            .as_deref()
            .unwrap_or("")
            .to_string();

        for call_idx in 0..schema.services[svc_idx].calls.len() {
            let method_name = schema.services[svc_idx].calls[call_idx]
                .name
                .as_deref()
                .unwrap_or("")
                .to_string();

            // Resolve request type
            let req_name = schema.services[svc_idx].calls[call_idx]
                .request
                .as_ref()
                .and_then(|r| r.name.as_deref())
                .map(|s| s.to_string());

            if let Some(req_name) = req_name {
                let call = &schema.services[svc_idx].calls[call_idx];
                let span = call.span.clone();
                let type_ref =
                    index
                        .resolve(&req_name, current_ns.as_deref())
                        .ok_or_else(|| AnalyzeError::UnresolvedType {
                            name: req_name.clone(),
                            context: format!("{svc_name}.{method_name} request"),
                            span: span.clone(),
                        })?;

                match type_ref {
                    TypeRef::Object(idx) => {
                        let obj = &schema.objects[idx];
                        if obj.is_struct {
                            return Err(AnalyzeError::InvalidRpcType {
                                service: svc_name.clone(),
                                method: method_name.clone(),
                                type_name: req_name,
                                span,
                            });
                        }
                        schema.services[svc_idx].calls[call_idx].request_index = Some(idx as i32);
                    }
                    TypeRef::Enum(_) => {
                        return Err(AnalyzeError::InvalidRpcType {
                            service: svc_name.clone(),
                            method: method_name.clone(),
                            type_name: req_name,
                            span,
                        });
                    }
                }
            }

            // Resolve response type
            let resp_name = schema.services[svc_idx].calls[call_idx]
                .response
                .as_ref()
                .and_then(|r| r.name.as_deref())
                .map(|s| s.to_string());

            if let Some(resp_name) = resp_name {
                let call = &schema.services[svc_idx].calls[call_idx];
                let span = call.span.clone();
                let type_ref = index
                    .resolve(&resp_name, current_ns.as_deref())
                    .ok_or_else(|| AnalyzeError::UnresolvedType {
                        name: resp_name.clone(),
                        context: format!("{svc_name}.{method_name} response"),
                        span: span.clone(),
                    })?;

                match type_ref {
                    TypeRef::Object(idx) => {
                        let obj = &schema.objects[idx];
                        if obj.is_struct {
                            return Err(AnalyzeError::InvalidRpcType {
                                service: svc_name.clone(),
                                method: method_name.clone(),
                                type_name: resp_name,
                                span,
                            });
                        }
                        schema.services[svc_idx].calls[call_idx].response_index = Some(idx as i32);
                    }
                    TypeRef::Enum(_) => {
                        return Err(AnalyzeError::InvalidRpcType {
                            service: svc_name.clone(),
                            method: method_name.clone(),
                            type_name: resp_name,
                            span,
                        });
                    }
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn validate_schema(schema: &schema::Schema, state: &ParserState) -> Result<()> {
    // File identifier must be exactly 4 bytes
    if let Some(ref ident) = schema.file_ident {
        if ident.len() != 4 {
            return Err(AnalyzeError::InvalidFileIdentifier {
                ident: ident.clone(),
                len: ident.len(),
                span: state.file_ident_span.clone(),
            });
        }
    }

    // Validate enum definitions
    for enum_def in &schema.enums {
        let enum_name = enum_def.name.as_deref().unwrap_or("");

        // Enum underlying type must be integer (not float/string/etc.)
        if !enum_def.is_union {
            validate_enum_underlying_type(enum_def)?;
        }

        // G3.3: Validate union NONE variant position and value (before duplicate
        // name check, so explicit NONE at wrong position is caught first)
        if enum_def.is_union {
            validate_union_none_variant(enum_def)?;
        }

        // Check for duplicate enum value names
        validate_enum_value_names(enum_def)?;

        // Check enum values fit in underlying type
        if !enum_def.is_union {
            validate_enum_value_ranges(enum_def)?;
        }

        // Check bit_flags enum values are valid bit positions
        if !enum_def.is_union {
            let is_bitflags = enum_def
                .attributes
                .as_ref()
                .map(|a| {
                    a.entries
                        .iter()
                        .any(|e| e.key.as_deref() == Some("bit_flags"))
                })
                .unwrap_or(false);
            if is_bitflags {
                validate_bitflags_values(enum_def)?;
            }
        }

        // force_align is not valid on enums
        if let Some(attrs) = enum_def.attributes.as_ref() {
            if attrs
                .entries
                .iter()
                .any(|e| e.key.as_deref() == Some("force_align"))
            {
                return Err(AnalyzeError::ForceAlignOnNonStruct {
                    name: enum_name.to_string(),
                    span: enum_def.span.clone(),
                });
            }
        }
    }

    for obj in &schema.objects {
        let obj_name = obj.name.as_deref().unwrap_or("");
        let is_struct = obj.is_struct;

        // Check for duplicate field names
        validate_duplicate_fields(obj)?;

        // Check field id consistency: all or none must have id
        let ids_present = obj.fields.iter().filter(|f| f.id.is_some()).count();
        if ids_present > 0 && ids_present < obj.fields.len() {
            return Err(AnalyzeError::MixedIdAssignment {
                table_name: obj_name.to_string(),
                span: obj.span.clone(),
            });
        }

        // G3.1: Union fields with explicit id:0 cause collision with companion _type field
        for field in &obj.fields {
            let bt = field
                .type_
                .as_ref()
                .and_then(|t| t.base_type)
                .unwrap_or(BaseType::BASE_TYPE_NONE);
            if bt == BaseType::BASE_TYPE_UNION {
                if let Some(0) = field.id {
                    return Err(AnalyzeError::UnionFieldIdZero {
                        table_name: obj_name.to_string(),
                        field_name: field.name.as_deref().unwrap_or("").to_string(),
                        span: field.span.clone(),
                    });
                }
            }
        }

        // When all fields have explicit IDs, validate them
        if ids_present == obj.fields.len() && !obj.fields.is_empty() {
            // Check for out-of-range and duplicate IDs
            let mut id_to_field: std::collections::HashMap<u32, &str> =
                std::collections::HashMap::new();
            for field in &obj.fields {
                let raw_id = field.id.unwrap();
                let fname = field.name.as_deref().unwrap_or("");
                if raw_id > 65535 {
                    return Err(AnalyzeError::FieldIdOutOfRange {
                        table_name: obj_name.to_string(),
                        field_name: fname.to_string(),
                        id: raw_id,
                        span: field.span.clone(),
                    });
                }
                if let Some(existing) = id_to_field.get(&raw_id) {
                    return Err(AnalyzeError::DuplicateFieldId {
                        table_name: obj_name.to_string(),
                        id: raw_id,
                        field_a: existing.to_string(),
                        field_b: fname.to_string(),
                        span: field.span.clone(),
                    });
                }
                id_to_field.insert(raw_id, fname);
            }
            // Check for contiguity: IDs must be 0..N-1
            let n = obj.fields.len();
            let all_present = (0..n as u32).all(|i| id_to_field.contains_key(&i));
            if !all_present {
                return Err(AnalyzeError::NonContiguousFieldIds {
                    table_name: obj_name.to_string(),
                    expected: n,
                    span: obj.span.clone(),
                });
            }
        }

        if is_struct {
            // Empty structs are invalid
            if obj.fields.is_empty() {
                return Err(AnalyzeError::EmptyStruct {
                    name: obj_name.to_string(),
                    span: obj.span.clone(),
                });
            }

            // Validate force_align is power of 2
            validate_force_align(obj)?;

            for field in &obj.fields {
                let fname = field.name.as_deref().unwrap_or("");
                // Struct fields cannot have default values
                if field.default_integer.is_some() || field.default_real.is_some() {
                    return Err(AnalyzeError::StructDefaultValue {
                        struct_name: obj_name.to_string(),
                        field_name: fname.to_string(),
                        span: field.span.clone(),
                    });
                }

                // Struct fields cannot be deprecated
                if field.is_deprecated {
                    return Err(AnalyzeError::StructDeprecatedField {
                        struct_name: obj_name.to_string(),
                        field_name: fname.to_string(),
                        span: field.span.clone(),
                    });
                }

                // Validate field type
                if let Some(ty) = field.type_.as_ref() {
                    validate_struct_field_type(obj_name, field, ty)?;
                }
            }
        } else {
            // Tables: force_align is not valid
            if let Some(attrs) = obj.attributes.as_ref() {
                if attrs
                    .entries
                    .iter()
                    .any(|e| e.key.as_deref() == Some("force_align"))
                {
                    return Err(AnalyzeError::ForceAlignOnNonStruct {
                        name: obj_name.to_string(),
                        span: obj.span.clone(),
                    });
                }
            }

            // Validate default values fit in their type
            validate_default_value_ranges(obj)?;
        }

        // G3.4: Validate key attribute constraints (applies to both structs and tables)
        validate_key_attribute(obj)?;
    }

    Ok(())
}

fn validate_enum_underlying_type(enum_def: &schema::Enum) -> Result<()> {
    let enum_name = enum_def.name.as_deref().unwrap_or("");
    if let Some(ty) = enum_def.underlying_type.as_ref() {
        let bt = ty.base_type.unwrap_or(BaseType::BASE_TYPE_NONE);
        match bt {
            BaseType::BASE_TYPE_BYTE
            | BaseType::BASE_TYPE_U_BYTE
            | BaseType::BASE_TYPE_SHORT
            | BaseType::BASE_TYPE_U_SHORT
            | BaseType::BASE_TYPE_INT
            | BaseType::BASE_TYPE_U_INT
            | BaseType::BASE_TYPE_LONG
            | BaseType::BASE_TYPE_U_LONG => Ok(()),
            _ => Err(AnalyzeError::InvalidEnumUnderlyingType {
                enum_name: enum_name.to_string(),
                span: ty.span.clone(),
            }),
        }
    } else {
        Ok(())
    }
}

fn validate_enum_value_names(enum_def: &schema::Enum) -> Result<()> {
    let enum_name = enum_def.name.as_deref().unwrap_or("");
    let mut seen = std::collections::HashSet::new();
    for val in &enum_def.values {
        let name = val.name.as_deref().unwrap_or("");
        if !seen.insert(name) {
            return Err(AnalyzeError::DuplicateEnumValueName {
                enum_name: enum_name.to_string(),
                value_name: name.to_string(),
                span: val.span.clone(),
            });
        }
    }
    Ok(())
}

fn validate_duplicate_fields(obj: &schema::Object) -> Result<()> {
    let obj_name = obj.name.as_deref().unwrap_or("");
    let mut seen = std::collections::HashSet::new();
    for field in &obj.fields {
        let name = field.name.as_deref().unwrap_or("");
        if !seen.insert(name) {
            return Err(AnalyzeError::DuplicateFieldName {
                type_name: obj_name.to_string(),
                field_name: name.to_string(),
                span: field.span.clone(),
            });
        }
    }
    Ok(())
}

fn validate_force_align(obj: &schema::Object) -> Result<()> {
    let obj_name = obj.name.as_deref().unwrap_or("");
    if let Some(attrs) = obj.attributes.as_ref() {
        for entry in &attrs.entries {
            if entry.key.as_deref() == Some("force_align") {
                if let Some(ref val) = entry.value {
                    // G3.5: Report error on unparseable values instead of silently ignoring
                    match val.parse::<i64>() {
                        Ok(n) => {
                            if n <= 0 || (n & (n - 1)) != 0 {
                                return Err(AnalyzeError::ForceAlignNotPowerOf2 {
                                    name: obj_name.to_string(),
                                    value: n,
                                    span: obj.span.clone(),
                                });
                            }
                        }
                        Err(_) => {
                            return Err(AnalyzeError::InvalidForceAlignValue {
                                name: obj_name.to_string(),
                                value: val.clone(),
                                span: obj.span.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_enum_value_ranges(enum_def: &schema::Enum) -> Result<()> {
    let enum_name = enum_def.name.as_deref().unwrap_or("");
    let bt = enum_def
        .underlying_type
        .as_ref()
        .and_then(|t| t.base_type)
        .unwrap_or(BaseType::BASE_TYPE_NONE);

    let (min, max) = match bt {
        BaseType::BASE_TYPE_BYTE => (i8::MIN as i64, i8::MAX as i64),
        BaseType::BASE_TYPE_U_BYTE => (0i64, u8::MAX as i64),
        BaseType::BASE_TYPE_SHORT => (i16::MIN as i64, i16::MAX as i64),
        BaseType::BASE_TYPE_U_SHORT => (0i64, u16::MAX as i64),
        BaseType::BASE_TYPE_INT => (i32::MIN as i64, i32::MAX as i64),
        BaseType::BASE_TYPE_U_INT => (0i64, u32::MAX as i64),
        BaseType::BASE_TYPE_LONG => (i64::MIN, i64::MAX),
        BaseType::BASE_TYPE_U_LONG => (0i64, i64::MAX), // values stored as i64
        _ => return Ok(()),
    };

    let type_name = base_type_name(bt);

    for val in &enum_def.values {
        if let Some(v) = val.value {
            if v < min || v > max {
                return Err(AnalyzeError::EnumValueOutOfRange {
                    enum_name: enum_name.to_string(),
                    value: v,
                    type_name: type_name.to_string(),
                    span: val.span.clone(),
                });
            }
        }
    }
    Ok(())
}

/// Validate that bit_flags enum values are valid bit positions for the underlying type.
fn validate_bitflags_values(enum_def: &schema::Enum) -> Result<()> {
    let enum_name = enum_def.name.as_deref().unwrap_or("");
    let bt = enum_def
        .underlying_type
        .as_ref()
        .and_then(|t| t.base_type)
        .unwrap_or(BaseType::BASE_TYPE_NONE);

    let max_bits: u32 = match bt {
        BaseType::BASE_TYPE_BYTE | BaseType::BASE_TYPE_U_BYTE => 8,
        BaseType::BASE_TYPE_SHORT | BaseType::BASE_TYPE_U_SHORT => 16,
        BaseType::BASE_TYPE_INT | BaseType::BASE_TYPE_U_INT => 32,
        BaseType::BASE_TYPE_LONG | BaseType::BASE_TYPE_U_LONG => 64,
        _ => return Ok(()),
    };

    let type_name = base_type_name(bt);

    for val in &enum_def.values {
        if let Some(v) = val.value {
            if v < 0 || v >= max_bits as i64 {
                return Err(AnalyzeError::BitFlagsValueOutOfRange {
                    enum_name: enum_name.to_string(),
                    value: v,
                    max_bits,
                    type_name: type_name.to_string(),
                });
            }
        }
    }
    Ok(())
}

fn base_type_name(bt: BaseType) -> &'static str {
    match bt {
        BaseType::BASE_TYPE_BYTE => "byte",
        BaseType::BASE_TYPE_U_BYTE => "ubyte",
        BaseType::BASE_TYPE_SHORT => "short",
        BaseType::BASE_TYPE_U_SHORT => "ushort",
        BaseType::BASE_TYPE_INT => "int",
        BaseType::BASE_TYPE_U_INT => "uint",
        BaseType::BASE_TYPE_LONG => "long",
        BaseType::BASE_TYPE_U_LONG => "ulong",
        BaseType::BASE_TYPE_FLOAT => "float",
        BaseType::BASE_TYPE_DOUBLE => "double",
        BaseType::BASE_TYPE_BOOL => "bool",
        BaseType::BASE_TYPE_STRING => "string",
        _ => "unknown",
    }
}

fn validate_default_value_ranges(obj: &schema::Object) -> Result<()> {
    let obj_name = obj.name.as_deref().unwrap_or("");

    for field in &obj.fields {
        let field_name = field.name.as_deref().unwrap_or("");
        let bt = field
            .type_
            .as_ref()
            .and_then(|t| t.base_type)
            .unwrap_or(BaseType::BASE_TYPE_NONE);

        // Only check integer defaults for scalar integer types
        if let Some(default_val) = field.default_integer {
            let range = match bt {
                BaseType::BASE_TYPE_BYTE => Some((i8::MIN as i64, i8::MAX as i64)),
                BaseType::BASE_TYPE_U_BYTE | BaseType::BASE_TYPE_BOOL => {
                    Some((0i64, u8::MAX as i64))
                }
                BaseType::BASE_TYPE_SHORT => Some((i16::MIN as i64, i16::MAX as i64)),
                BaseType::BASE_TYPE_U_SHORT => Some((0i64, u16::MAX as i64)),
                BaseType::BASE_TYPE_INT => Some((i32::MIN as i64, i32::MAX as i64)),
                BaseType::BASE_TYPE_U_INT => Some((0i64, u32::MAX as i64)),
                BaseType::BASE_TYPE_LONG => Some((i64::MIN, i64::MAX)),
                BaseType::BASE_TYPE_U_LONG => Some((0i64, i64::MAX)),
                _ => None,
            };

            if let Some((min, max)) = range {
                if default_val < min || default_val > max {
                    return Err(AnalyzeError::DefaultValueOutOfRange {
                        table_name: obj_name.to_string(),
                        field_name: field_name.to_string(),
                        value: default_val,
                        type_name: base_type_name(bt).to_string(),
                        span: field.span.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn validate_struct_field_type(
    struct_name: &str,
    field: &schema::Field,
    ty: &schema::Type,
) -> Result<()> {
    let bt = ty.base_type.unwrap_or(BaseType::BASE_TYPE_NONE);

    match bt {
        // Allowed in structs: scalars, other structs
        BaseType::BASE_TYPE_BOOL
        | BaseType::BASE_TYPE_BYTE
        | BaseType::BASE_TYPE_U_BYTE
        | BaseType::BASE_TYPE_SHORT
        | BaseType::BASE_TYPE_U_SHORT
        | BaseType::BASE_TYPE_INT
        | BaseType::BASE_TYPE_U_INT
        | BaseType::BASE_TYPE_LONG
        | BaseType::BASE_TYPE_U_LONG
        | BaseType::BASE_TYPE_FLOAT
        | BaseType::BASE_TYPE_DOUBLE
        | BaseType::BASE_TYPE_STRUCT => Ok(()),

        // Fixed arrays: allowed, but element type must also be struct-safe
        BaseType::BASE_TYPE_ARRAY => {
            let et = ty.element_type.unwrap_or(BaseType::BASE_TYPE_NONE);
            match et {
                BaseType::BASE_TYPE_BOOL
                | BaseType::BASE_TYPE_BYTE
                | BaseType::BASE_TYPE_U_BYTE
                | BaseType::BASE_TYPE_SHORT
                | BaseType::BASE_TYPE_U_SHORT
                | BaseType::BASE_TYPE_INT
                | BaseType::BASE_TYPE_U_INT
                | BaseType::BASE_TYPE_LONG
                | BaseType::BASE_TYPE_U_LONG
                | BaseType::BASE_TYPE_FLOAT
                | BaseType::BASE_TYPE_DOUBLE
                | BaseType::BASE_TYPE_STRUCT => Ok(()),
                _ => Err(AnalyzeError::InvalidStructField {
                    struct_name: struct_name.to_string(),
                    field_name: field.name.as_deref().unwrap_or("").to_string(),
                    reason: format!("array element type {:?} is not allowed in structs", et),
                    span: field.span.clone(),
                }),
            }
        }

        // Not allowed: tables, strings, vectors, unions
        _ => Err(AnalyzeError::InvalidStructField {
            struct_name: struct_name.to_string(),
            field_name: field.name.as_deref().unwrap_or("").to_string(),
            reason: format!("type {:?} is not allowed in structs", bt),
            span: field.span.clone(),
        }),
    }
}

/// G3.3: Validate union NONE variant position and value.
/// NONE must be the first variant (index 0) and have value 0.
fn validate_union_none_variant(enum_def: &schema::Enum) -> Result<()> {
    let union_name = enum_def.name.as_deref().unwrap_or("");

    for (idx, val) in enum_def.values.iter().enumerate() {
        if val.name.as_deref() == Some("NONE") {
            if idx != 0 {
                return Err(AnalyzeError::InvalidUnionNone {
                    union_name: union_name.to_string(),
                    reason: "must be the first variant (index 0)".to_string(),
                    span: val.span.clone(),
                });
            }
            if let Some(v) = val.value {
                if v != 0 {
                    return Err(AnalyzeError::InvalidUnionNone {
                        union_name: union_name.to_string(),
                        reason: format!("must have value 0, got {v}"),
                        span: val.span.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

/// G3.4: Validate key attribute constraints.
/// - At most one key field per table/struct
/// - Key field must be scalar or string (not table, vector, union, etc.)
/// - Key field cannot be deprecated
fn validate_key_attribute(obj: &schema::Object) -> Result<()> {
    let obj_name = obj.name.as_deref().unwrap_or("");
    let mut first_key: Option<&str> = None;

    for field in &obj.fields {
        if !field.is_key {
            continue;
        }
        let fname = field.name.as_deref().unwrap_or("");

        // Check for multiple keys
        if let Some(existing) = first_key {
            return Err(AnalyzeError::MultipleKeys {
                table_name: obj_name.to_string(),
                field_a: existing.to_string(),
                field_b: fname.to_string(),
                span: field.span.clone(),
            });
        }
        first_key = Some(fname);

        // Key field must be scalar or string
        if let Some(ty) = field.type_.as_ref() {
            let bt = ty.base_type.unwrap_or(BaseType::BASE_TYPE_NONE);
            match bt {
                BaseType::BASE_TYPE_BOOL
                | BaseType::BASE_TYPE_BYTE
                | BaseType::BASE_TYPE_U_BYTE
                | BaseType::BASE_TYPE_SHORT
                | BaseType::BASE_TYPE_U_SHORT
                | BaseType::BASE_TYPE_INT
                | BaseType::BASE_TYPE_U_INT
                | BaseType::BASE_TYPE_LONG
                | BaseType::BASE_TYPE_U_LONG
                | BaseType::BASE_TYPE_FLOAT
                | BaseType::BASE_TYPE_DOUBLE
                | BaseType::BASE_TYPE_STRING => {}
                _ => {
                    return Err(AnalyzeError::InvalidKeyFieldType {
                        table_name: obj_name.to_string(),
                        field_name: fname.to_string(),
                        actual_type: format!("{:?}", bt),
                        span: field.span.clone(),
                    });
                }
            }
        }

        // Key field cannot be deprecated
        if field.is_deprecated {
            return Err(AnalyzeError::DeprecatedKeyField {
                table_name: obj_name.to_string(),
                field_name: fname.to_string(),
                span: field.span.clone(),
            });
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Private leak validation (--no-leak-private-annotation)
// ---------------------------------------------------------------------------

fn has_private_attr(attrs: Option<&Attributes>) -> bool {
    attrs.is_some_and(|a| {
        a.entries
            .iter()
            .any(|kv| kv.key.as_deref() == Some("private"))
    })
}

/// Check that public types do not expose private types through their fields.
///
/// Matches C++ flatc `CheckPrivateLeak()`: for each struct/table, if it is NOT
/// private but a field's type (enum or struct/table) IS private, that's an error.
/// Similarly for union types: if a union is not private but a variant's struct is.
pub fn check_private_leak(schema: &schema::Schema) -> Result<()> {
    // Check struct/table fields
    for obj in &schema.objects {
        let obj_name = obj.name.as_deref().unwrap_or("");
        let obj_is_private = has_private_attr(obj.attributes.as_ref());

        for field in &obj.fields {
            if let Some(ref ty) = field.type_ {
                let field_type_bt = ty.base_type.unwrap_or(BaseType::BASE_TYPE_NONE);
                match field_type_bt {
                    BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => {
                        if let Some(idx) = ty.index {
                            if let Some(ref_obj) = schema.objects.get(idx as usize) {
                                if !obj_is_private && has_private_attr(ref_obj.attributes.as_ref())
                                {
                                    return Err(AnalyzeError::PrivateLeak {
                                        public_type: obj_name.to_string(),
                                        private_type: ref_obj
                                            .name
                                            .as_deref()
                                            .unwrap_or("")
                                            .to_string(),
                                    });
                                }
                            }
                        }
                    }
                    _ => {
                        // Check enum types via index
                        if let Some(idx) = ty.index {
                            if let Some(ref_enum) = schema.enums.get(idx as usize) {
                                if !ref_enum.is_union
                                    && !obj_is_private
                                    && has_private_attr(ref_enum.attributes.as_ref())
                                {
                                    return Err(AnalyzeError::PrivateLeak {
                                        public_type: obj_name.to_string(),
                                        private_type: ref_enum
                                            .name
                                            .as_deref()
                                            .unwrap_or("")
                                            .to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Check union variants
    for enum_def in &schema.enums {
        if !enum_def.is_union {
            continue;
        }
        let enum_name = enum_def.name.as_deref().unwrap_or("");
        let enum_is_private = has_private_attr(enum_def.attributes.as_ref());

        for val in &enum_def.values {
            if let Some(ref union_type) = val.union_type {
                let bt = union_type.base_type.unwrap_or(BaseType::BASE_TYPE_NONE);
                if bt == BaseType::BASE_TYPE_TABLE || bt == BaseType::BASE_TYPE_STRUCT {
                    if let Some(idx) = union_type.index {
                        if let Some(ref_obj) = schema.objects.get(idx as usize) {
                            if !enum_is_private && has_private_attr(ref_obj.attributes.as_ref()) {
                                return Err(AnalyzeError::PrivateLeak {
                                    public_type: enum_name.to_string(),
                                    private_type: ref_obj.name.as_deref().unwrap_or("").to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
