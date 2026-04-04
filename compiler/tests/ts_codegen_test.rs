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

fn generate_ts_default(schema_src: &str) -> String {
    generate_ts(
        schema_src,
        &TsCodeGenOptions {
            gen_object_api: true,
            gen_only_files: None,
            gen_mutable: true,
        },
    )
}

// ---------------------------------------------------------------------------
// Inline code generation tests
// ---------------------------------------------------------------------------

#[test]
fn ts_gen_struct_simple() {
    let schema = "struct Vec3 { x: float; y: float; z: float; }";
    let code = generate_ts_default(schema);
    assert!(
        code.contains("export class Vec3"),
        "should generate Vec3 class"
    );
    assert!(code.contains("x():number"), "should generate x getter");
    assert!(code.contains("y():number"), "should generate y getter");
    assert!(code.contains("z():number"), "should generate z getter");
}

#[test]
fn ts_gen_table_basic() {
    let schema = "table Monster { hp: int; mana: short = 150; name: string; } root_type Monster;";
    let code = generate_ts_default(schema);
    assert!(
        code.contains("export class Monster"),
        "should generate Monster class"
    );
    assert!(code.contains("hp():number"), "should generate hp getter");
    assert!(
        code.contains("mana():number"),
        "should generate mana getter"
    );
    assert!(
        code.contains("name():string|null"),
        "should generate name getter"
    );
}

#[test]
fn ts_gen_enum_basic() {
    let schema = "enum Color: byte { Red = 1, Green = 2, Blue = 8 }";
    let code = generate_ts_default(schema);
    assert!(
        code.contains("export enum Color"),
        "should generate Color enum"
    );
    assert!(code.contains("Red = 1"), "should generate Red constant");
    assert!(code.contains("Green = 2"), "should generate Green constant");
}

#[test]
fn ts_gen_enum_bitflags() {
    let schema = "enum Equipment: byte (bit_flags) { None = 0, Weapon = 1 }";
    let code = generate_ts_default(schema);
    assert!(
        code.contains("export enum Equipment"),
        "should generate Equipment enum"
    );
    assert!(code.contains("None = 0"), "should generate None constant");
}

#[test]
fn ts_gen_optional_scalars() {
    let schema = "table Options { value: int = null; } root_type Options;";
    let code = generate_ts_default(schema);
    assert!(
        code.contains("value():number|null"),
        "should generate nullable value getter"
    );
}

#[test]
fn ts_gen_object_api() {
    let schema = "struct Vec3 { x: float; y: float; z: float; }";
    let code = generate_ts_default(schema);
    assert!(
        code.contains("export class Vec3T"),
        "should generate Vec3T class"
    );
    assert!(
        code.contains("pack(builder:flatbuffers.Builder):flatbuffers.Offset"),
        "should generate pack method"
    );
}

#[test]
fn ts_gen_namespace() {
    let schema = "namespace Game.Items; table Item { name: string; } root_type Item;";
    let code = generate_ts_default(schema);
    assert!(
        code.contains("export class Item"),
        "should generate Item class"
    );
}

#[test]
fn ts_gen_nested_struct() {
    let schema = "struct Inner { x: int; } struct Outer { inner: Inner; }";
    let code = generate_ts_default(schema);
    assert!(
        code.contains("export class Inner"),
        "should generate Inner class"
    );
    assert!(
        code.contains("export class Outer"),
        "should generate Outer class"
    );
}

#[test]
fn ts_gen_vector_field() {
    let schema = "table Monster { items: [int]; } root_type Monster;";
    let code = generate_ts_default(schema);
    assert!(
        code.contains("items(index: number):number|null"),
        "should generate items getter"
    );
}

#[test]
fn ts_gen_keyword_escape() {
    let schema = "table MyTable { type_: int; } root_type MyTable;";
    let code = generate_ts_default(schema);
    assert!(
        code.contains("type_()"),
        "should escape 'type' keyword to 'type_'"
    );
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
