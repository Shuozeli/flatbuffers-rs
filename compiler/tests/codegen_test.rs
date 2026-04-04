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

// ---------------------------------------------------------------------------
// Inline tests
// ---------------------------------------------------------------------------

fn generate_rust_code(schema_src: &str) -> String {
    let parser = FbsParser::new(schema_src).with_file_name("test.fbs".to_string());
    let parse_output = parser.parse().unwrap();
    let schema = analyze(parse_output).unwrap();
    let opts = CodeGenOptions {
        gen_name_constants: true,
        gen_object_api: true,
        gen_only_files: None,
        ..CodeGenOptions::default()
    };
    generate_rust(&schema, &opts).unwrap()
}

#[test]
fn rust_gen_struct_simple() {
    let schema = "struct Vec3 { x: float; y: float; z: float; }";
    let code = generate_rust_code(schema);
    assert!(
        code.contains("pub struct Vec3"),
        "should generate Vec3 struct"
    );
    assert!(
        code.contains("pub fn x(&self) -> f32"),
        "should generate x getter"
    );
    assert!(
        code.contains("pub fn y(&self) -> f32"),
        "should generate y getter"
    );
    assert!(
        code.contains("pub fn z(&self) -> f32"),
        "should generate z getter"
    );
}

#[test]
fn rust_gen_table_basic() {
    let schema = "table Monster { hp: int; mana: short = 150; name: string; } root_type Monster;";
    let code = generate_rust_code(schema);
    assert!(
        code.contains("pub struct Monster"),
        "should generate Monster struct"
    );
    assert!(
        code.contains("pub fn root_as_monster"),
        "should generate root accessor"
    );
    assert!(code.contains("pub fn hp("), "should generate hp getter");
    assert!(code.contains("pub fn mana("), "should generate mana getter");
    assert!(code.contains("pub fn name("), "should generate name getter");
}

#[test]
fn rust_gen_enum_basic() {
    let schema = "enum Color: byte { Red = 1, Green = 2, Blue = 8 }";
    let code = generate_rust_code(schema);
    assert!(
        code.contains("pub struct Color"),
        "should generate Color struct"
    );
    assert!(
        code.contains("pub const Red: Self = Self(1)"),
        "should generate Red constant"
    );
    assert!(
        code.contains("pub const Green: Self = Self(2)"),
        "should generate Green constant"
    );
}

#[test]
fn rust_gen_enum_bitflags() {
    let schema = "enum Equipment: byte (bit_flags) { None = 0, Weapon = 1 }";
    let code = generate_rust_code(schema);
    assert!(
        code.contains("bitflags::bitflags!"),
        "should generate bitflags macro"
    );
    assert!(
        code.contains("pub struct Equipment"),
        "should generate Equipment struct"
    );
}

#[test]
fn rust_gen_optional_scalars() {
    let schema = "table Options { value: int = null; } root_type Options;";
    let code = generate_rust_code(schema);
    assert!(
        code.contains("pub fn value(&self) -> Option<i32>"),
        "should generate optional value getter"
    );
}

#[test]
fn rust_gen_object_api() {
    let schema = "struct Vec3 { x: float; y: float; z: float; }";
    let code = generate_rust_code(schema);
    assert!(
        code.contains("pub struct Vec3T"),
        "should generate Vec3T struct"
    );
    assert!(
        code.contains("pub fn pack(&self) -> Vec3"),
        "should generate pack method"
    );
    assert!(
        code.contains("pub fn unpack(&self) -> Vec3T"),
        "should generate unpack method"
    );
}

#[test]
fn rust_gen_namespace() {
    let schema = "namespace Game.Items; table Item { name: string; } root_type Item;";
    let code = generate_rust_code(schema);
    assert!(
        code.contains("pub struct Item"),
        "should generate Item struct"
    );
}

#[test]
fn rust_gen_nested_struct() {
    let schema = "struct Inner { x: int; } struct Outer { inner: Inner; }";
    let code = generate_rust_code(schema);
    assert!(
        code.contains("pub struct Inner"),
        "should generate Inner struct"
    );
    assert!(
        code.contains("pub struct Outer"),
        "should generate Outer struct"
    );
}

#[test]
fn rust_gen_vector_field() {
    let schema = "table Monster { scores: [int]; } root_type Monster;";
    let code = generate_rust_code(schema);
    assert!(
        code.contains("pub fn scores("),
        "should generate scores getter"
    );
}

#[test]
fn rust_gen_keyword_escape() {
    let schema = "table MyTable { type_: int; } root_type MyTable;";
    let code = generate_rust_code(schema);
    assert!(
        code.contains("pub fn type_(&self)"),
        "should escape 'type' keyword to 'type_'"
    );
}
