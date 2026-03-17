pub mod analyzer;
pub mod bfbs;
pub mod compiler;
pub mod conform;
pub mod error;
pub mod json;
// Generated from reflection.fbs by the FlatBuffers compiler (C++ flatc).
// Lint suppressions are needed because the generated code doesn't follow Rust conventions.
#[allow(
    unused_imports,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_variables,
    unused_mut,
    deprecated,
    elided_lifetimes_in_paths,
    mismatched_lifetime_syntaxes,
    clippy::duplicated_attributes,
    clippy::extra_unused_lifetimes,
    clippy::missing_safety_doc,
    clippy::module_inception,
    clippy::wrong_self_convention,
)]
pub mod reflection;
pub mod struct_layout;
pub mod type_index;

/// Re-export the codegen crate as a module for backward compatibility.
pub use flatc_rs_codegen as codegen;

pub use analyzer::{analyze, check_private_leak};
pub use codegen::generate_rust;
pub use codegen::generate_typescript;
pub use codegen::CodeGenError;
pub use compiler::{compile, compile_single, CompilationResult, CompilerError, CompilerOptions};
pub use error::AnalyzeError;
pub use flatc_rs_parser as parser;
pub use flatc_rs_schema as schema;
pub use flatc_rs_schema::resolved::ResolvedSchema;
