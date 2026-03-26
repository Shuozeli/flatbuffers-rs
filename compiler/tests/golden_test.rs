use flatc_rs_compiler::{analyze, parser::FbsParser};
use flatc_rs_test_utils::{GoldenTestCase, GoldenTestOptions};
use std::path::PathBuf;

fn run_single_golden(name: &str) {
    let input_path = format!("testdata/golden/{name}.fbs");
    let transform = move |input: &str| {
        let parser = FbsParser::new(input).with_file_name(input_path.clone());
        let parse_output = match parser.parse() {
            Ok(output) => output,
            Err(e) => return format!("PARSE ERROR: {e}\n"),
        };
        match analyze(parse_output) {
            Ok(schema) => serde_json::to_string_pretty(&schema.as_legacy().unwrap()).unwrap(),
            Err(e) => format!("ANALYZE ERROR: {e}\n"),
        }
    };

    let case = GoldenTestCase {
        name: name.to_string(),
        input_path: PathBuf::from(format!("testdata/golden/{name}.fbs")),
        expected_path: PathBuf::from(format!("testdata/golden/{name}.expected")),
    };
    flatc_rs_test_utils::run_golden_test(&case, &transform, &GoldenTestOptions::from_env())
        .unwrap();
}

include!(concat!(env!("OUT_DIR"), "/golden_tests_generated.rs"));
