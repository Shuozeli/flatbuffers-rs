mod enum_gen;
mod namespace_tree;
mod python_gen;
mod rust_gen;
mod rust_runtime_gen;
mod rust_table_gen;
#[cfg(feature = "grpc")]
mod service_gen;
mod struct_gen;
mod ts_enum_gen;
mod ts_gen;
mod ts_struct_gen;
mod ts_table_gen;
mod ts_type_map;
pub mod type_map;

// Re-export CodeWriter from codegen-core
pub use codegen_core::CodeWriter;

use std::collections::HashSet;

use flatc_rs_schema::resolved::{
    ResolvedEnumVal, ResolvedField, ResolvedObject, ResolvedSchema, ResolvedType,
};
use flatc_rs_schema::{Attributes, BaseType};
use python_gen::PythonGenerator;
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
    let index = field.type_.index.ok_or_else(|| {
        CodeGenError::Internal(format!("field '{}' type has no index", field.name))
    })?;
    usize::try_from(index).map_err(|_| {
        CodeGenError::Internal(format!(
            "field '{}' has negative type index {index}",
            field.name
        ))
    })
}

/// Get the enum/object index from a Type descriptor.
fn type_index(ty: &ResolvedType, context: &str) -> Result<usize, CodeGenError> {
    let index = ty
        .index
        .ok_or_else(|| CodeGenError::Internal(format!("type has no index in {context}")))?;
    usize::try_from(index).map_err(|_| {
        CodeGenError::Internal(format!("type has negative index {index} in {context}"))
    })
}

/// Get the type index for a union variant's type.
fn union_variant_type_index(val: &ResolvedEnumVal) -> Result<usize, CodeGenError> {
    val.union_type
        .as_ref()
        .and_then(|t| t.index)
        .and_then(|i| usize::try_from(i).ok())
        .ok_or_else(|| {
            CodeGenError::Internal(format!("union variant '{}' has no type index", val.name))
        })
}

/// Get the byte_size of an object (struct).
fn obj_byte_size(obj: &ResolvedObject) -> Result<usize, CodeGenError> {
    obj.byte_size
        .map(|s| s as usize)
        .ok_or_else(|| CodeGenError::Internal(format!("object '{}' has no byte_size", obj.name)))
}

/// Get the min_align of an object (struct).
fn obj_min_align(obj: &ResolvedObject) -> Result<usize, CodeGenError> {
    obj.min_align
        .map(|a| a as usize)
        .ok_or_else(|| CodeGenError::Internal(format!("object '{}' has no min_align", obj.name)))
}

/// Get a struct field's byte offset.
fn field_offset(field: &ResolvedField) -> Result<usize, CodeGenError> {
    field
        .offset
        .map(|o| o as usize)
        .ok_or_else(|| CodeGenError::Internal(format!("field '{}' has no offset", field.name)))
}

/// Get a table field's ID.
fn field_id(field: &ResolvedField) -> Result<u32, CodeGenError> {
    field
        .id
        .ok_or_else(|| CodeGenError::Internal(format!("field '{}' has no id", field.name)))
}

fn validate_type_reference(
    schema: &ResolvedSchema,
    ty: &ResolvedType,
    context: &str,
) -> Result<(), CodeGenError> {
    let check_index = |kind: &str, len: usize| -> Result<(), CodeGenError> {
        let index = type_index(ty, context)?;
        if index >= len {
            return Err(CodeGenError::Internal(format!(
                "{context} references {kind} index {index}, but only {len} {kind}s exist"
            )));
        }
        Ok(())
    };

    match ty.base_type {
        BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => {
            check_index("object", schema.objects.len())?;
        }
        BaseType::BASE_TYPE_UNION => check_index("enum", schema.enums.len())?,
        BaseType::BASE_TYPE_ARRAY => {
            match ty.fixed_length {
                Some(length) if length > 0 => {}
                _ => {
                    return Err(CodeGenError::Internal(format!(
                        "{context} has no positive fixed array length"
                    )));
                }
            }
            match ty.element_type_or_none() {
                BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => {
                    check_index("object", schema.objects.len())?;
                }
                BaseType::BASE_TYPE_UNION => check_index("enum", schema.enums.len())?,
                element if element.is_scalar() && ty.index.is_some() => {
                    check_index("enum", schema.enums.len())?;
                }
                _ => {}
            }
        }
        BaseType::BASE_TYPE_VECTOR => match ty.element_type_or_none() {
            BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => {
                check_index("object", schema.objects.len())?;
            }
            BaseType::BASE_TYPE_UNION => check_index("enum", schema.enums.len())?,
            element if element.is_scalar() && ty.index.is_some() => {
                check_index("enum", schema.enums.len())?;
            }
            _ => {}
        },
        scalar if scalar.is_scalar() && ty.index.is_some() => {
            check_index("enum", schema.enums.len())?;
        }
        _ => {}
    }
    Ok(())
}

fn validate_schema_for_codegen(schema: &ResolvedSchema) -> Result<(), CodeGenError> {
    if let Some(index) = schema.root_table_index {
        if index >= schema.objects.len() {
            return Err(CodeGenError::Internal(format!(
                "root table index {index} is out of bounds for {} objects",
                schema.objects.len()
            )));
        }
    }

    for object in &schema.objects {
        if object.is_struct {
            let byte_size = obj_byte_size(object)?;
            let min_align = obj_min_align(object)?;
            if byte_size == 0 || min_align == 0 {
                return Err(CodeGenError::Internal(format!(
                    "struct '{}' has zero byte size or alignment",
                    object.name
                )));
            }
        }
        for field in &object.fields {
            let context = format!("field '{}.{}'", object.name, field.name);
            if object.is_struct {
                field_offset(field)?;
            } else {
                field_id(field)?;
            }
            validate_type_reference(schema, &field.type_, &context)?;
        }
    }

    for enum_def in &schema.enums {
        for variant in &enum_def.values {
            if let Some(union_type) = &variant.union_type {
                validate_type_reference(
                    schema,
                    union_type,
                    &format!("union variant '{}.{}'", enum_def.name, variant.name),
                )?;
            }
        }
    }

    for service in &schema.services {
        for call in &service.calls {
            if call.request_index >= schema.objects.len()
                || call.response_index >= schema.objects.len()
            {
                return Err(CodeGenError::Internal(format!(
                    "RPC method '{}.{}' references an out-of-bounds object",
                    service.name, call.name
                )));
            }
        }
    }
    Ok(())
}

/// Options for Rust code generation.
#[derive(Default)]
pub struct CodeGenOptions {
    /// Generate fully-qualified name constants on tables (--gen-name-strings).
    pub gen_name_constants: bool,
    /// Generate Object API types (owned `*T` structs with `pack`/`unpack` methods).
    /// Requires `--gen-object-api` to enable (matches C++ flatc behavior).
    pub gen_object_api: bool,
    /// Implement serde::Serialize/Deserialize on generated Rust types (--rust-serialize).
    /// Generates manual Serialize/Deserialize for enums and bitflags, derived impls
    /// for Object API types, and manual Serialize for struct readers.
    pub rust_serialize: bool,
    /// When set, only generate code for types whose `declaration_file` matches
    /// one of these paths. When `None`, generate for all types (--gen-all).
    pub gen_only_files: Option<HashSet<String>>,
    /// Don't generate `use` import statements for dependent schemas (--no-includes).
    pub no_includes: bool,
    /// Generate `pub(crate)` instead of `pub` for types with `(private)` attribute.
    /// Also validates that public types don't expose private types through fields.
    pub no_leak_private: bool,
    /// Generate Rust readers over a pluggable byte-buffer abstraction.
    pub rust_pluggable_buffer: bool,
}

/// Return the Rust visibility keyword for a type based on its attributes and options.
///
/// When `opts.no_leak_private` is true and the type has a `(private)` attribute,
/// returns `"pub(crate)"`. Otherwise returns `"pub"`.
pub fn type_visibility(attrs: Option<&Attributes>, opts: &CodeGenOptions) -> &'static str {
    if opts.no_leak_private {
        if let Some(attrs) = attrs {
            if attrs.has("private") {
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

/// Options for Python code generation.
#[derive(Default)]
pub struct PythonCodeGenOptions {
    /// When set, only generate code for types whose `declaration_file` matches
    /// one of these paths. When `None`, generate for all types (--gen-all).
    pub gen_only_files: Option<HashSet<String>>,
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
    validate_schema_for_codegen(schema)?;
    #[cfg(feature = "grpc")]
    if schema
        .services
        .iter()
        .any(|service| should_generate(service.declaration_file.as_deref(), &opts.gen_only_files))
        && !opts.gen_object_api
    {
        return Err(CodeGenError::Internal(
            "FlatBuffers gRPC code generation requires gen_object_api".to_string(),
        ));
    }

    let gen = RustGenerator::new(schema, opts);
    let code = gen.generate()?;

    // Append gRPC service stubs when grpc feature is enabled
    #[cfg(feature = "grpc")]
    let code = {
        let service_code = service_gen::generate_services(schema, &opts.gen_only_files)?;
        if service_code.is_empty() {
            code
        } else {
            format!("{code}\n{service_code}")
        }
    };

    Ok(code)
}

/// Generate TypeScript source code from a fully resolved FlatBuffers schema.
///
/// The generated code is compatible with the `flatbuffers` npm package and
/// includes reader classes, builder static methods, and Object API classes.
pub fn generate_typescript(
    schema: &ResolvedSchema,
    opts: &TsCodeGenOptions,
) -> Result<String, CodeGenError> {
    validate_schema_for_codegen(schema)?;
    let gen = TsGenerator::new(schema, opts);
    gen.generate()
}

/// Generate Python source code from a fully resolved FlatBuffers schema.
///
/// The generated code is dependency-free model code using dataclasses and
/// IntEnum. It preserves table/struct fields, scalar defaults, enums, unions,
/// vectors, fixed arrays, namespaces, documentation, and keyword-safe names.
pub fn generate_python(
    schema: &ResolvedSchema,
    opts: &PythonCodeGenOptions,
) -> Result<String, CodeGenError> {
    let gen = PythonGenerator::new(schema, opts);
    gen.generate()
}
