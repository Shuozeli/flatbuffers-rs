use std::path::Path;
use std::process::{Command, Output};

const PURE_GRPC_REV: &str = "2f13145a9d1f0dfa36a430a44e2fceb9ca24b44e";

fn write_schema(crate_dir: &Path) {
    std::fs::create_dir_all(crate_dir).expect("create generated crate directory");
    std::fs::write(
        crate_dir.join("greeter.fbs"),
        r#"
namespace hello.v1;

table HelloRequest {
  name: string;
}

table HelloReply {
  message: string;
}

rpc_service Greeter {
  SayHello(HelloRequest): HelloReply;
}
"#,
    )
    .expect("write FlatBuffers service schema");
}

fn generate_and_check_crate(crate_dir: &Path) -> Output {
    write_schema(crate_dir);
    let generated_dir = crate_dir.join("src");
    let generation = Command::new(env!("CARGO_BIN_EXE_flatc"))
        .arg("--rust")
        .arg("--gen-object-api")
        .arg("-o")
        .arg(&generated_dir)
        .arg(crate_dir.join("greeter.fbs"))
        .output()
        .expect("run grpc-enabled flatc");
    if !generation.status.success() {
        panic!(
            "gRPC code generation failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&generation.stdout),
            String::from_utf8_lossy(&generation.stderr)
        );
    }

    std::fs::rename(
        generated_dir.join("greeter_generated.rs"),
        generated_dir.join("lib.rs"),
    )
    .expect("install generated crate root");
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        format!(
            r#"
[package]
name = "flatc_grpc_generated_check"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
bytes = "1"
flatbuffers = "=25.12.19"
grpc-client = {{ git = "https://github.com/Shuozeli/pure-grpc-rs.git", rev = "{PURE_GRPC_REV}" }}
grpc-codec-flatbuffers = {{ git = "https://github.com/Shuozeli/pure-grpc-rs.git", rev = "{PURE_GRPC_REV}" }}
grpc-core = {{ git = "https://github.com/Shuozeli/pure-grpc-rs.git", rev = "{PURE_GRPC_REV}", default-features = false }}
grpc-server = {{ git = "https://github.com/Shuozeli/pure-grpc-rs.git", rev = "{PURE_GRPC_REV}" }}
http = "1"
http-body = "1"
tower-service = "0.3"
"#,
        ),
    )
    .expect("write generated crate manifest");

    Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .arg("--release")
        .arg("--manifest-path")
        .arg(crate_dir.join("Cargo.toml"))
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .expect("check generated gRPC crate")
}

#[test]
fn grpc_enabled_cli_output_compiles_with_the_pinned_transport() {
    // Arrange
    let temp = tempfile::tempdir().expect("create temporary generated crate");

    // Act
    let output = generate_and_check_crate(temp.path());

    // Assert
    if !output.status.success() {
        panic!(
            "generated gRPC crate did not compile\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
