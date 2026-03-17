mod builder;
mod helpers;
mod object_api;
mod reader;

use flatc_rs_schema::resolved::ResolvedSchema;
use flatc_rs_schema::BaseType;

use super::code_writer::CodeWriter;
use super::ts_type_map;
use super::type_map;
use super::TsCodeGenOptions;

/// Generate TypeScript code for the table at `schema.objects[index]`.
pub fn generate(
    w: &mut CodeWriter,
    schema: &ResolvedSchema,
    index: usize,
    opts: &TsCodeGenOptions,
) {
    let gen_object_api = opts.gen_object_api;
    let obj = &schema.objects[index];
    let name = &obj.name;
    let fqn = ts_type_map::build_fqn(obj);

    // Check if this table is the root table using the authoritative index
    let is_root = schema
        .root_table_index
        .map(|idx| idx == index)
        .unwrap_or(false);
    let file_ident = schema.file_ident.as_deref().unwrap_or("");

    let implements = if gen_object_api {
        format!(" implements flatbuffers.IUnpackableObject<{name}T>")
    } else {
        String::new()
    };

    // Documentation
    ts_type_map::gen_doc_comment(w, obj.documentation.as_ref());

    w.block(&format!("export class {name}{implements}"), |w| {
        // bb and bb_pos properties
        w.line("bb: flatbuffers.ByteBuffer|null = null;");
        w.line("bb_pos = 0;");
        w.blank();

        // __init
        w.block(
            &format!("__init(i:number, bb:flatbuffers.ByteBuffer):{name}"),
            |w| {
                w.line("this.bb_pos = i;");
                w.line("this.bb = bb;");
                w.line("return this;");
            },
        );
        w.blank();

        // getRootAsXxx
        reader::gen_get_root_as(w, name);
        w.blank();

        // getSizePrefixedRootAsXxx
        reader::gen_get_size_prefixed_root_as(w, name);
        w.blank();

        // bufferHasIdentifier (only for root table with file_ident)
        if is_root && !file_ident.is_empty() {
            w.block(
                "static bufferHasIdentifier(bb:flatbuffers.ByteBuffer):boolean",
                |w| {
                    w.line(&format!("return bb.__has_identifier('{file_ident}');"));
                },
            );
            w.blank();
        }

        // getFullyQualifiedName
        w.block(
            &format!("static getFullyQualifiedName(): \"{fqn}\""),
            |w| {
                w.line(&format!("return '{fqn}';"));
            },
        );
        w.blank();

        // Field accessors
        for field in &obj.fields {
            if field.is_deprecated {
                continue;
            }
            ts_type_map::gen_doc_comment(w, field.documentation.as_ref());
            reader::gen_field_accessor(w, schema, obj, field);
            w.blank();
        }

        // Field mutators (for mutable fields, only with --gen-mutable)
        if opts.gen_mutable {
            for field in &obj.fields {
                if field.is_deprecated {
                    continue;
                }
                let bt = type_map::get_base_type(&field.type_);
                if type_map::is_scalar(bt) || bt == BaseType::BASE_TYPE_BOOL {
                    reader::gen_field_mutator(w, schema, field);
                    w.blank();
                }
            }
        }

        // Static builder methods
        builder::gen_start_method(w, obj, name);
        w.blank();

        for field in &obj.fields {
            if field.is_deprecated {
                continue;
            }
            builder::gen_add_method(w, schema, obj, field, name);
            w.blank();

            // Vector create/start helpers
            let bt = type_map::get_base_type(&field.type_);
            if bt == BaseType::BASE_TYPE_VECTOR {
                builder::gen_vector_helpers(w, schema, field);
                w.blank();
            }
        }

        builder::gen_end_method(w, obj, name);
        w.blank();

        // finishXxxBuffer / finishSizePrefixedXxxBuffer (root table with file_ident)
        if is_root && !file_ident.is_empty() {
            w.block(
                &format!("static finish{name}Buffer(builder:flatbuffers.Builder, offset:flatbuffers.Offset)"),
                |w| {
                    w.line(&format!("builder.finish(offset, '{file_ident}');"));
                },
            );
            w.blank();
            w.block(
                &format!("static finishSizePrefixed{name}Buffer(builder:flatbuffers.Builder, offset:flatbuffers.Offset)"),
                |w| {
                    w.line(&format!("builder.finish(offset, '{file_ident}', true);"));
                },
            );
            w.blank();
        }

        // Convenience create function
        builder::gen_create_fn(w, schema, obj, name);

        // serialize / deserialize
        w.blank();
        w.block("serialize():Uint8Array", |w| {
            w.line("return this.bb!.bytes();");
        });
        w.blank();
        w.block(
            &format!("static deserialize(buffer: Uint8Array):{name}"),
            |w| {
                w.line(&format!(
                    "return {name}.getRootAs{name}(new flatbuffers.ByteBuffer(buffer))"
                ));
            },
        );

        // unpack / unpackTo (Object API)
        if gen_object_api {
            w.blank();
            object_api::gen_unpack(w, schema, obj, name);
            w.blank();
            object_api::gen_unpack_to(w, schema, obj, name);
        }
    });

    // Object API T class
    if gen_object_api {
        w.blank();
        object_api::gen_object_api_class(w, schema, obj, name);
    }
}
