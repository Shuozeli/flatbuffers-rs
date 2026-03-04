use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::analyzer;
use crate::error::AnalyzeError;
use flatc_rs_parser::{FbsParser, ParseOutput, ParserState};
use flatc_rs_schema as schema;

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

    #[error("semantic error: {0}")]
    AnalyzeError(#[from] AnalyzeError),
}

#[derive(Debug, Clone, Default)]
pub struct CompilerOptions {
    /// Search paths for include directives (like flatc -I).
    pub include_paths: Vec<PathBuf>,
}

/// Result of compiling one or more .fbs files.
#[derive(Debug)]
pub struct CompilationResult {
    /// The fully resolved schema.
    pub schema: schema::Schema,
}

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

struct ParsedFile {
    path: PathBuf,
    schema: schema::Schema,
    state: ParserState,
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
        include_paths,
        parsed_files: Vec::new(),
        seen: HashSet::new(),
        visiting: HashSet::new(),
    };

    // Parse each input file and its transitive includes.
    for file in input_files {
        resolver.resolve_file(file, 0)?;
    }

    // Collect parsed files in dependency order (includes first).
    let parsed = resolver.into_parsed_files();

    // Merge schemas from all files.
    let merged = merge_schemas(&parsed);

    // Run semantic analysis on the merged schema.
    let schema = analyzer::analyze(merged)?;

    Ok(CompilationResult { schema })
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

// ---------------------------------------------------------------------------
// Include resolver
// ---------------------------------------------------------------------------

struct IncludeResolver {
    include_paths: Vec<PathBuf>,
    /// Parsed files in dependency order (includes before includers).
    parsed_files: Vec<ParsedFile>,
    /// Canonical paths of files already parsed (deduplication).
    seen: HashSet<PathBuf>,
    /// Files currently being processed (for circular include detection).
    visiting: HashSet<PathBuf>,
}

/// Maximum include depth to prevent stack overflow from deep include chains.
const MAX_INCLUDE_DEPTH: usize = 64;

impl IncludeResolver {
    fn resolve_file(&mut self, file_path: &Path, depth: usize) -> Result<(), CompilerError> {
        // G3.8: Prevent stack overflow from deep (non-circular) include chains
        if depth > MAX_INCLUDE_DEPTH {
            return Err(CompilerError::IncludeDepthLimit {
                depth,
                file: file_path.to_path_buf(),
            });
        }

        let canonical = std::fs::canonicalize(file_path)
            .map_err(|_| CompilerError::FileNotFound(file_path.to_path_buf()))?;

        // Already parsed -- skip.
        if self.seen.contains(&canonical) {
            return Ok(());
        }

        // Circular include -- skip (not an error; flatc handles this gracefully).
        if self.visiting.contains(&canonical) {
            return Ok(());
        }

        self.visiting.insert(canonical.clone());

        // Read and parse.
        let source = std::fs::read_to_string(&canonical).map_err(|e| CompilerError::IoError {
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
        for fbs_file in &output.schema.fbs_files {
            if let Some(include_name) = &fbs_file.filename {
                let include_path = self.find_include(include_name, &canonical)?;
                self.resolve_file(&include_path, depth + 1)?;
            }
        }

        self.visiting.remove(&canonical);
        self.seen.insert(canonical.clone());
        self.parsed_files.push(ParsedFile {
            path: canonical,
            schema: output.schema,
            state: output.state,
        });

        Ok(())
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
            if relative.exists() {
                self.validate_no_traversal(name, &relative, parent, from_file)?;
                return Ok(relative);
            }
        }

        // 2. Try each include search path.
        for path in &self.include_paths {
            let candidate = path.join(name);
            if candidate.exists() {
                self.validate_no_traversal(name, &candidate, path, from_file)?;
                return Ok(candidate);
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
    ) -> Result<(), CompilerError> {
        // Fast path: no ".." means no traversal possible.
        if !include_name.contains("..") {
            return Ok(());
        }

        let canonical_resolved = match std::fs::canonicalize(resolved) {
            Ok(c) => c,
            Err(_) => return Ok(()), // Let resolve_file handle the error downstream.
        };

        let canonical_root = match std::fs::canonicalize(search_root) {
            Ok(c) => c,
            Err(_) => return Ok(()),
        };

        if canonical_resolved.starts_with(&canonical_root) {
            return Ok(());
        }

        Err(CompilerError::PathTraversal {
            include: include_name.to_string(),
            resolved: canonical_resolved,
            from: from_file.to_path_buf(),
        })
    }

    fn into_parsed_files(self) -> Vec<ParsedFile> {
        self.parsed_files
    }
}

// ---------------------------------------------------------------------------
// Schema merging
// ---------------------------------------------------------------------------

/// Merge schemas from multiple parsed files into a single `ParseOutput`.
fn merge_schemas(files: &[ParsedFile]) -> ParseOutput {
    let mut merged_schema = schema::Schema::new();
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
        // Create dir structure: root/sub/main.fbs includes "../../escape.fbs"
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();

        // Create a file that would be outside the search root
        let escape_path = dir.path().parent().unwrap().join("escape.fbs");
        fs::write(&escape_path, "table Escaped { x:int; }").unwrap();

        let fbs_path = sub.join("main.fbs");
        fs::write(
            &fbs_path,
            "include \"../../escape.fbs\";\ntable T { x:int; }",
        )
        .unwrap();

        let options = CompilerOptions::default();
        let result = compile(&[fbs_path], &options);

        // Clean up the escape file before asserting
        let _ = fs::remove_file(&escape_path);

        match result {
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("path traversal") || msg.contains("include not found"),
                    "expected traversal or not-found error, got: {msg}"
                );
            }
            Ok(_) => panic!("expected path traversal error, but compilation succeeded"),
        }
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
}
