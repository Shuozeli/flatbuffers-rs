use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    fs::write(".flatc-rs-build-out-dir", out_dir.to_string_lossy().as_bytes())?;

    flatc_rs_build::Builder::new()
        .schemas([
            "schemas/first/schema.fbs",
            "schemas/second/schema.fbs",
        ])
        .compile()?;
    Ok(())
}
