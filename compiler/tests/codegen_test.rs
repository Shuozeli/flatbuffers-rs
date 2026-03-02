use flatc_rs_compiler::{
    analyze,
    codegen::{generate_rust, CodeGenOptions},
    parser::FbsParser,
};
use flatc_rs_test_utils::{GoldenTestCase, GoldenTestOptions};
use std::path::PathBuf;

fn run_single_codegen_golden(name: &str) {
    let input_path = format!("testdata/codegen_golden/{name}.fbs");
    let transform = move |input: &str| {
        let parser = FbsParser::new(input).with_file_name(input_path.clone());
        let parse_output = match parser.parse() {
            Ok(output) => output,
            Err(e) => return format!("PARSE ERROR: {e}\n"),
        };
        let schema = match analyze(parse_output) {
            Ok(schema) => schema,
            Err(e) => return format!("ANALYZE ERROR: {e}\n"),
        };
        let opts = CodeGenOptions {
            gen_name_constants: true,
            gen_object_api: true,
            gen_only_files: None,
            ..CodeGenOptions::default()
        };
        match generate_rust(&schema, &opts) {
            Ok(code) => code,
            Err(e) => format!("CODEGEN ERROR: {e}\n"),
        }
    };

    let case = GoldenTestCase {
        name: name.to_string(),
        input_path: PathBuf::from(format!("testdata/codegen_golden/{name}.fbs")),
        expected_path: PathBuf::from(format!("testdata/codegen_golden/{name}.expected")),
    };
    flatc_rs_test_utils::run_golden_test(&case, &transform, &GoldenTestOptions::from_env())
        .unwrap();
}

fn run_single_serde_codegen_golden(name: &str) {
    let input_path = format!("testdata/serde_codegen_golden/{name}.fbs");
    let transform = move |input: &str| {
        let parser = FbsParser::new(input).with_file_name(input_path.clone());
        let parse_output = match parser.parse() {
            Ok(output) => output,
            Err(e) => return format!("PARSE ERROR: {e}\n"),
        };
        let schema = match analyze(parse_output) {
            Ok(schema) => schema,
            Err(e) => return format!("ANALYZE ERROR: {e}\n"),
        };
        let opts = CodeGenOptions {
            gen_name_constants: true,
            gen_object_api: true,
            rust_serialize: true,
            gen_only_files: None,
            ..CodeGenOptions::default()
        };
        match generate_rust(&schema, &opts) {
            Ok(code) => code,
            Err(e) => format!("CODEGEN ERROR: {e}\n"),
        }
    };

    let case = GoldenTestCase {
        name: name.to_string(),
        input_path: PathBuf::from(format!("testdata/serde_codegen_golden/{name}.fbs")),
        expected_path: PathBuf::from(format!("testdata/serde_codegen_golden/{name}.expected")),
    };
    flatc_rs_test_utils::run_golden_test(&case, &transform, &GoldenTestOptions::from_env())
        .unwrap();
}

include!(concat!(env!("OUT_DIR"), "/codegen_tests_generated.rs"));
include!(concat!(
    env!("OUT_DIR"),
    "/serde_codegen_tests_generated.rs"
));
