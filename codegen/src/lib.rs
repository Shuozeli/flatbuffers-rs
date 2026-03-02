mod code_writer;
mod enum_gen;
mod rust_gen;
mod struct_gen;
mod table_gen;
mod ts_enum_gen;
mod ts_gen;
mod ts_struct_gen;
mod ts_table_gen;
mod ts_type_map;
pub mod type_map;

use std::collections::HashSet;
use std::panic::{self, AssertUnwindSafe};

use flatc_rs_schema as schema;
use rust_gen::RustGenerator;
use ts_gen::TsGenerator;

/// Errors that can occur during code generation.
#[derive(Debug, thiserror::Error)]
pub enum CodeGenError {
    #[error("internal codegen error: {0}")]
    Internal(String),
}

// -- Schema access helpers --
// These helpers replace bare `.unwrap()` calls with descriptive messages.
// After the analyzer validates the schema, these fields should always be present.
// A failure here indicates a bug in the analyzer, not a user error.

/// Get the type descriptor for a field. Panics with a descriptive message if missing.
fn field_type(field: &schema::Field) -> &schema::Type {
    field.type_.as_ref().unwrap_or_else(|| {
        panic!(
            "BUG: field '{}' has no type descriptor (schema should have been validated by analyzer)",
            field.name.as_deref().unwrap_or("<unknown>")
        )
    })
}

/// Get the enum/object index from a field's type. Panics with a descriptive message if missing.
fn field_type_index(field: &schema::Field) -> usize {
    let ty = field_type(field);
    ty.index.unwrap_or_else(|| {
        panic!(
            "BUG: field '{}' type has no index (schema should have been validated by analyzer)",
            field.name.as_deref().unwrap_or("<unknown>")
        )
    }) as usize
}

/// Options for Rust code generation.
pub struct CodeGenOptions {
    /// Generate fully-qualified name constants on tables (--gen-name-strings).
    pub gen_name_constants: bool,
    /// Generate Object API types (owned `*T` structs with `pack`/`unpack` methods).
    /// Requires `--gen-object-api` to enable (matches C++ flatc behavior).
    pub gen_object_api: bool,
    /// Implement serde::Serialize on generated Rust types (--rust-serialize).
    /// Not yet implemented; accepted for forward compatibility.
    pub rust_serialize: bool,
    /// When set, only generate code for types whose `declaration_file` matches
    /// one of these paths. When `None`, generate for all types (--gen-all).
    pub gen_only_files: Option<HashSet<String>>,
}

impl Default for CodeGenOptions {
    fn default() -> Self {
        Self {
            gen_name_constants: false,
            gen_object_api: false,
            rust_serialize: false,
            gen_only_files: None,
        }
    }
}

/// Options for TypeScript code generation.
pub struct TsCodeGenOptions {
    /// Generate Object API types (`*T` classes with `pack`/`unpack` methods).
    /// Requires `--gen-object-api` to enable (matches C++ flatc behavior).
    pub gen_object_api: bool,
    /// When set, only generate code for types whose `declaration_file` matches
    /// one of these paths. When `None`, generate for all types (--gen-all).
    pub gen_only_files: Option<HashSet<String>>,
}

impl Default for TsCodeGenOptions {
    fn default() -> Self {
        Self {
            gen_object_api: false,
            gen_only_files: None,
        }
    }
}

/// Check if a type should be included based on its declaration file and the filter.
fn should_generate(declaration_file: Option<&str>, filter: &Option<HashSet<String>>) -> bool {
    match filter {
        None => true,
        Some(files) => match declaration_file {
            Some(df) => files.contains(df),
            // Types without a declaration_file (e.g., from compile_single) always pass.
            None => true,
        },
    }
}

/// Generate Rust source code from a fully resolved FlatBuffers schema.
///
/// The generated code is compatible with the `flatbuffers` runtime crate and
/// includes readers, builders, and trait implementations for all types.
pub fn generate_rust(
    schema: &schema::Schema,
    opts: &CodeGenOptions,
) -> Result<String, CodeGenError> {
    let schema = AssertUnwindSafe(schema);
    let opts = AssertUnwindSafe(opts);
    panic::catch_unwind(move || {
        let gen = RustGenerator::new(&schema, &opts);
        gen.generate()
    })
    .map_err(panic_to_codegen_error)
}

/// Generate TypeScript source code from a fully resolved FlatBuffers schema.
///
/// The generated code is compatible with the `flatbuffers` npm package and
/// includes reader classes, builder static methods, and Object API classes.
pub fn generate_typescript(
    schema: &schema::Schema,
    opts: &TsCodeGenOptions,
) -> Result<String, CodeGenError> {
    let schema = AssertUnwindSafe(schema);
    let opts = AssertUnwindSafe(opts);
    panic::catch_unwind(move || {
        let gen = TsGenerator::new(&schema, &opts);
        gen.generate()
    })
    .map_err(panic_to_codegen_error)
}

fn panic_to_codegen_error(payload: Box<dyn std::any::Any + Send>) -> CodeGenError {
    let msg = if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown codegen error".to_string()
    };
    CodeGenError::Internal(msg)
}
