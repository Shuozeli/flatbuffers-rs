//! Resolved schema types where all analyzer-guaranteed fields are non-optional.
//!
//! These types mirror the parsed schema types in the parent module but with
//! stronger guarantees: fields that the analyzer always populates become
//! non-optional, and transient parsing artifacts (like stub Objects on RpcCall)
//! are dropped entirely.

use std::collections::HashMap;

use super::{AdvancedFeatures, Attributes, BaseType, Documentation, Namespace, SchemaFile, Span};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Error returned when a schema cannot be converted or violates a resolved invariant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveError {
    pub path: String,
    pub reason: String,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid resolved schema at '{}': {}",
            self.path, self.reason
        )
    }
}

impl std::error::Error for ResolveError {}

/// Error returned when an object name cannot be resolved uniquely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectLookupError {
    NotFound {
        name: String,
    },
    Ambiguous {
        name: String,
        candidates: Vec<String>,
    },
}

impl std::fmt::Display for ObjectLookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { name } => write!(f, "object type '{name}' not found in schema"),
            Self::Ambiguous { name, candidates } => write!(
                f,
                "object type '{name}' is ambiguous; use one of: {}",
                candidates.join(", ")
            ),
        }
    }
}

impl std::error::Error for ObjectLookupError {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IndexedObject {
    index: usize,
    fully_qualified_name: String,
}

/// Lookup table for fully-qualified and unambiguous short object names.
#[derive(Debug, Clone, Default)]
pub struct ObjectIndex {
    by_name: HashMap<String, Vec<IndexedObject>>,
}

impl ObjectIndex {
    fn insert(&mut self, name: String, object: IndexedObject) {
        let candidates = self.by_name.entry(name).or_default();
        if !candidates
            .iter()
            .any(|candidate| candidate.index == object.index)
        {
            candidates.push(object);
        }
    }

    /// Resolve an FQN or an unambiguous short object name.
    pub fn resolve(&self, name: &str) -> Result<usize, ObjectLookupError> {
        let candidates = self
            .by_name
            .get(name)
            .ok_or_else(|| ObjectLookupError::NotFound {
                name: name.to_string(),
            })?;

        if candidates.len() == 1 {
            return Ok(candidates[0].index);
        }

        Err(ObjectLookupError::Ambiguous {
            name: name.to_string(),
            candidates: candidates
                .iter()
                .map(|candidate| candidate.fully_qualified_name.clone())
                .collect(),
        })
    }
}

fn fully_qualified_name(name: &str, namespace: Option<&Namespace>) -> String {
    if name.contains('.') {
        return name.to_string();
    }

    match namespace.and_then(|namespace| namespace.namespace.as_deref()) {
        Some(namespace) if !namespace.is_empty() => format!("{namespace}.{name}"),
        _ => name.to_string(),
    }
}

fn insert_object(index: &mut ObjectIndex, object_index: usize, fqn: String) {
    let object = IndexedObject {
        index: object_index,
        fully_qualified_name: fqn.clone(),
    };
    index.insert(fqn.clone(), object.clone());
    let short_name = fqn.rsplit('.').next().unwrap_or(&fqn);
    if short_name != fqn {
        index.insert(short_name.to_string(), object);
    }
}

// ---------------------------------------------------------------------------
// ResolvedType
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedType {
    pub base_type: BaseType,
    pub base_size: Option<u32>,
    pub element_size: Option<u32>,
    pub element_type: Option<BaseType>,
    pub index: Option<i32>,
    pub fixed_length: Option<u32>,
}

impl ResolvedType {
    /// Returns the element type, defaulting to `BASE_TYPE_NONE` if not set.
    pub fn element_type_or_none(&self) -> BaseType {
        self.element_type.unwrap_or(BaseType::BASE_TYPE_NONE)
    }

    /// Convert back to the parsed `Type` representation.
    /// Used by [`ResolvedSchema::as_legacy()`] -- prefer working with
    /// `ResolvedType` directly when possible.
    pub fn to_parsed(&self) -> super::Type {
        super::Type {
            base_type: Some(self.base_type),
            base_size: self.base_size,
            element_size: self.element_size,
            element_type: self.element_type,
            index: self.index,
            fixed_length: self.fixed_length,
            unresolved_name: None,
            span: None,
        }
    }
}

// ---------------------------------------------------------------------------
// ResolvedField
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedField {
    pub name: String,
    pub type_: ResolvedType,
    pub id: Option<u32>,
    pub offset: Option<u32>,
    pub default_integer: Option<i64>,
    pub default_real: Option<f64>,
    pub default_string: Option<String>,
    pub is_deprecated: bool,
    pub is_required: bool,
    pub is_key: bool,
    pub is_optional: bool,
    pub attributes: Option<Attributes>,
    pub documentation: Option<Documentation>,
    pub padding: Option<u32>,
    pub is_offset_64: bool,
    pub span: Option<Span>,
}

// ---------------------------------------------------------------------------
// ResolvedEnumVal
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedEnumVal {
    pub name: String,
    pub value: i64,
    pub union_type: Option<ResolvedType>,
    pub documentation: Option<Documentation>,
    pub attributes: Option<Attributes>,
    pub span: Option<Span>,
}

// ---------------------------------------------------------------------------
// ResolvedEnum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedEnum {
    pub name: String,
    pub values: Vec<ResolvedEnumVal>,
    pub is_union: bool,
    pub underlying_type: ResolvedType,
    pub attributes: Option<Attributes>,
    pub documentation: Option<Documentation>,
    pub declaration_file: Option<String>,
    pub namespace: Option<Namespace>,
    pub span: Option<Span>,
}

// ---------------------------------------------------------------------------
// ResolvedObject
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedObject {
    pub name: String,
    pub fields: Vec<ResolvedField>,
    pub is_struct: bool,
    pub min_align: Option<i32>,
    pub byte_size: Option<i32>,
    pub attributes: Option<Attributes>,
    pub documentation: Option<Documentation>,
    pub declaration_file: Option<String>,
    pub namespace: Option<Namespace>,
    pub span: Option<Span>,
}

impl ResolvedObject {
    /// Return the canonical fully-qualified object name.
    pub fn fully_qualified_name(&self) -> String {
        fully_qualified_name(&self.name, self.namespace.as_ref())
    }
}

// ---------------------------------------------------------------------------
// ResolvedRpcCall
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedRpcCall {
    pub name: String,
    pub request_index: usize,
    pub response_index: usize,
    pub attributes: Option<Attributes>,
    pub documentation: Option<Documentation>,
    pub span: Option<Span>,
}

// ---------------------------------------------------------------------------
// ResolvedService
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedService {
    pub name: String,
    pub calls: Vec<ResolvedRpcCall>,
    pub attributes: Option<Attributes>,
    pub documentation: Option<Documentation>,
    pub declaration_file: Option<String>,
    pub namespace: Option<Namespace>,
    pub span: Option<Span>,
}

// ---------------------------------------------------------------------------
// ResolvedSchema
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedSchema {
    pub objects: Vec<ResolvedObject>,
    pub enums: Vec<ResolvedEnum>,
    pub file_ident: Option<String>,
    pub file_ext: Option<String>,
    pub root_table_index: Option<usize>,
    pub services: Vec<ResolvedService>,
    pub advanced_features: AdvancedFeatures,
    pub fbs_files: Vec<SchemaFile>,
}

impl ResolvedSchema {
    /// Build a lookup table containing object FQNs and short names.
    ///
    /// Short names that refer to multiple objects remain in the index and
    /// return [`ObjectLookupError::Ambiguous`] when resolved.
    pub fn build_object_index(&self) -> ObjectIndex {
        let mut index = ObjectIndex::default();
        for (i, obj) in self.objects.iter().enumerate() {
            insert_object(&mut index, i, obj.fully_qualified_name());
        }
        index
    }

    /// Validate every reference and layout invariant required by schema consumers.
    pub fn validate(&self) -> Result<(), ResolveError> {
        if let Some(index) = self.root_table_index {
            let root = self.objects.get(index).ok_or_else(|| {
                invalid(
                    "root_table_index",
                    format!(
                        "object index {index} is out of bounds for {} objects",
                        self.objects.len()
                    ),
                )
            })?;
            if root.is_struct {
                return Err(invalid(
                    "root_table_index",
                    format!("'{}' is a struct, not a table", root.name),
                ));
            }
        }

        if let Some(identifier) = &self.file_ident {
            if identifier.len() != 4 {
                return Err(invalid(
                    "file_ident",
                    format!("must be exactly 4 bytes, got {}", identifier.len()),
                ));
            }
        }

        self.validate_enums()?;
        self.validate_objects()?;
        self.validate_services()?;
        Ok(())
    }

    fn validate_enums(&self) -> Result<(), ResolveError> {
        for (enum_index, enum_def) in self.enums.iter().enumerate() {
            let path = format!("enums[{enum_index}] ({})", enum_def.name);
            if enum_def.is_union {
                if enum_def.underlying_type.base_type != BaseType::BASE_TYPE_U_TYPE {
                    return Err(invalid(
                        format!("{path}.underlying_type"),
                        "a union must use U_TYPE as its underlying type",
                    ));
                }
                for (value_index, value) in enum_def.values.iter().enumerate() {
                    let value_path = format!("{path}.values[{value_index}] ({})", value.name);
                    if value_index == 0 && value.name == "NONE" && value.value == 0 {
                        if value.union_type.is_some() {
                            return Err(invalid(
                                format!("{value_path}.union_type"),
                                "the NONE union variant must not reference an object",
                            ));
                        }
                    } else {
                        let ty = value.union_type.as_ref().ok_or_else(|| {
                            invalid(
                                format!("{value_path}.union_type"),
                                "a non-NONE union variant must reference a table or struct",
                            )
                        })?;
                        let type_path = format!("{value_path}.union_type");
                        match ty.base_type {
                            BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => {
                                self.validate_object_type(ty, &type_path)?;
                            }
                            BaseType::BASE_TYPE_STRING => require_no_index(ty, &type_path)?,
                            other => {
                                return Err(invalid(
                                    type_path,
                                    format!(
                                        "a union variant cannot use unsupported type {other:?}"
                                    ),
                                ));
                            }
                        }
                    }
                }
            } else {
                if !is_integer_type(enum_def.underlying_type.base_type) {
                    return Err(invalid(
                        format!("{path}.underlying_type"),
                        "an enum must use an integer scalar underlying type",
                    ));
                }
                if enum_def
                    .values
                    .iter()
                    .any(|value| value.union_type.is_some())
                {
                    return Err(invalid(
                        format!("{path}.values"),
                        "a non-union enum value must not have a union type",
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_objects(&self) -> Result<(), ResolveError> {
        for (object_index, object) in self.objects.iter().enumerate() {
            let path = format!("objects[{object_index}] ({})", object.name);
            if object.is_struct {
                self.validate_struct(object, &path)?;
            } else {
                self.validate_table(object, &path)?;
            }
        }
        Ok(())
    }

    fn validate_table(&self, object: &ResolvedObject, path: &str) -> Result<(), ResolveError> {
        let mut ids = vec![false; object.fields.len()];
        for (field_index, field) in object.fields.iter().enumerate() {
            let field_path = format!("{path}.fields[{field_index}] ({})", field.name);
            let id = field.id.ok_or_else(|| {
                invalid(format!("{field_path}.id"), "a table field must have an ID")
            })? as usize;
            if id >= ids.len() {
                return Err(invalid(
                    format!("{field_path}.id"),
                    format!("ID {id} is outside the contiguous range 0..{}", ids.len()),
                ));
            }
            if std::mem::replace(&mut ids[id], true) {
                return Err(invalid(
                    format!("{field_path}.id"),
                    format!("duplicate table field ID {id}"),
                ));
            }
            self.validate_field_type(&field.type_, false, &format!("{field_path}.type"))?;
        }
        Ok(())
    }

    fn validate_struct(&self, object: &ResolvedObject, path: &str) -> Result<(), ResolveError> {
        let min_align = positive_layout(object.min_align, &format!("{path}.min_align"))?;
        let byte_size = positive_layout(object.byte_size, &format!("{path}.byte_size"))?;
        if !min_align.is_power_of_two() {
            return Err(invalid(
                format!("{path}.min_align"),
                format!("alignment {min_align} is not a power of two"),
            ));
        }
        if !byte_size.is_multiple_of(min_align) {
            return Err(invalid(
                format!("{path}.byte_size"),
                format!("size {byte_size} is not aligned to {min_align}"),
            ));
        }
        if object.fields.is_empty() {
            return Err(invalid(path, "a struct must contain at least one field"));
        }

        let mut ranges = Vec::with_capacity(object.fields.len());
        for (field_index, field) in object.fields.iter().enumerate() {
            let field_path = format!("{path}.fields[{field_index}] ({})", field.name);
            let offset = field.offset.ok_or_else(|| {
                invalid(
                    format!("{field_path}.offset"),
                    "a struct field must have a byte offset",
                )
            })? as usize;
            self.validate_field_type(&field.type_, true, &format!("{field_path}.type"))?;
            let (size, align) = self.inline_layout(&field.type_, &format!("{field_path}.type"))?;
            if !offset.is_multiple_of(align) {
                return Err(invalid(
                    format!("{field_path}.offset"),
                    format!("offset {offset} is not aligned to {align}"),
                ));
            }
            let end = offset.checked_add(size).ok_or_else(|| {
                invalid(
                    format!("{field_path}.offset"),
                    "field extent overflows usize",
                )
            })?;
            if end > byte_size {
                return Err(invalid(
                    format!("{field_path}.offset"),
                    format!("field range {offset}..{end} exceeds struct size {byte_size}"),
                ));
            }
            ranges.push((offset, end, field_path));
        }
        ranges.sort_by_key(|range| range.0);
        for pair in ranges.windows(2) {
            if pair[0].1 > pair[1].0 {
                return Err(invalid(
                    format!("{}.offset", pair[1].2),
                    format!("field overlaps {}", pair[0].2),
                ));
            }
        }
        Ok(())
    }

    fn validate_field_type(
        &self,
        ty: &ResolvedType,
        in_struct: bool,
        path: &str,
    ) -> Result<(), ResolveError> {
        match ty.base_type {
            BaseType::BASE_TYPE_NONE => Err(invalid(path, "NONE is not a valid field type")),
            BaseType::BASE_TYPE_VECTOR64 => Err(invalid(
                path,
                "Vector64 is not supported by the Rust and TypeScript generators",
            )),
            BaseType::BASE_TYPE_VECTOR => {
                if in_struct {
                    return Err(invalid(path, "a struct cannot contain a vector"));
                }
                self.validate_sequence_element(ty, false, path)
            }
            BaseType::BASE_TYPE_ARRAY => {
                if !in_struct {
                    return Err(invalid(path, "a fixed array is only valid inside a struct"));
                }
                if ty.fixed_length.is_none_or(|length| length == 0) {
                    return Err(invalid(path, "a fixed array must have a positive length"));
                }
                self.validate_sequence_element(ty, true, path)
            }
            BaseType::BASE_TYPE_STRING => {
                if in_struct {
                    Err(invalid(path, "a struct cannot contain a string"))
                } else {
                    require_no_index(ty, path)
                }
            }
            BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => {
                if in_struct && ty.base_type == BaseType::BASE_TYPE_TABLE {
                    return Err(invalid(path, "a struct cannot contain a table"));
                }
                self.validate_object_type(ty, path)
            }
            BaseType::BASE_TYPE_UNION => {
                if in_struct {
                    return Err(invalid(path, "a struct cannot contain a union"));
                }
                self.validate_enum_reference(ty, true, path)
            }
            BaseType::BASE_TYPE_U_TYPE => self.validate_enum_reference(ty, true, path),
            scalar if scalar.is_scalar() => {
                if ty.index.is_some() {
                    self.validate_enum_reference(ty, false, path)
                } else {
                    Ok(())
                }
            }
            other => Err(invalid(path, format!("unsupported field type {other:?}"))),
        }
    }

    fn validate_sequence_element(
        &self,
        ty: &ResolvedType,
        is_array: bool,
        path: &str,
    ) -> Result<(), ResolveError> {
        let element = ty.element_type.ok_or_else(|| {
            invalid(
                format!("{path}.element_type"),
                "a vector or array needs an element type",
            )
        })?;
        match element {
            BaseType::BASE_TYPE_NONE
            | BaseType::BASE_TYPE_VECTOR
            | BaseType::BASE_TYPE_VECTOR64
            | BaseType::BASE_TYPE_ARRAY => Err(invalid(
                format!("{path}.element_type"),
                format!("{element:?} is not a valid sequence element"),
            )),
            BaseType::BASE_TYPE_STRING => {
                if is_array {
                    Err(invalid(path, "a fixed array cannot contain strings"))
                } else {
                    require_no_index(ty, path)
                }
            }
            BaseType::BASE_TYPE_TABLE => {
                if is_array {
                    return Err(invalid(path, "a fixed array cannot contain tables"));
                }
                self.validate_object_element(ty, false, path)
            }
            BaseType::BASE_TYPE_STRUCT => self.validate_object_element(ty, true, path),
            BaseType::BASE_TYPE_UNION => {
                if is_array {
                    return Err(invalid(path, "a fixed array cannot contain unions"));
                }
                self.validate_enum_reference(ty, true, path)
            }
            BaseType::BASE_TYPE_U_TYPE => self.validate_enum_reference(ty, true, path),
            scalar if scalar.is_scalar() => {
                if ty.index.is_some() {
                    self.validate_enum_reference_for_base(ty, scalar, false, path)
                } else {
                    Ok(())
                }
            }
            other => Err(invalid(
                format!("{path}.element_type"),
                format!("unsupported sequence element {other:?}"),
            )),
        }
    }

    fn validate_object_type(&self, ty: &ResolvedType, path: &str) -> Result<(), ResolveError> {
        self.validate_object_element(ty, ty.base_type == BaseType::BASE_TYPE_STRUCT, path)
    }

    fn validate_object_element(
        &self,
        ty: &ResolvedType,
        expect_struct: bool,
        path: &str,
    ) -> Result<(), ResolveError> {
        let index = required_index(ty, path)?;
        let object = self.objects.get(index).ok_or_else(|| {
            invalid(
                format!("{path}.index"),
                format!(
                    "object index {index} is out of bounds for {} objects",
                    self.objects.len()
                ),
            )
        })?;
        if object.is_struct != expect_struct {
            let expected = if expect_struct { "struct" } else { "table" };
            return Err(invalid(
                format!("{path}.index"),
                format!(
                    "object index {index} references '{}', which is not a {expected}",
                    object.name
                ),
            ));
        }
        Ok(())
    }

    fn validate_enum_reference(
        &self,
        ty: &ResolvedType,
        expect_union: bool,
        path: &str,
    ) -> Result<(), ResolveError> {
        self.validate_enum_reference_for_base(ty, ty.base_type, expect_union, path)
    }

    fn validate_enum_reference_for_base(
        &self,
        ty: &ResolvedType,
        referenced_base: BaseType,
        expect_union: bool,
        path: &str,
    ) -> Result<(), ResolveError> {
        let index = required_index(ty, path)?;
        let enum_def = self.enums.get(index).ok_or_else(|| {
            invalid(
                format!("{path}.index"),
                format!(
                    "enum index {index} is out of bounds for {} enums",
                    self.enums.len()
                ),
            )
        })?;
        if enum_def.is_union != expect_union {
            let expected = if expect_union { "union" } else { "enum" };
            return Err(invalid(
                format!("{path}.index"),
                format!(
                    "enum index {index} references '{}', which is not a {expected}",
                    enum_def.name
                ),
            ));
        }
        if !expect_union && enum_def.underlying_type.base_type != referenced_base {
            return Err(invalid(
                format!("{path}.index"),
                format!(
                    "enum '{}' uses {:?}, not {:?}",
                    enum_def.name, enum_def.underlying_type.base_type, referenced_base
                ),
            ));
        }
        Ok(())
    }

    fn inline_layout(&self, ty: &ResolvedType, path: &str) -> Result<(usize, usize), ResolveError> {
        match ty.base_type {
            BaseType::BASE_TYPE_STRUCT => {
                let index = required_index(ty, path)?;
                let object = &self.objects[index];
                Ok((
                    positive_layout(object.byte_size, &format!("objects[{index}].byte_size"))?,
                    positive_layout(object.min_align, &format!("objects[{index}].min_align"))?,
                ))
            }
            BaseType::BASE_TYPE_ARRAY => {
                let element = ty.element_type.ok_or_else(|| {
                    invalid(format!("{path}.element_type"), "missing array element type")
                })?;
                let length = ty
                    .fixed_length
                    .filter(|length| *length > 0)
                    .ok_or_else(|| invalid(path, "missing positive array length"))?
                    as usize;
                let (element_size, element_align) = if element == BaseType::BASE_TYPE_STRUCT {
                    let index = required_index(ty, path)?;
                    let object = &self.objects[index];
                    (
                        positive_layout(object.byte_size, &format!("objects[{index}].byte_size"))?,
                        positive_layout(object.min_align, &format!("objects[{index}].min_align"))?,
                    )
                } else {
                    let size = element.byte_size().ok_or_else(|| {
                        invalid(
                            path,
                            format!("cannot determine inline size for {element:?}"),
                        )
                    })? as usize;
                    (size, size)
                };
                let size = element_size
                    .checked_mul(length)
                    .ok_or_else(|| invalid(path, "fixed array byte size overflows usize"))?;
                Ok((size, element_align))
            }
            scalar if scalar.is_scalar() => {
                let size = scalar
                    .byte_size()
                    .ok_or_else(|| invalid(path, format!("missing byte size for {scalar:?}")))?
                    as usize;
                Ok((size, size))
            }
            other => Err(invalid(
                path,
                format!("{other:?} has no inline struct layout"),
            )),
        }
    }

    fn validate_services(&self) -> Result<(), ResolveError> {
        for (service_index, service) in self.services.iter().enumerate() {
            for (call_index, call) in service.calls.iter().enumerate() {
                let path = format!(
                    "services[{service_index}] ({}).calls[{call_index}] ({})",
                    service.name, call.name
                );
                for (label, index) in [
                    ("request_index", call.request_index),
                    ("response_index", call.response_index),
                ] {
                    let object = self.objects.get(index).ok_or_else(|| {
                        invalid(
                            format!("{path}.{label}"),
                            format!(
                                "object index {index} is out of bounds for {} objects",
                                self.objects.len()
                            ),
                        )
                    })?;
                    if object.is_struct {
                        return Err(invalid(
                            format!("{path}.{label}"),
                            format!("'{}' is a struct; RPC messages must be tables", object.name),
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

impl super::Object {
    /// Return the canonical fully-qualified object name when the parsed object
    /// has a name.
    pub fn fully_qualified_name(&self) -> Option<String> {
        self.name
            .as_deref()
            .map(|name| fully_qualified_name(name, self.namespace.as_ref()))
    }
}

impl super::Schema {
    /// Build a lookup table containing object FQNs and short names.
    pub fn build_object_index(&self) -> ObjectIndex {
        let mut index = ObjectIndex::default();
        for (i, obj) in self.objects.iter().enumerate() {
            if let Some(fqn) = obj.fully_qualified_name() {
                insert_object(&mut index, i, fqn);
            }
        }
        index
    }
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

fn invalid(path: impl Into<String>, reason: impl Into<String>) -> ResolveError {
    ResolveError {
        path: path.into(),
        reason: reason.into(),
    }
}

fn require<T>(value: Option<T>, field: &'static str, context: &str) -> Result<T, ResolveError> {
    value.ok_or_else(|| invalid(format!("{context}.{field}"), "missing required field"))
}

fn required_index(ty: &ResolvedType, path: &str) -> Result<usize, ResolveError> {
    let index = ty
        .index
        .ok_or_else(|| invalid(format!("{path}.index"), "missing required type index"))?;
    usize::try_from(index).map_err(|_| {
        invalid(
            format!("{path}.index"),
            format!("type index {index} must not be negative"),
        )
    })
}

fn require_no_index(ty: &ResolvedType, path: &str) -> Result<(), ResolveError> {
    if let Some(index) = ty.index {
        Err(invalid(
            format!("{path}.index"),
            format!("unexpected type index {index}"),
        ))
    } else {
        Ok(())
    }
}

fn positive_layout(value: Option<i32>, path: &str) -> Result<usize, ResolveError> {
    let value = value.ok_or_else(|| invalid(path, "missing required layout value"))?;
    usize::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| invalid(path, format!("layout value {value} must be positive")))
}

fn is_integer_type(base_type: BaseType) -> bool {
    matches!(
        base_type,
        BaseType::BASE_TYPE_BYTE
            | BaseType::BASE_TYPE_U_BYTE
            | BaseType::BASE_TYPE_SHORT
            | BaseType::BASE_TYPE_U_SHORT
            | BaseType::BASE_TYPE_INT
            | BaseType::BASE_TYPE_U_INT
            | BaseType::BASE_TYPE_LONG
            | BaseType::BASE_TYPE_U_LONG
    )
}

// ---------------------------------------------------------------------------
// Conversion: Type -> ResolvedType
// ---------------------------------------------------------------------------

impl ResolvedType {
    fn try_from_parsed(t: &super::Type, context: &str) -> Result<Self, ResolveError> {
        Ok(Self {
            base_type: require(t.base_type, "base_type", context)?,
            base_size: t.base_size,
            element_size: t.element_size,
            element_type: t.element_type,
            index: t.index,
            fixed_length: t.fixed_length,
        })
    }
}

// ---------------------------------------------------------------------------
// Conversion: Field -> ResolvedField
// ---------------------------------------------------------------------------

impl ResolvedField {
    fn try_from_parsed(f: &super::Field, parent: &str) -> Result<Self, ResolveError> {
        let name = require(f.name.clone(), "name", &format!("Field in {parent}"))?;
        let context = format!("Field '{name}' in {parent}");
        let type_ = match &f.type_ {
            Some(t) => ResolvedType::try_from_parsed(t, &context)?,
            None => return Err(invalid(format!("{context}.type"), "missing required field")),
        };
        Ok(Self {
            name,
            type_,
            id: f.id,
            offset: f.offset,
            default_integer: f.default_integer,
            default_real: f.default_real,
            default_string: f.default_string.clone(),
            is_deprecated: f.is_deprecated,
            is_required: f.is_required,
            is_key: f.is_key,
            is_optional: f.is_optional,
            attributes: f.attributes.clone(),
            documentation: f.documentation.clone(),
            padding: f.padding,
            is_offset_64: f.is_offset_64,
            span: f.span.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Conversion: EnumVal -> ResolvedEnumVal
// ---------------------------------------------------------------------------

impl ResolvedEnumVal {
    fn try_from_parsed(v: &super::EnumVal, parent: &str) -> Result<Self, ResolveError> {
        let name = require(v.name.clone(), "name", &format!("EnumVal in {parent}"))?;
        let context = format!("EnumVal '{name}' in {parent}");
        let value = require(v.value, "value", &context)?;
        let union_type = v
            .union_type
            .as_ref()
            .map(|t| ResolvedType::try_from_parsed(t, &context))
            .transpose()?;

        Ok(Self {
            name,
            value,
            union_type,
            documentation: v.documentation.clone(),
            attributes: v.attributes.clone(),
            span: v.span.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Conversion: Enum -> ResolvedEnum
// ---------------------------------------------------------------------------

impl ResolvedEnum {
    fn try_from_parsed(e: &super::Enum) -> Result<Self, ResolveError> {
        let name = require(e.name.clone(), "name", "Enum")?;
        let context = format!("Enum '{name}'");
        let underlying_type = match &e.underlying_type {
            Some(t) => ResolvedType::try_from_parsed(t, &context)?,
            None => {
                return Err(invalid(
                    format!("{context}.underlying_type"),
                    "missing required field",
                ));
            }
        };
        let values = e
            .values
            .iter()
            .map(|v| ResolvedEnumVal::try_from_parsed(v, &context))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            name,
            values,
            is_union: e.is_union,
            underlying_type,
            attributes: e.attributes.clone(),
            documentation: e.documentation.clone(),
            declaration_file: e.declaration_file.clone(),
            namespace: e.namespace.clone(),
            span: e.span.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Conversion: Object -> ResolvedObject
// ---------------------------------------------------------------------------

impl ResolvedObject {
    fn try_from_parsed(o: &super::Object) -> Result<Self, ResolveError> {
        let name = require(o.name.clone(), "name", "Object")?;
        let context = format!("Object '{name}'");
        let fields = o
            .fields
            .iter()
            .map(|f| ResolvedField::try_from_parsed(f, &context))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            name,
            fields,
            is_struct: o.is_struct,
            min_align: o.min_align,
            byte_size: o.byte_size,
            attributes: o.attributes.clone(),
            documentation: o.documentation.clone(),
            declaration_file: o.declaration_file.clone(),
            namespace: o.namespace.clone(),
            span: o.span.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Conversion: RpcCall -> ResolvedRpcCall
// ---------------------------------------------------------------------------

impl ResolvedRpcCall {
    fn try_from_parsed(c: &super::RpcCall, parent: &str) -> Result<Self, ResolveError> {
        let name = require(c.name.clone(), "name", &format!("RpcCall in {parent}"))?;
        let context = format!("RpcCall '{name}' in {parent}");
        let request_index = require(c.request_index, "request_index", &context)?;
        let response_index = require(c.response_index, "response_index", &context)?;

        // Convert i32 indices to usize, failing on negative values.
        let request_index = usize::try_from(request_index).map_err(|_| {
            invalid(
                format!("{context}.request_index"),
                format!("index {request_index} must not be negative"),
            )
        })?;
        let response_index = usize::try_from(response_index).map_err(|_| {
            invalid(
                format!("{context}.response_index"),
                format!("index {response_index} must not be negative"),
            )
        })?;

        Ok(Self {
            name,
            request_index,
            response_index,
            attributes: c.attributes.clone(),
            documentation: c.documentation.clone(),
            span: c.span.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Conversion: Service -> ResolvedService
// ---------------------------------------------------------------------------

impl ResolvedService {
    fn try_from_parsed(s: &super::Service) -> Result<Self, ResolveError> {
        let name = require(s.name.clone(), "name", "Service")?;
        let context = format!("Service '{name}'");
        let calls = s
            .calls
            .iter()
            .map(|c| ResolvedRpcCall::try_from_parsed(c, &context))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            name,
            calls,
            attributes: s.attributes.clone(),
            documentation: s.documentation.clone(),
            declaration_file: s.declaration_file.clone(),
            namespace: s.namespace.clone(),
            span: s.span.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Conversion: Schema -> ResolvedSchema
// ---------------------------------------------------------------------------

impl ResolvedSchema {
    /// Convert this resolved schema back to the legacy parsed `Schema` type.
    ///
    /// This is a lossy conversion used for backward compatibility with code
    /// that has not yet been migrated to accept `ResolvedSchema` directly
    /// (e.g., codegen, JSON conversion, BFBS serialization). The transient
    /// parsing artifacts like `RpcCall.request`/`response` stub objects are
    /// not restored; callers should use index-based lookups instead.
    pub fn as_legacy(&self) -> Result<super::Schema, ResolveError> {
        self.validate()?;
        let objects = self
            .objects
            .iter()
            .map(|o| super::Object {
                name: Some(o.name.clone()),
                fields: o
                    .fields
                    .iter()
                    .map(|f| super::Field {
                        name: Some(f.name.clone()),
                        type_: Some(f.type_.to_parsed()),
                        id: f.id,
                        offset: f.offset,
                        default_integer: f.default_integer,
                        default_real: f.default_real,
                        default_string: f.default_string.clone(),
                        is_deprecated: f.is_deprecated,
                        is_required: f.is_required,
                        is_key: f.is_key,
                        is_optional: f.is_optional,
                        attributes: f.attributes.clone(),
                        documentation: f.documentation.clone(),
                        padding: f.padding,
                        is_offset_64: f.is_offset_64,
                        span: f.span.clone(),
                    })
                    .collect(),
                is_struct: o.is_struct,
                min_align: o.min_align,
                byte_size: o.byte_size,
                attributes: o.attributes.clone(),
                documentation: o.documentation.clone(),
                declaration_file: o.declaration_file.clone(),
                namespace: o.namespace.clone(),
                span: o.span.clone(),
            })
            .collect::<Vec<_>>();

        let enums = self
            .enums
            .iter()
            .map(|e| super::Enum {
                name: Some(e.name.clone()),
                values: e
                    .values
                    .iter()
                    .map(|v| super::EnumVal {
                        name: Some(v.name.clone()),
                        value: Some(v.value),
                        union_type: v.union_type.as_ref().map(|t| t.to_parsed()),
                        documentation: v.documentation.clone(),
                        attributes: v.attributes.clone(),
                        span: v.span.clone(),
                    })
                    .collect(),
                is_union: e.is_union,
                underlying_type: Some(e.underlying_type.to_parsed()),
                attributes: e.attributes.clone(),
                documentation: e.documentation.clone(),
                declaration_file: e.declaration_file.clone(),
                namespace: e.namespace.clone(),
                span: e.span.clone(),
            })
            .collect::<Vec<_>>();

        let services = self
            .services
            .iter()
            .map(|s| {
                let calls = s
                    .calls
                    .iter()
                    .map(|c| {
                        let request_index = i32::try_from(c.request_index).map_err(|_| {
                            invalid(
                                format!("RpcCall '{}'.request_index", c.name),
                                format!("index {} overflows i32", c.request_index),
                            )
                        })?;
                        let response_index = i32::try_from(c.response_index).map_err(|_| {
                            invalid(
                                format!("RpcCall '{}'.response_index", c.name),
                                format!("index {} overflows i32", c.response_index),
                            )
                        })?;
                        Ok(super::RpcCall {
                            name: Some(c.name.clone()),
                            request_index: Some(request_index),
                            response_index: Some(response_index),
                            request: None,
                            response: None,
                            attributes: c.attributes.clone(),
                            documentation: c.documentation.clone(),
                            span: c.span.clone(),
                        })
                    })
                    .collect::<Result<Vec<_>, ResolveError>>()?;
                Ok(super::Service {
                    name: Some(s.name.clone()),
                    calls,
                    attributes: s.attributes.clone(),
                    documentation: s.documentation.clone(),
                    declaration_file: s.declaration_file.clone(),
                    namespace: s.namespace.clone(),
                    span: s.span.clone(),
                })
            })
            .collect::<Result<Vec<_>, ResolveError>>()?;

        let root_table = self.root_table_index.map(|idx| objects[idx].clone());

        Ok(super::Schema {
            objects,
            enums,
            file_ident: self.file_ident.clone(),
            file_ext: self.file_ext.clone(),
            root_table,
            root_table_index: self.root_table_index,
            services,
            advanced_features: self.advanced_features,
            fbs_files: self.fbs_files.clone(),
        })
    }

    /// Convert a parsed `Schema` into a `ResolvedSchema`, returning an error
    /// if any required field is missing (i.e., the schema has not been fully
    /// analyzed).
    pub fn try_from_parsed(schema: &super::Schema) -> Result<Self, ResolveError> {
        let objects = schema
            .objects
            .iter()
            .map(ResolvedObject::try_from_parsed)
            .collect::<Result<Vec<_>, _>>()?;

        let enums = schema
            .enums
            .iter()
            .map(ResolvedEnum::try_from_parsed)
            .collect::<Result<Vec<_>, _>>()?;

        let services = schema
            .services
            .iter()
            .map(ResolvedService::try_from_parsed)
            .collect::<Result<Vec<_>, _>>()?;

        let resolved = Self {
            objects,
            enums,
            file_ident: schema.file_ident.clone(),
            file_ext: schema.file_ext.clone(),
            root_table_index: schema.root_table_index,
            services,
            advanced_features: schema.advanced_features,
            fbs_files: schema.fbs_files.clone(),
        };
        resolved.validate()?;
        Ok(resolved)
    }
}

#[cfg(test)]
mod object_index_tests {
    use super::*;

    fn object(name: &str, namespace: Option<&str>) -> ResolvedObject {
        ResolvedObject {
            name: name.to_string(),
            fields: Vec::new(),
            is_struct: false,
            min_align: None,
            byte_size: None,
            attributes: None,
            documentation: None,
            declaration_file: None,
            namespace: namespace.map(|namespace| Namespace {
                namespace: Some(namespace.to_string()),
            }),
            span: None,
        }
    }

    fn schema(objects: Vec<ResolvedObject>) -> ResolvedSchema {
        ResolvedSchema {
            objects,
            enums: Vec::new(),
            file_ident: None,
            file_ext: None,
            root_table_index: None,
            services: Vec::new(),
            advanced_features: AdvancedFeatures::default(),
            fbs_files: Vec::new(),
        }
    }

    #[test]
    fn resolves_nested_fqns_and_rejects_ambiguous_short_names() {
        let schema = schema(vec![
            object("Root", Some("A.Nested")),
            object("Root", Some("B")),
        ]);
        let index = schema.build_object_index();

        assert_eq!(index.resolve("A.Nested.Root"), Ok(0));
        assert_eq!(index.resolve("B.Root"), Ok(1));
        assert_eq!(
            index.resolve("Root"),
            Err(ObjectLookupError::Ambiguous {
                name: "Root".to_string(),
                candidates: vec!["A.Nested.Root".to_string(), "B.Root".to_string()],
            })
        );
    }

    #[test]
    fn preserves_names_that_are_already_fully_qualified() {
        let schema = schema(vec![object("A.Nested.Root", Some("A.Nested"))]);
        let index = schema.build_object_index();

        assert_eq!(index.resolve("A.Nested.Root"), Ok(0));
        assert_eq!(index.resolve("Root"), Ok(0));
        assert!(matches!(
            index.resolve("A.Nested.A.Nested.Root"),
            Err(ObjectLookupError::NotFound { .. })
        ));
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    fn ty(base_type: BaseType) -> ResolvedType {
        ResolvedType {
            base_type,
            base_size: base_type.byte_size(),
            element_size: None,
            element_type: None,
            index: None,
            fixed_length: None,
        }
    }

    fn field(
        name: &str,
        type_: ResolvedType,
        id: Option<u32>,
        offset: Option<u32>,
    ) -> ResolvedField {
        ResolvedField {
            name: name.to_string(),
            type_,
            id,
            offset,
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
        }
    }

    fn object(name: &str, fields: Vec<ResolvedField>, is_struct: bool) -> ResolvedObject {
        ResolvedObject {
            name: name.to_string(),
            fields,
            is_struct,
            min_align: is_struct.then_some(4),
            byte_size: is_struct.then_some(4),
            attributes: None,
            documentation: None,
            declaration_file: None,
            namespace: None,
            span: None,
        }
    }

    fn schema() -> ResolvedSchema {
        let root = object(
            "Root",
            vec![field("value", ty(BaseType::BASE_TYPE_INT), Some(0), None)],
            false,
        );
        let point = object(
            "Point",
            vec![field("x", ty(BaseType::BASE_TYPE_INT), None, Some(0))],
            true,
        );
        let color = ResolvedEnum {
            name: "Color".to_string(),
            values: vec![ResolvedEnumVal {
                name: "Red".to_string(),
                value: 0,
                union_type: None,
                documentation: None,
                attributes: None,
                span: None,
            }],
            is_union: false,
            underlying_type: ty(BaseType::BASE_TYPE_INT),
            attributes: None,
            documentation: None,
            declaration_file: None,
            namespace: None,
            span: None,
        };
        let any = ResolvedEnum {
            name: "Any".to_string(),
            values: vec![
                ResolvedEnumVal {
                    name: "NONE".to_string(),
                    value: 0,
                    union_type: None,
                    documentation: None,
                    attributes: None,
                    span: None,
                },
                ResolvedEnumVal {
                    name: "Root".to_string(),
                    value: 1,
                    union_type: Some(ResolvedType {
                        index: Some(0),
                        ..ty(BaseType::BASE_TYPE_TABLE)
                    }),
                    documentation: None,
                    attributes: None,
                    span: None,
                },
            ],
            is_union: true,
            underlying_type: ty(BaseType::BASE_TYPE_U_TYPE),
            attributes: None,
            documentation: None,
            declaration_file: None,
            namespace: None,
            span: None,
        };
        ResolvedSchema {
            objects: vec![root, point],
            enums: vec![color, any],
            file_ident: Some("TEST".to_string()),
            file_ext: None,
            root_table_index: Some(0),
            services: vec![ResolvedService {
                name: "Api".to_string(),
                calls: vec![ResolvedRpcCall {
                    name: "Get".to_string(),
                    request_index: 0,
                    response_index: 0,
                    attributes: None,
                    documentation: None,
                    span: None,
                }],
                attributes: None,
                documentation: None,
                declaration_file: None,
                namespace: None,
                span: None,
            }],
            advanced_features: AdvancedFeatures::default(),
            fbs_files: Vec::new(),
        }
    }

    fn assert_invalid(schema: ResolvedSchema, expected: &str) {
        let result = std::panic::catch_unwind(|| schema.validate());
        let error = result.expect("validation must not panic").unwrap_err();
        assert!(
            error.to_string().contains(expected),
            "expected '{expected}' in '{error}'"
        );
    }

    #[test]
    fn accepts_a_complete_resolved_schema() {
        assert_eq!(schema().validate(), Ok(()));
    }

    #[test]
    fn rejects_missing_and_negative_struct_layout() {
        let mut missing = schema();
        missing.objects[1].min_align = None;
        assert_invalid(missing, "min_align");

        let mut negative = schema();
        negative.objects[1].byte_size = Some(-4);
        assert_invalid(negative, "must be positive");
    }

    #[test]
    fn requires_table_ids_and_struct_offsets() {
        let mut table = schema();
        table.objects[0].fields[0].id = None;
        assert_invalid(table, "table field must have an ID");

        let mut strukt = schema();
        strukt.objects[1].fields[0].offset = None;
        assert_invalid(strukt, "struct field must have a byte offset");
    }

    #[test]
    fn rejects_negative_and_out_of_bounds_type_indices() {
        let mut negative = schema();
        negative.objects[0].fields[0].type_ = ResolvedType {
            index: Some(-1),
            ..ty(BaseType::BASE_TYPE_TABLE)
        };
        assert_invalid(negative, "must not be negative");

        let mut negative_enum = schema();
        negative_enum.objects[0].fields[0].type_.index = Some(-1);
        assert_invalid(negative_enum, "must not be negative");

        let mut object_oob = schema();
        object_oob.objects[0].fields[0].type_ = ResolvedType {
            index: Some(99),
            ..ty(BaseType::BASE_TYPE_TABLE)
        };
        assert_invalid(object_oob, "object index 99");

        let mut enum_oob = schema();
        enum_oob.objects[0].fields[0].type_.index = Some(99);
        assert_invalid(enum_oob, "enum index 99");
    }

    #[test]
    fn rejects_invalid_vector_array_union_and_rpc_references() {
        let mut vector = schema();
        vector.objects[0].fields[0].type_ = ty(BaseType::BASE_TYPE_VECTOR);
        assert_invalid(vector, "needs an element type");

        let mut array = schema();
        array.objects[1].fields[0].type_ = ResolvedType {
            element_type: Some(BaseType::BASE_TYPE_INT),
            fixed_length: Some(0),
            ..ty(BaseType::BASE_TYPE_ARRAY)
        };
        assert_invalid(array, "positive length");

        let mut union = schema();
        union.enums[1].values[1].union_type.as_mut().unwrap().index = Some(1);
        assert_invalid(union, "not a table");

        let mut rpc = schema();
        rpc.services[0].calls[0].response_index = 2;
        assert_invalid(rpc, "object index 2");
    }

    #[test]
    fn rejects_contextually_invalid_and_unsupported_base_types() {
        let mut string_struct = schema();
        string_struct.objects[1].fields[0].type_ = ty(BaseType::BASE_TYPE_STRING);
        assert_invalid(string_struct, "struct cannot contain a string");

        let mut vector64 = schema();
        vector64.objects[0].fields[0].type_ = ty(BaseType::BASE_TYPE_VECTOR64);
        assert_invalid(vector64, "Vector64 is not supported");
    }
}
