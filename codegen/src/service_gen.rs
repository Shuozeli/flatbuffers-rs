//! gRPC service code generation for FlatBuffers schemas.
//!
//! Bridges flatbuffers-rs schema types to grpc-codegen for generating
//! server traits and client stubs.

use crate::CodeGenError;
use flatc_rs_schema::resolved::ResolvedSchema;
use grpc_codegen::flatbuffers::service_from_fbs;
use grpc_codegen::{client_gen, server_gen};

/// Generate gRPC service code (server + client) for all services in a schema.
///
/// Returns the generated Rust source code as a string, or empty string if
/// no services are defined.
pub fn generate_services(
    schema: &ResolvedSchema,
    proto_path: &str,
) -> Result<String, CodeGenError> {
    if schema.services.is_empty() {
        return Ok(String::new());
    }

    let mut tokens = proc_macro2::TokenStream::new();

    for service in &schema.services {
        let svc_def = service_from_fbs(service, schema, proto_path);
        tokens.extend(server_gen::generate(&svc_def));
        tokens.extend(client_gen::generate(&svc_def));
    }

    let file = syn::parse2::<syn::File>(tokens)
        .map_err(|e| CodeGenError::Internal(format!("gRPC service generation error: {e}")))?;
    Ok(prettyplease::unparse(&file))
}
