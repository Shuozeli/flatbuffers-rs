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
    // Arrange
    let schema = "table Monster { hp: int; mana: short = 150; name: string; } root_type Monster;";

    // Act
    let code = generate_rust_code(schema);

    // Assert
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
    assert!(
        !code.contains("pub fn createMonster"),
        "standalone camel-case constructors should not be generated"
    );
}

#[test]
fn rust_gen_deprecated_fields_are_read_only_and_omitted_from_derived_views() {
    // Arrange
    let schema =
        "table MobileEvent { old_id: string (deprecated); count: int; } root_type MobileEvent;";

    // Act
    let parser = FbsParser::new(schema).with_file_name("deprecated.fbs".to_string());
    let parsed = parser.parse().expect("parse deprecated-field schema");
    let resolved = analyze(parsed).expect("analyze deprecated-field schema");
    let code = generate_rust(
        &resolved,
        &CodeGenOptions {
            gen_object_api: true,
            rust_serialize: true,
            ..CodeGenOptions::default()
        },
    )
    .expect("generate Rust with Object API and serde");

    // Assert
    assert!(
        code.contains("pub fn old_id("),
        "deprecated reader compatibility should remain available"
    );
    assert!(
        code.contains("ds.field(\"count\", &self.count());"),
        "Debug should include active fields"
    );
    assert!(
        !code.contains("self.old_id()"),
        "Debug, serde, and Object API unpack must not call deprecated accessors"
    );
    assert!(
        !code.contains("pub fn add_old_id(")
            && !code.contains("pub old_id:")
            && !code.contains("args.old_id"),
        "deprecated fields should be read-only and absent from builders, args, and Object API"
    );
    assert!(
        code.contains("pub struct MobileEventArgs {"),
        "a deprecated string alone must not introduce an unused Args lifetime"
    );
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
fn rust_gen_matches_official_naming_root_helpers_and_follow_types() {
    // Arrange
    let schema = r#"
namespace xlqy3;
enum EntityKind:byte { Unknown }
table SnapEntity { kind:EntityKind; }
table S2cWorldSnapshot { entities:[SnapEntity]; }
root_type S2cWorldSnapshot;
"#;

    // Act
    let code = generate_rust_code(schema);

    // Assert
    assert!(code.contains("pub mod xlqy_3 {"));
    assert!(code.contains("ENUM_MIN_ENTITY_KIND"));
    assert!(code.contains("pub fn root_as_s2c_world_snapshot_with_opts"));
    assert!(code.contains("pub unsafe fn size_prefixed_root_as_s2c_world_snapshot_unchecked"));
    assert!(code.contains("pub fn finish_s2c_world_snapshot_buffer"));
    assert!(code.contains(
        "pub fn entities(&self) -> Option<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<SnapEntity<'a>>>>"
    ));
    assert!(code.contains(
        "self._tab.get::<::flatbuffers::ForwardsUOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<SnapEntity>>>>"
    ));

    let helper_position = code.find("pub fn root_as_s2c_world_snapshot").unwrap();
    let namespace_end = code.find("} // pub mod xlqy3").unwrap();
    assert!(
        helper_position < namespace_end,
        "root helpers must remain inside the root table namespace"
    );
}

#[test]
fn rust_gen_preserves_schema_documentation() {
    // Arrange
    let schema = r#"
/// A position.
struct Position {
  /// Horizontal coordinate.
  x:float;
}
/// Entity kinds.
enum EntityKind:byte {
  /// An unknown entity.
  Unknown
}
/// A world entity.
table Entity {
  /// Its position.
  position:Position;
}
root_type Entity;
"#;

    // Act
    let code = generate_rust_code(schema);

    // Assert
    assert!(code.contains("/// A position.\n#[repr(transparent)]"));
    assert!(code.contains("/// Horizontal coordinate.\n  pub fn x("));
    assert!(code.contains("/// Entity kinds.\n#[derive(Clone, Copy"));
    assert!(code.contains("/// An unknown entity.\n  pub const Unknown"));
    assert!(code.contains("/// A world entity.\n#[derive(Copy, Clone"));
    assert!(code.contains("/// Its position.\n  #[inline]\n  pub fn position("));
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
