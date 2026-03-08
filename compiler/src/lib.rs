pub mod analyzer;
pub mod bfbs;
pub mod compiler;
pub mod conform;
pub mod error;
pub mod json;
#[allow(
    unused_imports,
    dead_code,
    clippy::all,
    non_camel_case_types,
    non_snake_case,
    unused_variables,
    unused_mut,
    deprecated
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
