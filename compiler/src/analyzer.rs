use crate::error::{AnalyzeError, Result};
use crate::struct_layout;
use crate::type_index::{TypeIndex, TypeRef};
use flatc_rs_parser::{ParseOutput, ParserState};
use flatc_rs_schema::{self as schema, BaseType};

/// Analyze a parsed schema, resolving type references, assigning enum values,
/// computing struct layouts, and validating correctness.
///
/// Returns a fully resolved `Schema` ready for code generation.
pub fn analyze(output: ParseOutput) -> Result<schema::Schema> {
    let ParseOutput { mut schema, state } = output;

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

    Ok(schema)
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
                .map(|o| o.is_struct == Some(true))
                .collect(),
            enum_is_union: schema
                .enums
                .iter()
                .map(|e| e.is_union == Some(true))
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
        if schema.objects[obj_idx].is_struct == Some(true) {
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
        if enum_decl.is_union == Some(true) {
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
        if schema.enums[enum_idx].is_union != Some(true) {
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
            if obj.is_struct == Some(true) {
                return Err(AnalyzeError::RootTypeMustBeTable {
                    name: root_name,
                    span: state.root_type_span.clone(),
                });
            }
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
                        if obj.is_struct == Some(true) {
                            return Err(AnalyzeError::InvalidRpcType {
                                service: svc_name.clone(),
                                method: method_name.clone(),
                                type_name: req_name,
                                span,
                            });
                        }
                        schema.services[svc_idx].calls[call_idx].request = Some(obj.clone());
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
                        if obj.is_struct == Some(true) {
                            return Err(AnalyzeError::InvalidRpcType {
                                service: svc_name.clone(),
                                method: method_name.clone(),
                                type_name: resp_name,
                                span,
                            });
                        }
                        schema.services[svc_idx].calls[call_idx].response = Some(obj.clone());
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
        if enum_def.is_union != Some(true) {
            validate_enum_underlying_type(enum_def)?;
        }

        // Check for duplicate enum value names
        validate_enum_value_names(enum_def)?;

        // Check enum values fit in underlying type
        if enum_def.is_union != Some(true) {
            validate_enum_value_ranges(enum_def)?;
        }

        // Check bit_flags enum values are valid bit positions
        if enum_def.is_union != Some(true) {
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
        let is_struct = obj.is_struct == Some(true);

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
                if field.is_deprecated == Some(true) {
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
                    if let Ok(n) = val.parse::<i64>() as std::result::Result<i64, _> {
                        if n <= 0 || (n & (n - 1)) != 0 {
                            return Err(AnalyzeError::ForceAlignNotPowerOf2 {
                                name: obj_name.to_string(),
                                value: n,
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
