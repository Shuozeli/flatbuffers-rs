//! Compilation tests: verify that generated Rust code actually compiles.
//!
//! Unlike golden tests (which compare text), these tests generate Rust code
//! from schemas, write it to a temp file, and invoke `rustc` to check that
//! the output is syntactically and semantically valid.

use flatc_rs_compiler::{
    analyze,
    codegen::{generate_rust, CodeGenOptions},
    parser::FbsParser,
};
use std::process::Command;

/// Generate Rust code from a schema string, write it to a temp file, and
/// run `rustc --edition 2021 --crate-type lib` on it.
///
/// Returns the rustc stderr on failure; panics with diagnostics.
fn assert_compiles(schema_source: &str, opts: &CodeGenOptions, test_name: &str) {
    let parser = FbsParser::new(schema_source).with_file_name(format!("{test_name}.fbs"));
    let parse_output = parser.parse().unwrap_or_else(|e| {
        panic!("[{test_name}] parse failed: {e}");
    });
    let schema = analyze(parse_output).unwrap_or_else(|e| {
        panic!("[{test_name}] analyze failed: {e}");
    });
    let code = generate_rust(&schema, opts).unwrap_or_else(|e| {
        panic!("[{test_name}] codegen failed: {e}");
    });

    // Wrap generated code with the required preamble
    let full_source = format!(
        "#![allow(unused_imports, dead_code, non_camel_case_types, non_snake_case, non_upper_case_globals, clippy::all)]\n\
         extern crate flatbuffers;\n\
         {code}"
    );

    let dir = tempfile::tempdir().unwrap();
    let src_path = dir.path().join(format!("{test_name}.rs"));
    std::fs::write(&src_path, full_source.as_bytes()).unwrap();

    // Find the flatbuffers crate in our dependency tree
    let flatbuffers_dir = find_flatbuffers_rlib();

    let mut cmd = Command::new("rustc");
    cmd.arg("--edition=2021")
        .arg("--crate-type=lib")
        .arg("--crate-name")
        .arg(test_name)
        .arg("-o")
        .arg(dir.path().join("out.rlib"));

    if let Some(ref deps_dir) = flatbuffers_dir {
        cmd.arg("-L").arg(deps_dir);
        // Also add --extern for flatbuffers if we can find the rlib
        if let Some(rlib) = find_extern_rlib(deps_dir, "flatbuffers") {
            cmd.arg("--extern").arg(format!("flatbuffers={rlib}"));
        }
    }

    cmd.arg(&src_path);

    let output = cmd.output().unwrap_or_else(|e| {
        panic!("[{test_name}] failed to run rustc: {e}");
    });

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "[{test_name}] generated code failed to compile:\n\
             --- rustc stderr ---\n{stderr}\n\
             --- generated code (first 100 lines) ---\n{}",
            full_source.lines().take(100).collect::<Vec<_>>().join("\n")
        );
    }
}

/// Locate the directory containing the compiled `flatbuffers` rlib/rmeta.
/// Searches the target/debug/deps directory.
fn find_flatbuffers_rlib() -> Option<String> {
    // Walk up from the test binary to find target/debug/deps
    let exe = std::env::current_exe().ok()?;
    let mut dir = exe.parent()?;

    // test binary is in target/debug/deps/<binary>
    // we want target/debug/deps/
    if dir.ends_with("deps") {
        return Some(dir.to_string_lossy().to_string());
    }

    // Try parent directories
    for _ in 0..5 {
        let deps_dir = dir.join("deps");
        if deps_dir.is_dir() {
            return Some(deps_dir.to_string_lossy().to_string());
        }
        dir = dir.parent()?;
    }

    None
}

/// Find a specific crate's rlib file in a deps directory.
fn find_extern_rlib(deps_dir: &str, crate_name: &str) -> Option<String> {
    let dir = std::path::Path::new(deps_dir);
    if !dir.is_dir() {
        return None;
    }
    // Look for libflatbuffers-*.rlib
    let prefix = format!("lib{crate_name}-");
    for entry in std::fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(&prefix) && name.ends_with(".rlib") {
            return Some(entry.path().to_string_lossy().to_string());
        }
    }
    None
}

fn default_opts() -> CodeGenOptions {
    CodeGenOptions {
        gen_name_constants: true,
        gen_object_api: true,
        gen_only_files: None,
        ..CodeGenOptions::default()
    }
}

fn serde_opts() -> CodeGenOptions {
    CodeGenOptions {
        gen_name_constants: true,
        gen_object_api: true,
        rust_serialize: true,
        gen_only_files: None,
        ..CodeGenOptions::default()
    }
}

// ---------------------------------------------------------------------------
// Test cases: each schema exercises a different codegen feature
// ---------------------------------------------------------------------------

#[test]
fn compile_table_with_scalars() {
    assert_compiles(
        r#"
table Monster {
  hp: short = 100;
  mana: short = 150;
  speed: float = 1.5;
  active: bool = true;
}
"#,
        &default_opts(),
        "table_scalars",
    );
}

#[test]
fn compile_table_with_strings_and_vectors() {
    assert_compiles(
        r#"
table Monster {
  name: string (required);
  inventory: [ubyte];
  tags: [string];
}
"#,
        &default_opts(),
        "table_strings_vectors",
    );
}

#[test]
fn compile_struct() {
    assert_compiles(
        r#"
struct Vec3 {
  x: float;
  y: float;
  z: float;
}

table Monster {
  pos: Vec3;
}
"#,
        &default_opts(),
        "struct",
    );
}

#[test]
#[ignore] // Known issue: HeroArgs<'a> has unused lifetime when table contains union fields.
fn compile_enum_and_union() {
    assert_compiles(
        r#"
enum Color : byte { Red = 0, Green, Blue }

table Sword { damage: int; }
table Shield { armor: int; }

union Equipment { Sword, Shield }

table Hero {
  color: Color = Green;
  equipped: Equipment;
}
"#,
        &default_opts(),
        "enum_union",
    );
}

#[test]
fn compile_bitflags() {
    assert_compiles(
        r#"
enum Flags : uint (bit_flags) {
  HasHP,
  HasMana,
  HasName,
}

table Monster {
  flags: Flags;
}
"#,
        &default_opts(),
        "bitflags",
    );
}

#[test]
fn compile_nested_namespace() {
    assert_compiles(
        r#"
namespace Game.Characters;

struct Vec2 { x: float; y: float; }

table Monster {
  pos: Vec2;
  hp: int;
}
"#,
        &default_opts(),
        "namespace",
    );
}

#[test]
fn compile_optional_scalars() {
    assert_compiles(
        r#"
table ScalarStuff {
  just_i8: int8;
  maybe_i8: int8 = null;
  default_i8: int8 = 42;
  just_f64: float64;
  maybe_f64: float64 = null;
  just_bool: bool;
  maybe_bool: bool = null;
}
"#,
        &default_opts(),
        "optional_scalars",
    );
}

#[test]
fn compile_keyword_escape() {
    assert_compiles(
        r#"
table Message {
  type: string;
  match: int;
  ref: float;
}
"#,
        &default_opts(),
        "keyword_escape",
    );
}

#[test]
#[ignore] // Known issue (G3.21): Object API skips struct types with array fields.
fn compile_struct_array() {
    assert_compiles(
        r#"
struct Matrix {
  values: [float:9];
}

table Transform {
  m: Matrix;
}
"#,
        &default_opts(),
        "struct_array",
    );
}

#[test]
fn compile_serde_table() {
    assert_compiles(
        r#"
enum Color : byte { Red = 0, Green, Blue }

table Monster {
  name: string;
  hp: int = 100;
  color: Color = Blue;
  inventory: [ubyte];
}
"#,
        &serde_opts(),
        "serde_table",
    );
}

#[test]
fn compile_table_with_key() {
    assert_compiles(
        r#"
table Entry {
  name: string (key);
  value: int;
}
"#,
        &default_opts(),
        "table_key",
    );
}

#[test]
fn compile_nested_table() {
    assert_compiles(
        r#"
table Inner {
  x: int;
}

table Outer {
  inner: Inner;
  items: [Inner];
}
"#,
        &default_opts(),
        "nested_table",
    );
}

// ---------------------------------------------------------------------------
// --no-includes flag tests
// ---------------------------------------------------------------------------

/// Generate Rust code from a schema string, returning the generated source.
fn generate_code(schema_source: &str, opts: &CodeGenOptions) -> String {
    let parser = FbsParser::new(schema_source).with_file_name("test.fbs".to_string());
    let parse_output = parser.parse().unwrap();
    let schema = analyze(parse_output).unwrap();
    generate_rust(&schema, opts).unwrap()
}

#[test]
fn no_includes_omits_use_super() {
    let schema = r#"
namespace Game;
table Monster { hp: int; }
"#;
    // Default: should contain "use super::*;"
    let code_with = generate_code(schema, &default_opts());
    assert!(
        code_with.contains("use super::*;"),
        "default codegen should include 'use super::*;'"
    );

    // With --no-includes: should NOT contain "use super::*;"
    let mut opts = default_opts();
    opts.no_includes = true;
    let code_without = generate_code(schema, &opts);
    assert!(
        !code_without.contains("use super::*;"),
        "codegen with no_includes should omit 'use super::*;'"
    );
}

#[test]
fn no_includes_still_compiles() {
    assert_compiles(
        r#"
namespace Game;
table Monster { hp: int; name: string; }
"#,
        &CodeGenOptions {
            no_includes: true,
            ..default_opts()
        },
        "no_includes",
    );
}

// ---------------------------------------------------------------------------
// --no-leak-private-annotation flag tests
// ---------------------------------------------------------------------------

#[test]
fn private_annotation_generates_pub_crate() {
    let schema = r#"
table PublicTable { x: int; }
table PrivateTable (private) { y: int; }
"#;
    let mut opts = default_opts();
    opts.no_leak_private = true;
    let code = generate_code(schema, &opts);

    // PublicTable should use "pub"
    assert!(
        code.contains("pub struct PublicTable"),
        "PublicTable should be pub"
    );

    // PrivateTable should use "pub(crate)"
    assert!(
        code.contains("pub(crate) struct PrivateTable"),
        "PrivateTable should be pub(crate), got:\n{}",
        code.lines()
            .filter(|l| l.contains("PrivateTable"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn private_annotation_ignored_without_flag() {
    let schema = r#"
table PrivateTable (private) { y: int; }
"#;
    let opts = default_opts(); // no_leak_private = false (default)
    let code = generate_code(schema, &opts);

    // Without the flag, even (private)-annotated types should be "pub"
    assert!(
        code.contains("pub struct PrivateTable"),
        "without --no-leak-private-annotation, PrivateTable should still be pub"
    );
    assert!(
        !code.contains("pub(crate)"),
        "no pub(crate) without the flag"
    );
}

#[test]
fn private_annotation_compiles() {
    assert_compiles(
        r#"
table PublicTable { x: int; }
table PrivateTable (private) { y: int; }
"#,
        &CodeGenOptions {
            no_leak_private: true,
            ..default_opts()
        },
        "private_annotation",
    );
}
