//! FlatBuffers schema adapter for codegen-schema.
//!
//! Converts from [`flatc_rs_schema::resolved::ResolvedSchema`] to the common
//! schema definition types.

use codegen_core::CodeGenError;
use codegen_schema::{
    EnumDef, EnumValue, FieldDef, MessageDef, MethodDef, OneOfVariant, ScalarType, SchemaDef,
    ServiceDef, StreamingType, Type,
};

use flatc_rs_schema::resolved::{ResolvedSchema, ResolvedType};
use flatc_rs_schema::BaseType;

// ---------------------------------------------------------------------------
// BaseType -> ScalarType
// ---------------------------------------------------------------------------

fn base_type_to_scalar(bt: BaseType) -> Option<ScalarType> {
    match bt {
        BaseType::BASE_TYPE_BOOL => Some(ScalarType::Bool),
        BaseType::BASE_TYPE_BYTE => Some(ScalarType::Int8),
        BaseType::BASE_TYPE_U_BYTE => Some(ScalarType::Uint8),
        BaseType::BASE_TYPE_SHORT => Some(ScalarType::Int16),
        BaseType::BASE_TYPE_U_SHORT => Some(ScalarType::Uint16),
        BaseType::BASE_TYPE_INT => Some(ScalarType::Int32),
        BaseType::BASE_TYPE_U_INT => Some(ScalarType::Uint32),
        BaseType::BASE_TYPE_LONG => Some(ScalarType::Int64),
        BaseType::BASE_TYPE_U_LONG => Some(ScalarType::Uint64),
        BaseType::BASE_TYPE_FLOAT => Some(ScalarType::Float32),
        BaseType::BASE_TYPE_DOUBLE => Some(ScalarType::Float64),
        // BASE_TYPE_STRING is NOT a scalar in FlatBuffers (it's an offset type)
        // but we handle it here for convenience
        BaseType::BASE_TYPE_STRING => Some(ScalarType::String),
        // These are handled elsewhere
        BaseType::BASE_TYPE_VECTOR | BaseType::BASE_TYPE_VECTOR64 | BaseType::BASE_TYPE_ARRAY => {
            None
        }
        BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => None,
        BaseType::BASE_TYPE_UNION => None,
        BaseType::BASE_TYPE_NONE | BaseType::BASE_TYPE_U_TYPE => None,
    }
}

// ---------------------------------------------------------------------------
// ResolvedType -> Schema Type
// ---------------------------------------------------------------------------

fn convert_type(rt: &ResolvedType, schema: &ResolvedSchema) -> Result<Type, CodeGenError> {
    use flatc_rs_schema::BaseType as B;

    // Handle scalar types
    if rt.base_type.is_scalar() {
        if let Some(scalar) = base_type_to_scalar(rt.base_type) {
            return Ok(Type::Scalar(scalar));
        }
    }

    // Handle string (not a scalar in FlatBuffers but we treat it as one)
    if rt.base_type == B::BASE_TYPE_STRING {
        return Ok(Type::Scalar(ScalarType::String));
    }

    // Handle vectors
    if matches!(rt.base_type, B::BASE_TYPE_VECTOR | B::BASE_TYPE_VECTOR64) {
        let element_type = rt.element_type.unwrap_or(B::BASE_TYPE_NONE);
        let element = convert_type(
            &ResolvedType {
                base_type: element_type,
                base_size: rt.base_size,
                element_size: rt.element_size,
                element_type: None,
                index: rt.index,
                fixed_length: rt.fixed_length,
            },
            schema,
        )?;
        return Ok(Type::Vector(Box::new(element)));
    }

    // Handle arrays
    if matches!(rt.base_type, B::BASE_TYPE_ARRAY) {
        let element_type = rt.element_type.unwrap_or(B::BASE_TYPE_NONE);
        let element = convert_type(
            &ResolvedType {
                base_type: element_type,
                base_size: rt.base_size,
                element_size: rt.element_size,
                element_type: None,
                index: rt.index,
                fixed_length: rt.fixed_length,
            },
            schema,
        )?;
        return Ok(Type::Vector(Box::new(element)));
    }

    // Handle union types
    if rt.base_type == B::BASE_TYPE_UNION {
        if let Some(idx) = rt.index {
            let idx_usize = usize::try_from(idx).map_err(|_| {
                CodeGenError::Internal(format!("enum index out of bounds: {}", idx as usize))
            })?;

            if idx_usize >= schema.enums.len() {
                return Err(CodeGenError::Internal(format!(
                    "enum index out of bounds: {}",
                    idx_usize
                )));
            }

            let union_enum = &schema.enums[idx_usize];
            let variants: Vec<OneOfVariant> = union_enum
                .values
                .iter()
                .filter_map(|v| {
                    let ty = if let Some(ref type_resolved) = v.union_type {
                        let type_idx = type_resolved.index?;
                        let type_idx_usize = usize::try_from(type_idx).ok()?;
                        if type_idx_usize >= schema.objects.len() {
                            return None;
                        }
                        let obj = &schema.objects[type_idx_usize];
                        let namespace = obj.namespace.as_ref().and_then(|n| n.namespace.clone());
                        Type::Message {
                            name: obj.name.clone(),
                            package: namespace,
                        }
                    } else {
                        return None;
                    };
                    Some(OneOfVariant {
                        name: v.name.clone(),
                        ty,
                    })
                })
                .collect();

            if variants.is_empty() {
                return Err(CodeGenError::Internal(
                    "union has no variants with types".to_string(),
                ));
            }

            return Ok(Type::OneOf {
                name: union_enum.name.clone(),
                variants,
            });
        }
    }

    // Handle tables/structs by index - resolve actual type name
    if let Some(idx) = rt.index {
        let idx_usize = usize::try_from(idx).map_err(|_| {
            CodeGenError::Internal(format!("object index out of bounds: {}", idx as usize))
        })?;

        if idx_usize >= schema.objects.len() {
            return Err(CodeGenError::Internal(format!(
                "object index out of bounds: {}",
                idx_usize
            )));
        }

        let obj = &schema.objects[idx_usize];
        let namespace = obj.namespace.as_ref().and_then(|n| n.namespace.clone());

        return Ok(Type::Message {
            name: obj.name.clone(),
            package: namespace,
        });
    }

    Err(CodeGenError::Internal(format!(
        "unsupported base type: {:?}",
        rt.base_type
    )))
}

// ---------------------------------------------------------------------------
// Streaming attribute parsing
// ---------------------------------------------------------------------------

/// Parse streaming mode from FlatBuffers attributes.
///
/// FlatBuffers uses `streaming: "server"`, `streaming: "client"`,
/// or `streaming: "bidi"` key-value attributes on RPC methods.
fn parse_streaming(attrs: &Option<flatc_rs_schema::Attributes>) -> StreamingType {
    let attrs = match attrs {
        Some(a) => a,
        None => return StreamingType::None,
    };

    for kv in &attrs.entries {
        if kv.key.as_deref() == Some("streaming") {
            return match kv.value.as_deref() {
                Some("server") => StreamingType::Server,
                Some("client") => StreamingType::Client,
                Some("bidi") => StreamingType::BiDi,
                _ => StreamingType::None,
            };
        }
    }

    StreamingType::None
}

// ---------------------------------------------------------------------------
// ResolvedSchema -> SchemaDef
// ---------------------------------------------------------------------------

/// Convert a ResolvedSchema to a SchemaDef.
pub fn from_resolved_schema(schema: &ResolvedSchema) -> Result<SchemaDef, CodeGenError> {
    let messages: Vec<MessageDef> = schema
        .objects
        .iter()
        .map(|o| {
            let namespace = o.namespace.as_ref().and_then(|n| n.namespace.clone());
            let comments = o
                .documentation
                .as_ref()
                .map(|d| d.lines.clone())
                .unwrap_or_default();

            let fields: Vec<FieldDef> = o
                .fields
                .iter()
                .map(|f| {
                    let ty = convert_type(&f.type_, schema)?;
                    let default_value = f
                        .default_string
                        .clone()
                        .or_else(|| f.default_integer.map(|i| i.to_string()));
                    let comments = f
                        .documentation
                        .as_ref()
                        .map(|d| d.lines.clone())
                        .unwrap_or_default();

                    Ok(FieldDef {
                        name: f.name.clone(),
                        ty,
                        is_optional: f.is_optional,
                        default_value,
                        id: f.id,
                        comments,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok(MessageDef {
                name: o.name.clone(),
                fields,
                is_struct: o.is_struct,
                namespace,
                comments,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let enums: Vec<EnumDef> = schema
        .enums
        .iter()
        .map(|e| {
            let namespace = e.namespace.as_ref().and_then(|n| n.namespace.clone());

            EnumDef {
                name: e.name.clone(),
                values: e
                    .values
                    .iter()
                    .map(|v| EnumValue {
                        name: v.name.clone(),
                        value: v.value,
                        comments: v
                            .documentation
                            .as_ref()
                            .map(|d| d.lines.clone())
                            .unwrap_or_default(),
                    })
                    .collect(),
                is_union: e.is_union,
                namespace,
                comments: e
                    .documentation
                    .as_ref()
                    .map(|d| d.lines.clone())
                    .unwrap_or_default(),
            }
        })
        .collect();

    let services: Vec<ServiceDef> = schema
        .services
        .iter()
        .map(|s| {
            let comments = s
                .documentation
                .as_ref()
                .map(|d| d.lines.clone())
                .unwrap_or_default();
            let namespace = s.namespace.as_ref().and_then(|n| n.namespace.clone());
            let methods = s
                .calls
                .iter()
                .map(|c| {
                    let request_idx = c.request_index;
                    let response_idx = c.response_index;

                    if request_idx >= schema.objects.len() {
                        return Err(CodeGenError::Internal(format!(
                            "request index {} out of bounds for service '{}', method '{}'",
                            request_idx, s.name, c.name,
                        )));
                    }
                    if response_idx >= schema.objects.len() {
                        return Err(CodeGenError::Internal(format!(
                            "response index {} out of bounds for service '{}', method '{}'",
                            response_idx, s.name, c.name,
                        )));
                    }

                    let req = &schema.objects[request_idx];
                    let res = &schema.objects[response_idx];

                    // NOTE: FlatBuffers supports streaming via attributes:
                    // streaming: "server", "client", or "bidi"
                    Ok(MethodDef {
                        name: c.name.clone(),
                        rust_name: None,
                        input_type: req.name.clone(),
                        output_type: res.name.clone(),
                        streaming: parse_streaming(&c.attributes),
                        // FlatBuffers does not have a concept of codec path in its
                        // schema. We use a default that can be overridden by the
                        // code generator or user configuration.
                        codec_path: "crate::codec::Codec".to_string(),
                        comments: c
                            .documentation
                            .as_ref()
                            .map(|d| d.lines.clone())
                            .unwrap_or_default(),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok(ServiceDef {
                name: s.name.clone(),
                methods,
                package: namespace,
                comments,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(SchemaDef {
        name: schema.file_ident.clone().unwrap_or_default(),
        messages,
        enums,
        services,
        file_ident: schema.file_ident.clone(),
        root_table: schema
            .root_table_index
            .and_then(|idx| schema.objects.get(idx).map(|o| o.name.clone())),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use flatc_rs_schema::resolved::{
        ResolvedEnum, ResolvedEnumVal, ResolvedField, ResolvedObject, ResolvedType as RT,
    };
    use flatc_rs_schema::BaseType as B;
    use flatc_rs_schema::Namespace;

    fn make_test_schema() -> ResolvedSchema {
        let monster_fields = vec![
            ResolvedField {
                name: "hp".to_string(),
                type_: RT {
                    base_type: B::BASE_TYPE_SHORT,
                    base_size: None,
                    element_size: None,
                    element_type: None,
                    index: None,
                    fixed_length: None,
                },
                id: Some(0),
                offset: Some(4),
                default_integer: Some(100),
                default_real: None,
                default_string: None,
                is_deprecated: false,
                is_required: false,
                is_key: false,
                is_optional: false,
                attributes: None,
                documentation: Some(flatc_rs_schema::Documentation {
                    lines: vec!["Monster HP".to_string()],
                }),
                padding: None,
                is_offset_64: false,
                span: None,
            },
            ResolvedField {
                name: "name".to_string(),
                type_: RT {
                    base_type: B::BASE_TYPE_STRING,
                    base_size: None,
                    element_size: None,
                    element_type: None,
                    index: None,
                    fixed_length: None,
                },
                id: Some(1),
                offset: Some(6),
                default_integer: None,
                default_real: None,
                default_string: None,
                is_deprecated: false,
                is_required: false,
                is_key: false,
                is_optional: true,
                attributes: None,
                documentation: None,
                padding: None,
                is_offset_64: false,
                span: None,
            },
        ];

        let weapon_fields = vec![ResolvedField {
            name: "damage".to_string(),
            type_: RT {
                base_type: B::BASE_TYPE_SHORT,
                base_size: None,
                element_size: None,
                element_type: None,
                index: None,
                fixed_length: None,
            },
            id: Some(0),
            offset: Some(4),
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
        }];

        let color_enum = ResolvedEnum {
            name: "Color".to_string(),
            values: vec![
                ResolvedEnumVal {
                    name: "Red".to_string(),
                    value: 0,
                    union_type: None,
                    documentation: None,
                    attributes: None,
                    span: None,
                },
                ResolvedEnumVal {
                    name: "Green".to_string(),
                    value: 1,
                    union_type: None,
                    documentation: None,
                    attributes: None,
                    span: None,
                },
            ],
            is_union: false,
            underlying_type: RT {
                base_type: B::BASE_TYPE_BYTE,
                base_size: None,
                element_size: None,
                element_type: None,
                index: None,
                fixed_length: None,
            },
            attributes: None,
            documentation: None,
            declaration_file: None,
            namespace: Some(Namespace {
                namespace: Some("MyGame".to_string()),
            }),
            span: None,
        };

        ResolvedSchema {
            objects: vec![
                ResolvedObject {
                    name: "Monster".to_string(),
                    fields: monster_fields,
                    is_struct: false,
                    min_align: Some(2),
                    byte_size: Some(10),
                    attributes: None,
                    documentation: None,
                    declaration_file: None,
                    namespace: Some(Namespace {
                        namespace: Some("MyGame".to_string()),
                    }),
                    span: None,
                },
                ResolvedObject {
                    name: "Weapon".to_string(),
                    fields: weapon_fields,
                    is_struct: true,
                    min_align: Some(2),
                    byte_size: Some(4),
                    attributes: None,
                    documentation: None,
                    declaration_file: None,
                    namespace: Some(Namespace {
                        namespace: Some("MyGame".to_string()),
                    }),
                    span: None,
                },
            ],
            enums: vec![color_enum],
            file_ident: Some("MyGame".to_string()),
            file_ext: Some("bin".to_string()),
            root_table_index: Some(0),
            services: vec![],
            advanced_features: flatc_rs_schema::AdvancedFeatures::default(),
            fbs_files: vec![],
        }
    }

    #[test]
    fn test_messages_include_structs_and_tables() {
        let schema = make_test_schema();
        let result = from_resolved_schema(&schema).unwrap();
        // Both tables (is_struct=false) and structs (is_struct=true) should be included
        assert_eq!(result.messages.len(), 2);
        let names: Vec<_> = result.messages.iter().map(|m| &m.name).collect();
        assert!(names.contains(&&"Monster".to_string()));
        assert!(names.contains(&&"Weapon".to_string()));
        // Verify is_struct flag is correctly preserved
        let monster = result
            .messages
            .iter()
            .find(|m| m.name == "Monster")
            .unwrap();
        let weapon = result.messages.iter().find(|m| m.name == "Weapon").unwrap();
        assert!(!monster.is_struct); // Monster is a table
        assert!(weapon.is_struct); // Weapon is a struct
    }

    #[test]
    fn test_messages_contain_correct_fields() {
        let schema = make_test_schema();
        let result = from_resolved_schema(&schema).unwrap();
        let monster = &result.messages[0];

        assert_eq!(monster.name, "Monster");
        assert_eq!(monster.fields.len(), 2);
        assert_eq!(monster.fields[0].name, "hp");
        assert_eq!(monster.fields[1].name, "name");
        assert!(monster.fields[1].is_optional); // name is optional in our test
    }

    #[test]
    fn test_enums() {
        let schema = make_test_schema();
        let result = from_resolved_schema(&schema).unwrap();

        assert_eq!(result.enums.len(), 1);
        assert_eq!(result.enums[0].name, "Color");
        assert_eq!(result.enums[0].values.len(), 2);
        assert_eq!(result.enums[0].values[0].name, "Red");
        assert_eq!(result.enums[0].values[1].name, "Green");
    }

    #[test]
    fn test_file_ident() {
        let schema = make_test_schema();
        let result = from_resolved_schema(&schema).unwrap();

        assert_eq!(result.file_ident, Some("MyGame".to_string()));
    }

    #[test]
    fn test_root_table() {
        let schema = make_test_schema();
        let result = from_resolved_schema(&schema).unwrap();

        assert_eq!(result.root_table, Some("Monster".to_string()));
    }

    #[test]
    fn test_convert_scalar_type() {
        let schema = make_test_schema();
        let rt = RT {
            base_type: B::BASE_TYPE_INT,
            base_size: None,
            element_size: None,
            element_type: None,
            index: None,
            fixed_length: None,
        };

        let result = convert_type(&rt, &schema);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Type::Scalar(ScalarType::Int32)));
    }

    #[test]
    fn test_convert_string_type() {
        let schema = make_test_schema();
        let rt = RT {
            base_type: B::BASE_TYPE_STRING,
            base_size: None,
            element_size: None,
            element_type: None,
            index: None,
            fixed_length: None,
        };

        let result = convert_type(&rt, &schema);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Type::Scalar(ScalarType::String)));
    }

    #[test]
    fn test_convert_message_type() {
        let schema = make_test_schema();
        // Reference to Weapon (index 1)
        let rt = RT {
            base_type: B::BASE_TYPE_TABLE,
            base_size: None,
            element_size: None,
            element_type: None,
            index: Some(1),
            fixed_length: None,
        };

        let result = convert_type(&rt, &schema);
        assert!(result.is_ok());
        match result.unwrap() {
            Type::Message { name, package } => {
                assert_eq!(name, "Weapon");
                assert_eq!(package, Some("MyGame".to_string()));
            }
            _ => panic!("Expected Message type"),
        }
    }

    #[test]
    fn test_convert_vector_type() {
        let schema = make_test_schema();
        // Vector of strings
        let rt = RT {
            base_type: B::BASE_TYPE_VECTOR,
            base_size: None,
            element_size: None,
            element_type: Some(B::BASE_TYPE_STRING),
            index: None,
            fixed_length: None,
        };

        let result = convert_type(&rt, &schema);
        assert!(result.is_ok());
        match result.unwrap() {
            Type::Vector(inner) => {
                assert!(matches!(*inner, Type::Scalar(ScalarType::String)));
            }
            _ => panic!("Expected Vector type"),
        }
    }
}
