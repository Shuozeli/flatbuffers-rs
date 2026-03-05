//! Tests for JSON <-> binary conversion.

use flatc_rs_compiler::{
    compile_single,
    json::{binary_to_json, json_to_binary, json_to_binary_with_opts, EncoderOptions, JsonOptions},
};
use serde_json::json;

fn default_opts() -> JsonOptions {
    JsonOptions {
        strict_json: false,
        output_defaults: false,
        output_enum_identifiers: true,
        size_prefixed: false,
    }
}

// ---------------------------------------------------------------------------
// Round-trip: JSON -> binary -> JSON
// ---------------------------------------------------------------------------

#[test]
fn round_trip_simple_table() {
    let result = compile_single(
        "table Monster { name:string; hp:short = 100; mana:short = 150; }
         root_type Monster;",
    )
    .unwrap();

    let input = json!({
        "name": "Orc",
        "hp": 80,
        "mana": 200
    });

    let bin = json_to_binary(&input, &result.schema, "Monster").unwrap();
    let output = binary_to_json(&bin, &result.schema, "Monster", &default_opts()).unwrap();

    assert_eq!(output["name"], "Orc");
    assert_eq!(output["hp"], 80);
    assert_eq!(output["mana"], 200);
}

#[test]
fn round_trip_with_struct() {
    let result = compile_single(
        "struct Vec3 { x:float; y:float; z:float; }
         table Monster { name:string; pos:Vec3; }
         root_type Monster;",
    )
    .unwrap();

    let input = json!({
        "name": "Dragon",
        "pos": { "x": 1.0, "y": 2.0, "z": 3.0 }
    });

    let bin = json_to_binary(&input, &result.schema, "Monster").unwrap();
    let output = binary_to_json(&bin, &result.schema, "Monster", &default_opts()).unwrap();

    assert_eq!(output["name"], "Dragon");
    let pos = &output["pos"];
    assert_eq!(pos["x"], 1.0);
    assert_eq!(pos["y"], 2.0);
    assert_eq!(pos["z"], 3.0);
}

#[test]
fn round_trip_with_vector_of_scalars() {
    let result = compile_single(
        "table Stats { values:[int]; }
         root_type Stats;",
    )
    .unwrap();

    let input = json!({
        "values": [10, 20, 30, 40]
    });

    let bin = json_to_binary(&input, &result.schema, "Stats").unwrap();
    let output = binary_to_json(&bin, &result.schema, "Stats", &default_opts()).unwrap();

    assert_eq!(output["values"], json!([10, 20, 30, 40]));
}

#[test]
fn round_trip_with_vector_of_strings() {
    let result = compile_single(
        "table Names { items:[string]; }
         root_type Names;",
    )
    .unwrap();

    let input = json!({
        "items": ["Alice", "Bob", "Charlie"]
    });

    let bin = json_to_binary(&input, &result.schema, "Names").unwrap();
    let output = binary_to_json(&bin, &result.schema, "Names", &default_opts()).unwrap();

    assert_eq!(output["items"], json!(["Alice", "Bob", "Charlie"]));
}

#[test]
fn round_trip_with_nested_table() {
    let result = compile_single(
        "table Weapon { name:string; damage:int; }
         table Monster { name:string; weapon:Weapon; }
         root_type Monster;",
    )
    .unwrap();

    let input = json!({
        "name": "Orc",
        "weapon": { "name": "Axe", "damage": 15 }
    });

    let bin = json_to_binary(&input, &result.schema, "Monster").unwrap();
    let output = binary_to_json(&bin, &result.schema, "Monster", &default_opts()).unwrap();

    assert_eq!(output["name"], "Orc");
    assert_eq!(output["weapon"]["name"], "Axe");
    assert_eq!(output["weapon"]["damage"], 15);
}

#[test]
fn round_trip_with_vector_of_tables() {
    let result = compile_single(
        "table Item { name:string; count:int; }
         table Inventory { items:[Item]; }
         root_type Inventory;",
    )
    .unwrap();

    let input = json!({
        "items": [
            { "name": "Sword", "count": 1 },
            { "name": "Shield", "count": 2 }
        ]
    });

    let bin = json_to_binary(&input, &result.schema, "Inventory").unwrap();
    let output = binary_to_json(&bin, &result.schema, "Inventory", &default_opts()).unwrap();

    let items = output["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["name"], "Sword");
    assert_eq!(items[0]["count"], 1);
    assert_eq!(items[1]["name"], "Shield");
    assert_eq!(items[1]["count"], 2);
}

#[test]
fn round_trip_with_enum() {
    let result = compile_single(
        "enum Color:byte { Red = 0, Green = 1, Blue = 2 }
         table Monster { name:string; color:Color; }
         root_type Monster;",
    )
    .unwrap();

    // Enum as string name
    let input = json!({
        "name": "Goblin",
        "color": "Green"
    });

    let bin = json_to_binary(&input, &result.schema, "Monster").unwrap();
    let output = binary_to_json(&bin, &result.schema, "Monster", &default_opts()).unwrap();

    assert_eq!(output["name"], "Goblin");
    assert_eq!(output["color"], "Green");
}

#[test]
fn round_trip_with_enum_as_number() {
    let result = compile_single(
        "enum Color:byte { Red = 0, Green = 1, Blue = 2 }
         table Monster { name:string; color:Color; }
         root_type Monster;",
    )
    .unwrap();

    // Enum as integer
    let input = json!({
        "name": "Goblin",
        "color": 2
    });

    let bin = json_to_binary(&input, &result.schema, "Monster").unwrap();

    // With enum identifiers enabled, should output name
    let output = binary_to_json(&bin, &result.schema, "Monster", &default_opts()).unwrap();
    assert_eq!(output["color"], "Blue");

    // With enum identifiers disabled, should output number
    let mut opts = default_opts();
    opts.output_enum_identifiers = false;
    let output = binary_to_json(&bin, &result.schema, "Monster", &opts).unwrap();
    assert_eq!(output["color"], 2);
}

#[test]
fn round_trip_with_bool() {
    let result = compile_single(
        "table Config { enabled:bool; count:int; }
         root_type Config;",
    )
    .unwrap();

    let input = json!({
        "enabled": true,
        "count": 42
    });

    let bin = json_to_binary(&input, &result.schema, "Config").unwrap();
    let output = binary_to_json(&bin, &result.schema, "Config", &default_opts()).unwrap();

    assert_eq!(output["enabled"], true);
    assert_eq!(output["count"], 42);
}

#[test]
fn round_trip_with_vector_of_structs() {
    let result = compile_single(
        "struct Vec2 { x:float; y:float; }
         table Path { points:[Vec2]; }
         root_type Path;",
    )
    .unwrap();

    let input = json!({
        "points": [
            { "x": 1.0, "y": 2.0 },
            { "x": 3.0, "y": 4.0 }
        ]
    });

    let bin = json_to_binary(&input, &result.schema, "Path").unwrap();
    let output = binary_to_json(&bin, &result.schema, "Path", &default_opts()).unwrap();

    let points = output["points"].as_array().unwrap();
    assert_eq!(points.len(), 2);
    assert_eq!(points[0]["x"], 1.0);
    assert_eq!(points[0]["y"], 2.0);
    assert_eq!(points[1]["x"], 3.0);
    assert_eq!(points[1]["y"], 4.0);
}

#[test]
fn round_trip_sparse_table() {
    // Test that omitted fields remain absent (not output as default)
    let result = compile_single(
        "table Monster { name:string; hp:short = 100; mana:short = 150; }
         root_type Monster;",
    )
    .unwrap();

    let input = json!({
        "name": "Orc"
    });

    let bin = json_to_binary(&input, &result.schema, "Monster").unwrap();
    let output = binary_to_json(&bin, &result.schema, "Monster", &default_opts()).unwrap();

    assert_eq!(output["name"], "Orc");
    // hp and mana not present in binary, should not appear
    assert!(output.get("hp").is_none());
    assert!(output.get("mana").is_none());
}

#[test]
fn round_trip_with_defaults_option() {
    let result = compile_single(
        "table Monster { name:string; hp:short = 100; mana:short = 150; }
         root_type Monster;",
    )
    .unwrap();

    let input = json!({
        "name": "Orc"
    });

    let bin = json_to_binary(&input, &result.schema, "Monster").unwrap();

    let mut opts = default_opts();
    opts.output_defaults = true;
    let output = binary_to_json(&bin, &result.schema, "Monster", &opts).unwrap();

    assert_eq!(output["name"], "Orc");
    // With defaults enabled, missing fields should appear with default values
    assert_eq!(output["hp"], 100);
    assert_eq!(output["mana"], 150);
}

#[test]
fn round_trip_all_scalar_types() {
    let result = compile_single(
        "table AllScalars {
            f_bool:bool;
            f_byte:byte;
            f_ubyte:ubyte;
            f_short:short;
            f_ushort:ushort;
            f_int:int;
            f_uint:uint;
            f_long:long;
            f_ulong:ulong;
            f_float:float;
            f_double:double;
         }
         root_type AllScalars;",
    )
    .unwrap();

    let input = json!({
        "f_bool": true,
        "f_byte": -42,
        "f_ubyte": 200,
        "f_short": -1000,
        "f_ushort": 60000,
        "f_int": -100000,
        "f_uint": 3000000000u64,
        "f_long": -9000000000000i64,
        "f_ulong": 18000000000000000000u64,
        "f_float": 3.14,
        "f_double": 2.718281828
    });

    let bin = json_to_binary(&input, &result.schema, "AllScalars").unwrap();
    let output = binary_to_json(&bin, &result.schema, "AllScalars", &default_opts()).unwrap();

    assert_eq!(output["f_bool"], true);
    assert_eq!(output["f_byte"], -42);
    assert_eq!(output["f_ubyte"], 200);
    assert_eq!(output["f_short"], -1000);
    assert_eq!(output["f_ushort"], 60000);
    assert_eq!(output["f_int"], -100000);
    assert_eq!(output["f_uint"], 3000000000u64);
    assert_eq!(output["f_long"], -9000000000000i64);
    assert_eq!(output["f_ulong"], 18000000000000000000u64);

    // Float precision: f32 round-trip may lose precision
    let f = output["f_float"].as_f64().unwrap();
    assert!((f - 3.14).abs() < 0.001);

    let d = output["f_double"].as_f64().unwrap();
    assert!((d - 2.718281828).abs() < 1e-9);
}

#[test]
fn round_trip_union() {
    let result = compile_single(
        "table Sword { damage:int; }
         table Shield { defense:int; }
         union Equipment { Sword, Shield }
         table Hero { name:string; equipment:Equipment; }
         root_type Hero;",
    )
    .unwrap();

    let input = json!({
        "name": "Knight",
        "equipment_type": "Sword",
        "equipment": { "damage": 25 }
    });

    let bin = json_to_binary(&input, &result.schema, "Hero").unwrap();
    let output = binary_to_json(&bin, &result.schema, "Hero", &default_opts()).unwrap();

    assert_eq!(output["name"], "Knight");
    assert_eq!(output["equipment_type"], "Sword");
    assert_eq!(output["equipment"]["damage"], 25);
}

// ---------------------------------------------------------------------------
// Cross-compat: read C++ flatc binary data
// ---------------------------------------------------------------------------

#[test]
fn decode_cpp_monsterdata() {
    // The official monsterdata_test.mon is built by C++ flatc
    let mon_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../original-flatc/tests/monsterdata_test.mon"
    );
    let schema_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../testdata/reference/schemas/monster_test.fbs"
    );
    let include_dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../testdata/reference/schemas/include_test"
    );

    let buf = match std::fs::read(mon_path) {
        Ok(b) => b,
        Err(_) => {
            eprintln!("skipping decode_cpp_monsterdata: monsterdata_test.mon not found");
            return;
        }
    };

    let schema_file = std::path::PathBuf::from(schema_path);
    let opts = flatc_rs_compiler::CompilerOptions {
        include_paths: vec![
            schema_file.parent().unwrap().to_path_buf(),
            std::path::PathBuf::from(include_dir),
        ],
    };
    let result = flatc_rs_compiler::compile(&[schema_file], &opts).unwrap();

    // The schema has two "Monster" objects (MyGame.Example2.Monster and
    // MyGame.Example.Monster). The root_table is the full one, which is
    // stored at a specific index. Find it by field count.
    let root_name = "Monster";
    // Verify the root_table is set
    assert!(result.schema.root_table.is_some());
    let root_obj = result.schema.root_table.as_ref().unwrap();
    assert!(
        root_obj.fields.len() > 5,
        "root Monster should have many fields"
    );

    let json_val = binary_to_json(&buf, &result.schema, root_name, &default_opts()).unwrap();

    // Verify key fields match the known monsterdata_test values
    assert_eq!(json_val["name"], "MyMonster");
    assert_eq!(json_val["hp"], 80);
    // mana=150 is the default value and not stored in the binary,
    // so it won't appear without output_defaults

    // pos struct
    let pos = &json_val["pos"];
    assert_eq!(pos["x"], 1.0);
    assert_eq!(pos["y"], 2.0);
    assert_eq!(pos["z"], 3.0);

    // inventory vector (byte vector)
    let inv = json_val["inventory"].as_array().unwrap();
    assert_eq!(inv.len(), 5);
    assert_eq!(inv[0], 0);
    assert_eq!(inv[1], 1);
    assert_eq!(inv[2], 2);
    assert_eq!(inv[3], 3);
    assert_eq!(inv[4], 4);

    // testarrayofstring
    let strings = json_val["testarrayofstring"].as_array().unwrap();
    assert!(strings.len() >= 2);
    assert_eq!(strings[0], "test1");
    assert_eq!(strings[1], "test2");
}

// ---------------------------------------------------------------------------
// Error cases
// ---------------------------------------------------------------------------

#[test]
fn decode_empty_buffer_fails() {
    let result = compile_single("table Monster { name:string; } root_type Monster;").unwrap();

    let err = binary_to_json(&[], &result.schema, "Monster", &default_opts());
    assert!(err.is_err());
}

#[test]
fn encode_wrong_root_type_fails() {
    let result = compile_single("table Monster { name:string; } root_type Monster;").unwrap();

    let input = json!({"name": "test"});
    let err = json_to_binary(&input, &result.schema, "NonExistent");
    assert!(err.is_err());
}

#[test]
fn encode_wrong_json_type_fails() {
    let result = compile_single("table Monster { name:string; } root_type Monster;").unwrap();

    let input = json!("not an object");
    let err = json_to_binary(&input, &result.schema, "Monster");
    assert!(err.is_err());
}

// ---------------------------------------------------------------------------
// --unknown-json flag tests
// ---------------------------------------------------------------------------

#[test]
fn unknown_json_field_errors_by_default() {
    let result =
        compile_single("table Monster { name:string; hp:int; } root_type Monster;").unwrap();

    let input = json!({
        "name": "Orc",
        "hp": 100,
        "unknown_field": 42
    });

    // Default: strict mode should error on unknown fields
    let err = json_to_binary(&input, &result.schema, "Monster");
    assert!(err.is_err());
    let err_msg = format!("{}", err.unwrap_err());
    assert!(
        err_msg.contains("unknown_field"),
        "error should mention the unknown field name"
    );
}

#[test]
fn unknown_json_field_skipped_with_flag() {
    let result =
        compile_single("table Monster { name:string; hp:int; } root_type Monster;").unwrap();

    let input = json!({
        "name": "Orc",
        "hp": 100,
        "unknown_field": 42,
        "another_unknown": "ignored"
    });

    // With unknown_json=true, unknown fields should be silently skipped
    let opts = EncoderOptions {
        unknown_json: true,
        ..Default::default()
    };
    let bin = json_to_binary_with_opts(&input, &result.schema, "Monster", &opts).unwrap();
    let output = binary_to_json(&bin, &result.schema, "Monster", &default_opts()).unwrap();

    assert_eq!(output["name"], "Orc");
    assert_eq!(output["hp"], 100);
    // Unknown fields should not appear in round-tripped output
    assert!(output.get("unknown_field").is_none());
    assert!(output.get("another_unknown").is_none());
}

// ---------------------------------------------------------------------------
// --size-prefixed flag tests
// ---------------------------------------------------------------------------

#[test]
fn size_prefixed_binary_decode() {
    let result =
        compile_single("table Monster { name:string; hp:int; } root_type Monster;").unwrap();

    let input = json!({
        "name": "Dragon",
        "hp": 200
    });

    // Encode to binary (not size-prefixed)
    let bin = json_to_binary(&input, &result.schema, "Monster").unwrap();

    // Create a size-prefixed version: 4-byte little-endian length + binary
    let len = bin.len() as u32;
    let mut prefixed = len.to_le_bytes().to_vec();
    prefixed.extend_from_slice(&bin);

    // Decode with size_prefixed=true
    let mut opts = default_opts();
    opts.size_prefixed = true;
    let output = binary_to_json(&prefixed, &result.schema, "Monster", &opts).unwrap();

    assert_eq!(output["name"], "Dragon");
    assert_eq!(output["hp"], 200);
}

#[test]
fn size_prefixed_false_rejects_prefixed_binary() {
    let result =
        compile_single("table Monster { name:string; hp:int; } root_type Monster;").unwrap();

    let input = json!({ "name": "Orc", "hp": 50 });
    let bin = json_to_binary(&input, &result.schema, "Monster").unwrap();

    // Size-prefixed binary read without the flag should fail or produce wrong data
    let len = bin.len() as u32;
    let mut prefixed = len.to_le_bytes().to_vec();
    prefixed.extend_from_slice(&bin);

    // Without size_prefixed, the decoder reads from offset 0 which is the length bytes,
    // treating them as the root offset -- this should produce garbage or error
    let result_decode = binary_to_json(&prefixed, &result.schema, "Monster", &default_opts());
    // Either it errors or produces incorrect data (not "Orc")
    match result_decode {
        Err(_) => {} // expected
        Ok(val) => assert_ne!(
            val["name"], "Orc",
            "should not correctly decode without size_prefixed flag"
        ),
    }
}
