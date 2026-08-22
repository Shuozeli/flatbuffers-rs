//! Cargo build-script integration for generating Rust from FlatBuffers schemas.
//!
//! The builder compiles each direct schema into one Rust source file under
//! `OUT_DIR`, emits Cargo change-tracking directives for every transitive
//! include, and preserves generated-file mtimes when their contents are
//! unchanged.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use flatc_rs_compiler::{
    check_private_leak,
    codegen::{generate_rust, CodeGenOptions},
    compile_inputs, CompilerError, CompilerOptions,
};

/// Errors produced while resolving schemas or writing generated Rust sources.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("at least one FlatBuffers schema is required")]
    NoSchemas,

    #[error("OUT_DIR is not set; call Builder::out_dir outside a Cargo build script")]
    MissingOutDir,

    #[error("input path has no file stem: {0}")]
    MissingFileStem(PathBuf),

    #[error("schemas '{first}' and '{second}' both generate the output '{output}'")]
    OutputCollision {
        output: PathBuf,
        first: PathBuf,
        second: PathBuf,
    },

    #[error(transparent)]
    Compiler(#[from] CompilerError),

    #[error(transparent)]
    Codegen(#[from] flatc_rs_compiler::CodeGenError),

    #[error("failed to {operation} {path}: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        source: io::Error,
    },
}

/// Information about a completed schema generation run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOutput {
    /// All generated Rust source paths, one per direct schema.
    pub generated_files: Vec<PathBuf>,
    /// Generated files whose contents changed during this run.
    pub updated_files: Vec<PathBuf>,
    /// Canonical direct and transitive schema paths watched by Cargo.
    pub source_files: Vec<PathBuf>,
}

/// Configures Rust generation for use from a Cargo `build.rs` script.
#[derive(Debug, Clone)]
pub struct Builder {
    schemas: Vec<PathBuf>,
    include_dirs: Vec<PathBuf>,
    out_dir: Option<PathBuf>,
    filename_suffix: String,
    rerun_if_env_changed: BTreeSet<String>,
    gen_all: bool,
    gen_name_constants: bool,
    gen_object_api: bool,
    rust_serialize: bool,
    no_includes: bool,
    no_leak_private: bool,
    rust_pluggable_buffer: bool,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            schemas: Vec::new(),
            include_dirs: Vec::new(),
            out_dir: None,
            filename_suffix: "_generated".to_string(),
            rerun_if_env_changed: BTreeSet::new(),
            gen_all: false,
            gen_name_constants: false,
            gen_object_api: false,
            rust_serialize: false,
            no_includes: false,
            no_leak_private: false,
            rust_pluggable_buffer: false,
        }
    }
}

impl Builder {
    /// Creates an empty build configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one direct `.fbs` input.
    pub fn schema(mut self, path: impl Into<PathBuf>) -> Self {
        self.schemas.push(path.into());
        self
    }

    /// Adds multiple direct `.fbs` inputs.
    pub fn schemas<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.schemas.extend(paths.into_iter().map(Into::into));
        self
    }

    /// Adds a search path for `include` directives.
    pub fn include_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.include_dirs.push(path.into());
        self
    }

    /// Overrides Cargo's `OUT_DIR`, primarily for tests and standalone tools.
    pub fn out_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.out_dir = Some(path.into());
        self
    }

    /// Overrides the default `_generated` output filename suffix.
    pub fn filename_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.filename_suffix = suffix.into();
        self
    }

    /// Asks Cargo to rerun the build script when an environment variable changes.
    pub fn rerun_if_env_changed(mut self, variable: impl Into<String>) -> Self {
        self.rerun_if_env_changed.insert(variable.into());
        self
    }

    /// Generates declarations from included schemas as well as direct inputs.
    pub fn gen_all(mut self) -> Self {
        self.gen_all = true;
        self
    }

    /// Generates fully-qualified type name constants.
    pub fn gen_name_strings(mut self) -> Self {
        self.gen_name_constants = true;
        self
    }

    /// Generates owned Object API types and pack/unpack methods.
    pub fn gen_object_api(mut self) -> Self {
        self.gen_object_api = true;
        self
    }

    /// Generates serde serialization implementations.
    pub fn rust_serialize(mut self) -> Self {
        self.rust_serialize = true;
        self
    }

    /// Omits generated imports for dependent schemas.
    pub fn no_includes(mut self) -> Self {
        self.no_includes = true;
        self
    }

    /// Enforces and generates crate-private declarations for `(private)` types.
    pub fn no_leak_private(mut self) -> Self {
        self.no_leak_private = true;
        self
    }

    /// Generates readers over `flatc-rs-runtime::FlatBufferRead`.
    pub fn rust_pluggable_buffer(mut self) -> Self {
        self.rust_pluggable_buffer = true;
        self
    }

    /// Generates all configured schemas and emits Cargo directives to stdout.
    pub fn compile(self) -> Result<BuildOutput, Error> {
        let stdout = io::stdout();
        self.compile_with_writer(&mut stdout.lock())
    }

    /// Generates all configured schemas and writes Cargo directives to `writer`.
    ///
    /// This is useful for tests and build-system adapters that need to capture
    /// the directives rather than sending them directly to Cargo.
    pub fn compile_with_writer(self, writer: &mut impl Write) -> Result<BuildOutput, Error> {
        if self.schemas.is_empty() {
            return Err(Error::NoSchemas);
        }

        let out_dir = self.resolve_out_dir()?;
        let results = compile_inputs(
            &self.schemas,
            &CompilerOptions {
                include_paths: self.include_dirs.clone(),
            },
        )?;

        let output_paths = self.output_paths(&out_dir, &results)?;
        let mut generated_files = Vec::with_capacity(results.len());
        let mut updated_files = Vec::new();
        let mut source_files = BTreeSet::new();

        for (result, output_path) in results.iter().zip(output_paths) {
            source_files.extend(result.source_files.iter().cloned());
            if self.no_leak_private {
                check_private_leak(&result.schema).map_err(CompilerError::from)?;
            }
            let gen_only_files = if self.gen_all {
                None
            } else {
                Some(HashSet::from([result
                    .input_file
                    .to_string_lossy()
                    .to_string()]))
            };
            let code = generate_rust(
                &result.schema,
                &CodeGenOptions {
                    gen_name_constants: self.gen_name_constants,
                    gen_object_api: self.gen_object_api,
                    rust_serialize: self.rust_serialize,
                    gen_only_files,
                    no_includes: self.no_includes,
                    no_leak_private: self.no_leak_private,
                    rust_pluggable_buffer: self.rust_pluggable_buffer,
                },
            )?;

            if write_if_changed(&output_path, code.as_bytes())? {
                updated_files.push(output_path.clone());
            }
            generated_files.push(output_path);
        }

        for path in &source_files {
            writeln!(writer, "cargo::rerun-if-changed={}", path.display()).map_err(|source| {
                Error::Io {
                    operation: "write Cargo directive for",
                    path: path.clone(),
                    source,
                }
            })?;
        }
        for variable in &self.rerun_if_env_changed {
            writeln!(writer, "cargo::rerun-if-env-changed={variable}").map_err(|source| {
                Error::Io {
                    operation: "write Cargo environment directive for",
                    path: PathBuf::from(variable),
                    source,
                }
            })?;
        }

        Ok(BuildOutput {
            generated_files,
            updated_files,
            source_files: source_files.into_iter().collect(),
        })
    }

    fn resolve_out_dir(&self) -> Result<PathBuf, Error> {
        self.out_dir
            .clone()
            .or_else(|| env::var_os("OUT_DIR").map(PathBuf::from))
            .ok_or(Error::MissingOutDir)
    }

    fn output_paths(
        &self,
        out_dir: &Path,
        results: &[flatc_rs_compiler::InputCompilationResult],
    ) -> Result<Vec<PathBuf>, Error> {
        let mut owners = BTreeMap::<PathBuf, PathBuf>::new();
        let mut paths = Vec::with_capacity(results.len());

        for result in results {
            let stem = result
                .input_file
                .file_stem()
                .ok_or_else(|| Error::MissingFileStem(result.input_file.clone()))?
                .to_string_lossy();
            let path = out_dir.join(format!("{stem}{}.rs", self.filename_suffix));
            if let Some(first) = owners.insert(path.clone(), result.input_file.clone()) {
                return Err(Error::OutputCollision {
                    output: path,
                    first,
                    second: result.input_file.clone(),
                });
            }
            paths.push(path);
        }

        Ok(paths)
    }
}

fn write_if_changed(path: &Path, content: &[u8]) -> Result<bool, Error> {
    match fs::read(path) {
        Ok(existing) if existing == content => return Ok(false),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(Error::Io {
                operation: "read",
                path: path.to_path_buf(),
                source,
            })
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::Io {
            operation: "create directory",
            path: parent.to_path_buf(),
            source,
        })?;
    }

    fs::write(path, content).map_err(|source| Error::Io {
        operation: "write",
        path: path.to_path_buf(),
        source,
    })?;

    Ok(true)
}
