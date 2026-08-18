use flatc_rs_annotator::walk_binary;
use flatc_rs_codegen::{generate_rust, generate_typescript, CodeGenOptions, TsCodeGenOptions};
use flatc_rs_compiler::bfbs::serialize_schema;
use flatc_rs_compiler::compile_single;
use flatc_rs_compiler::json::{binary_to_json, json_to_binary, JsonOptions};
use serde_json::json;

#[test]
fn public_schema_consumers_reject_invalid_schemas_without_panicking() {
    // Arrange
    let mut schema = compile_single("table Child {} table Root { child: Child; } root_type Root;")
        .expect("compile valid fixture")
        .schema;
    let root = schema
        .objects
        .iter_mut()
        .find(|object| object.name == "Root")
        .expect("root object");
    root.fields[0].type_.index = Some(-1);

    // Act
    let results = std::panic::catch_unwind(|| {
        [
            generate_rust(&schema, &CodeGenOptions::default()).map(|_| ()),
            generate_typescript(&schema, &TsCodeGenOptions::default()).map(|_| ()),
        ]
        .into_iter()
        .map(|result| result.map_err(|error| error.to_string()))
        .chain([
            serialize_schema(&schema)
                .map(|_| ())
                .map_err(|error| error.to_string()),
            json_to_binary(&json!({}), &schema, "Root")
                .map(|_| ())
                .map_err(|error| error.to_string()),
            binary_to_json(&[], &schema, "Root", &JsonOptions::default())
                .map(|_| ())
                .map_err(|error| error.to_string()),
            walk_binary(&[], &schema, "Root")
                .map(|_| ())
                .map_err(|error| error.to_string()),
        ])
        .collect::<Vec<_>>()
    });

    // Assert
    let results = results.expect("public schema consumers must not panic");
    assert_eq!(results.len(), 6);
    for result in results {
        let error = result.expect_err("invalid schema must be rejected");
        assert!(error.contains("must not be negative"), "{error}");
    }
}
