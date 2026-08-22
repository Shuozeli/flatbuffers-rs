use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    fs::write(".flatc-rs-build-out-dir", out_dir.to_string_lossy().as_bytes())?;
    let counter = out_dir.join("flatc-rs-build-runs");
    let runs = fs::read_to_string(&counter)
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0)
        + 1;
    fs::write(counter, runs.to_string())?;

    let mut builder = flatc_rs_build::Builder::new()
        .schema("schemas/game.fbs")
        .include_dir("schemas")
        .gen_all()
        .rerun_if_env_changed("FLATC_RS_E2E_OUT_DIR");
    if let Some(path) = std::env::var_os("FLATC_RS_E2E_OUT_DIR") {
        builder = builder.out_dir(path);
    }
    builder.compile()?;
    Ok(())
}
