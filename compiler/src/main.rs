use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use clap::Parser;
use flatc_rs_compiler::{
    bfbs::serialize_schema,
    check_private_leak,
    codegen::{
        generate_dart, generate_rust, generate_typescript, CodeGenOptions, DartCodeGenOptions,
        TsCodeGenOptions,
    },
    compile,
    conform::check_conform,
    json::{binary_to_json, json_to_binary_with_opts, EncoderOptions, JsonOptions},
    CompilerOptions,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "flatc")]
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

    /// Generate Dart code for tables/structs.
    #[arg(short = 'D', long = "dart")]
    dart: bool,

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

    // -- JSON conversion (matching C++ flatc -t) --
    /// Convert FlatBuffer binary to JSON (provide data files after --).
    #[arg(short = 't', long = "json")]
    to_json: bool,

    /// Use strict JSON format.
    #[arg(long)]
    strict_json: bool,

    /// Output fields with default values in JSON.
    #[arg(long)]
    defaults_json: bool,

    /// When encoding JSON -> binary, emit fields even when they equal the default.
    #[arg(long)]
    force_defaults: bool,

    /// Allow unknown fields in JSON input instead of erroring.
    #[arg(long)]
    unknown_json: bool,

    /// Treat input binary as size-prefixed (4-byte length header).
    #[arg(long)]
    size_prefixed: bool,

    // -- Codegen control --
    /// Don't generate include statements for dependent schemas.
    #[arg(long)]
    no_includes: bool,

    /// Generate pub(crate) for types with (private) attribute and validate
    /// that public types don't expose private types.
    #[arg(long)]
    no_leak_private_annotation: bool,

    /// Generate mutate methods for scalar fields (TypeScript).
    #[arg(long)]
    gen_mutable: bool,

    // -- BFBS options --
    /// Include doc comments in binary schema output.
    #[arg(long)]
    bfbs_comments: bool,

    /// Set root path for relative filenames in BFBS output.
    #[arg(long)]
    bfbs_filenames: Option<PathBuf>,

    /// Use absolute paths in BFBS output (modifier for --bfbs-filenames).
    #[arg(long)]
    bfbs_absolute_paths: bool,

    // -- Schema evolution --
    /// Check backwards compatibility against a base schema file.
    #[arg(long)]
    conform: Option<PathBuf>,

    /// Search path for includes when compiling the --conform schema.
    #[arg(long)]
    conform_includes: Vec<PathBuf>,

    /// Data files (binary or JSON) for conversion. Specified after -- separator.
    #[arg(last = true)]
    data_files: Vec<PathBuf>,

    /// Annotate a FlatBuffers binary with schema information.
    /// Outputs an .afb file with hex dumps and field annotations.
    #[arg(long)]
    annotate: bool,

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

    let has_action = cli.rust
        || cli.ts
        || cli.dart
        || cli.binary_schema
        || cli.dump_schema
        || cli.to_json
        || cli.annotate
        || cli.conform.is_some();
    if !has_action {
        eprintln!(
            "error: no action specified, use --rust, --ts, --dart, --json, --schema, --annotate, --conform, or --dump-schema"
        );
        process::exit(1);
    }

    // Validate JSON conversion: -t requires data files
    if cli.to_json && cli.data_files.is_empty() {
        eprintln!("error: --json (-t) requires data files after -- separator");
        eprintln!("usage: flatc -t schema.fbs -- data.bin");
        process::exit(1);
    }

    // Validate --annotate requires data files
    if cli.annotate && cli.data_files.is_empty() {
        eprintln!("error: --annotate requires data files after -- separator");
        eprintln!("usage: flatc --annotate schema.fbs -- data.bin");
        process::exit(1);
    }

    // -b with data files: JSON -> binary
    let json_to_bin = cli.binary_schema
        && !cli.data_files.is_empty()
        && cli.data_files.iter().all(|f| {
            f.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        });

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

    let schema = &result.schema;

    // -- Conform check --
    if let Some(ref conform_file) = cli.conform {
        let conform_opts = CompilerOptions {
            include_paths: if cli.conform_includes.is_empty() {
                cli.include.clone()
            } else {
                cli.conform_includes.clone()
            },
        };
        let base_result = match compile(std::slice::from_ref(conform_file), &conform_opts) {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "error: failed to compile conform schema {}: {e}",
                    conform_file.display()
                );
                process::exit(1);
            }
        };
        if let Err(errors) = check_conform(schema, &base_result.schema) {
            for err in &errors {
                eprintln!("error: {err}");
            }
            eprintln!("{} conformance error(s) found", errors.len());
            process::exit(1);
        }
    }

    // -- Private leak check --
    if cli.no_leak_private_annotation {
        if let Err(e) = check_private_leak(schema) {
            eprintln!("error: {e}");
            process::exit(1);
        }
    }

    // -- Output --
    let output_dir = cli.output_path.as_deref().unwrap_or(Path::new("."));

    if cli.dump_schema {
        // For dump_schema, serialize the legacy schema for backward compat
        let legacy_schema = match schema.as_legacy() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: failed to convert schema: {e}");
                process::exit(1);
            }
        };
        match serde_json::to_string_pretty(&legacy_schema) {
            Ok(json) => println!("{json}"),
            Err(e) => {
                eprintln!("error: failed to serialize schema: {e}");
                process::exit(1);
            }
        }
    }

    if cli.binary_schema {
        // Apply --bfbs-filenames path rewriting if set
        let schema_for_bfbs = if let Some(ref bfbs_root) = cli.bfbs_filenames {
            let mut schema = schema.clone();
            let bfbs_root = fs::canonicalize(bfbs_root).unwrap_or_else(|_| bfbs_root.clone());
            let root_str = bfbs_root.to_string_lossy();
            for obj in &mut schema.objects {
                if let Some(ref mut df) = obj.declaration_file {
                    if !cli.bfbs_absolute_paths {
                        if let Some(rel) = df.strip_prefix(&*root_str) {
                            *df = rel.trim_start_matches('/').to_string();
                        }
                    }
                }
            }
            for enum_def in &mut schema.enums {
                if let Some(ref mut df) = enum_def.declaration_file {
                    if !cli.bfbs_absolute_paths {
                        if let Some(rel) = df.strip_prefix(&*root_str) {
                            *df = rel.trim_start_matches('/').to_string();
                        }
                    }
                }
            }
            schema
        } else {
            schema.clone()
        };
        let bfbs = serialize_schema(&schema_for_bfbs);
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

    // -- JSON conversion --
    if cli.to_json || json_to_bin {
        // Determine root type from schema or --root-type flag
        let root_type_name = cli
            .root_type
            .as_deref()
            .or(schema
                .root_table_index
                .map(|idx| schema.objects[idx].name.as_str()))
            .unwrap_or_else(|| {
                eprintln!("error: no root_type in schema and --root-type not specified");
                process::exit(1);
            })
            .to_string();

        let json_opts = JsonOptions {
            strict_json: cli.strict_json,
            output_defaults: cli.defaults_json,
            output_enum_identifiers: true,
            size_prefixed: cli.size_prefixed,
        };

        let enc_opts = EncoderOptions {
            unknown_json: cli.unknown_json,
            force_defaults: cli.force_defaults,
        };

        for data_file in &cli.data_files {
            if cli.to_json {
                // Binary -> JSON
                let buf = match fs::read(data_file) {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("error: failed to read {}: {e}", data_file.display());
                        process::exit(1);
                    }
                };
                let json_val = match binary_to_json(&buf, schema, &root_type_name, &json_opts) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("error: failed to decode {}: {e}", data_file.display());
                        process::exit(1);
                    }
                };

                let json_str = if cli.strict_json {
                    serde_json::to_string(&json_val)
                } else {
                    serde_json::to_string_pretty(&json_val)
                }
                .unwrap_or_else(|e| {
                    eprintln!("error: failed to serialize JSON: {e}");
                    process::exit(1);
                });

                // Write to output file or stdout
                let stem = data_file
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "output".to_string());
                let out_path = output_dir.join(format!("{stem}.json"));

                if cli.file_names_only {
                    println!("{}", out_path.display());
                } else if let Err(e) = write_output(&out_path, &json_str) {
                    eprintln!("error: {e}");
                    process::exit(1);
                }
            } else if json_to_bin {
                // JSON -> Binary
                let json_str = match fs::read_to_string(data_file) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("error: failed to read {}: {e}", data_file.display());
                        process::exit(1);
                    }
                };
                let json_val: serde_json::Value = match serde_json::from_str(&json_str) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("error: failed to parse JSON {}: {e}", data_file.display());
                        process::exit(1);
                    }
                };
                let bin =
                    match json_to_binary_with_opts(&json_val, schema, &root_type_name, &enc_opts) {
                        Ok(b) => b,
                        Err(e) => {
                            eprintln!("error: failed to encode {}: {e}", data_file.display());
                            process::exit(1);
                        }
                    };

                let stem = data_file
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "output".to_string());
                let out_path = output_dir.join(format!("{stem}.bin"));

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
                    if let Err(e) = fs::write(&out_path, &bin) {
                        eprintln!("error: failed to write {}: {e}", out_path.display());
                        process::exit(1);
                    }
                }
            }
        }
    }

    // -- Annotate --
    if cli.annotate {
        let root_type_name = cli
            .root_type
            .as_deref()
            .or(schema
                .root_table_index
                .map(|idx| schema.objects[idx].name.as_str()))
            .unwrap_or_else(|| {
                eprintln!("error: no root_type in schema and --root-type not specified");
                process::exit(1);
            })
            .to_string();

        let schema_name = cli
            .files
            .first()
            .map(|p| {
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_else(|| "schema.fbs".to_string());

        for data_file in &cli.data_files {
            let buf = match fs::read(data_file) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("error: failed to read {}: {e}", data_file.display());
                    process::exit(1);
                }
            };

            let binary_name = data_file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let afb = match flatc_rs_annotator::annotate_binary(
                &buf,
                schema,
                &root_type_name,
                &schema_name,
                &binary_name,
            ) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: failed to annotate {}: {e}", data_file.display());
                    process::exit(1);
                }
            };

            let stem = data_file
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "output".to_string());
            let out_path = output_dir.join(format!("{stem}.afb"));

            if cli.file_names_only {
                println!("{}", out_path.display());
            } else if let Err(e) = write_output(&out_path, &afb) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }

    // Build the declaration_file filter for --gen-all behavior.
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

    for input_file in &cli.files {
        if cli.rust {
            let rust_opts = CodeGenOptions {
                gen_name_constants: cli.gen_name_strings,
                gen_object_api: cli.gen_object_api,
                rust_serialize: cli.rust_serialize,
                gen_only_files: gen_only_files.clone(),
                no_includes: cli.no_includes,
                no_leak_private: cli.no_leak_private_annotation,
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
                gen_mutable: cli.gen_mutable,
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

        if cli.dart {
            let dart_opts = DartCodeGenOptions {
                gen_object_api: cli.gen_object_api,
                gen_only_files: gen_only_files.clone(),
            };
            let ext = cli.filename_ext.as_deref().unwrap_or("dart");
            let code = match generate_dart(&result.schema, &dart_opts) {
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
