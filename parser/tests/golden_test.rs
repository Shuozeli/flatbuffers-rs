use flatc_rs_parser::FbsParser;
use flatc_rs_test_utils::{GoldenTestCase, GoldenTestOptions};
use std::path::PathBuf;

fn transform(input: &str) -> String {
    let parser = FbsParser::new(input);
    match parser.parse() {
        Ok(output) => {
            let mut text = serde_json::to_string_pretty(&output.schema).unwrap();
            if let Some(root) = &output.state.root_type_name {
                text.push_str(&format!("\n# root_type: {root}\n"));
            }
            if !output.state.declared_attributes.is_empty() {
                for attr in &output.state.declared_attributes {
                    text.push_str(&format!("# declared_attribute: {attr}\n"));
                }
            }
            text
        }
        Err(e) => format!("ERROR: {e}\n"),
    }
}

fn run_single_golden(name: &str) {
    let case = GoldenTestCase {
        name: name.to_string(),
        input_path: PathBuf::from(format!("testdata/golden/{name}.fbs")),
        expected_path: PathBuf::from(format!("testdata/golden/{name}.expected")),
    };
    flatc_rs_test_utils::run_golden_test(&case, &transform, &GoldenTestOptions::from_env())
        .unwrap();
}

include!(concat!(env!("OUT_DIR"), "/golden_tests_generated.rs"));
