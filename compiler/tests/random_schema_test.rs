//! Property tests: randomly generated FBS schemas must compile and codegen
//! without errors. Uses deterministic seeds for reproducibility.

mod fbs_gen;

use fbs_gen::{GenConfig, SchemaBuilder};
use flatc_rs_compiler::{
    compile_single,
    codegen::{generate_rust, generate_typescript, CodeGenOptions, TsCodeGenOptions},
};

const NUM_SEEDS: u64 = 500;

#[test]
fn random_schema_compiles_rust() {
    for seed in 0..NUM_SEEDS {
        let config = GenConfig::default();
        let fbs_text = SchemaBuilder::generate(seed, config);

        let result = compile_single(&fbs_text).unwrap_or_else(|e| {
            panic!("seed {seed} failed to compile:\n---\n{fbs_text}---\nerror: {e}");
        });

        let opts = CodeGenOptions {
            gen_object_api: true,
            gen_name_constants: true,
            ..CodeGenOptions::default()
        };
        generate_rust(&result.schema, &opts).unwrap_or_else(|e| {
            panic!("seed {seed} failed Rust codegen:\n---\n{fbs_text}---\nerror: {e}");
        });
    }
}

#[test]
fn random_schema_compiles_typescript() {
    for seed in 0..NUM_SEEDS {
        let config = GenConfig::default();
        let fbs_text = SchemaBuilder::generate(seed, config);

        let result = compile_single(&fbs_text).unwrap_or_else(|e| {
            panic!("seed {seed} failed to compile:\n---\n{fbs_text}---\nerror: {e}");
        });

        let opts = TsCodeGenOptions {
            gen_object_api: true,
            gen_only_files: None,
        };
        generate_typescript(&result.schema, &opts).unwrap_or_else(|e| {
            panic!("seed {seed} failed TS codegen:\n---\n{fbs_text}---\nerror: {e}");
        });
    }
}

/// Cross-compatibility test with C++ flatc. Gated on FLATC_PATH env var.
#[test]
fn random_schema_matches_cpp_flatc() {
    let flatc_path = match std::env::var("FLATC_PATH") {
        Ok(p) => std::path::PathBuf::from(p),
        Err(_) => {
            eprintln!("FLATC_PATH not set, skipping C++ comparison test");
            return;
        }
    };

    for seed in 0..50 {
        let config = GenConfig::default();
        let fbs_text = SchemaBuilder::generate(seed, config);

        let dir = tempfile::tempdir().unwrap();
        let fbs_path = dir.path().join("test.fbs");
        std::fs::write(&fbs_path, &fbs_text).unwrap();

        // Compile with flatc-rs
        let rs_result = match compile_single(&fbs_text) {
            Ok(r) => r,
            Err(e) => {
                panic!("seed {seed}: flatc-rs rejected schema:\n{fbs_text}\nerror: {e}");
            }
        };
        let rs_opts = CodeGenOptions {
            gen_object_api: false,
            ..CodeGenOptions::default()
        };
        let rs_code = generate_rust(&rs_result.schema, &rs_opts).unwrap();

        // Compile with C++ flatc
        let cpp_output = std::process::Command::new(&flatc_path)
            .args(["--rust", "-o", dir.path().to_str().unwrap()])
            .arg(&fbs_path)
            .output()
            .unwrap();

        if !cpp_output.status.success() {
            let stderr = String::from_utf8_lossy(&cpp_output.stderr);
            panic!(
                "seed {seed}: C++ flatc rejected schema that flatc-rs accepted:\n\
                 {fbs_text}\nflatc stderr: {stderr}"
            );
        }

        let cpp_path = dir.path().join("test_generated.rs");
        if cpp_path.exists() {
            let cpp_code = std::fs::read_to_string(&cpp_path).unwrap();
            if normalize(&rs_code) != normalize(&cpp_code) {
                let rs_path = dir.path().join("rs_output.rs");
                std::fs::write(&rs_path, &rs_code).unwrap();
                panic!(
                    "seed {seed}: output mismatch!\nSchema:\n{fbs_text}\n\
                     Diff: diff {} {}",
                    rs_path.display(),
                    cpp_path.display()
                );
            }
        }
    }
}

/// Normalize whitespace for comparison: collapse runs of blank lines.
fn normalize(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_blank = false;
    for line in s.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            if !prev_blank {
                result.push('\n');
            }
            prev_blank = true;
        } else {
            result.push_str(trimmed);
            result.push('\n');
            prev_blank = false;
        }
    }
    result
}
