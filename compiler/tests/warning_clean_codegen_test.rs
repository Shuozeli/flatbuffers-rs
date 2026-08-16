//! Downstream warning contracts for generated Rust.

use flatc_rs_compiler::{
    analyze,
    codegen::{generate_rust, CodeGenOptions},
    parser::FbsParser,
};
use std::fs;
use std::process::Command;

#[test]
fn generated_rust_compiles_with_warnings_denied() {
    // Arrange
    let source = r#"
namespace warning.clean;

table TextPayload {
  value: string;
}

table EmptyPayload {
}

union Payload {
  TextPayload
}

table MobileEvent {
  old_id: string (deprecated);
  old_payload: Payload (deprecated);
  count: int;
}

root_type MobileEvent;
"#;
    let parsed = FbsParser::new(source)
        .with_file_name("mobile_event.fbs".to_string())
        .parse()
        .expect("parse warning-clean fixture");
    let schema = analyze(parsed).expect("analyze warning-clean fixture");
    let generated = generate_rust(
        &schema,
        &CodeGenOptions {
            gen_object_api: true,
            rust_serialize: true,
            ..CodeGenOptions::default()
        },
    )
    .expect("generate Object API and serde Rust");
    assert!(
        generated.contains("_args: &'args EmptyPayloadArgs"),
        "empty table create arguments must be explicitly unused"
    );
    assert!(
        generated.contains("let builder = EmptyPayloadBuilder::new(_fbb);"),
        "empty table builders must not be unnecessarily mutable"
    );
    assert!(
        generated.contains("let s = serializer.serialize_struct(\"EmptyPayload\", 0)?;"),
        "empty table serializers must not be unnecessarily mutable"
    );
    assert!(
        generated.contains(
            ".visit_field::<::flatbuffers::ForwardsUOffset<&str>>(\"old_id\", Self::VT_OLD_ID, false)?"
        ),
        "deprecated readable fields must remain covered by generated verification"
    );
    assert!(
        generated.contains(
            ".visit_union::<Payload, _>(\"old_payload_type\", Self::VT_OLD_PAYLOAD_TYPE, \"old_payload\", Self::VT_OLD_PAYLOAD, false"
        ),
        "deprecated readable unions must verify their discriminator and payload together"
    );
    let project = tempfile::tempdir().expect("create downstream project");
    fs::create_dir(project.path().join("src")).expect("create downstream src");
    fs::write(
        project.path().join("Cargo.toml"),
        r#"[package]
name = "warning-clean-deprecated-codegen"
version = "0.0.0"
edition = "2021"

[dependencies]
flatbuffers = { version = "25.12.19", features = ["serialize"] }
serde = { version = "1", features = ["derive"] }
"#,
    )
    .expect("write downstream manifest");
    fs::write(
        project.path().join("src/lib.rs"),
        "#![deny(warnings)]\ninclude!(\"generated.rs\");\n",
    )
    .expect("write downstream crate root");
    fs::write(project.path().join("src/generated.rs"), generated).expect("write generated Rust");

    // Act
    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
        .args(["check", "--release", "--quiet"])
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-Dwarnings")
        .env_remove("CARGO_TARGET_DIR")
        .current_dir(project.path())
        .output()
        .expect("check downstream generated crate");

    // Assert
    assert!(
        output.status.success(),
        "generated deprecated-field Object API was not warning-clean\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
