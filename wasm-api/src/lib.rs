use std::panic;
use wasm_bindgen::prelude::*;

/// Catch panics and convert them to JsError.
fn catch<F, T>(f: F) -> Result<T, JsError>
where
    F: FnOnce() -> Result<T, JsError> + panic::UnwindSafe,
{
    match panic::catch_unwind(f) {
        Ok(result) => result,
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "internal compiler panic".to_string()
            };
            Err(JsError::new(&msg))
        }
    }
}

/// Compile a .fbs schema and generate Rust code.
///
/// Takes a FlatBuffers schema source string and returns generated Rust code.
/// Does not support `include` directives (single-file only).
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

/// Compile a .fbs schema and generate TypeScript code.
///
/// Takes a FlatBuffers schema source string and returns generated TypeScript code.
/// Does not support `include` directives (single-file only).
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

/// Compile a .fbs schema and return the binary schema (.bfbs) as bytes.
#[wasm_bindgen]
pub fn compile_fbs_to_bfbs(source: &str) -> Result<Vec<u8>, JsError> {
    let source = source.to_string();
    catch(move || {
        let result =
            flatc_rs_compiler::compile_single(&source).map_err(|e| JsError::new(&e.to_string()))?;

        Ok(flatc_rs_compiler::bfbs::serialize_schema(&result.schema))
    })
}

/// Annotate a FlatBuffers binary using a schema source and root type name.
///
/// Returns a human-readable annotation string showing the binary structure
/// with byte offsets, field names, and values.
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
