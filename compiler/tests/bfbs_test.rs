//! Tests for the binary schema (.bfbs) serializer.

use flatc_rs_compiler::{bfbs::serialize_schema, schema::BaseType};

fn read_u32_le(buf: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3]])
}

fn read_u16_le(buf: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([buf[offset], buf[offset + 1]])
}

fn read_i32_le(buf: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes([buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3]])
}

/// Read the number of elements in a vector field of the root Schema table.
/// field_index: 0=objects, 1=enums
fn read_root_vector_len(buf: &[u8], field_index: usize) -> usize {
    let root_offset = read_u32_le(buf, 0) as usize;
    let table_start = root_offset; // root_offset is absolute from buffer start
    let vtable_soffset = read_i32_le(buf, table_start);
    let vtable_loc = (table_start as i64 - vtable_soffset as i64) as usize;

    let vtable_size = read_u16_le(buf, vtable_loc) as usize;
    let slot = 4 + field_index * 2;
    if vtable_size <= slot {
        return 0;
    }

    let field_voff = read_u16_le(buf, vtable_loc + slot) as usize;
    if field_voff == 0 {
        return 0;
    }

    let vec_offset_loc = table_start + field_voff;
    let vec_rel = read_u32_le(buf, vec_offset_loc) as usize;
    let vec_start = vec_offset_loc + vec_rel;
    read_u32_le(buf, vec_start) as usize
}

fn assert_valid_bfbs(buf: &[u8]) {
    assert!(buf.len() >= 8, "buffer too small: {} bytes", buf.len());
    assert_eq!(&buf[4..8], b"BFBS", "missing BFBS file identifier");
}

#[test]
fn test_bfbs_simple_table() {
    let schema = flatc_rs_compiler::compile_single("table Monster { hp: short; name: string; }")
        .expect("compile_single failed")
        .schema;

    let buf = serialize_schema(&schema);
    assert_valid_bfbs(&buf);

    let num_objects = read_root_vector_len(&buf, 0);
    let num_enums = read_root_vector_len(&buf, 1);
    assert_eq!(num_objects, 1, "expected 1 object (Monster)");
    assert_eq!(num_enums, 0, "expected 0 enums");
}

#[test]
fn test_bfbs_enum_and_table() {
    let schema = flatc_rs_compiler::compile_single(
        "enum Color : byte { Red, Green, Blue } table Monster { color: Color; }",
    )
    .expect("compile_single failed")
    .schema;

    let buf = serialize_schema(&schema);
    assert_valid_bfbs(&buf);

    let num_objects = read_root_vector_len(&buf, 0);
    let num_enums = read_root_vector_len(&buf, 1);
    assert_eq!(num_objects, 1);
    assert_eq!(num_enums, 1);
}

#[test]
fn test_bfbs_struct_and_table() {
    let schema = flatc_rs_compiler::compile_single(
        "struct Vec3 { x: float; y: float; z: float; } table Monster { pos: Vec3; }",
    )
    .expect("compile_single failed")
    .schema;

    let buf = serialize_schema(&schema);
    assert_valid_bfbs(&buf);

    let num_objects = read_root_vector_len(&buf, 0);
    assert_eq!(num_objects, 2, "expected 2 objects (Monster + Vec3)");
}

#[test]
fn test_bfbs_file_identifier() {
    let schema = flatc_rs_compiler::compile_single(
        "table Root { x: int; } root_type Root; file_identifier \"TEST\";",
    )
    .expect("compile_single failed")
    .schema;

    let buf = serialize_schema(&schema);
    assert_valid_bfbs(&buf);

    // Schema contains the original file_ident
    assert_eq!(schema.file_ident.as_deref(), Some("TEST"));
    // But the .bfbs file itself always has "BFBS"
    assert_eq!(&buf[4..8], b"BFBS");
}

#[test]
fn test_bfbs_base_type_discriminants() {
    assert_eq!(BaseType::BASE_TYPE_NONE as u8, 0);
    assert_eq!(BaseType::BASE_TYPE_U_TYPE as u8, 1);
    assert_eq!(BaseType::BASE_TYPE_BOOL as u8, 2);
    assert_eq!(BaseType::BASE_TYPE_BYTE as u8, 3);
    assert_eq!(BaseType::BASE_TYPE_U_BYTE as u8, 4);
    assert_eq!(BaseType::BASE_TYPE_SHORT as u8, 5);
    assert_eq!(BaseType::BASE_TYPE_U_SHORT as u8, 6);
    assert_eq!(BaseType::BASE_TYPE_INT as u8, 7);
    assert_eq!(BaseType::BASE_TYPE_U_INT as u8, 8);
    assert_eq!(BaseType::BASE_TYPE_LONG as u8, 9);
    assert_eq!(BaseType::BASE_TYPE_U_LONG as u8, 10);
    assert_eq!(BaseType::BASE_TYPE_FLOAT as u8, 11);
    assert_eq!(BaseType::BASE_TYPE_DOUBLE as u8, 12);
    assert_eq!(BaseType::BASE_TYPE_STRING as u8, 13);
    assert_eq!(BaseType::BASE_TYPE_VECTOR as u8, 14);
    assert_eq!(BaseType::BASE_TYPE_TABLE as u8, 15);
    assert_eq!(BaseType::BASE_TYPE_UNION as u8, 16);
    assert_eq!(BaseType::BASE_TYPE_ARRAY as u8, 17);
    assert_eq!(BaseType::BASE_TYPE_VECTOR64 as u8, 18);
    // STRUCT maps to Obj (15) for binary serialization
    assert_eq!(BaseType::BASE_TYPE_STRUCT.to_reflection_byte(), 15);
}

#[test]
fn test_bfbs_roundtrip_size_sanity() {
    // A schema with multiple types should produce a reasonably sized .bfbs
    let schema = flatc_rs_compiler::compile_single(
        r#"
        namespace MyGame;
        enum Color : byte { Red = 1, Green, Blue }
        struct Vec3 { x: float; y: float; z: float; }
        table Monster {
            pos: Vec3;
            hp: short = 100;
            name: string;
            color: Color = Blue;
            inventory: [ubyte];
        }
        root_type Monster;
        file_identifier "MONS";
        "#,
    )
    .expect("compile_single failed")
    .schema;

    let buf = serialize_schema(&schema);
    assert_valid_bfbs(&buf);
    assert!(buf.len() > 100, "buf too small for multi-type schema");

    let num_objects = read_root_vector_len(&buf, 0);
    let num_enums = read_root_vector_len(&buf, 1);
    assert_eq!(num_objects, 2, "expected Monster + Vec3");
    assert_eq!(num_enums, 1, "expected Color");
}
