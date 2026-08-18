use std::collections::BTreeMap;
use std::panic;
use std::path::PathBuf;

use flatc_rs_compiler::{CompilationResult, CompilerError, CompilerOptions, VirtualFile};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_TYPES: &str = r#"
export interface VirtualSchemaFile {
  path: string;
  source: string;
}

export interface VirtualSchemaRequest {
  entryPath: string;
  files: VirtualSchemaFile[];
  includePaths?: string[];
}

export interface FlatcRsError {
  code: string;
  message: string;
  details?: Record<string, string>;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "VirtualSchemaRequest")]
    pub type JsVirtualSchemaRequest;
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VirtualSchemaRequest {
    entry_path: String,
    files: Vec<VirtualFileInput>,
    #[serde(default)]
    include_paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VirtualFileInput {
    path: String,
    source: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ApiError {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    details: BTreeMap<String, String>,
}

impl ApiError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            details: BTreeMap::new(),
        }
    }

    fn with_detail(mut self, key: &str, value: impl Into<String>) -> Self {
        self.details.insert(key.to_string(), value.into());
        self
    }
}

/// Catch panics and convert them to JsError for the original single-file API.
fn catch<F, T>(f: F) -> Result<T, JsError>
where
    F: FnOnce() -> Result<T, JsError>,
{
    match panic::catch_unwind(panic::AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(payload) => Err(JsError::new(&panic_message(payload))),
    }
}

fn catch_api<F, T>(f: F) -> Result<T, ApiError>
where
    F: FnOnce() -> Result<T, ApiError>,
{
    match panic::catch_unwind(panic::AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(payload) => Err(ApiError::new("internal_panic", panic_message(payload))),
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "internal compiler panic".to_string()
    }
}

fn api_error_to_js(error: ApiError) -> JsValue {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    error
        .serialize(&serializer)
        .unwrap_or_else(|serialization_error| JsError::new(&serialization_error.to_string()).into())
}

fn parse_request(request: JsValue) -> Result<VirtualSchemaRequest, ApiError> {
    serde_wasm_bindgen::from_value(request)
        .map_err(|error| ApiError::new("invalid_request", error.to_string()))
}

fn request_js_value(request: &JsVirtualSchemaRequest) -> JsValue {
    let value: &JsValue = request.as_ref();
    value.clone()
}

fn compile_request(request: VirtualSchemaRequest) -> Result<CompilationResult, ApiError> {
    let files = request
        .files
        .into_iter()
        .map(|file| VirtualFile::new(file.path, file.source))
        .collect::<Vec<_>>();
    let options = CompilerOptions {
        include_paths: request
            .include_paths
            .into_iter()
            .map(PathBuf::from)
            .collect(),
    };

    flatc_rs_compiler::compile_virtual(
        PathBuf::from(request.entry_path).as_path(),
        &files,
        &options,
    )
    .map_err(compiler_error)
}

fn compiler_error(error: CompilerError) -> ApiError {
    let message = error.to_string();
    match error {
        CompilerError::FileNotFound(path) => {
            ApiError::new("file_not_found", message).with_detail("path", path.to_string_lossy())
        }
        CompilerError::IoError { path, .. } => {
            ApiError::new("io_error", message).with_detail("path", path.to_string_lossy())
        }
        CompilerError::ParseError { file, .. } => {
            ApiError::new("parse_error", message).with_detail("file", file.to_string_lossy())
        }
        CompilerError::IncludeNotFound { include, from } => {
            ApiError::new("include_not_found", message)
                .with_detail("include", include)
                .with_detail("from", from.to_string_lossy())
        }
        CompilerError::PathTraversal {
            include,
            resolved,
            from,
        } => ApiError::new("path_traversal", message)
            .with_detail("include", include)
            .with_detail("resolved", resolved.to_string_lossy())
            .with_detail("from", from.to_string_lossy()),
        CompilerError::AbsoluteIncludePath { include, from } => {
            ApiError::new("absolute_include_path", message)
                .with_detail("include", include)
                .with_detail("from", from.to_string_lossy())
        }
        CompilerError::IncludeDepthLimit { depth, file } => {
            ApiError::new("include_depth_limit", message)
                .with_detail("depth", depth.to_string())
                .with_detail("file", file.to_string_lossy())
        }
        CompilerError::IncludedFileLimit { count, limit } => {
            ApiError::new("included_file_limit", message)
                .with_detail("count", count.to_string())
                .with_detail("limit", limit.to_string())
        }
        CompilerError::IncludeCycle { file } => {
            ApiError::new("include_cycle", message).with_detail("file", file.to_string_lossy())
        }
        CompilerError::InvalidVirtualPath { path, reason } => {
            ApiError::new("invalid_virtual_path", message)
                .with_detail("path", path.to_string_lossy())
                .with_detail("reason", reason)
        }
        CompilerError::DuplicateVirtualPath { path } => {
            ApiError::new("duplicate_virtual_path", message)
                .with_detail("path", path.to_string_lossy())
        }
        CompilerError::AnalyzeError(_) => ApiError::new("semantic_error", message),
    }
}

fn codegen_error(error: impl std::fmt::Display) -> ApiError {
    ApiError::new("codegen_error", error.to_string())
}

fn annotation_error(error: impl std::fmt::Display) -> ApiError {
    ApiError::new("annotation_error", error.to_string())
}

fn bfbs_error(error: impl std::fmt::Display) -> ApiError {
    ApiError::new("bfbs_error", error.to_string())
}

/// Compile a .fbs schema and generate Rust code.
///
/// This original single-source function remains available for compatibility.
/// Use [`compile_fbs_files_to_rust`] for schemas containing includes.
#[wasm_bindgen]
pub fn compile_fbs_to_rust(source: &str, gen_object_api: bool) -> Result<String, JsError> {
    let source = source.to_string();
    catch(move || {
        let result =
            flatc_rs_compiler::compile_single(&source).map_err(|e| JsError::new(&e.to_string()))?;

        let opts = flatc_rs_codegen::CodeGenOptions {
            gen_object_api,
            ..Default::default()
        };

        flatc_rs_codegen::generate_rust(&result.schema, &opts)
            .map_err(|e| JsError::new(&e.to_string()))
    })
}

/// Compile a virtual multi-file schema graph and generate Rust code.
#[wasm_bindgen]
pub fn compile_fbs_files_to_rust(
    request: &JsVirtualSchemaRequest,
    gen_object_api: bool,
) -> Result<String, JsValue> {
    let request = request_js_value(request);
    catch_api(move || {
        let result = compile_request(parse_request(request)?)?;
        let options = flatc_rs_codegen::CodeGenOptions {
            gen_object_api,
            ..Default::default()
        };
        flatc_rs_codegen::generate_rust(&result.schema, &options).map_err(codegen_error)
    })
    .map_err(api_error_to_js)
}

/// Compile a .fbs schema and generate TypeScript code.
///
/// This original single-source function remains available for compatibility.
/// Use [`compile_fbs_files_to_ts`] for schemas containing includes.
#[wasm_bindgen]
pub fn compile_fbs_to_ts(source: &str, gen_object_api: bool) -> Result<String, JsError> {
    let source = source.to_string();
    catch(move || {
        let result =
            flatc_rs_compiler::compile_single(&source).map_err(|e| JsError::new(&e.to_string()))?;

        let opts = flatc_rs_codegen::TsCodeGenOptions {
            gen_object_api,
            gen_mutable: true,
            ..Default::default()
        };

        flatc_rs_codegen::generate_typescript(&result.schema, &opts)
            .map_err(|e| JsError::new(&e.to_string()))
    })
}

/// Compile a virtual multi-file schema graph and generate TypeScript code.
#[wasm_bindgen]
pub fn compile_fbs_files_to_ts(
    request: &JsVirtualSchemaRequest,
    gen_object_api: bool,
) -> Result<String, JsValue> {
    let request = request_js_value(request);
    catch_api(move || {
        let result = compile_request(parse_request(request)?)?;
        let options = flatc_rs_codegen::TsCodeGenOptions {
            gen_object_api,
            gen_mutable: true,
            ..Default::default()
        };
        flatc_rs_codegen::generate_typescript(&result.schema, &options).map_err(codegen_error)
    })
    .map_err(api_error_to_js)
}

/// Compile a .fbs schema and return the binary schema (.bfbs) as bytes.
#[wasm_bindgen]
pub fn compile_fbs_to_bfbs(source: &str) -> Result<Vec<u8>, JsError> {
    let source = source.to_string();
    catch(move || {
        let result =
            flatc_rs_compiler::compile_single(&source).map_err(|e| JsError::new(&e.to_string()))?;

        flatc_rs_compiler::bfbs::serialize_schema(&result.schema)
            .map_err(|error| JsError::new(&error.to_string()))
    })
}

/// Compile a virtual multi-file schema graph to binary schema (.bfbs) bytes.
#[wasm_bindgen]
pub fn compile_fbs_files_to_bfbs(request: &JsVirtualSchemaRequest) -> Result<Vec<u8>, JsValue> {
    let request = request_js_value(request);
    catch_api(move || {
        let result = compile_request(parse_request(request)?)?;
        flatc_rs_compiler::bfbs::serialize_schema(&result.schema).map_err(bfbs_error)
    })
    .map_err(api_error_to_js)
}

/// Annotate a FlatBuffers binary using a single schema source and root type.
#[wasm_bindgen]
pub fn annotate_flatbuffer(
    binary: &[u8],
    schema_source: &str,
    root_type_name: &str,
) -> Result<String, JsError> {
    let schema_source = schema_source.to_string();
    let root_type_name = root_type_name.to_string();
    let binary = binary.to_vec();
    catch(move || {
        let result = flatc_rs_compiler::compile_single(&schema_source)
            .map_err(|e| JsError::new(&e.to_string()))?;

        flatc_rs_annotator::annotate_binary(
            &binary,
            &result.schema,
            &root_type_name,
            "<schema>",
            "<binary>",
        )
        .map_err(|e| JsError::new(&e.to_string()))
    })
}

/// Annotate a FlatBuffers binary using a virtual multi-file schema graph.
#[wasm_bindgen]
pub fn annotate_flatbuffer_files(
    binary: &[u8],
    request: &JsVirtualSchemaRequest,
    root_type_name: &str,
) -> Result<String, JsValue> {
    let binary = binary.to_vec();
    let request = request_js_value(request);
    let root_type_name = root_type_name.to_string();
    catch_api(move || {
        let result = compile_request(parse_request(request)?)?;
        flatc_rs_annotator::annotate_binary(
            &binary,
            &result.schema,
            &root_type_name,
            "<virtual-schema>",
            "<binary>",
        )
        .map_err(annotation_error)
    })
    .map_err(api_error_to_js)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic_is_converted_to_a_stable_api_error() {
        let error = catch_api::<_, ()>(|| panic!("test panic")).unwrap_err();

        assert_eq!(error.code, "internal_panic");
        assert_eq!(error.message, "test panic");
    }

    #[test]
    fn compiler_errors_have_stable_codes_and_details() {
        let error = compiler_error(CompilerError::IncludeNotFound {
            include: "types.fbs".to_string(),
            from: PathBuf::from("schemas/main.fbs"),
        });

        assert_eq!(error.code, "include_not_found");
        assert_eq!(error.details["include"], "types.fbs");
        assert_eq!(error.details["from"], "schemas/main.fbs");
    }
}
