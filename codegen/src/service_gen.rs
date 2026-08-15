//! gRPC service code generation for FlatBuffers schemas.
//!
//! Maps the local resolved schema directly into pure-grpc's codec-neutral IR.
//! The generated transport uses owned FlatBuffers Object API types (`*T`) so
//! request and response values satisfy gRPC's `Clone + Send + Sync + 'static`
//! requirements without introducing a dependency back to this repository.

use std::collections::{BTreeSet, HashSet};

use crate::{type_map, CodeGenError};
use flatc_rs_schema::resolved::{ResolvedObject, ResolvedRpcCall, ResolvedSchema, ResolvedService};
use flatc_rs_schema::{Attributes, Documentation};
use grpc_codegen::ir::{MethodDef, ServiceDef, StreamingType};
use grpc_codegen::{client_gen, server_gen};
use proc_macro2::TokenStream;
use quote::quote;

const FLATBUFFERS_CODEC_PATH: &str = "grpc_codec_flatbuffers::FlatBuffersCodec";

/// Generate FlatBuffers codec implementations plus pure-grpc server and client
/// stubs for every service in a schema.
pub fn generate_services(
    schema: &ResolvedSchema,
    filter: &Option<HashSet<String>>,
) -> Result<String, CodeGenError> {
    let services = schema
        .services
        .iter()
        .filter(|service| super::should_generate(service.declaration_file.as_deref(), filter))
        .collect::<Vec<_>>();
    if services.is_empty() {
        return Ok(String::new());
    }

    let mut tokens = TokenStream::new();

    for object_index in rpc_message_indices(schema, &services)? {
        let object = schema.objects.get(object_index).ok_or_else(|| {
            CodeGenError::Internal(format!(
                "RPC message index {object_index} is outside schema.objects"
            ))
        })?;
        tokens.extend(generate_codec_impl(object)?);
    }

    for service in services {
        let service = service_from_fbs(service, schema)?;
        tokens.extend(server_gen::generate(&service));
        tokens.extend(client_gen::generate(&service));
    }

    let file = syn::parse2::<syn::File>(tokens)
        .map_err(|e| CodeGenError::Internal(format!("gRPC service generation error: {e}")))?;
    Ok(prettyplease::unparse(&file))
}

fn rpc_message_indices(
    schema: &ResolvedSchema,
    services: &[&ResolvedService],
) -> Result<BTreeSet<usize>, CodeGenError> {
    let mut indices = BTreeSet::new();
    for service in services {
        for call in &service.calls {
            request_object(call, schema)?;
            response_object(call, schema)?;
            indices.insert(call.request_index);
            indices.insert(call.response_index);
        }
    }
    Ok(indices)
}

fn generate_codec_impl(object: &ResolvedObject) -> Result<TokenStream, CodeGenError> {
    if object.is_struct {
        return Err(CodeGenError::Internal(format!(
            "FlatBuffers gRPC message '{}' must be a table, not a struct",
            object.name
        )));
    }

    let owned_path = rust_type_path(object, true);
    let reader_path = rust_type_path(object, false);
    let owned: TokenStream = owned_path.parse().map_err(|e| {
        CodeGenError::Internal(format!(
            "invalid owned Rust path '{owned_path}' for RPC message '{}': {e}",
            object.name
        ))
    })?;
    let reader: TokenStream = reader_path.parse().map_err(|e| {
        CodeGenError::Internal(format!(
            "invalid reader Rust path '{reader_path}' for RPC message '{}': {e}",
            object.name
        ))
    })?;
    let error_label = format!("invalid {}: {{e}}", object.name);

    Ok(quote! {
        impl grpc_codec_flatbuffers::FlatBufferGrpcMessage for #owned {
            fn encode_flatbuffer(&self) -> ::std::vec::Vec<u8> {
                let mut builder =
                    grpc_codec_flatbuffers::flatbuffers::FlatBufferBuilder::new();
                let root = #owned::pack(self, &mut builder);
                builder.finish(root, None);
                builder.finished_data().to_vec()
            }

            fn decode_flatbuffer(
                data: &[u8],
            ) -> ::std::result::Result<Self, ::std::string::String> {
                let reader = grpc_codec_flatbuffers::flatbuffers::root::<#reader>(data)
                    .map_err(|e| format!(#error_label))?;
                Ok(reader.unpack())
            }
        }
    })
}

fn service_from_fbs(
    service: &ResolvedService,
    schema: &ResolvedSchema,
) -> Result<ServiceDef, CodeGenError> {
    let methods = service
        .calls
        .iter()
        .map(|call| method_from_fbs(call, schema))
        .collect::<Result<Vec<_>, _>>()?;
    let service = ServiceDef {
        name: service.name.clone(),
        package: service
            .namespace
            .as_ref()
            .and_then(|namespace| namespace.namespace.clone()),
        methods,
        comments: extract_comments(&service.documentation),
    };
    let errors = service.validate();
    if !errors.is_empty() {
        return Err(CodeGenError::Internal(format!(
            "invalid gRPC service definition: {}",
            errors.join("; ")
        )));
    }
    Ok(service)
}

fn method_from_fbs(
    call: &ResolvedRpcCall,
    schema: &ResolvedSchema,
) -> Result<MethodDef, CodeGenError> {
    let request = request_object(call, schema)?;
    let response = response_object(call, schema)?;
    let streaming = parse_streaming(&call.attributes)?;
    if !matches!(streaming, StreamingType::None) {
        return Err(CodeGenError::Internal(format!(
            "FlatBuffers gRPC code generation does not yet support {} RPC '{}'",
            streaming_mode(streaming),
            call.name
        )));
    }

    Ok(MethodDef {
        name: call.name.clone(),
        rust_name: Some(type_map::to_snake_case(&call.name)),
        input_type: format!("super::{}", rust_type_path(request, true)),
        output_type: format!("super::{}", rust_type_path(response, true)),
        streaming,
        codec_path: FLATBUFFERS_CODEC_PATH.to_string(),
        comments: extract_comments(&call.documentation),
    })
}

fn request_object<'a>(
    call: &ResolvedRpcCall,
    schema: &'a ResolvedSchema,
) -> Result<&'a ResolvedObject, CodeGenError> {
    schema.objects.get(call.request_index).ok_or_else(|| {
        CodeGenError::Internal(format!(
            "RPC '{}' request index {} is outside schema.objects",
            call.name, call.request_index
        ))
    })
}

fn response_object<'a>(
    call: &ResolvedRpcCall,
    schema: &'a ResolvedSchema,
) -> Result<&'a ResolvedObject, CodeGenError> {
    schema.objects.get(call.response_index).ok_or_else(|| {
        CodeGenError::Internal(format!(
            "RPC '{}' response index {} is outside schema.objects",
            call.name, call.response_index
        ))
    })
}

fn rust_type_path(object: &ResolvedObject, owned: bool) -> String {
    let mut segments = object
        .namespace
        .as_ref()
        .and_then(|namespace| namespace.namespace.as_deref())
        .into_iter()
        .flat_map(|namespace| namespace.split('.'))
        .filter(|segment| !segment.is_empty())
        .map(type_map::to_snake_case)
        .collect::<Vec<_>>();
    segments.push(if owned {
        format!("{}T", object.name)
    } else {
        object.name.clone()
    });
    segments.join("::")
}

fn parse_streaming(attributes: &Option<Attributes>) -> Result<StreamingType, CodeGenError> {
    let value = attributes
        .as_ref()
        .and_then(|attributes| attributes.get("streaming"))
        .map(|value| {
            value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .unwrap_or(value)
        });
    match value {
        None | Some("none") => Ok(StreamingType::None),
        Some("server") => Ok(StreamingType::Server),
        Some("client") => Ok(StreamingType::Client),
        Some("bidi") => Ok(StreamingType::BiDi),
        Some(value) => Err(CodeGenError::Internal(format!(
            "unknown FlatBuffers gRPC streaming mode '{value}'"
        ))),
    }
}

fn streaming_mode(streaming: StreamingType) -> &'static str {
    match streaming {
        StreamingType::None => "unary",
        StreamingType::Server => "server-streaming",
        StreamingType::Client => "client-streaming",
        StreamingType::BiDi => "bidirectional-streaming",
    }
}

fn extract_comments(documentation: &Option<Documentation>) -> Vec<String> {
    documentation
        .as_ref()
        .map(|documentation| documentation.lines.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use flatc_rs_schema::resolved::{ResolvedObject, ResolvedRpcCall, ResolvedService};
    use flatc_rs_schema::{AdvancedFeatures, KeyValue, Namespace};

    fn schema_with_service() -> ResolvedSchema {
        let object = |name: &str| ResolvedObject {
            name: name.into(),
            fields: vec![],
            is_struct: false,
            min_align: None,
            byte_size: None,
            attributes: None,
            documentation: None,
            declaration_file: None,
            namespace: Some(Namespace {
                namespace: Some("hello.v1".into()),
            }),
            span: None,
        };
        ResolvedSchema {
            objects: vec![object("HelloRequest"), object("HelloReply")],
            enums: vec![],
            file_ident: None,
            file_ext: None,
            root_table_index: None,
            services: vec![ResolvedService {
                name: "Greeter".into(),
                calls: vec![ResolvedRpcCall {
                    name: "SayHello".into(),
                    request_index: 0,
                    response_index: 1,
                    attributes: None,
                    documentation: Some(Documentation {
                        lines: vec!["Greets one caller.".into()],
                    }),
                    span: None,
                }],
                attributes: None,
                documentation: Some(Documentation {
                    lines: vec!["Greeting service.".into()],
                }),
                declaration_file: None,
                namespace: Some(Namespace {
                    namespace: Some("hello.v1".into()),
                }),
                span: None,
            }],
            advanced_features: AdvancedFeatures::default(),
            fbs_files: vec![],
        }
    }

    #[test]
    fn local_schema_mapping_preserves_names_owned_paths_codec_and_comments() {
        // Arrange
        let schema = schema_with_service();

        // Act
        let service =
            service_from_fbs(&schema.services[0], &schema).expect("map local FlatBuffers schema");

        // Assert
        assert_eq!(service.name, "Greeter");
        assert_eq!(service.package.as_deref(), Some("hello.v1"));
        assert_eq!(service.comments, ["Greeting service."]);
        assert_eq!(service.methods.len(), 1);
        let method = &service.methods[0];
        assert_eq!(method.name, "SayHello");
        assert_eq!(method.rust_name.as_deref(), Some("say_hello"));
        assert_eq!(method.input_type, "super::hello::v1::HelloRequestT");
        assert_eq!(method.output_type, "super::hello::v1::HelloReplyT");
        assert_eq!(method.streaming, StreamingType::None);
        assert_eq!(method.codec_path, FLATBUFFERS_CODEC_PATH);
        assert_eq!(method.comments, ["Greets one caller."]);
    }

    #[test]
    fn local_schema_mapping_rejects_an_out_of_bounds_request_index() {
        // Arrange
        let mut schema = schema_with_service();
        schema.services[0].calls[0].request_index = usize::MAX;

        // Act
        let result = service_from_fbs(&schema.services[0], &schema);

        // Assert
        assert!(matches!(
            result,
            Err(CodeGenError::Internal(message))
                if message.contains("SayHello") && message.contains("request index")
        ));
    }

    #[test]
    fn local_schema_mapping_rejects_streaming_before_emitting_partial_stubs() {
        // Arrange
        let mut schema = schema_with_service();
        schema.services[0].calls[0].attributes = Some(Attributes {
            entries: vec![KeyValue {
                key: Some("streaming".into()),
                value: Some("\"server\"".into()),
            }],
        });

        // Act
        let result = generate_services(&schema, &None);

        // Assert
        assert!(matches!(
            result,
            Err(CodeGenError::Internal(message))
                if message.contains("server-streaming") && message.contains("SayHello")
        ));
    }

    #[test]
    fn local_schema_mapping_generates_codecs_server_client_path_and_comments() {
        // Arrange
        let schema = schema_with_service();

        // Act
        let generated = generate_services(&schema, &None).expect("generate gRPC services");

        // Assert
        assert!(
            generated.contains("FlatBufferGrpcMessage for hello::v1::HelloRequestT"),
            "{generated}"
        );
        assert!(generated.contains("hello::v1::HelloReply"), "{generated}");
        assert!(generated.contains("pub mod greeter_server"), "{generated}");
        assert!(generated.contains("pub mod greeter_client"), "{generated}");
        assert!(
            generated.contains("/hello.v1.Greeter/SayHello"),
            "{generated}"
        );
        assert!(generated.contains("Greets one caller."), "{generated}");
    }
}
