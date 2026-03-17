mod code_writer;
mod enum_gen;
mod namespace_tree;
mod rust_gen;
mod rust_table_gen;
mod struct_gen;
mod ts_enum_gen;
mod ts_gen;
mod ts_struct_gen;
mod ts_table_gen;
mod ts_type_map;
pub mod type_map;

use std::collections::HashSet;
use std::panic;

use flatc_rs_schema::resolved::{
    ResolvedEnumVal, ResolvedField, ResolvedObject, ResolvedSchema, ResolvedType,
};
use flatc_rs_schema::Attributes;
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

/// Get the enum/object index from a field's type.
fn field_type_index(field: &ResolvedField) -> Result<usize, CodeGenError> {
    field.type_.index.map(|i| i as usize).ok_or_else(|| {
        CodeGenError::Internal(format!(
            "field '{}' type has no index",
            &field.name
        ))
    })
}

/// Get the enum/object index from a Type descriptor.
fn type_index(ty: &ResolvedType, context: &str) -> Result<usize, CodeGenError> {
    ty.index.map(|i| i as usize).ok_or_else(|| {
        CodeGenError::Internal(format!("type has no index in {context}"))
    })
}

/// Get the type index for a union variant's type.
fn union_variant_type_index(val: &ResolvedEnumVal) -> Result<usize, CodeGenError> {
    val.union_type
        .as_ref()
        .and_then(|t| t.index)
        .map(|i| i as usize)
        .ok_or_else(|| {
            CodeGenError::Internal(format!(
                "union variant '{}' has no type index",
                &val.name
            ))
        })
}

/// Get the byte_size of an object (struct).
fn obj_byte_size(obj: &ResolvedObject) -> Result<usize, CodeGenError> {
    obj.byte_size.map(|s| s as usize).ok_or_else(|| {
        CodeGenError::Internal(format!(
            "object '{}' has no byte_size",
            &obj.name
        ))
    })
}

/// Get the min_align of an object (struct).
fn obj_min_align(obj: &ResolvedObject) -> Result<usize, CodeGenError> {
    obj.min_align.map(|a| a as usize).ok_or_else(|| {
        CodeGenError::Internal(format!(
            "object '{}' has no min_align",
            &obj.name
        ))
    })
}

/// Get a struct field's byte offset.
fn field_offset(field: &ResolvedField) -> Result<usize, CodeGenError> {
    field.offset.map(|o| o as usize).ok_or_else(|| {
        CodeGenError::Internal(format!(
            "field '{}' has no offset",
            &field.name
        ))
    })
}

/// Get a table field's ID.
fn field_id(field: &ResolvedField) -> Result<u32, CodeGenError> {
    field.id.ok_or_else(|| {
        CodeGenError::Internal(format!(
            "field '{}' has no id",
            &field.name
        ))
    })
}

/// Options for Rust code generation.
#[derive(Default)]
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
    /// Don't generate `use` import statements for dependent schemas (--no-includes).
    pub no_includes: bool,
    /// Generate `pub(crate)` instead of `pub` for types with `(private)` attribute.
    /// Also validates that public types don't expose private types through fields.
    pub no_leak_private: bool,
}

/// Return the Rust visibility keyword for a type based on its attributes and options.
///
/// When `opts.no_leak_private` is true and the type has a `(private)` attribute,
/// returns `"pub(crate)"`. Otherwise returns `"pub"`.
pub fn type_visibility(attrs: Option<&Attributes>, opts: &CodeGenOptions) -> &'static str {
    if opts.no_leak_private {
        if let Some(attrs) = attrs {
            if attrs
                .entries
                .iter()
                .any(|kv| kv.key.as_deref() == Some("private"))
            {
                return "pub(crate)";
            }
        }
    }
    "pub"
}

/// Options for TypeScript code generation.
#[derive(Default)]
pub struct TsCodeGenOptions {
    /// Generate Object API types (`*T` classes with `pack`/`unpack` methods).
    /// Requires `--gen-object-api` to enable (matches C++ flatc behavior).
    pub gen_object_api: bool,
    /// When set, only generate code for types whose `declaration_file` matches
    /// one of these paths. When `None`, generate for all types (--gen-all).
    pub gen_only_files: Option<HashSet<String>>,
    /// Generate `mutate_*` methods for scalar fields in TypeScript (--gen-mutable).
    pub gen_mutable: bool,
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
    schema: &ResolvedSchema,
    opts: &CodeGenOptions,
) -> Result<String, CodeGenError> {
    let gen = RustGenerator::new(schema, opts);
    gen.generate()
}

/// Generate TypeScript source code from a fully resolved FlatBuffers schema.
///
/// The generated code is compatible with the `flatbuffers` npm package and
/// includes reader classes, builder static methods, and Object API classes.
///
/// Panics in the TS codegen are caught and converted to `CodeGenError::Internal`.
pub fn generate_typescript(
    schema: &ResolvedSchema,
    opts: &TsCodeGenOptions,
) -> Result<String, CodeGenError> {
    catch_codegen_panic(|| {
        let gen = TsGenerator::new(schema, opts);
        gen.generate()
    })
}

/// Run a codegen closure, catching any panics and converting them to errors.
/// Used as a safety net for the TypeScript codegen which has not yet been
/// converted to use `Result` returns.
fn catch_codegen_panic<F: FnOnce() -> String + panic::UnwindSafe>(
    f: F,
) -> Result<String, CodeGenError> {
    panic::catch_unwind(f).map_err(|payload| {
        let msg = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown codegen error".to_string()
        };
        CodeGenError::Internal(msg)
    })
}
