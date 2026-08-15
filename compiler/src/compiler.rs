use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};

use crate::analyzer;
use crate::error::AnalyzeError;
use flatc_rs_parser::{FbsParser, ParseOutput, ParserState};
use flatc_rs_schema as schema;
use flatc_rs_schema::resolved::ResolvedSchema;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum CompilerError {
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("I/O error reading {path}: {source}")]
    IoError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("parse error in {file}: {message}")]
    ParseError { file: PathBuf, message: String },

    #[error("include not found: '{include}' (referenced from {from})")]
    IncludeNotFound { include: String, from: PathBuf },

    #[error("include path traversal: '{include}' resolves to {resolved} which is outside all allowed search roots (referenced from {from})")]
    PathTraversal {
        include: String,
        resolved: PathBuf,
        from: PathBuf,
    },

    #[error("absolute include path not allowed: '{include}' (referenced from {from})")]
    AbsoluteIncludePath { include: String, from: PathBuf },

    #[error("include depth limit exceeded ({depth} levels) while processing {file}")]
    IncludeDepthLimit { depth: usize, file: PathBuf },

    #[error("included file limit exceeded: {count} files exceeds limit of {limit}")]
    IncludedFileLimit { count: usize, limit: usize },

    #[error("include cycle detected while processing {file}")]
    IncludeCycle { file: PathBuf },

    #[error("invalid virtual file path '{path}': {reason}")]
    InvalidVirtualPath { path: PathBuf, reason: String },

    #[error("duplicate virtual file path after normalization: {path}")]
    DuplicateVirtualPath { path: PathBuf },

    #[error("semantic error: {0}")]
    AnalyzeError(#[from] AnalyzeError),
}

#[derive(Debug, Clone, Default)]
pub struct CompilerOptions {
    /// Search paths for include directives (like flatc -I).
    pub include_paths: Vec<PathBuf>,
}

/// One source file in an in-memory schema filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualFile {
    pub path: PathBuf,
    pub source: String,
}

impl VirtualFile {
    pub fn new(path: impl Into<PathBuf>, source: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            source: source.into(),
        }
    }
}

/// Result of compiling one or more .fbs files.
#[derive(Debug)]
pub struct CompilationResult {
    /// The fully resolved schema.
    pub schema: ResolvedSchema,
}

/// Result of compiling one direct input and its transitive include closure.
#[derive(Debug)]
pub struct InputCompilationResult {
    /// Canonical path of the direct input file.
    pub input_file: PathBuf,
    /// The fully resolved schema for this input and its includes.
    pub schema: ResolvedSchema,
}

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

struct ParsedFile {
    path: PathBuf,
    schema: schema::Schema,
    state: ParserState,
}

struct ResolvedFiles {
    parsed_files: Vec<ParsedFile>,
    dependencies: HashMap<PathBuf, Vec<PathBuf>>,
    input_files: Vec<PathBuf>,
}

// ---------------------------------------------------------------------------
// Compile
// ---------------------------------------------------------------------------

/// Compile one or more .fbs files into a resolved schema.
///
/// Resolves include directives, merges all schemas, and runs semantic analysis.
pub fn compile(
    input_files: &[PathBuf],
    options: &CompilerOptions,
) -> Result<CompilationResult, CompilerError> {
    let resolved = resolve_files(input_files, options)?;
    let merged = merge_schemas(&resolved.parsed_files);
    let schema = analyzer::analyze(merged)?;

    Ok(CompilationResult { schema })
}

/// Compile each direct input against only its transitive include closure.
///
/// All files are parsed once, even when direct inputs share includes. Unlike
/// [`compile`], independent direct inputs are not merged before analysis, so
/// their root metadata and declarations cannot affect each other.
pub fn compile_inputs(
    input_files: &[PathBuf],
    options: &CompilerOptions,
) -> Result<Vec<InputCompilationResult>, CompilerError> {
    let resolved = resolve_files(input_files, options)?;
    let mut results = Vec::with_capacity(resolved.input_files.len());

    for input_file in &resolved.input_files {
        let closure = dependency_closure(input_file, &resolved.dependencies);
        let merged = merge_schemas(
            resolved
                .parsed_files
                .iter()
                .filter(|file| closure.contains(&file.path) && file.path != *input_file)
                .chain(
                    resolved
                        .parsed_files
                        .iter()
                        .filter(|file| file.path == *input_file),
                ),
        );
        let schema = analyzer::analyze(merged)?;
        results.push(InputCompilationResult {
            input_file: input_file.clone(),
            schema,
        });
    }

    Ok(results)
}

fn resolve_files(
    input_files: &[PathBuf],
    options: &CompilerOptions,
) -> Result<ResolvedFiles, CompilerError> {
    resolve_files_with(input_files, options, &NativeFileSystem)
}

fn resolve_files_with<F: SchemaFileSystem>(
    input_files: &[PathBuf],
    options: &CompilerOptions,
    file_system: &F,
) -> Result<ResolvedFiles, CompilerError> {
    // Build include search paths: user-supplied paths + parent dirs of input files.
    let mut include_paths = options.include_paths.clone();
    for file in input_files {
        if let Some(parent) = file.parent() {
            let dir = if parent.as_os_str().is_empty() {
                PathBuf::from(".")
            } else {
                parent.to_path_buf()
            };
            if !include_paths.contains(&dir) {
                include_paths.push(dir);
            }
        }
    }

    let mut resolver = IncludeResolver {
        file_system,
        include_paths,
        parsed_files: Vec::new(),
        dependencies: HashMap::new(),
        seen: HashSet::new(),
        visiting: HashSet::new(),
    };

    // Parse each input file and its transitive includes.
    let mut canonical_inputs = Vec::with_capacity(input_files.len());
    for file in input_files {
        canonical_inputs.push(resolver.resolve_file(file, 0)?);
    }

    let (parsed_files, dependencies) = resolver.into_parts();
    Ok(ResolvedFiles {
        parsed_files,
        dependencies,
        input_files: canonical_inputs,
    })
}

fn dependency_closure(
    root: &Path,
    dependencies: &HashMap<PathBuf, Vec<PathBuf>>,
) -> HashSet<PathBuf> {
    let mut closure = HashSet::new();
    let mut pending = vec![root.to_path_buf()];

    while let Some(file) = pending.pop() {
        if !closure.insert(file.clone()) {
            continue;
        }
        if let Some(includes) = dependencies.get(&file) {
            pending.extend(includes.iter().cloned());
        }
    }

    closure
}

/// Compile a single source string (no include resolution).
///
/// Useful for testing and programmatic use when includes are not needed.
pub fn compile_single(source: &str) -> Result<CompilationResult, CompilerError> {
    let parser = FbsParser::new(source).with_file_name("<input>".to_string());
    let output = parser.parse().map_err(|e| CompilerError::ParseError {
        file: PathBuf::from("<input>"),
        message: e.to_string(),
    })?;

    let schema = analyzer::analyze(output)?;
    Ok(CompilationResult { schema })
}

/// Compile an entry schema and its transitive includes from an in-memory
/// filesystem.
///
/// Paths are normalized lexically and must be relative to the virtual root.
/// Include resolution uses the same traversal, cycle, depth, and file-count
/// rules as [`compile`].
pub fn compile_virtual(
    entry_file: &Path,
    files: &[VirtualFile],
    options: &CompilerOptions,
) -> Result<CompilationResult, CompilerError> {
    let file_system = VirtualFileSystem::new(files)?;
    let entry_file = normalize_virtual_file_path(entry_file)?;
    let include_paths = options
        .include_paths
        .iter()
        .map(|path| {
            normalize_virtual_path(path).map_err(|reason| invalid_virtual_path(path, reason))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let options = CompilerOptions { include_paths };
    let resolved = resolve_files_with(&[entry_file], &options, &file_system)?;
    let merged = merge_schemas(&resolved.parsed_files);
    let schema = analyzer::analyze(merged)?;

    Ok(CompilationResult { schema })
}

// ---------------------------------------------------------------------------
// Include resolver
// ---------------------------------------------------------------------------

trait SchemaFileSystem {
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;
    fn exists(&self, path: &Path) -> bool;
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
}

struct NativeFileSystem;

impl SchemaFileSystem for NativeFileSystem {
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        std::fs::canonicalize(path)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        std::fs::read_to_string(path)
    }
}

struct VirtualFileSystem {
    files: HashMap<PathBuf, String>,
    directories: HashSet<PathBuf>,
}

impl VirtualFileSystem {
    fn new(files: &[VirtualFile]) -> Result<Self, CompilerError> {
        let mut sources = HashMap::with_capacity(files.len());
        let mut directories = HashSet::new();
        directories.insert(PathBuf::new());

        for file in files {
            let path = normalize_virtual_file_path(&file.path)?;
            if sources.insert(path.clone(), file.source.clone()).is_some() {
                return Err(CompilerError::DuplicateVirtualPath { path });
            }

            let mut parent = path.parent();
            while let Some(directory) = parent {
                directories.insert(directory.to_path_buf());
                if directory.as_os_str().is_empty() {
                    break;
                }
                parent = directory.parent();
            }
        }

        Ok(Self {
            files: sources,
            directories,
        })
    }
}

impl SchemaFileSystem for VirtualFileSystem {
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        let normalized = normalize_virtual_path(path)
            .map_err(|reason| io::Error::new(io::ErrorKind::InvalidInput, reason))?;
        if self.files.contains_key(&normalized) || self.directories.contains(&normalized) {
            Ok(normalized)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "virtual path not found",
            ))
        }
    }

    fn exists(&self, path: &Path) -> bool {
        normalize_virtual_path(path)
            .ok()
            .is_some_and(|normalized| self.files.contains_key(&normalized))
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        let normalized = normalize_virtual_path(path)
            .map_err(|reason| io::Error::new(io::ErrorKind::InvalidInput, reason))?;
        self.files
            .get(&normalized)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "virtual file not found"))
    }
}

fn normalize_virtual_file_path(path: &Path) -> Result<PathBuf, CompilerError> {
    let normalized =
        normalize_virtual_path(path).map_err(|reason| invalid_virtual_path(path, reason))?;
    if normalized.as_os_str().is_empty() {
        return Err(invalid_virtual_path(path, "file path is empty"));
    }
    Ok(normalized)
}

fn normalize_virtual_path(path: &Path) -> Result<PathBuf, &'static str> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(part) => normalized.push(part),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    return Err("path escapes the virtual root");
                }
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err("absolute paths are not allowed");
            }
        }
    }
    Ok(normalized)
}

fn invalid_virtual_path(path: &Path, reason: impl Into<String>) -> CompilerError {
    CompilerError::InvalidVirtualPath {
        path: path.to_path_buf(),
        reason: reason.into(),
    }
}

struct IncludeResolver<'a, F> {
    file_system: &'a F,
    include_paths: Vec<PathBuf>,
    /// Parsed files in dependency order (includes before includers).
    parsed_files: Vec<ParsedFile>,
    /// Canonical include targets keyed by canonical including file.
    dependencies: HashMap<PathBuf, Vec<PathBuf>>,
    /// Canonical paths of files already parsed (deduplication).
    seen: HashSet<PathBuf>,
    /// Files currently being processed (for circular include detection).
    visiting: HashSet<PathBuf>,
}

/// Maximum include depth to prevent stack overflow from deep include chains.
const MAX_INCLUDE_DEPTH: usize = 64;

/// Maximum number of included files to prevent OOM on malicious schemas (G3.20).
const MAX_INCLUDED_FILES: usize = 1000;

impl<F: SchemaFileSystem> IncludeResolver<'_, F> {
    fn resolve_file(&mut self, file_path: &Path, depth: usize) -> Result<PathBuf, CompilerError> {
        // G3.8: Prevent stack overflow from deep (non-circular) include chains
        if depth > MAX_INCLUDE_DEPTH {
            return Err(CompilerError::IncludeDepthLimit {
                depth,
                file: file_path.to_path_buf(),
            });
        }

        let canonical = self
            .file_system
            .canonicalize(file_path)
            .map_err(|_| CompilerError::FileNotFound(file_path.to_path_buf()))?;

        // Already parsed -- skip.
        if self.seen.contains(&canonical) {
            return Ok(canonical);
        }

        // Reject circular includes explicitly so all frontends receive a
        // deterministic, structured error instead of a partial dependency graph.
        if self.visiting.contains(&canonical) {
            return Err(CompilerError::IncludeCycle { file: canonical });
        }

        self.visiting.insert(canonical.clone());

        // Read and parse.
        let source =
            self.file_system
                .read_to_string(&canonical)
                .map_err(|e| CompilerError::IoError {
                    path: canonical.clone(),
                    source: e,
                })?;

        let parser =
            FbsParser::new(&source).with_file_name(canonical.to_string_lossy().to_string());
        let output = parser.parse().map_err(|e| CompilerError::ParseError {
            file: canonical.clone(),
            message: e.to_string(),
        })?;

        // Recursively resolve includes.
        let mut dependencies = Vec::new();
        for fbs_file in &output.schema.fbs_files {
            if let Some(include_name) = &fbs_file.filename {
                let include_path = self.find_include(include_name, &canonical)?;
                dependencies.push(self.resolve_file(&include_path, depth + 1)?);
            }
        }

        self.visiting.remove(&canonical);
        self.seen.insert(canonical.clone());
        self.dependencies.insert(canonical.clone(), dependencies);
        self.parsed_files.push(ParsedFile {
            path: canonical.clone(),
            schema: output.schema,
            state: output.state,
        });

        // G3.20: Limit total number of included files to prevent OOM
        if self.parsed_files.len() > MAX_INCLUDED_FILES {
            return Err(CompilerError::IncludedFileLimit {
                count: self.parsed_files.len(),
                limit: MAX_INCLUDED_FILES,
            });
        }

        Ok(canonical)
    }

    fn find_include(&self, name: &str, from_file: &Path) -> Result<PathBuf, CompilerError> {
        // Reject absolute include paths -- includes must be relative.
        if Path::new(name).is_absolute() {
            return Err(CompilerError::AbsoluteIncludePath {
                include: name.to_string(),
                from: from_file.to_path_buf(),
            });
        }

        // 1. Try relative to the including file's directory.
        if let Some(parent) = from_file.parent() {
            let relative = parent.join(name);
            if self.file_system.exists(&relative) {
                return self.validate_no_traversal(name, &relative, parent, from_file);
            }
        }

        // 2. Try each include search path.
        for path in &self.include_paths {
            let candidate = path.join(name);
            if self.file_system.exists(&candidate) {
                return self.validate_no_traversal(name, &candidate, path, from_file);
            }
        }

        Err(CompilerError::IncludeNotFound {
            include: name.to_string(),
            from: from_file.to_path_buf(),
        })
    }

    /// Verify that a resolved include path stays within its search root.
    ///
    /// After joining the include name to a search root directory, the canonical
    /// result must be a descendant of the canonical root. This prevents path
    /// traversal attacks like `include "../../etc/passwd"`.
    fn validate_no_traversal(
        &self,
        include_name: &str,
        resolved: &Path,
        search_root: &Path,
        from_file: &Path,
    ) -> Result<PathBuf, CompilerError> {
        let not_found = || CompilerError::IncludeNotFound {
            include: include_name.to_string(),
            from: from_file.to_path_buf(),
        };
        let canonical_resolved = self
            .file_system
            .canonicalize(resolved)
            .map_err(|_| not_found())?;
        let canonical_root = self
            .file_system
            .canonicalize(search_root)
            .map_err(|_| not_found())?;

        let is_within_another_root = from_file
            .parent()
            .into_iter()
            .chain(self.include_paths.iter().map(PathBuf::as_path))
            .filter(|root| *root != search_root)
            .filter_map(|root| self.file_system.canonicalize(root).ok())
            .any(|root| canonical_resolved.starts_with(root));

        if canonical_resolved.starts_with(&canonical_root) || is_within_another_root {
            return Ok(canonical_resolved);
        }

        Err(CompilerError::PathTraversal {
            include: include_name.to_string(),
            resolved: canonical_resolved,
            from: from_file.to_path_buf(),
        })
    }

    fn into_parts(self) -> (Vec<ParsedFile>, HashMap<PathBuf, Vec<PathBuf>>) {
        (self.parsed_files, self.dependencies)
    }
}

// ---------------------------------------------------------------------------
// Schema merging
// ---------------------------------------------------------------------------

/// Merge schemas from multiple parsed files into a single `ParseOutput`.
fn merge_schemas<'a>(files: impl IntoIterator<Item = &'a ParsedFile>) -> ParseOutput {
    let mut merged_schema = schema::Schema::default();
    let mut merged_state = ParserState::default();

    for file in files {
        let decl_file = file.path.to_string_lossy().to_string();

        for obj in &file.schema.objects {
            let mut obj = obj.clone();
            obj.declaration_file = Some(decl_file.clone());
            merged_schema.objects.push(obj);
        }

        for enum_decl in &file.schema.enums {
            let mut enum_decl = enum_decl.clone();
            enum_decl.declaration_file = Some(decl_file.clone());
            merged_schema.enums.push(enum_decl);
        }

        for service in &file.schema.services {
            let mut service = service.clone();
            service.declaration_file = Some(decl_file.clone());
            merged_schema.services.push(service);
        }

        for fbs_file in &file.schema.fbs_files {
            merged_schema.fbs_files.push(fbs_file.clone());
        }

        // Use file-level metadata from the root file (last one wins).
        // G3.11: Warn when conflicting values are detected across includes.
        if file.schema.file_ident.is_some() {
            if let Some(ref existing) = merged_schema.file_ident {
                if file.schema.file_ident.as_ref() != Some(existing) {
                    eprintln!(
                        "warning: conflicting file_identifier in {}: '{}' overrides '{}'",
                        file.path.display(),
                        file.schema.file_ident.as_deref().unwrap_or(""),
                        existing
                    );
                }
            }
            merged_schema.file_ident = file.schema.file_ident.clone();
        }
        if file.schema.file_ext.is_some() {
            if let Some(ref existing) = merged_schema.file_ext {
                if file.schema.file_ext.as_ref() != Some(existing) {
                    eprintln!(
                        "warning: conflicting file_extension in {}: '{}' overrides '{}'",
                        file.path.display(),
                        file.schema.file_ext.as_deref().unwrap_or(""),
                        existing
                    );
                }
            }
            merged_schema.file_ext = file.schema.file_ext.clone();
        }

        // Merge parser state.
        if file.state.root_type_name.is_some() {
            if let Some(ref existing) = merged_state.root_type_name {
                if file.state.root_type_name.as_ref() != Some(existing) {
                    eprintln!(
                        "warning: conflicting root_type in {}: '{}' overrides '{}'",
                        file.path.display(),
                        file.state.root_type_name.as_deref().unwrap_or(""),
                        existing
                    );
                }
            }
            merged_state.root_type_name = file.state.root_type_name.clone();
            merged_state.root_type_namespace = file.state.root_type_namespace.clone();
        }
        merged_state
            .declared_attributes
            .extend(file.state.declared_attributes.iter().cloned());
    }

    ParseOutput {
        schema: merged_schema,
        state: merged_state,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[cfg(unix)]
    fn create_file_symlink(target: &Path, link: &Path) -> bool {
        std::os::unix::fs::symlink(target, link).unwrap();
        true
    }

    #[cfg(windows)]
    fn create_file_symlink(target: &Path, link: &Path) -> bool {
        match std::os::windows::fs::symlink_file(target, link) {
            Ok(()) => true,
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => false,
            Err(error) => panic!("failed to create test symlink: {error}"),
        }
    }

    #[test]
    fn test_absolute_include_path_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let fbs_path = dir.path().join("test.fbs");
        fs::write(&fbs_path, "include \"/etc/passwd\";\ntable T { x:int; }").unwrap();

        let options = CompilerOptions::default();
        let err = compile(&[fbs_path], &options).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("absolute include path not allowed"),
            "expected absolute path error, got: {msg}"
        );
    }

    #[test]
    fn test_path_traversal_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let sub = root.join("sub");
        fs::create_dir_all(&sub).unwrap();
        let escape_path = dir.path().join("escape.fbs");
        fs::write(&escape_path, "table Escaped { x:int; }").unwrap();

        let fbs_path = sub.join("main.fbs");
        fs::write(
            &fbs_path,
            "include \"../../escape.fbs\";\ntable T { x:int; }",
        )
        .unwrap();

        let options = CompilerOptions::default();
        let error = compile(&[fbs_path], &options).unwrap_err();

        assert!(matches!(error, CompilerError::PathTraversal { .. }));
    }

    #[test]
    fn test_parent_include_within_explicit_search_root_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let schemas = root.join("schemas");
        fs::create_dir_all(&schemas).unwrap();
        fs::write(root.join("common.fbs"), "table Common { value:int; }").unwrap();

        let main = schemas.join("main.fbs");
        fs::write(
            &main,
            "include \"../common.fbs\";\ntable Main { common:Common; }\nroot_type Main;",
        )
        .unwrap();

        let options = CompilerOptions {
            include_paths: vec![root],
        };
        let result = compile(&[main], &options).unwrap();

        assert!(result
            .schema
            .objects
            .iter()
            .any(|object| object.name == "Common"));
    }

    #[test]
    fn test_relative_include_within_root_allowed() {
        // Create dir structure: root/sub/inner.fbs and root/main.fbs includes "sub/inner.fbs"
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();

        fs::write(sub.join("inner.fbs"), "table Inner { x:int; }").unwrap();
        let fbs_path = dir.path().join("main.fbs");
        fs::write(&fbs_path, "include \"sub/inner.fbs\";\ntable T { y:int; }").unwrap();

        let options = CompilerOptions::default();
        let result = compile(&[fbs_path], &options);
        assert!(
            result.is_ok(),
            "relative include should work: {:?}",
            result.err()
        );
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn test_symlink_include_outside_root_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let outside = dir.path().join("outside");
        fs::create_dir(&root).unwrap();
        fs::create_dir(&outside).unwrap();
        let outside_schema = outside.join("outside.fbs");
        fs::write(&outside_schema, "table Outside { secret:string; }").unwrap();
        if !create_file_symlink(Path::new("../outside/outside.fbs"), &root.join("link.fbs")) {
            return;
        }

        let main = root.join("main.fbs");
        fs::write(
            &main,
            "include \"link.fbs\";\ntable Main { value:int; }\nroot_type Main;",
        )
        .unwrap();

        let error = compile(&[main.clone()], &CompilerOptions::default()).unwrap_err();

        match error {
            CompilerError::PathTraversal {
                include,
                resolved,
                from,
            } => {
                assert_eq!(include, "link.fbs");
                assert_eq!(resolved, fs::canonicalize(outside_schema).unwrap());
                assert_eq!(from, fs::canonicalize(main).unwrap());
            }
            other => panic!("expected path traversal error, got: {other}"),
        }
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn test_symlink_include_inside_root_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let types = dir.path().join("types");
        fs::create_dir(&types).unwrap();
        fs::write(types.join("inside.fbs"), "table Inside { value:int; }").unwrap();
        if !create_file_symlink(Path::new("types/inside.fbs"), &dir.path().join("link.fbs")) {
            return;
        }

        let main = dir.path().join("main.fbs");
        fs::write(
            &main,
            "include \"link.fbs\";\ntable Main { inside:Inside; }\nroot_type Main;",
        )
        .unwrap();

        let result = compile(&[main], &CompilerOptions::default()).unwrap();

        assert!(result
            .schema
            .objects
            .iter()
            .any(|object| object.name == "Inside"));
        assert!(result
            .schema
            .objects
            .iter()
            .any(|object| object.name == "Main"));
    }
}
