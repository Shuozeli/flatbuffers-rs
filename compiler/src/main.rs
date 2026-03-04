use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use clap::Parser;
use flatc_rs_compiler::{
    bfbs::serialize_schema,
    codegen::{generate_rust, generate_typescript, CodeGenOptions, TsCodeGenOptions},
    compile, CompilerOptions,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "flatc-rs")]
#[command(about = "FlatBuffers schema compiler (Rust implementation)")]
#[command(version = VERSION)]
struct Cli {
    /// Input .fbs files to compile.
    files: Vec<PathBuf>,

    // -- General --
    /// Prefix PATH to all generated files.
    #[arg(short = 'o')]
    output_path: Option<PathBuf>,

    /// Search for includes in the specified path.
    #[arg(short = 'I')]
    include: Vec<PathBuf>,

    // -- Language selection (matching C++ flatc flags) --
    /// Generate Rust files for tables/structs.
    #[arg(short = 'r', long = "rust")]
    rust: bool,

    /// Generate TypeScript code for tables/structs.
    #[arg(short = 'T', long = "ts")]
    ts: bool,

    // -- Codegen options --
    /// Generate an additional object-based API.
    #[arg(long)]
    gen_object_api: bool,

    /// Generate type name functions for C++ and Rust.
    #[arg(long)]
    gen_name_strings: bool,

    /// Generate not just code for the current schema files, but for all
    /// files it includes as well.
    #[arg(long)]
    gen_all: bool,

    // -- Rust-specific --
    /// Implement serde::Serialize on generated Rust types.
    #[arg(long)]
    rust_serialize: bool,

    /// Generate rust code in individual files with a module root file.
    #[arg(long)]
    rust_module_root_file: bool,

    // -- File handling --
    /// The suffix appended to generated file names (default: '_generated').
    #[arg(long, default_value = "_generated")]
    filename_suffix: String,

    /// The extension appended to generated file names (overrides language default).
    #[arg(long)]
    filename_ext: Option<String>,

    /// Select or override the default root_type.
    #[arg(long)]
    root_type: Option<String>,

    /// When parsing schemas, require explicit ids (id: x).
    #[arg(long)]
    require_explicit_ids: bool,

    /// Print generated file names without writing to files.
    #[arg(long)]
    file_names_only: bool,

    /// Inhibit all warning messages.
    #[arg(long)]
    no_warnings: bool,

    /// Treat all warnings as errors.
    #[arg(long)]
    warnings_as_errors: bool,

    /// Serialize schemas to binary (.bfbs file).
    #[arg(short = 'b', long = "schema")]
    binary_schema: bool,

    // -- flatc-rs extensions --
    /// Output the resolved schema as JSON (flatc-rs only).
    #[arg(long)]
    dump_schema: bool,
}

/// Compute the output file path for a given input .fbs file.
///
/// Follows C++ flatc convention: `{output_dir}/{stem}{suffix}.{ext}`
fn output_file_path(
    input: &Path,
    suffix: &str,
    ext: &str,
    output_dir: &Path,
) -> Result<PathBuf, String> {
    let stem = input
        .file_stem()
        .ok_or_else(|| format!("input path has no file name: {}", input.display()))?
        .to_string_lossy();
    let filename = format!("{stem}{suffix}.{ext}");
    Ok(output_dir.join(filename))
}

/// Write generated code to a file, creating parent directories as needed.
/// G3.13: Uses write-to-temp + rename for atomic output (no partial files on interrupt).
fn write_output(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create directory {}: {e}", parent.display()))?;
    }
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, content)
        .map_err(|e| format!("failed to write {}: {e}", tmp_path.display()))?;
    fs::rename(&tmp_path, path).map_err(|e| {
        // Clean up temp file on rename failure
        let _ = fs::remove_file(&tmp_path);
        format!(
            "failed to rename {} -> {}: {e}",
            tmp_path.display(),
            path.display()
        )
    })
}

fn warn(msg: &str, no_warnings: bool) {
    if !no_warnings {
        eprintln!("warning: {msg}");
    }
}

fn main() {
    let cli = Cli::parse();

    // -- Validation --
    if cli.files.is_empty() {
        eprintln!("error: missing input files");
        process::exit(1);
    }

    let has_action = cli.rust || cli.ts || cli.binary_schema || cli.dump_schema;
    if !has_action {
        eprintln!("error: no action specified, use --rust, --ts, --schema, or --dump-schema");
        process::exit(1);
    }

    // Warn about unimplemented features.
    if cli.rust_module_root_file {
        warn(
            "--rust-module-root-file is not yet implemented, using single-file output",
            cli.no_warnings,
        );
    }
    if cli.require_explicit_ids {
        warn(
            "--require-explicit-ids is not yet implemented, ignoring",
            cli.no_warnings,
        );
    }

    // Warn about language-specific flags without the language.
    if cli.rust_serialize && !cli.rust {
        warn(
            "--rust-serialize has no effect without --rust",
            cli.no_warnings,
        );
    }
    if cli.rust_module_root_file && !cli.rust {
        warn(
            "--rust-module-root-file has no effect without --rust",
            cli.no_warnings,
        );
    }

    // -- Compile --
    let options = CompilerOptions {
        include_paths: cli.include.clone(),
    };

    let result = match compile(&cli.files, &options) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    // -- Output --
    let output_dir = cli.output_path.as_deref().unwrap_or(Path::new("."));

    if cli.dump_schema {
        match serde_json::to_string_pretty(&result.schema) {
            Ok(json) => println!("{json}"),
            Err(e) => {
                eprintln!("error: failed to serialize schema: {e}");
                process::exit(1);
            }
        }
    }

    if cli.binary_schema {
        let bfbs = serialize_schema(&result.schema);
        for input_file in &cli.files {
            let stem = input_file
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "schema".to_string());
            let out_path = output_dir.join(format!("{stem}.bfbs"));
            if cli.file_names_only {
                println!("{}", out_path.display());
            } else {
                if let Some(parent) = out_path.parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        eprintln!(
                            "error: failed to create directory {}: {e}",
                            parent.display()
                        );
                        process::exit(1);
                    }
                }
                if let Err(e) = fs::write(&out_path, &bfbs) {
                    eprintln!("error: failed to write {}: {e}", out_path.display());
                    process::exit(1);
                }
            }
        }
    }

    // Build the declaration_file filter for --gen-all behavior.
    // When --gen-all is NOT set, only generate code for types from the
    // direct input files. Canonicalize paths to match how the compiler
    // records declaration_file.
    // G3.12: Propagate canonicalize errors instead of silently dropping files
    let gen_only_files: Option<HashSet<String>> = if cli.gen_all {
        None
    } else {
        let mut files = HashSet::new();
        for f in &cli.files {
            match fs::canonicalize(f) {
                Ok(p) => {
                    files.insert(p.to_string_lossy().to_string());
                }
                Err(e) => {
                    eprintln!("error: failed to resolve input file {}: {e}", f.display());
                    process::exit(1);
                }
            }
        }
        Some(files)
    };

    // C++ flatc generates one output file per input .fbs file when not
    // using --rust-module-root-file. We match that behavior.
    for input_file in &cli.files {
        if cli.rust {
            let rust_opts = CodeGenOptions {
                gen_name_constants: cli.gen_name_strings,
                gen_object_api: cli.gen_object_api,
                rust_serialize: cli.rust_serialize,
                gen_only_files: gen_only_files.clone(),
            };
            let ext = cli.filename_ext.as_deref().unwrap_or("rs");
            let code = match generate_rust(&result.schema, &rust_opts) {
                Ok(code) => code,
                Err(e) => {
                    eprintln!("error: {e}");
                    process::exit(1);
                }
            };
            let out_path = match output_file_path(input_file, &cli.filename_suffix, ext, output_dir)
            {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("error: {e}");
                    process::exit(1);
                }
            };

            if cli.file_names_only {
                println!("{}", out_path.display());
            } else if let Err(e) = write_output(&out_path, &code) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }

        if cli.ts {
            let ts_opts = TsCodeGenOptions {
                gen_object_api: cli.gen_object_api,
                gen_only_files: gen_only_files.clone(),
            };
            let ext = cli.filename_ext.as_deref().unwrap_or("ts");
            let code = match generate_typescript(&result.schema, &ts_opts) {
                Ok(code) => code,
                Err(e) => {
                    eprintln!("error: {e}");
                    process::exit(1);
                }
            };
            let out_path = match output_file_path(input_file, &cli.filename_suffix, ext, output_dir)
            {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("error: {e}");
                    process::exit(1);
                }
            };

            if cli.file_names_only {
                println!("{}", out_path.display());
            } else if let Err(e) = write_output(&out_path, &code) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
}
