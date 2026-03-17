//! Tests for the random data generator.

use crate::{generate_json, DataGenConfig, DataGenError};
use flatc_rs_compiler::compile_single;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn compile_and_generate(fbs: &str, seed: u64) -> String {
    let result = compile_single(fbs).expect("schema should compile");
    let legacy = result.schema.as_legacy();
    let root_type = legacy
        .root_table
        .as_ref()
        .and_then(|t| t.name.as_deref())
        .expect("schema should have root_type");
    generate_json(&legacy, root_type, seed, DataGenConfig::default())
        .expect("generate_json should succeed")
}

fn compile_and_generate_config(fbs: &str, seed: u64, config: DataGenConfig) -> String {
    let result = compile_single(fbs).expect("schema should compile");
    let legacy = result.schema.as_legacy();
    let root_type = legacy
        .root_table
        .as_ref()
        .and_then(|t| t.name.as_deref())
        .expect("schema should have root_type");
    generate_json(&legacy, root_type, seed, config).expect("generate_json should succeed")
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn deterministic_same_seed_same_output() {
    let fbs = r#"
        table Monster { hp: int; name: string; }
        root_type Monster;
    "#;
    let a = compile_and_generate(fbs, 42);
    let b = compile_and_generate(fbs, 42);
    assert_eq!(a, b);
}

#[test]
fn different_seeds_different_output() {
    let fbs = r#"
        table Monster { hp: int; name: string; mana: short; }
        root_type Monster;
    "#;
    let a = compile_and_generate(fbs, 1);
    let b = compile_and_generate(fbs, 2);
    // Very unlikely to be identical with different seeds and multiple fields
    assert_ne!(a, b);
}

// ---------------------------------------------------------------------------
// Scalar types
// ---------------------------------------------------------------------------

#[test]
fn scalar_types() {
    let fbs = r#"
        table Scalars {
            a: bool;
            b: byte;
            c: ubyte;
            d: short;
            e: ushort;
            f: int;
            g: uint;
            h: long;
            i: ulong;
            j: float;
            k: double;
        }
        root_type Scalars;
    "#;
    let config = DataGenConfig {
        prob_include_field: 1.0,
        ..DataGenConfig::default()
    };
    let json = compile_and_generate_config(fbs, 42, config);
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    let obj = v.as_object().expect("root should be object");

    // All 11 fields should be present
    assert_eq!(obj.len(), 11, "all scalar fields present: {json}");

    // Type checks
    assert!(obj["a"].is_boolean());
    assert!(obj["b"].is_number());
    assert!(obj["j"].is_number());
    assert!(obj["k"].is_number());
}

// ---------------------------------------------------------------------------
// String fields
// ---------------------------------------------------------------------------

#[test]
fn string_field() {
    let fbs = r#"
        table Msg { text: string; }
        root_type Msg;
    "#;
    let config = DataGenConfig {
        prob_include_field: 1.0,
        ..DataGenConfig::default()
    };
    let json = compile_and_generate_config(fbs, 42, config);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v["text"].is_string());
    assert!(!v["text"].as_str().unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// Enum fields
// ---------------------------------------------------------------------------

#[test]
fn enum_field() {
    let fbs = r#"
        enum Color : byte { Red = 0, Green, Blue }
        table Palette { color: Color; }
        root_type Palette;
    "#;
    let config = DataGenConfig {
        prob_include_field: 1.0,
        ..DataGenConfig::default()
    };
    let json = compile_and_generate_config(fbs, 42, config);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let color = v["color"].as_str().expect("enum should be string");
    assert!(
        ["Red", "Green", "Blue"].contains(&color),
        "unexpected color: {color}"
    );
}

// ---------------------------------------------------------------------------
// Struct fields
// ---------------------------------------------------------------------------

#[test]
fn struct_field() {
    let fbs = r#"
        struct Vec3 { x: float; y: float; z: float; }
        table Entity { pos: Vec3; }
        root_type Entity;
    "#;
    let config = DataGenConfig {
        prob_include_field: 1.0,
        ..DataGenConfig::default()
    };
    let json = compile_and_generate_config(fbs, 42, config);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let pos = v["pos"].as_object().expect("struct should be object");
    assert!(pos.contains_key("x"));
    assert!(pos.contains_key("y"));
    assert!(pos.contains_key("z"));
}

// ---------------------------------------------------------------------------
// Nested table
// ---------------------------------------------------------------------------

#[test]
fn nested_table() {
    let fbs = r#"
        table Inner { value: int; }
        table Outer { child: Inner; }
        root_type Outer;
    "#;
    let config = DataGenConfig {
        prob_include_field: 1.0,
        ..DataGenConfig::default()
    };
    let json = compile_and_generate_config(fbs, 42, config);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v["child"].is_object(), "nested table should be object");
}

// ---------------------------------------------------------------------------
// Vectors
// ---------------------------------------------------------------------------

#[test]
fn vector_of_scalars() {
    let fbs = r#"
        table Data { values: [int]; }
        root_type Data;
    "#;
    let config = DataGenConfig {
        prob_include_field: 1.0,
        min_vector_len: 1,
        max_vector_len: 5,
        ..DataGenConfig::default()
    };
    let json = compile_and_generate_config(fbs, 42, config);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = v["values"].as_array().expect("vector should be array");
    assert!(!arr.is_empty());
    assert!(arr[0].is_number());
}

#[test]
fn vector_of_strings() {
    let fbs = r#"
        table Data { names: [string]; }
        root_type Data;
    "#;
    let config = DataGenConfig {
        prob_include_field: 1.0,
        min_vector_len: 1,
        max_vector_len: 3,
        ..DataGenConfig::default()
    };
    let json = compile_and_generate_config(fbs, 42, config);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = v["names"].as_array().expect("vector should be array");
    assert!(!arr.is_empty());
    assert!(arr[0].is_string());
}

#[test]
fn vector_of_tables() {
    let fbs = r#"
        table Item { id: int; }
        table Container { items: [Item]; }
        root_type Container;
    "#;
    let config = DataGenConfig {
        prob_include_field: 1.0,
        min_vector_len: 1,
        max_vector_len: 3,
        ..DataGenConfig::default()
    };
    let json = compile_and_generate_config(fbs, 42, config);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = v["items"].as_array().expect("vector should be array");
    assert!(!arr.is_empty());
    assert!(arr[0].is_object());
}

// ---------------------------------------------------------------------------
// Union
// ---------------------------------------------------------------------------

#[test]
fn union_field() {
    let fbs = r#"
        table Sword { damage: int; }
        table Shield { defense: int; }
        union Equipment { Sword, Shield }
        table Hero { equip: Equipment; }
        root_type Hero;
    "#;
    let config = DataGenConfig {
        prob_include_field: 1.0,
        ..DataGenConfig::default()
    };
    // Try multiple seeds to ensure at least one produces a union
    let mut found_union = false;
    for seed in 0..20 {
        let json = compile_and_generate_config(fbs, seed, config.clone());
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        if v.get("equip_type").is_some() {
            found_union = true;
            let type_name = v["equip_type"].as_str().unwrap();
            assert!(
                ["Sword", "Shield"].contains(&type_name),
                "unexpected union type: {type_name}"
            );
            assert!(v["equip"].is_object());
            break;
        }
    }
    assert!(found_union, "should generate union in 20 seeds");
}

// ---------------------------------------------------------------------------
// Depth limiting
// ---------------------------------------------------------------------------

#[test]
fn depth_limiting_prevents_infinite_recursion() {
    let fbs = r#"
        table Node { child: Node; value: int; }
        root_type Node;
    "#;
    let config = DataGenConfig {
        max_depth: 2,
        prob_include_field: 1.0,
        ..DataGenConfig::default()
    };
    // Should complete without stack overflow
    let json = compile_and_generate_config(fbs, 42, config);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.is_object());
}

// ---------------------------------------------------------------------------
// Required fields
// ---------------------------------------------------------------------------

#[test]
fn required_field_always_present() {
    let fbs = r#"
        table Msg { name: string (required); }
        root_type Msg;
    "#;
    // Even with prob_include_field = 0, required fields must appear
    let config = DataGenConfig {
        prob_include_field: 0.0,
        ..DataGenConfig::default()
    };
    for seed in 0..10 {
        let json = compile_and_generate_config(fbs, seed, config.clone());
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(
            v.get("name").is_some(),
            "seed {seed}: required field 'name' missing"
        );
    }
}

// ---------------------------------------------------------------------------
// Error cases
// ---------------------------------------------------------------------------

#[test]
fn error_root_type_not_found() {
    let fbs = r#"
        table Monster { hp: int; }
        root_type Monster;
    "#;
    let result = compile_single(fbs).unwrap();
    let err = generate_json(
        &result.schema.as_legacy(),
        "NonExistent",
        42,
        DataGenConfig::default(),
    );
    assert!(err.is_err());
    match err.unwrap_err() {
        DataGenError::RootTypeNotFound { name } => assert_eq!(name, "NonExistent"),
        other => panic!("expected RootTypeNotFound, got: {other}"),
    }
}

// ---------------------------------------------------------------------------
// Round-trip: generate -> encode -> walk
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_with_random_schemas() {
    use flatc_rs_fbs_gen::{GenConfig, SchemaBuilder};

    let gen_config = GenConfig {
        max_enums: 2,
        max_structs: 2,
        max_tables: 3,
        max_unions: 1,
        max_fields_per_type: 4,
        use_namespace: false,
        use_file_ident: false,
        prob_deprecated: 0.0,
        prob_null_default: 0.0,
        prob_nan_inf_default: 0.0,
        prob_rpc_service: 0.0,
        prob_doc_comment: 0.0,
        prob_fixed_array: 0.0,
        ..GenConfig::default()
    };

    let data_config = DataGenConfig {
        max_depth: 2,
        max_vector_len: 3,
        min_vector_len: 0,
        prob_include_field: 0.8,
        max_string_len: 10,
    };

    let mut pass = 0;
    let mut fail = 0;

    for seed in 0..100 {
        let fbs_text = SchemaBuilder::generate(seed, gen_config.clone());

        let compile_result = match compile_single(&fbs_text) {
            Ok(r) => r,
            Err(_) => {
                fail += 1;
                continue;
            }
        };

        let legacy = compile_result.schema.as_legacy();
        let root_type = match legacy.root_table.as_ref().and_then(|t| t.name.as_deref()) {
            Some(n) => n.to_string(),
            None => {
                fail += 1;
                continue;
            }
        };

        let json = match generate_json(&legacy, &root_type, seed, data_config.clone()) {
            Ok(j) => j,
            Err(e) => {
                panic!("seed {seed}: data-gen failed:\nschema:\n{fbs_text}\nerror: {e}");
            }
        };

        // Parse JSON and try to encode it
        let json_value: serde_json::Value = serde_json::from_str(&json).unwrap_or_else(|e| {
            panic!("seed {seed}: invalid JSON produced:\n{json}\nerror: {e}");
        });

        // Skip encode for now -- just verify JSON is valid and parseable
        let _ = json_value;
        pass += 1;
    }

    assert!(
        pass > 50,
        "at least 50/100 seeds should produce valid JSON (got {pass} pass, {fail} fail)"
    );
}
