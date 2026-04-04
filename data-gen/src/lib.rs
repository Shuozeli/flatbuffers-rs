//! Random FlatBuffers JSON data generator from a compiled Schema.
//!
//! Given a compiled [`Schema`] and a root type name, generates random JSON
//! data that conforms to the schema's structure. Uses a deterministic seed
//! so the same `(schema, seed, config)` always produces the same JSON.

mod builder;

use flatc_rs_schema::Schema;
use rand::rngs::StdRng;
use rand::SeedableRng;

pub use builder::DataBuilder;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Controls the shape and size of generated JSON data.
#[derive(Clone)]
pub struct DataGenConfig {
    /// Maximum recursion depth for nested tables (prevents infinite recursion).
    pub max_depth: usize,
    /// Maximum number of elements in generated vectors.
    pub max_vector_len: usize,
    /// Minimum number of elements in generated vectors.
    pub min_vector_len: usize,
    /// Probability that a non-required field is included (0.0 to 1.0).
    pub prob_include_field: f64,
    /// Maximum length for randomly generated strings.
    pub max_string_len: usize,
}

impl Default for DataGenConfig {
    fn default() -> Self {
        Self {
            max_depth: 3,
            max_vector_len: 5,
            min_vector_len: 0,
            prob_include_field: 0.95,
            max_string_len: 20,
        }
    }
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum DataGenError {
    RootTypeNotFound { name: String },
    ObjectIndexOutOfRange { index: usize, count: usize },
    EnumIndexOutOfRange { index: usize, count: usize },
    NoRootTable,
    JsonSerialization { source: serde_json::Error },
}

impl std::fmt::Display for DataGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataGenError::RootTypeNotFound { name } => {
                write!(f, "root type '{name}' not found in schema")
            }
            DataGenError::ObjectIndexOutOfRange { index, count } => {
                write!(
                    f,
                    "object index {index} out of range (have {count} objects)"
                )
            }
            DataGenError::EnumIndexOutOfRange { index, count } => {
                write!(f, "enum index {index} out of range (have {count} enums)")
            }
            DataGenError::NoRootTable => write!(f, "schema has no root_table defined"),
            DataGenError::JsonSerialization { source } => {
                write!(f, "JSON serialization failed: {source}")
            }
        }
    }
}

impl std::error::Error for DataGenError {}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate random JSON data conforming to the given schema.
///
/// Returns a pretty-printed JSON string representing a valid instance of the
/// root table identified by `root_type`.
pub fn generate_json(
    schema: &Schema,
    root_type: &str,
    seed: u64,
    config: DataGenConfig,
) -> Result<String, DataGenError> {
    let rng = StdRng::seed_from_u64(seed);
    let mut builder = DataBuilder::new(schema, rng, config);
    let value = builder.build_root(root_type)?;
    serde_json::to_string_pretty(&value)
        .map_err(|source| DataGenError::JsonSerialization { source })
}

#[cfg(test)]
mod tests;
