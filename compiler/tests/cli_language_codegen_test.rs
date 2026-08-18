#[cfg(feature = "grpc")]
use std::path::Path;
use std::process::Command;

#[cfg(feature = "grpc")]
fn assert_multi_input_services_are_isolated(flag: &str, extension: &str, extra_flags: &[&str]) {
    // Arrange
    let tmp = tempfile::tempdir().unwrap();
    let schema_a = tmp.path().join("a.fbs");
    let schema_b = tmp.path().join("b.fbs");
    let out_dir = tmp.path().join(extension);
    write_service_schema(&schema_a, "A", "A");
    write_service_schema(&schema_b, "B", "B");

    // Act
    let mut command = Command::new(env!("CARGO_BIN_EXE_flatc"));
    command.arg("-o").arg(&out_dir).arg(flag);
    for extra_flag in extra_flags {
        command.arg(extra_flag);
    }
    let output = command.arg(&schema_a).arg(&schema_b).output().unwrap();

    // Assert
    assert!(
        output.status.success(),
        "{flag} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let a_code = std::fs::read_to_string(out_dir.join(format!("a_generated.{extension}"))).unwrap();
    let b_code = std::fs::read_to_string(out_dir.join(format!("b_generated.{extension}"))).unwrap();
    assert!(a_code.contains("ServiceA"));
    assert!(!a_code.contains("ServiceB"));
    assert!(!b_code.contains("ServiceA"));
    assert!(b_code.contains("ServiceB"));
}

#[cfg(feature = "grpc")]
fn write_service_schema(path: &Path, namespace: &str, suffix: &str) {
    std::fs::write(
        path,
        format!(
            "namespace {namespace};\ntable Req{suffix} {{ value:int; }}\ntable Res{suffix} {{ value:int; }}\nrpc_service Service{suffix} {{ Call{suffix}(Req{suffix}):Res{suffix}; }}\n"
        ),
    )
    .unwrap();
}

fn run_flatc(test_name: &str, flags: &[&str], expected_files: &[&str]) {
    // Arrange
    let tmp = tempfile::tempdir()
        .unwrap_or_else(|e| panic!("{test_name}: failed to create tempdir: {e}"));
    let schema_path = tmp.path().join("monster.fbs");
    let out_dir = tmp.path().join("out");
    std::fs::write(
        &schema_path,
        r#"
            namespace Cli.Test;
            enum Color: byte { Red = 1, Green = 2, Blue = 8 }
            struct Vec3 { x: float; y: float; z: float; }
            table Monster {
                pos: Vec3;
                hp: short = 100;
                name: string;
                color: Color = Blue;
                inventory: [ubyte];
            }
            root_type Monster;
        "#,
    )
    .unwrap_or_else(|e| panic!("{test_name}: failed to write schema: {e}"));

    // Act
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_flatc"));
    cmd.arg("-o").arg(&out_dir);
    for flag in flags {
        cmd.arg(flag);
    }
    cmd.arg(&schema_path);
    let output = cmd
        .output()
        .unwrap_or_else(|e| panic!("{test_name}: failed to run flatc: {e}"));

    // Assert
    if !output.status.success() {
        panic!(
            "{test_name}: flatc failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    for expected in expected_files {
        let path = out_dir.join(expected);
        assert!(
            path.exists(),
            "{test_name}: expected generated file {}",
            path.display()
        );
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{test_name}: failed to read {}: {e}", path.display()));
        assert!(
            !content.trim().is_empty(),
            "{test_name}: generated file {} is empty",
            path.display()
        );
    }
}

#[test]
fn cli_generates_python() {
    run_flatc("python_codegen", &["--python"], &["monster_generated.py"]);
}

#[test]
fn cli_nodejs_alias_generates_typescript() {
    run_flatc("nodejs_alias", &["--nodejs"], &["monster_generated.ts"]);
}

#[test]
fn cli_rejects_removed_dart_backend() {
    // Arrange
    let tmp = tempfile::tempdir().unwrap();
    let schema = tmp.path().join("empty.fbs");
    std::fs::write(&schema, "table Empty {}\n").unwrap();

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_flatc"))
        .arg("--dart")
        .arg(&schema)
        .output()
        .unwrap();

    // Assert
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument '--dart'"));
}

#[test]
fn cli_multi_input_codegen_isolates_each_output_for_every_language() {
    // Arrange
    let tmp = tempfile::tempdir().unwrap();
    let schema_a = tmp.path().join("a.fbs");
    let schema_b = tmp.path().join("b.fbs");
    std::fs::write(
        &schema_a,
        "file_identifier \"AAAA\";\ntable TableA { value:int; }\nroot_type TableA;\n",
    )
    .unwrap();
    std::fs::write(
        &schema_b,
        "file_identifier \"BBBB\";\ntable TableB { value:int; }\nroot_type TableB;\n",
    )
    .unwrap();

    for (flag, extension) in [("--rust", "rs"), ("--ts", "ts"), ("--python", "py")] {
        let out_dir = tmp.path().join(extension);

        // Act
        let output = Command::new(env!("CARGO_BIN_EXE_flatc"))
            .arg("-o")
            .arg(&out_dir)
            .arg(flag)
            .arg(&schema_a)
            .arg(&schema_b)
            .output()
            .unwrap();

        // Assert
        assert!(
            output.status.success(),
            "{flag} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let a_code =
            std::fs::read_to_string(out_dir.join(format!("a_generated.{extension}"))).unwrap();
        let b_code =
            std::fs::read_to_string(out_dir.join(format!("b_generated.{extension}"))).unwrap();
        assert!(a_code.contains("TableA"), "{flag} omitted TableA");
        assert!(!a_code.contains("TableB"), "{flag} leaked TableB into A");
        assert!(b_code.contains("TableB"), "{flag} omitted TableB");
        assert!(!b_code.contains("TableA"), "{flag} leaked TableA into B");
        assert_ne!(a_code, b_code, "{flag} generated duplicate outputs");

        if flag == "--rust" {
            assert!(a_code.contains("root_as_table_a"));
            assert!(!a_code.contains("root_as_table_b"));
            assert!(a_code.contains("\"AAAA\""));
            assert!(!a_code.contains("\"BBBB\""));
            assert!(!b_code.contains("root_as_table_a"));
            assert!(b_code.contains("root_as_table_b"));
            assert!(!b_code.contains("\"AAAA\""));
            assert!(b_code.contains("\"BBBB\""));
        }
    }
}

#[test]
fn cli_multi_input_allows_independent_duplicate_type_names() {
    // Arrange
    let tmp = tempfile::tempdir().unwrap();
    let schema_a = tmp.path().join("a.fbs");
    let schema_b = tmp.path().join("b.fbs");
    let out_dir = tmp.path().join("out");
    std::fs::write(&schema_a, "table Root { alpha:int; }\nroot_type Root;\n").unwrap();
    std::fs::write(&schema_b, "table Root { beta:string; }\nroot_type Root;\n").unwrap();

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_flatc"))
        .arg("-o")
        .arg(&out_dir)
        .arg("--rust")
        .arg(&schema_a)
        .arg(&schema_b)
        .output()
        .unwrap();

    // Assert
    assert!(
        output.status.success(),
        "flatc failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let a_code = std::fs::read_to_string(out_dir.join("a_generated.rs")).unwrap();
    let b_code = std::fs::read_to_string(out_dir.join("b_generated.rs")).unwrap();
    assert!(a_code.contains("pub fn alpha"));
    assert!(!a_code.contains("pub fn beta"));
    assert!(!b_code.contains("pub fn alpha"));
    assert!(b_code.contains("pub fn beta"));
    assert!(a_code.contains("root_as_root"));
    assert!(b_code.contains("root_as_root"));
}

#[cfg(feature = "grpc")]
#[test]
fn cli_multi_input_rust_services_are_isolated() {
    assert_multi_input_services_are_isolated("--rust", "rs", &["--gen-object-api"]);
}

#[cfg(feature = "grpc")]
#[test]
fn cli_rust_ignores_included_services_without_gen_all() {
    // Arrange
    let tmp = tempfile::tempdir().unwrap();
    let shared = tmp.path().join("shared.fbs");
    let schema = tmp.path().join("main.fbs");
    let out_dir = tmp.path().join("out");
    write_service_schema(&shared, "Shared", "Shared");
    std::fs::write(
        &schema,
        "include \"shared.fbs\";\ntable Local { value:int; }\n",
    )
    .unwrap();

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_flatc"))
        .arg("-o")
        .arg(&out_dir)
        .arg("--rust")
        .arg(&schema)
        .output()
        .unwrap();

    // Assert
    assert!(
        output.status.success(),
        "flatc failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let code = std::fs::read_to_string(out_dir.join("main_generated.rs")).unwrap();
    assert!(!code.contains("ServiceShared"));
}

#[test]
fn cli_multi_input_bfbs_is_scoped_to_each_schema() {
    // Arrange
    let tmp = tempfile::tempdir().unwrap();
    let schema_a = tmp.path().join("a.fbs");
    let schema_b = tmp.path().join("b.fbs");
    let out_dir = tmp.path().join("out");
    std::fs::write(
        &schema_a,
        "file_identifier \"AAAA\";\ntable RootA { value:int; }\nroot_type RootA;\n",
    )
    .unwrap();
    std::fs::write(
        &schema_b,
        "file_identifier \"BBBB\";\ntable RootB { value:int; }\nroot_type RootB;\n",
    )
    .unwrap();

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_flatc"))
        .arg("-o")
        .arg(&out_dir)
        .arg("--schema")
        .arg(&schema_a)
        .arg(&schema_b)
        .output()
        .unwrap();

    // Assert
    assert!(
        output.status.success(),
        "flatc failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let a_bytes = std::fs::read(out_dir.join("a.bfbs")).unwrap();
    let b_bytes = std::fs::read(out_dir.join("b.bfbs")).unwrap();
    let a = flatc_rs_compiler::bfbs::deserialize_schema(&a_bytes).unwrap();
    let b = flatc_rs_compiler::bfbs::deserialize_schema(&b_bytes).unwrap();
    assert_eq!(a.file_ident.as_deref(), Some("AAAA"));
    assert_eq!(b.file_ident.as_deref(), Some("BBBB"));
    assert_eq!(a.objects.len(), 1);
    assert_eq!(a.objects[0].name.as_deref(), Some("RootA"));
    assert_eq!(b.objects.len(), 1);
    assert_eq!(b.objects[0].name.as_deref(), Some("RootB"));
    assert_ne!(a_bytes, b_bytes);
}

#[test]
fn cli_gen_all_keeps_included_declarations() {
    // Arrange
    let tmp = tempfile::tempdir().unwrap();
    let shared = tmp.path().join("shared.fbs");
    let schema = tmp.path().join("message.fbs");
    let out_dir = tmp.path().join("out");
    std::fs::write(&shared, "table SharedValue { value:int; }\n").unwrap();
    std::fs::write(
        &schema,
        "include \"shared.fbs\";\ntable Message { shared:SharedValue; }\n",
    )
    .unwrap();

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_flatc"))
        .arg("-o")
        .arg(&out_dir)
        .arg("--rust")
        .arg("--gen-all")
        .arg(&schema)
        .output()
        .unwrap();

    // Assert
    assert!(
        output.status.success(),
        "flatc failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let code = std::fs::read_to_string(out_dir.join("message_generated.rs")).unwrap();
    assert!(code.contains("pub struct Message"));
    assert!(code.contains("pub struct SharedValue"));
}

#[test]
fn cli_rust_pluggable_buffer_generates_runtime_adapter() {
    // Arrange
    let tmp = tempfile::tempdir()
        .unwrap_or_else(|e| panic!("rust_pluggable_buffer: failed to create tempdir: {e}"));
    let schema_path = tmp.path().join("monster.fbs");
    let out_dir = tmp.path().join("out");
    std::fs::write(
        &schema_path,
        r#"
            namespace Cli.Test;
            table Monster {
                hp: short = 100;
                name: string;
                inventory: [ubyte];
            }
            root_type Monster;
        "#,
    )
    .unwrap_or_else(|e| panic!("rust_pluggable_buffer: failed to write schema: {e}"));

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_flatc"))
        .arg("-o")
        .arg(&out_dir)
        .arg("--rust")
        .arg("--rust-pluggable-buffer")
        .arg(&schema_path)
        .output()
        .unwrap_or_else(|e| panic!("rust_pluggable_buffer: failed to run flatc: {e}"));

    // Assert
    if !output.status.success() {
        panic!(
            "rust_pluggable_buffer: flatc failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let generated = std::fs::read_to_string(out_dir.join("monster_generated.rs"))
        .unwrap_or_else(|e| panic!("rust_pluggable_buffer: failed to read generated Rust: {e}"));
    assert!(generated.contains("pub mod __flatc_rs_runtime"));
    assert!(generated.contains("flatc_rs_runtime"));
    assert!(generated.contains("FlatBufferRead"));
    assert!(generated.contains("root_as_monster_in"));
}

#[cfg(feature = "grpc")]
#[test]
fn cli_requires_object_api_for_grpc_service_messages() {
    // Arrange
    let tmp = tempfile::tempdir()
        .unwrap_or_else(|e| panic!("grpc_object_api: failed to create tempdir: {e}"));
    let schema_path = tmp.path().join("greeter.fbs");
    let out_dir = tmp.path().join("out");
    std::fs::write(
        &schema_path,
        r#"
            table HelloRequest { name: string; }
            table HelloReply { message: string; }
            rpc_service Greeter { SayHello(HelloRequest): HelloReply; }
        "#,
    )
    .unwrap_or_else(|e| panic!("grpc_object_api: failed to write schema: {e}"));

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_flatc"))
        .arg("-o")
        .arg(&out_dir)
        .arg("--rust")
        .arg(&schema_path)
        .output()
        .unwrap_or_else(|e| panic!("grpc_object_api: failed to run flatc: {e}"));

    // Assert
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("gRPC code generation requires gen_object_api"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
