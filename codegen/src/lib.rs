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

/// Get the enum/object index from a Type descriptor. Panics with a descriptive message if missing.
fn type_index(ty: &schema::Type, context: &str) -> usize {
    ty.index.unwrap_or_else(|| {
        panic!(
            "BUG: type has no index in {context} (schema should have been validated by analyzer)"
        )
    }) as usize
}

/// Get the type index for a union variant's type. Panics with a descriptive message if missing.
fn union_variant_type_index(val: &schema::EnumVal) -> usize {
    val.union_type
        .as_ref()
        .and_then(|t| t.index)
        .unwrap_or_else(|| {
            panic!(
                "BUG: union variant '{}' has no type index (schema should have been validated by analyzer)",
                val.name.as_deref().unwrap_or("<unknown>")
            )
        }) as usize
}

/// Get the byte_size of an object (struct). Panics with a descriptive message if missing.
fn obj_byte_size(obj: &schema::Object) -> usize {
    obj.byte_size.unwrap_or_else(|| {
        panic!(
            "BUG: object '{}' has no byte_size (layout should have been computed by analyzer)",
            obj.name.as_deref().unwrap_or("<unknown>")
        )
    }) as usize
}

/// Get the min_align of an object (struct). Panics with a descriptive message if missing.
fn obj_min_align(obj: &schema::Object) -> usize {
    obj.min_align.unwrap_or_else(|| {
        panic!(
            "BUG: object '{}' has no min_align (layout should have been computed by analyzer)",
            obj.name.as_deref().unwrap_or("<unknown>")
        )
    }) as usize
}

/// Get a struct field's byte offset. Panics with a descriptive message if missing.
fn field_offset(field: &schema::Field) -> usize {
    field.offset.unwrap_or_else(|| {
        panic!(
            "BUG: field '{}' has no offset (layout should have been computed by analyzer)",
            field.name.as_deref().unwrap_or("<unknown>")
        )
    }) as usize
}

/// Get a table field's ID. Panics with a descriptive message if missing.
fn field_id(field: &schema::Field) -> u32 {
    field.id.unwrap_or_else(|| {
        panic!(
            "BUG: field '{}' has no id (should have been assigned by analyzer)",
            field.name.as_deref().unwrap_or("<unknown>")
        )
    })
}

/// Get an enum value's integer discriminator. Panics with a descriptive message if missing.
fn enum_val_value(val: &schema::EnumVal) -> i64 {
    val.value.unwrap_or_else(|| {
        panic!(
            "BUG: enum value '{}' has no value (should have been assigned by analyzer)",
            val.name.as_deref().unwrap_or("<unknown>")
        )
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
pub fn type_visibility(attrs: Option<&schema::Attributes>, opts: &CodeGenOptions) -> &'static str {
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
///
/// Panics in the codegen (which indicate bugs in the analyzer, not user errors)
/// are caught and converted to `CodeGenError::Internal`.
pub fn generate_rust(
    schema: &schema::Schema,
    opts: &CodeGenOptions,
) -> Result<String, CodeGenError> {
    catch_codegen_panic(|| {
        let gen = RustGenerator::new(schema, opts);
        gen.generate()
    })
}

/// Generate TypeScript source code from a fully resolved FlatBuffers schema.
///
/// The generated code is compatible with the `flatbuffers` npm package and
/// includes reader classes, builder static methods, and Object API classes.
///
/// Panics in the codegen (which indicate bugs in the analyzer, not user errors)
/// are caught and converted to `CodeGenError::Internal`.
pub fn generate_typescript(
    schema: &schema::Schema,
    opts: &TsCodeGenOptions,
) -> Result<String, CodeGenError> {
    catch_codegen_panic(|| {
        let gen = TsGenerator::new(schema, opts);
        gen.generate()
    })
}

/// Run a codegen closure, catching any panics and converting them to errors.
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
