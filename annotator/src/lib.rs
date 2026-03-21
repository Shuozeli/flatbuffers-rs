mod binary_walker;
mod formatter;
pub mod region;

pub use binary_walker::BinaryWalker;
pub use formatter::format_afb;
pub use region::{AnnotatedRegion, RegionType, WalkError};

pub use flatc_rs_schema::resolved::ResolvedSchema;

/// Walk a FlatBuffers binary using a Schema and root type name.
pub fn walk_binary(
    binary: &[u8],
    schema: &ResolvedSchema,
    root_type_name: &str,
) -> Result<Vec<AnnotatedRegion>, WalkError> {
    let walker = BinaryWalker::new(binary, schema);
    walker.walk(root_type_name)
}

/// Annotate a FlatBuffers binary and format the output as a `.afb` text file
/// matching the C++ `flatc --annotate` format.
pub fn annotate_binary(
    binary: &[u8],
    schema: &ResolvedSchema,
    root_type_name: &str,
    schema_file: &str,
    binary_file: &str,
) -> Result<String, WalkError> {
    let regions = walk_binary(binary, schema, root_type_name)?;
    Ok(format_afb(binary, &regions, schema_file, binary_file))
}
