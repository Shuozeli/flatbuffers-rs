use std::process::Command;

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
