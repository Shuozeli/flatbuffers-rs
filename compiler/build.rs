use std::env;
use std::fs;
use std::path::Path;

fn generate_tests_for_dir(dir: &Path, fn_prefix: &str, runner_fn: &str) -> String {
    let mut code = String::new();
    if dir.is_dir() {
        let mut names: Vec<String> = Vec::new();
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) == Some("fbs") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    names.push(stem.to_string());
                }
            }
            println!("cargo:rerun-if-changed={}", path.display());
        }
        names.sort();

        for name in &names {
            code.push_str(&format!(
                "#[test]\nfn {fn_prefix}{name}() {{\n    {runner_fn}(\"{name}\");\n}}\n\n"
            ));
        }
    }
    code
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // Analyzer golden tests
    let golden_dir = Path::new("testdata/golden");
    println!("cargo:rerun-if-changed=testdata/golden");
    let code = generate_tests_for_dir(golden_dir, "golden_", "run_single_golden");
    fs::write(Path::new(&out_dir).join("golden_tests_generated.rs"), code).unwrap();

    // Codegen golden tests
    let codegen_dir = Path::new("testdata/codegen_golden");
    println!("cargo:rerun-if-changed=testdata/codegen_golden");
    let code = generate_tests_for_dir(codegen_dir, "codegen_", "run_single_codegen_golden");
    fs::write(Path::new(&out_dir).join("codegen_tests_generated.rs"), code).unwrap();

    // TypeScript codegen golden tests
    let ts_codegen_dir = Path::new("testdata/ts_codegen_golden");
    println!("cargo:rerun-if-changed=testdata/ts_codegen_golden");
    let code = generate_tests_for_dir(
        ts_codegen_dir,
        "ts_codegen_",
        "run_single_ts_codegen_golden",
    );
    fs::write(
        Path::new(&out_dir).join("ts_codegen_tests_generated.rs"),
        code,
    )
    .unwrap();

    // Serde codegen golden tests
    let serde_codegen_dir = Path::new("testdata/serde_codegen_golden");
    println!("cargo:rerun-if-changed=testdata/serde_codegen_golden");
    let code = generate_tests_for_dir(
        serde_codegen_dir,
        "serde_codegen_",
        "run_single_serde_codegen_golden",
    );
    fs::write(
        Path::new(&out_dir).join("serde_codegen_tests_generated.rs"),
        code,
    )
    .unwrap();

    // Dart codegen golden tests
    let dart_codegen_dir = Path::new("testdata/dart_codegen_golden");
    println!("cargo:rerun-if-changed=testdata/dart_codegen_golden");
    let code = generate_tests_for_dir(
        dart_codegen_dir,
        "dart_codegen_",
        "run_single_dart_codegen_golden",
    );
    fs::write(
        Path::new(&out_dir).join("dart_codegen_tests_generated.rs"),
        code,
    )
    .unwrap();
}
