use flatc_rs_compiler::{
    analyze,
    codegen::{generate_typescript, TsCodeGenOptions},
    parser::FbsParser,
};
use flatc_rs_test_utils::{GoldenTestCase, GoldenTestOptions};
use std::path::PathBuf;

fn run_single_ts_codegen_golden(name: &str) {
    let input_path = format!("testdata/ts_codegen_golden/{name}.fbs");
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
        let opts = TsCodeGenOptions {
            gen_object_api: true,
            gen_only_files: None,
            gen_mutable: true,
        };
        match generate_typescript(&schema, &opts) {
            Ok(code) => code,
            Err(e) => format!("CODEGEN ERROR: {e}\n"),
        }
    };

    let case = GoldenTestCase {
        name: name.to_string(),
        input_path: PathBuf::from(format!("testdata/ts_codegen_golden/{name}.fbs")),
        expected_path: PathBuf::from(format!("testdata/ts_codegen_golden/{name}.expected")),
    };
    flatc_rs_test_utils::run_golden_test(&case, &transform, &GoldenTestOptions::from_env())
        .unwrap();
}

include!(concat!(env!("OUT_DIR"), "/ts_codegen_tests_generated.rs"));

// ---------------------------------------------------------------------------
// --gen-mutable flag tests
// ---------------------------------------------------------------------------

fn generate_ts(schema_src: &str, opts: &TsCodeGenOptions) -> String {
    let parser = FbsParser::new(schema_src).with_file_name("test.fbs".to_string());
    let parse_output = parser.parse().unwrap();
    let schema = analyze(parse_output).unwrap();
    generate_typescript(&schema, opts).unwrap()
}

#[test]
fn gen_mutable_generates_mutate_methods() {
    let schema = "table Monster { hp:int; mana:short; } root_type Monster;";

    let opts = TsCodeGenOptions {
        gen_object_api: false,
        gen_only_files: None,
        gen_mutable: true,
    };
    let code = generate_ts(schema, &opts);
    assert!(
        code.contains("mutateHp"),
        "with --gen-mutable, should generate mutateHp method"
    );
    assert!(
        code.contains("mutateMana"),
        "with --gen-mutable, should generate mutateMana method"
    );
}

#[test]
fn gen_mutable_off_omits_mutate_methods() {
    let schema = "table Monster { hp:int; mana:short; } root_type Monster;";

    let opts = TsCodeGenOptions {
        gen_object_api: false,
        gen_only_files: None,
        gen_mutable: false,
    };
    let code = generate_ts(schema, &opts);
    assert!(
        !code.contains("mutateHp"),
        "without --gen-mutable, should not generate mutateHp method"
    );
    assert!(
        !code.contains("mutateMana"),
        "without --gen-mutable, should not generate mutateMana method"
    );
}
