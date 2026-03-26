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

/// Error returned when converting a parsed schema to resolved types.
#[derive(Debug, Clone)]
pub struct ResolveError {
    pub field: &'static str,
    pub context: String,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "missing required field '{}' on {}",
            self.field, self.context
        )
    }
}

impl std::error::Error for ResolveError {}

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
    /// Build a lookup table mapping object names (both FQN and short names)
    /// to their index in `self.objects`.
    ///
    /// For objects with dotted names (e.g., "MyGame.Sample.Monster"), both
    /// the FQN and the short name ("Monster") are indexed. The first entry
    /// wins for duplicate short names.
    pub fn build_object_index(&self) -> HashMap<&str, usize> {
        let mut index = HashMap::new();
        for (i, obj) in self.objects.iter().enumerate() {
            let name = obj.name.as_str();
            index.entry(name).or_insert(i);
            if let Some(short) = name.rsplit('.').next() {
                if short != name {
                    index.entry(short).or_insert(i);
                }
            }
        }
        index
    }
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

fn require<T>(value: Option<T>, field: &'static str, context: &str) -> Result<T, ResolveError> {
    value.ok_or_else(|| ResolveError {
        field,
        context: context.to_string(),
    })
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
            None => {
                return Err(ResolveError {
                    field: "type",
                    context,
                });
            }
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
                return Err(ResolveError {
                    field: "underlying_type",
                    context,
                });
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
        let request_index = usize::try_from(request_index).map_err(|_| ResolveError {
            field: "request_index",
            context: format!("{context}: negative index {request_index}"),
        })?;
        let response_index = usize::try_from(response_index).map_err(|_| ResolveError {
            field: "response_index",
            context: format!("{context}: negative index {response_index}"),
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
                        let request_index =
                            i32::try_from(c.request_index).map_err(|_| ResolveError {
                                field: "request_index",
                                context: format!(
                                    "RpcCall '{}': index {} overflows i32",
                                    c.name, c.request_index
                                ),
                            })?;
                        let response_index =
                            i32::try_from(c.response_index).map_err(|_| ResolveError {
                                field: "response_index",
                                context: format!(
                                    "RpcCall '{}': index {} overflows i32",
                                    c.name, c.response_index
                                ),
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

        Ok(Self {
            objects,
            enums,
            file_ident: schema.file_ident.clone(),
            file_ext: schema.file_ext.clone(),
            root_table_index: schema.root_table_index,
            services,
            advanced_features: schema.advanced_features,
            fbs_files: schema.fbs_files.clone(),
        })
    }
}
