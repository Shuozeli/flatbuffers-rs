use std::path::Path;
use std::process::Command;

use flatc_rs_compiler::{
    analyze,
    codegen::{generate_rust, CodeGenOptions},
    parser::FbsParser,
};

const SCHEMA_SOURCE: &str = r#"
namespace MyGame.Sample;

table Monster {
  hp: short = 100;
  name: string (required);
  inventory: [ubyte];
}

root_type Monster;
file_identifier "MONS";
"#;

fn generated_pluggable_rust_for(schema_source: &str) -> String {
    // Arrange
    let parser = FbsParser::new(schema_source).with_file_name("schema.fbs".to_string());
    let parse_output = parser.parse().expect("parse schema");
    let schema = analyze(parse_output).expect("analyze schema");

    // Act
    generate_rust(
        &schema,
        &CodeGenOptions {
            rust_pluggable_buffer: true,
            gen_only_files: None,
            ..CodeGenOptions::default()
        },
    )
    .expect("generate Rust")
}

fn write_generated_crate_for(crate_dir: &Path, schema_source: &str, main_rs: &str) {
    // Arrange
    let src_dir = crate_dir.join("src");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    let runtime_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("runtime");

    // Act
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        format!(
            r#"
[package]
name = "pluggable_buffer_check"
version = "0.1.0"
edition = "2021"

[dependencies]
flatbuffers = "25.12.19"
flatc-rs-runtime = {{ path = "{}" }}
"#,
            runtime_dir.display()
        ),
    )
    .expect("write Cargo.toml");
    std::fs::write(
        src_dir.join("lib.rs"),
        generated_pluggable_rust_for(schema_source),
    )
    .expect("write generated lib.rs");
    std::fs::write(src_dir.join("main.rs"), main_rs).expect("write main.rs");
}

fn write_generated_crate(crate_dir: &Path, main_rs: &str) {
    write_generated_crate_for(crate_dir, SCHEMA_SOURCE, main_rs);
}

fn cargo_run(crate_dir: &Path) -> std::process::Output {
    // Act
    Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(crate_dir.join("Cargo.toml"))
        .env("CARGO_INCREMENTAL", "0")
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .expect("run generated crate")
}

fn cargo_check(crate_dir: &Path) -> std::process::Output {
    // Act
    Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(crate_dir.join("Cargo.toml"))
        .env("CARGO_INCREMENTAL", "0")
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .expect("check generated crate")
}

fn assert_success(output: std::process::Output, context: &str) {
    // Assert
    if !output.status.success() {
        panic!(
            "{context} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn assert_failure_contains(output: std::process::Output, context: &str, needle: &str) {
    // Assert
    if output.status.success() {
        panic!("{context} unexpectedly succeeded");
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(needle),
        "{context} stderr did not contain {needle:?}\n--- stderr ---\n{stderr}"
    );
}

#[test]
fn generated_reader_uses_custom_buffer() {
    // Arrange
    let tmp = tempfile::tempdir().expect("create tempdir");
    let crate_dir = tmp.path().join("check");
    write_generated_crate(
        &crate_dir,
        r#"
use core::cell::Cell;
use pluggable_buffer_check::__flatc_rs_runtime::FlatBufferRead;
use pluggable_buffer_check::my_game::sample::*;

struct CountingBuffer<'a> {
    bytes: &'a [u8],
    reads: Cell<usize>,
}

unsafe impl<'a> pluggable_buffer_check::__flatc_rs_runtime::FlatBufferRead for CountingBuffer<'a> {
    fn len(&self) -> usize {
        self.bytes.len()
    }

    fn range(&self, start: usize, len: usize) -> Option<&[u8]> {
        self.reads.set(self.reads.get() + 1);
        self.bytes.get(start..start.checked_add(len)?)
    }
}

fn main() {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("orc");
    let inventory = fbb.create_vector(&[1_u8, 2, 3, 4]);
    let root = Monster::create(
        &mut fbb,
        &MonsterArgs {
            hp: 42,
            name: Some(name),
            inventory: Some(inventory),
        },
    );
    pluggable_buffer_check::my_game::sample::finish_monster_buffer(&mut fbb, root);
    let bytes = fbb.finished_data();
    let counting = CountingBuffer {
        bytes,
        reads: Cell::new(0),
    };

    let monster = pluggable_buffer_check::my_game::sample::root_as_monster_in(&counting).expect("read root");
    assert_eq!(monster.hp(), 42);
    assert_eq!(monster.name(), "orc");
    let inventory = monster.inventory().expect("inventory vector");
    assert_eq!(inventory.len(), 4);
    assert_eq!(inventory.get(2), 3);
    assert!(counting.reads.get() > 0);
    assert_eq!(counting.read_byte(0), Some(bytes[0]));
}
"#,
    );

    // Act
    let output = cargo_run(&crate_dir);

    // Assert
    assert_success(output, "generated pluggable-buffer crate");
}

#[test]
fn verified_root_rejects_buffer_without_contiguous_view() {
    // Arrange
    let tmp = tempfile::tempdir().expect("create tempdir");
    let crate_dir = tmp.path().join("check");
    write_generated_crate(
        &crate_dir,
        r#"
use pluggable_buffer_check::my_game::sample::*;

struct RangeOnlyBuffer<'a> {
    bytes: &'a [u8],
}

unsafe impl<'a> pluggable_buffer_check::__flatc_rs_runtime::FlatBufferRead for RangeOnlyBuffer<'a> {
    fn len(&self) -> usize {
        self.bytes.len()
    }

    fn range(&self, start: usize, len: usize) -> Option<&[u8]> {
        self.bytes.get(start..start.checked_add(len)?)
    }

    fn all_bytes(&self) -> Option<&[u8]> {
        None
    }
}

fn main() {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("orc");
    let root = Monster::create(
        &mut fbb,
        &MonsterArgs {
            hp: 42,
            name: Some(name),
            inventory: None,
        },
    );
    pluggable_buffer_check::my_game::sample::finish_monster_buffer(&mut fbb, root);
    let buffer = RangeOnlyBuffer {
        bytes: fbb.finished_data(),
    };

    assert!(pluggable_buffer_check::my_game::sample::root_as_monster_in(&buffer).is_err());
}
"#,
    );

    // Act
    let output = cargo_run(&crate_dir);

    // Assert
    assert_success(output, "range-only verified-root check");
}

#[test]
fn reader_cannot_outlive_owned_buffer() {
    // Arrange
    let tmp = tempfile::tempdir().expect("create tempdir");
    let crate_dir = tmp.path().join("check");
    write_generated_crate(
        &crate_dir,
        r#"
use pluggable_buffer_check::my_game::sample::*;

fn build_reader() -> Monster<'static> {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("orc");
    let root = Monster::create(
        &mut fbb,
        &MonsterArgs {
            hp: 42,
            name: Some(name),
            inventory: None,
        },
    );
    pluggable_buffer_check::my_game::sample::finish_monster_buffer(&mut fbb, root);
    let bytes = fbb.finished_data();
    pluggable_buffer_check::my_game::sample::root_as_monster(bytes).unwrap()
}

fn main() {
    let _ = build_reader();
}
"#,
    );

    // Act
    let output = cargo_check(&crate_dir);

    // Assert
    assert_failure_contains(
        output,
        "reader outliving FlatBufferBuilder-owned bytes",
        "cannot return value referencing local variable",
    );
}

#[test]
fn buffer_cannot_be_mutated_while_reader_is_alive() {
    // Arrange
    let tmp = tempfile::tempdir().expect("create tempdir");
    let crate_dir = tmp.path().join("check");
    write_generated_crate(
        &crate_dir,
        r#"
use pluggable_buffer_check::my_game::sample::*;

struct OwnedBuffer {
    bytes: Vec<u8>,
}

unsafe impl pluggable_buffer_check::__flatc_rs_runtime::FlatBufferRead for OwnedBuffer {
    fn len(&self) -> usize {
        self.bytes.len()
    }

    fn range(&self, start: usize, len: usize) -> Option<&[u8]> {
        self.bytes.get(start..start.checked_add(len)?)
    }
}

fn main() {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("orc");
    let root = Monster::create(
        &mut fbb,
        &MonsterArgs {
            hp: 42,
            name: Some(name),
            inventory: None,
        },
    );
    pluggable_buffer_check::my_game::sample::finish_monster_buffer(&mut fbb, root);
    let mut buffer = OwnedBuffer {
        bytes: fbb.finished_data().to_vec(),
    };

    let monster = pluggable_buffer_check::my_game::sample::root_as_monster_in(&buffer).unwrap();
    buffer.bytes.clear();
    let _ = monster.hp();
}
"#,
    );

    // Act
    let output = cargo_check(&crate_dir);

    // Assert
    assert_failure_contains(
        output,
        "mutating owned buffer while reader is alive",
        "cannot borrow",
    );
}

#[test]
fn owned_buffer_can_be_mutated_after_reader_is_dropped() {
    // Arrange
    let tmp = tempfile::tempdir().expect("create tempdir");
    let crate_dir = tmp.path().join("check");
    write_generated_crate(
        &crate_dir,
        r#"
use pluggable_buffer_check::my_game::sample::*;

struct OwnedBuffer {
    bytes: Vec<u8>,
}

unsafe impl pluggable_buffer_check::__flatc_rs_runtime::FlatBufferRead for OwnedBuffer {
    fn len(&self) -> usize {
        self.bytes.len()
    }

    fn range(&self, start: usize, len: usize) -> Option<&[u8]> {
        self.bytes.get(start..start.checked_add(len)?)
    }
}

fn main() {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("orc");
    let root = Monster::create(
        &mut fbb,
        &MonsterArgs {
            hp: 42,
            name: Some(name),
            inventory: None,
        },
    );
    pluggable_buffer_check::my_game::sample::finish_monster_buffer(&mut fbb, root);
    let mut buffer = OwnedBuffer {
        bytes: fbb.finished_data().to_vec(),
    };

    {
        let monster = pluggable_buffer_check::my_game::sample::root_as_monster_in(&buffer).unwrap();
        assert_eq!(monster.hp(), 42);
    }

    buffer.bytes.clear();
    assert!(buffer.bytes.is_empty());
}
"#,
    );

    // Act
    let output = cargo_run(&crate_dir);

    // Assert
    assert_success(output, "mutating owned buffer after reader drop");
}

#[test]
fn builder_accepts_custom_allocator_for_write_ownership() {
    // Arrange
    let tmp = tempfile::tempdir().expect("create tempdir");
    let crate_dir = tmp.path().join("check");
    write_generated_crate(
        &crate_dir,
        r#"
use std::convert::Infallible;
use std::ops::{Deref, DerefMut};

use pluggable_buffer_check::my_game::sample::*;

struct TrackingAllocator {
    bytes: Vec<u8>,
    grows: usize,
}

impl TrackingAllocator {
    fn new(capacity: usize) -> Self {
        Self {
            bytes: vec![0; capacity],
            grows: 0,
        }
    }
}

impl Deref for TrackingAllocator {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl DerefMut for TrackingAllocator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytes
    }
}

unsafe impl flatbuffers::Allocator for TrackingAllocator {
    type Error = Infallible;

    fn grow_downwards(&mut self) -> Result<(), Self::Error> {
        let old_len = self.bytes.len();
        let new_len = std::cmp::max(1, old_len * 2);
        self.bytes.resize(new_len, 0);
        if old_len > 0 {
            self.bytes.copy_within(0..old_len, new_len - old_len);
            self.bytes[..new_len - old_len].fill(0);
        }
        self.grows += 1;
        Ok(())
    }

    fn len(&self) -> usize {
        self.bytes.len()
    }
}

fn main() {
    let mut scratch = TrackingAllocator::new(4);
    assert_eq!(
        pluggable_buffer_check::__flatc_rs_runtime::write_byte(&mut scratch, 0, 7),
        Some(())
    );
    assert_eq!(scratch.bytes[0], 7);

    let allocator = TrackingAllocator::new(1);
    let mut fbb = flatbuffers::FlatBufferBuilder::new_in(allocator);
    let name = fbb.create_string("orc");
    let inventory = fbb.create_vector(&[1_u8, 2, 3, 4]);
    let root = Monster::create(
        &mut fbb,
        &MonsterArgs {
            hp: 42,
            name: Some(name),
            inventory: Some(inventory),
        },
    );
    pluggable_buffer_check::my_game::sample::finish_monster_buffer(&mut fbb, root);
    let bytes = fbb.finished_data();
    let monster = pluggable_buffer_check::my_game::sample::root_as_monster(bytes).unwrap();
    assert_eq!(monster.hp(), 42);
    assert_eq!(monster.name(), "orc");
}
"#,
    );

    // Act
    let output = cargo_run(&crate_dir);

    // Assert
    assert_success(output, "custom allocator write path");
}

#[test]
fn safe_code_cannot_construct_unverified_reader_directly() {
    // Arrange
    let tmp = tempfile::tempdir().expect("create tempdir");
    let crate_dir = tmp.path().join("check");
    write_generated_crate(
        &crate_dir,
        r#"
use pluggable_buffer_check::my_game::sample::*;

fn main() {
    let bytes: &[u8] = &[0, 0, 0, 0];
    let _monster = Monster {
        _buf: bytes,
        _loc: 0,
    };
}
"#,
    );

    // Act
    let output = cargo_check(&crate_dir);

    // Assert
    assert_failure_contains(output, "direct safe reader construction", "private field");
}

#[test]
fn pluggable_reader_compiles_complex_schema_features() {
    // Arrange
    let tmp = tempfile::tempdir().expect("create tempdir");
    let crate_dir = tmp.path().join("check");
    let schema = r#"
namespace MyGame.Complex;

struct Vec3 {
  x: float;
  y: float;
  z: float;
}

table Weapon {
  damage: int;
}

union Equipment {
  Weapon,
  Vec3,
}

table Holder {
  pos: Vec3;
  path: [Vec3];
  weapon: Weapon;
  weapons: [Weapon];
  names: [string];
  equipped: Equipment;
}

root_type Holder;
"#;
    write_generated_crate_for(
        &crate_dir,
        schema,
        r#"
use pluggable_buffer_check::my_game::complex::*;

fn main() {
    let _ = core::mem::size_of::<Holder<'_, [u8]>>();
}
"#,
    );

    // Act
    let output = cargo_check(&crate_dir);

    // Assert
    assert_success(output, "complex pluggable schema compile");
}

#[test]
fn nested_flatbuffer_accessor_compiles_in_pluggable_mode() {
    // Arrange
    let tmp = tempfile::tempdir().expect("create tempdir");
    let crate_dir = tmp.path().join("check");
    let schema = r#"
table Inner {
  hp: int = 100;
}

table Outer {
  data: [ubyte] (nested_flatbuffer: "Inner");
}

root_type Outer;
"#;
    write_generated_crate_for(
        &crate_dir,
        schema,
        r#"
fn main() {
    let _ = core::mem::size_of::<pluggable_buffer_check::Outer<'_, [u8]>>();
}
"#,
    );

    // Act
    let output = cargo_check(&crate_dir);

    // Assert
    assert_success(output, "nested flatbuffer pluggable compile");
}

#[test]
fn size_prefixed_root_reads_custom_buffer() {
    // Arrange
    let tmp = tempfile::tempdir().expect("create tempdir");
    let crate_dir = tmp.path().join("check");
    write_generated_crate(
        &crate_dir,
        r#"
use pluggable_buffer_check::my_game::sample::*;

struct OwnedBuffer {
    bytes: Vec<u8>,
}

unsafe impl pluggable_buffer_check::__flatc_rs_runtime::FlatBufferRead for OwnedBuffer {
    fn len(&self) -> usize {
        self.bytes.len()
    }

    fn range(&self, start: usize, len: usize) -> Option<&[u8]> {
        self.bytes.get(start..start.checked_add(len)?)
    }
}

fn main() {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("orc");
    let root = Monster::create(
        &mut fbb,
        &MonsterArgs {
            hp: 42,
            name: Some(name),
            inventory: None,
        },
    );
    pluggable_buffer_check::my_game::sample::finish_size_prefixed_monster_buffer(&mut fbb, root);
    let buffer = OwnedBuffer {
        bytes: fbb.finished_data().to_vec(),
    };

    let monster = pluggable_buffer_check::my_game::sample::size_prefixed_root_as_monster_in(&buffer).unwrap();
    assert_eq!(monster.hp(), 42);
    assert_eq!(monster.name(), "orc");
}
"#,
    );

    // Act
    let output = cargo_run(&crate_dir);

    // Assert
    assert_success(output, "size-prefixed custom buffer");
}
