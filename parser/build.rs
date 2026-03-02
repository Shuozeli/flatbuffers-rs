use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let golden_dir = Path::new("testdata/golden");
    println!("cargo:rerun-if-changed=testdata/golden");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("golden_tests_generated.rs");

    let mut code = String::new();

    if golden_dir.is_dir() {
        let mut names: Vec<String> = Vec::new();
        for entry in fs::read_dir(golden_dir).unwrap() {
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
                "#[test]\nfn golden_{name}() {{\n    run_single_golden(\"{name}\");\n}}\n\n"
            ));
        }
    }

    fs::write(dest, code).unwrap();
}
