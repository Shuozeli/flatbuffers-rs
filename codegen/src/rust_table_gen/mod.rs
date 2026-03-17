mod builder;
mod helpers;
mod object_api;
mod reader;

use flatc_rs_schema::resolved::ResolvedSchema;

use super::code_writer::CodeWriter;
use super::type_map;
use super::{type_visibility, CodeGenError, CodeGenOptions};

/// Generate Rust code for the table at `schema.objects[index]`.
pub fn generate(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    index: usize,
    opts: &CodeGenOptions,
) -> Result<(), CodeGenError> {
    let obj = &schema.objects[index];
    let name = &obj.name;
    let vis = type_visibility(obj.attributes.as_ref(), opts);
    let current_ns = type_map::object_namespace(obj);

    reader::gen_offset_marker(w, name, vis);
    reader::gen_reader_struct(w, name, vis);
    w.blank();
    reader::gen_follow_impl(w, name);
    w.blank();
    reader::gen_impl_block(w, schema, obj, name, current_ns)?;
    w.blank();
    reader::gen_verifiable_impl(w, schema, obj, name, current_ns)?;
    builder::gen_args_struct(w, schema, obj, name, current_ns)?;
    w.blank();
    builder::gen_builder(w, schema, obj, name, current_ns)?;
    w.blank();
    reader::gen_debug_impl(w, obj, name, opts);
    w.blank();
    builder::gen_create_fn(w, obj, name);

    // Object API: owned T type with pack/unpack
    if opts.gen_object_api {
        w.blank();
        object_api::gen_object_api(w, schema, obj, name, current_ns, opts)?;
    }
    Ok(())
}
