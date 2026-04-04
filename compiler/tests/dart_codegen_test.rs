use flatc_rs_compiler::{
    analyze,
    codegen::{generate_dart, DartCodeGenOptions},
    parser::FbsParser,
};
use flatc_rs_test_utils::{GoldenTestCase, GoldenTestOptions};
use std::path::PathBuf;

fn run_single_dart_codegen_golden(name: &str) {
    let input_path = format!("testdata/dart_codegen_golden/{name}.fbs");
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
        let opts = DartCodeGenOptions {
            gen_object_api: true,
            gen_only_files: None,
        };
        match generate_dart(&schema, &opts) {
            Ok(code) => code,
            Err(e) => format!("CODEGEN ERROR: {e}\n"),
        }
    };

    let case = GoldenTestCase {
        name: name.to_string(),
        input_path: PathBuf::from(format!("testdata/dart_codegen_golden/{name}.fbs")),
        expected_path: PathBuf::from(format!("testdata/dart_codegen_golden/{name}.expected")),
    };
    flatc_rs_test_utils::run_golden_test(&case, &transform, &GoldenTestOptions::from_env())
        .unwrap();
}

include!(concat!(env!("OUT_DIR"), "/dart_codegen_tests_generated.rs"));

// ---------------------------------------------------------------------------
// Inline tests
// ---------------------------------------------------------------------------

fn generate_dart_code(schema_src: &str, opts: &DartCodeGenOptions) -> String {
    let parser = FbsParser::new(schema_src).with_file_name("test.fbs".to_string());
    let parse_output = parser.parse().unwrap();
    let schema = analyze(parse_output).unwrap();
    generate_dart(&schema, opts).unwrap()
}

#[test]
fn dart_gen_struct_simple() {
    let schema = "struct Vec3 { x: float; y: float; z: float; }";
    let opts = DartCodeGenOptions::default();
    let code = generate_dart_code(schema, &opts);
    assert!(code.contains("class Vec3"), "should generate Vec3 class");
    assert!(code.contains("double get x"), "should generate x accessor");
    assert!(code.contains("double get y"), "should generate y accessor");
    assert!(code.contains("double get z"), "should generate z accessor");
}

#[test]
fn dart_gen_table_basic() {
    let schema = "table Monster { hp: int; mana: short = 150; name: string; } root_type Monster;";
    let opts = DartCodeGenOptions::default();
    let code = generate_dart_code(schema, &opts);
    assert!(
        code.contains("class Monster"),
        "should generate Monster class"
    );
    assert!(
        code.contains("factory Monster(List<int> bytes)"),
        "should generate factory constructor"
    );
    assert!(code.contains("int get hp"), "should generate hp accessor");
    assert!(
        code.contains("int get mana"),
        "should generate mana accessor"
    );
    assert!(
        code.contains("String? get name"),
        "should generate name accessor"
    );
}

#[test]
fn dart_gen_enum_basic() {
    let schema = "enum Color: byte { Red = 1, Green = 2, Blue = 8 }";
    let opts = DartCodeGenOptions::default();
    let code = generate_dart_code(schema, &opts);
    assert!(
        code.contains("class Color"),
        "should generate Color enum class"
    );
    assert!(
        code.contains("factory Color.fromValue"),
        "should generate fromValue factory"
    );
    assert!(
        code.contains("static const Color Red"),
        "should generate Red constant"
    );
}

#[test]
fn dart_gen_enum_bitflags() {
    let schema = "enum Equipment: byte (bit_flags) { None = 0, Weapon = 1 }";
    let opts = DartCodeGenOptions::default();
    let code = generate_dart_code(schema, &opts);
    assert!(
        code.contains("static const int minValue"),
        "should generate minValue for bitflags"
    );
    assert!(
        code.contains("static const int maxValue"),
        "should generate maxValue for bitflags"
    );
    assert!(
        code.contains("static bool containsValue"),
        "should generate containsValue method"
    );
}

#[test]
fn dart_gen_optional_scalars() {
    let schema = "table Options { value: int (optional); } root_type Options;";
    let opts = DartCodeGenOptions::default();
    let code = generate_dart_code(schema, &opts);
    assert!(
        code.contains("int? get value"),
        "should generate nullable accessor for optional"
    );
}

#[test]
fn dart_gen_object_api() {
    // Note: Object API for tables is partially implemented - it generates
    // MonsterObjectBuilder but not MonsterT class yet
    let schema = "struct Vec3 { x: float; y: float; z: float; }";
    let opts = DartCodeGenOptions {
        gen_object_api: true,
        gen_only_files: None,
    };
    let code = generate_dart_code(schema, &opts);
    assert!(
        code.contains("class Vec3T"),
        "should generate Vec3T class for struct"
    );
    assert!(
        code.contains("int pack(fb.Builder fbBuilder)"),
        "should generate pack method"
    );
}

#[test]
fn dart_gen_namespace() {
    let schema = "namespace Game.Items; table Item { name: string; } root_type Item;";
    let opts = DartCodeGenOptions::default();
    let code = generate_dart_code(schema, &opts);
    // Namespaces are ignored in Dart codegen (flat_buffers package doesn't use them)
    assert!(code.contains("class Item"), "should generate Item class");
}

#[test]
fn dart_gen_nested_struct() {
    let schema = "struct Inner { x: int; } struct Outer { inner: Inner; }";
    let opts = DartCodeGenOptions::default();
    let code = generate_dart_code(schema, &opts);
    assert!(code.contains("class Inner"), "should generate Inner class");
    assert!(code.contains("class Outer"), "should generate Outer class");
}

#[test]
fn dart_gen_vector_field() {
    let schema = "table Monster { items: [int]; } root_type Monster;";
    let opts = DartCodeGenOptions::default();
    let code = generate_dart_code(schema, &opts);
    assert!(
        code.contains("int? get items"),
        "should generate items accessor"
    );
}

#[test]
fn dart_gen_string_default() {
    let schema = r#"table Monster { name: string = "default"; } root_type Monster;"#;
    let opts = DartCodeGenOptions::default();
    let code = generate_dart_code(schema, &opts);
    assert!(
        code.contains("defaultValue: 'default'"),
        "should generate default value for string"
    );
}

#[test]
fn dart_gen_keyword_escape() {
    let schema = "table MyTable { class: int; } root_type MyTable;";
    let opts = DartCodeGenOptions::default();
    let code = generate_dart_code(schema, &opts);
    assert!(
        code.contains("int get class_"),
        "should escape 'class' keyword to 'class_'"
    );
}
