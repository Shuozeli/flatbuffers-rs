use std::collections::HashSet;

use crate::builder::SchemaBuilder;
use crate::chooser::ScriptedChooser;
use crate::config::GenConfig;
use crate::types::ALL_SCALAR_TYPES;

// =========================================================================
// Migrated tests (from original fbs_gen.rs)
// =========================================================================

// -- Determinism --

#[test]
fn deterministic_output() {
    let a = SchemaBuilder::generate(42, GenConfig::default());
    let b = SchemaBuilder::generate(42, GenConfig::default());
    assert_eq!(a, b, "same seed must produce identical output");
}

#[test]
fn different_seeds_differ() {
    let a = SchemaBuilder::generate(1, GenConfig::default());
    let b = SchemaBuilder::generate(2, GenConfig::default());
    assert_ne!(a, b, "different seeds should produce different output");
}

// -- Capacity limits --

#[test]
fn enums_only() {
    let config = GenConfig {
        max_enums: 3,
        max_structs: 0,
        max_tables: 1,
        max_unions: 0,
        use_namespace: false,
        use_file_ident: false,
        ..GenConfig::default()
    };
    let out = SchemaBuilder::generate(10, config);
    assert!(out.contains("table "));
    assert!(!out.contains("struct "));
    assert!(!out.contains("union "));
}

#[test]
fn structs_only() {
    let config = GenConfig {
        max_enums: 0,
        max_structs: 3,
        max_tables: 1,
        max_unions: 0,
        use_namespace: false,
        use_file_ident: false,
        ..GenConfig::default()
    };
    let out = SchemaBuilder::generate(10, config);
    assert!(out.contains("table "));
    assert!(!out.contains("enum "));
    assert!(!out.contains("union "));
}

#[test]
fn tables_only() {
    let config = GenConfig {
        max_enums: 0,
        max_structs: 0,
        max_tables: 5,
        max_unions: 0,
        use_namespace: false,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        assert!(out.contains("table "));
        assert!(!out.contains("enum "));
        assert!(!out.contains("struct "));
        assert!(!out.contains("union "));
    }
}

#[test]
fn unions_require_variant_tables() {
    let config = GenConfig {
        max_enums: 0,
        max_structs: 0,
        max_tables: 3,
        max_unions: 2,
        max_union_variants: 2,
        use_namespace: false,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        if let Some(union_pos) = out.find("union ") {
            let union_block = &out[union_pos..];
            let brace_end = union_block.find('}').unwrap();
            let body = &union_block[..brace_end];
            for line in body.lines().skip(1) {
                let variant = line.trim().trim_end_matches(',');
                if variant.is_empty() {
                    continue;
                }
                let table_decl = format!("table {variant} {{");
                let table_pos = out.find(&table_decl).unwrap_or_else(|| {
                    panic!("seed {seed}: union variant '{variant}' has no table definition\n{out}")
                });
                assert!(
                    table_pos < union_pos,
                    "seed {seed}: variant table '{variant}' must appear before its union\n{out}"
                );
            }
        }
    }
}

#[test]
fn minimal_config_works() {
    let config = GenConfig {
        max_enums: 0,
        max_structs: 0,
        max_tables: 1,
        max_unions: 0,
        max_fields_per_type: 1,
        max_enum_values: 2,
        max_union_variants: 2,
        max_fixed_array_len: 0,
        use_namespace: false,
        use_root_type: true,
        use_file_ident: false,
        ..GenConfig::default()
    };
    let out = SchemaBuilder::generate(0, config);
    assert!(out.contains("table "), "must contain at least one table");
}

// -- Namespace --

#[test]
fn namespace_always_emitted_at_prob_1() {
    let config = GenConfig {
        use_namespace: true,
        prob_namespace: 1.0,
        prob_multi_namespace: 0.0,
        max_tables: 1,
        max_enums: 0,
        max_structs: 0,
        max_unions: 0,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        assert!(
            out.contains("namespace "),
            "seed {seed}: prob_namespace=1.0 should always emit namespace\n{out}"
        );
    }
}

#[test]
fn namespace_never_emitted_at_prob_0() {
    let config = GenConfig {
        use_namespace: true,
        prob_namespace: 0.0,
        max_tables: 1,
        max_enums: 0,
        max_structs: 0,
        max_unions: 0,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        assert!(
            !out.contains("namespace "),
            "seed {seed}: prob_namespace=0.0 should never emit namespace\n{out}"
        );
    }
}

#[test]
fn namespace_disabled_never_emits() {
    let config = GenConfig {
        use_namespace: false,
        prob_namespace: 1.0,
        max_tables: 1,
        max_enums: 0,
        max_structs: 0,
        max_unions: 0,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..10 {
        let out = SchemaBuilder::generate(seed, config.clone());
        assert!(!out.contains("namespace "), "seed {seed}");
    }
}

#[test]
fn multi_namespace_emits_multiple_sections() {
    let config = GenConfig {
        use_namespace: true,
        prob_namespace: 1.0,
        prob_multi_namespace: 1.0,
        max_namespaces: 3,
        max_tables: 3,
        max_enums: 2,
        max_structs: 1,
        max_unions: 0,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        let ns_count = out.matches("namespace ").count();
        assert!(
            ns_count >= 2,
            "seed {seed}: expected 2+ namespace sections, got {ns_count}\n{out}"
        );
    }
}

#[test]
fn multi_namespace_single_when_prob_0() {
    let config = GenConfig {
        use_namespace: true,
        prob_namespace: 1.0,
        prob_multi_namespace: 0.0,
        max_namespaces: 3,
        max_tables: 1,
        max_enums: 0,
        max_structs: 0,
        max_unions: 0,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        let ns_count = out.matches("namespace ").count();
        assert_eq!(
            ns_count, 1,
            "seed {seed}: prob_multi_namespace=0 should emit exactly 1 namespace\n{out}"
        );
    }
}

#[test]
fn multi_namespace_has_cross_references() {
    let config = GenConfig {
        use_namespace: true,
        prob_namespace: 1.0,
        prob_multi_namespace: 1.0,
        max_namespaces: 2,
        max_tables: 4,
        max_enums: 1,
        max_structs: 0,
        max_unions: 0,
        max_fields_per_type: 4,
        weight_scalar: 5,
        weight_string: 0,
        weight_enum: 5,
        weight_struct: 0,
        weight_table_ref: 40,
        weight_vector: 20,
        weight_union: 0,
        use_file_ident: false,
        prob_required: 0.0,
        prob_deprecated: 0.0,
        prob_key: 0.0,
        prob_default_value: 0.0,
        prob_field_ids: 0.0,
        ..GenConfig::default()
    };
    let mut found_cross_ref = false;
    for seed in 0..50 {
        let out = SchemaBuilder::generate(seed, config.clone());
        for line in out.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("field_") && trimmed.contains(": ") {
                let colon = trimmed.find(':').unwrap();
                let semi = trimmed.find(';').unwrap();
                let ftype = trimmed[colon + 1..semi].trim();
                let inner = ftype.trim_start_matches('[').trim_end_matches(']');
                if inner.contains('.') {
                    found_cross_ref = true;
                }
            }
        }
    }
    assert!(
        found_cross_ref,
        "expected at least one cross-namespace reference across 50 seeds"
    );
}

// -- File identifier --

#[test]
fn file_ident_always_at_prob_1() {
    let config = GenConfig {
        use_file_ident: true,
        prob_file_ident: 1.0,
        max_tables: 1,
        max_enums: 0,
        max_structs: 0,
        max_unions: 0,
        use_namespace: false,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        assert!(
            out.contains("file_identifier "),
            "seed {seed}: prob_file_ident=1.0 should always emit\n{out}"
        );
    }
}

#[test]
fn file_ident_never_at_prob_0() {
    let config = GenConfig {
        use_file_ident: true,
        prob_file_ident: 0.0,
        max_tables: 1,
        max_enums: 0,
        max_structs: 0,
        max_unions: 0,
        use_namespace: false,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        assert!(!out.contains("file_identifier "), "seed {seed}");
    }
}

#[test]
fn file_ident_is_4_chars() {
    let config = GenConfig {
        use_file_ident: true,
        prob_file_ident: 1.0,
        max_tables: 1,
        max_enums: 0,
        max_structs: 0,
        max_unions: 0,
        use_namespace: false,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        let start = out.find("file_identifier \"").unwrap() + "file_identifier \"".len();
        let end = out[start..].find('"').unwrap();
        assert_eq!(end, 4, "seed {seed}: file_identifier must be 4 chars");
    }
}

// -- Field IDs --

#[test]
fn field_ids_always_at_prob_1() {
    let config = GenConfig {
        prob_field_ids: 1.0,
        max_tables: 3,
        max_enums: 0,
        max_structs: 0,
        max_unions: 0,
        use_namespace: false,
        use_file_ident: false,
        prob_required: 0.0,
        prob_deprecated: 0.0,
        prob_key: 0.0,
        prob_default_value: 0.0,
        prob_rpc_service: 0.0,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        for line in out.lines() {
            let trimmed = line.trim();
            if trimmed.ends_with(';') && trimmed.contains(':') && !trimmed.starts_with("//") {
                assert!(
                    trimmed.contains("id: "),
                    "seed {seed}: field should have id when prob=1.0: {trimmed}\n{out}"
                );
            }
        }
    }
}

#[test]
fn field_ids_never_at_prob_0() {
    let config = GenConfig {
        prob_field_ids: 0.0,
        max_tables: 3,
        max_enums: 0,
        max_structs: 0,
        max_unions: 0,
        use_namespace: false,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        assert!(
            !out.contains("id: "),
            "seed {seed}: no field IDs expected\n{out}"
        );
    }
}

// -- Attributes --

#[test]
fn required_never_at_prob_0() {
    let config = GenConfig {
        prob_required: 0.0,
        max_tables: 3,
        max_enums: 0,
        max_structs: 0,
        max_unions: 0,
        use_namespace: false,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..30 {
        let out = SchemaBuilder::generate(seed, config.clone());
        assert!(!out.contains("required"), "seed {seed}\n{out}");
    }
}

#[test]
fn deprecated_never_at_prob_0() {
    let config = GenConfig {
        prob_deprecated: 0.0,
        prob_required: 0.0,
        max_tables: 3,
        max_enums: 0,
        max_structs: 0,
        max_unions: 0,
        use_namespace: false,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..30 {
        let out = SchemaBuilder::generate(seed, config.clone());
        assert!(!out.contains("deprecated"), "seed {seed}\n{out}");
    }
}

#[test]
fn key_never_at_prob_0() {
    let config = GenConfig {
        prob_key: 0.0,
        max_tables: 3,
        max_enums: 0,
        max_structs: 0,
        max_unions: 0,
        use_namespace: false,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..30 {
        let out = SchemaBuilder::generate(seed, config.clone());
        assert!(!out.contains("key"), "seed {seed}\n{out}");
    }
}

// -- Default values --

#[test]
fn defaults_never_at_prob_0() {
    let config = GenConfig {
        prob_default_value: 0.0,
        max_tables: 3,
        max_enums: 2,
        max_structs: 0,
        max_unions: 0,
        use_namespace: false,
        use_file_ident: false,
        prob_required: 0.0,
        prob_deprecated: 0.0,
        prob_key: 0.0,
        ..GenConfig::default()
    };
    for seed in 0..30 {
        let out = SchemaBuilder::generate(seed, config.clone());
        let mut in_table = false;
        for line in out.lines() {
            if line.starts_with("table ") {
                in_table = true;
            } else if line == "}" {
                in_table = false;
            } else if in_table && line.trim().ends_with(';') {
                assert!(
                    !line.contains(" = "),
                    "seed {seed}: no defaults expected: {line}\n{out}"
                );
            }
        }
    }
}

// -- Fixed arrays --

#[test]
fn fixed_arrays_never_at_prob_0() {
    let config = GenConfig {
        prob_fixed_array: 0.0,
        max_structs: 3,
        max_enums: 0,
        max_tables: 1,
        max_unions: 0,
        max_fields_per_type: 6,
        use_namespace: false,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..30 {
        let out = SchemaBuilder::generate(seed, config.clone());
        let mut in_struct = false;
        for line in out.lines() {
            if line.starts_with("struct ") {
                in_struct = true;
            } else if line == "}" {
                in_struct = false;
            } else if in_struct && line.trim().ends_with(';') {
                assert!(
                    !line.contains(":]"),
                    "seed {seed}: no fixed arrays expected: {line}\n{out}"
                );
            }
        }
    }
}

#[test]
fn fixed_arrays_disabled_when_max_len_0() {
    let config = GenConfig {
        prob_fixed_array: 1.0,
        max_fixed_array_len: 0,
        max_structs: 3,
        max_enums: 0,
        max_tables: 1,
        max_unions: 0,
        use_namespace: false,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        let mut in_struct = false;
        for line in out.lines() {
            if line.starts_with("struct ") {
                in_struct = true;
            } else if line == "}" {
                in_struct = false;
            } else if in_struct && line.trim().ends_with(';') {
                assert!(
                    !line.contains(":]"),
                    "seed {seed}: max_len=0 should disable arrays: {line}\n{out}"
                );
            }
        }
    }
}

// -- Table field type weights --

#[test]
fn weight_only_scalars() {
    let config = GenConfig {
        weight_scalar: 100,
        weight_string: 0,
        weight_enum: 0,
        weight_struct: 0,
        weight_table_ref: 0,
        weight_vector: 0,
        weight_union: 0,
        max_enums: 2,
        max_structs: 2,
        max_tables: 3,
        max_unions: 0,
        max_fields_per_type: 4,
        use_namespace: false,
        use_file_ident: false,
        prob_required: 0.0,
        prob_deprecated: 0.0,
        prob_key: 0.0,
        prob_default_value: 0.0,
        prob_field_ids: 0.0,
        prob_type_alias: 0.0,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        let mut in_main_table = false;
        let mut table_count = 0;
        for line in out.lines() {
            if line.starts_with("table ") {
                table_count += 1;
                in_main_table = true;
            } else if line == "}" {
                in_main_table = false;
            } else if in_main_table && line.trim().ends_with(';') {
                let field_line = line.trim();
                let colon = field_line.find(':').unwrap();
                let semi = field_line.find(';').unwrap();
                let ftype = field_line[colon + 1..semi].trim();
                assert!(
                    ALL_SCALAR_TYPES.contains(&ftype),
                    "seed {seed}: expected scalar, got '{ftype}'\n{out}"
                );
            }
        }
        assert!(table_count > 0, "seed {seed}: should have tables");
    }
}

#[test]
fn weight_only_strings() {
    let config = GenConfig {
        weight_scalar: 0,
        weight_string: 100,
        weight_enum: 0,
        weight_struct: 0,
        weight_table_ref: 0,
        weight_vector: 0,
        weight_union: 0,
        max_enums: 0,
        max_structs: 0,
        max_tables: 2,
        max_unions: 0,
        max_fields_per_type: 3,
        use_namespace: false,
        use_file_ident: false,
        prob_required: 0.0,
        prob_deprecated: 0.0,
        prob_key: 0.0,
        prob_default_value: 0.0,
        prob_field_ids: 0.0,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        let mut in_table = false;
        for line in out.lines() {
            if line.starts_with("table ") {
                in_table = true;
            } else if line == "}" {
                in_table = false;
            } else if in_table && line.trim().ends_with(';') {
                assert!(
                    line.contains(": string;"),
                    "seed {seed}: expected string field, got: {line}\n{out}"
                );
            }
        }
    }
}

#[test]
fn weight_only_vectors() {
    let config = GenConfig {
        weight_scalar: 0,
        weight_string: 0,
        weight_enum: 0,
        weight_struct: 0,
        weight_table_ref: 0,
        weight_vector: 100,
        weight_union: 0,
        max_enums: 0,
        max_structs: 0,
        max_tables: 2,
        max_unions: 0,
        max_fields_per_type: 3,
        use_namespace: false,
        use_file_ident: false,
        prob_required: 0.0,
        prob_deprecated: 0.0,
        prob_key: 0.0,
        prob_default_value: 0.0,
        prob_field_ids: 0.0,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        let mut in_table = false;
        for line in out.lines() {
            if line.starts_with("table ") {
                in_table = true;
            } else if line == "}" {
                in_table = false;
            } else if in_table && line.trim().ends_with(';') {
                assert!(
                    line.contains('[') && line.contains(']'),
                    "seed {seed}: expected vector field, got: {line}\n{out}"
                );
            }
        }
    }
}

// -- Root type --

#[test]
fn root_type_emitted_when_enabled() {
    let config = GenConfig {
        use_root_type: true,
        max_tables: 2,
        max_enums: 0,
        max_structs: 0,
        max_unions: 0,
        use_namespace: false,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        assert!(out.contains("root_type "), "seed {seed}\n{out}");
    }
}

#[test]
fn root_type_not_emitted_when_disabled() {
    let config = GenConfig {
        use_root_type: false,
        max_tables: 2,
        max_enums: 0,
        max_structs: 0,
        max_unions: 0,
        use_namespace: false,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, config.clone());
        assert!(!out.contains("root_type "), "seed {seed}");
    }
}

// -- Name uniqueness --

#[test]
fn no_duplicate_type_names() {
    let config = GenConfig {
        max_enums: 4,
        max_structs: 3,
        max_tables: 5,
        max_unions: 2,
        ..GenConfig::default()
    };
    for seed in 0..50 {
        let out = SchemaBuilder::generate(seed, config.clone());
        let mut names: Vec<String> = Vec::new();
        for line in out.lines() {
            for prefix in &["enum ", "struct ", "table ", "union "] {
                if let Some(rest) = line.strip_prefix(prefix) {
                    let name = rest
                        .split(|c: char| !c.is_alphanumeric())
                        .next()
                        .unwrap_or("");
                    assert!(
                        !names.contains(&name.to_string()),
                        "seed {seed}: duplicate type name '{name}'\n{out}"
                    );
                    names.push(name.to_string());
                }
            }
        }
    }
}

// -- Struct field validity --

#[test]
fn struct_fields_are_valid_types() {
    let config = GenConfig {
        max_enums: 2,
        max_structs: 3,
        max_tables: 1,
        max_unions: 0,
        max_fields_per_type: 4,
        use_namespace: false,
        use_file_ident: false,
        ..GenConfig::default()
    };
    for seed in 0..30 {
        let out = SchemaBuilder::generate(seed, config.clone());
        let mut known_types: HashSet<String> =
            ALL_SCALAR_TYPES.iter().map(|s| s.to_string()).collect();
        let mut in_struct = false;
        for line in out.lines() {
            if line.starts_with("enum ") {
                let name = line.split_whitespace().nth(1).unwrap();
                known_types.insert(name.to_string());
            } else if line.starts_with("struct ") {
                let name = line.split_whitespace().nth(1).unwrap();
                in_struct = true;
                known_types.insert(name.to_string());
            } else if line == "}" {
                in_struct = false;
            } else if in_struct && line.trim().ends_with(';') {
                let trimmed = line.trim();
                let colon = trimmed.find(':').unwrap();
                let semi = trimmed.find(';').unwrap();
                let ftype = trimmed[colon + 1..semi].trim();
                let base_type = if ftype.starts_with('[') && ftype.contains(':') {
                    let inner = &ftype[1..ftype.len() - 1];
                    inner.split(':').next().unwrap()
                } else {
                    ftype
                };
                assert!(
                    known_types.contains(base_type),
                    "seed {seed}: struct field type '{base_type}' not in known types\n{out}"
                );
            }
        }
    }
}

// -- Output non-empty --

#[test]
fn output_is_nonempty() {
    for seed in 0..20 {
        let out = SchemaBuilder::generate(seed, GenConfig::default());
        assert!(!out.is_empty(), "seed {seed} produced empty output");
    }
}

// -- Feature coverage --

#[test]
fn generates_table_references() {
    let config = GenConfig::default();
    let mut found = false;
    for seed in 0..100 {
        let out = SchemaBuilder::generate(seed, config.clone());
        for line in out.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("field_") && trimmed.contains(": ") {
                let colon = trimmed.find(':').unwrap();
                let semi = trimmed.find(';').unwrap_or(trimmed.len());
                let ftype = trimmed[colon + 1..semi]
                    .split_whitespace()
                    .next()
                    .unwrap_or("");
                if ftype.contains("Table") && !ftype.starts_with('[') {
                    found = true;
                }
            }
        }
    }
    assert!(
        found,
        "expected at least one table reference across 100 schemas"
    );
}

#[test]
fn generates_vec_of_tables() {
    let config = GenConfig::default();
    let mut found = false;
    for seed in 0..100 {
        let out = SchemaBuilder::generate(seed, config.clone());
        for line in out.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("field_") && trimmed.contains("[") {
                let bracket_start = trimmed.find('[').unwrap();
                let bracket_end = trimmed.find(']').unwrap();
                let inner = &trimmed[bracket_start + 1..bracket_end];
                if inner.contains("Table") {
                    found = true;
                }
            }
        }
    }
    assert!(
        found,
        "expected at least one [Table] vector across 100 schemas"
    );
}

#[test]
fn generates_namespaces() {
    let config = GenConfig::default();
    let mut found = false;
    for seed in 0..100 {
        let out = SchemaBuilder::generate(seed, config.clone());
        if out.contains("namespace ") {
            found = true;
        }
    }
    assert!(
        found,
        "expected at least one schema with namespace across 100 schemas"
    );
}

// =========================================================================
// ScriptedChooser-based branch tests
//
// Each test uses `with_defaults()` so only the decisions being tested need
// explicit scripting. Defaults: flip->false, pick->0, weighted->0.
// =========================================================================

/// Helper: minimal config that generates no enums/structs/unions and 1 table.
fn minimal_config() -> GenConfig {
    GenConfig {
        max_enums: 0,
        max_structs: 0,
        max_tables: 1,
        max_unions: 0,
        max_fields_per_type: 1,
        use_namespace: false,
        use_root_type: false,
        use_file_ident: false,
        prob_doc_comment: 0.0,
        prob_file_extension: 0.0,
        prob_rpc_service: 0.0,
        prob_required: 0.0,
        prob_deprecated: 0.0,
        prob_key: 0.0,
        prob_default_value: 0.0,
        prob_field_ids: 0.0,
        ..GenConfig::default()
    }
}

// -- Enum branch tests --

#[test]
fn scripted_enum_bit_flags_on() {
    let config = GenConfig {
        max_enums: 1,
        ..minimal_config()
    };
    // Picks: [1] = 1 enum (rest default to 0, clamped appropriately)
    // Flips: type_alias=false, bit_flags=TRUE (rest default to false)
    let chooser = ScriptedChooser::new()
        .with_defaults()
        .with_picks([1])
        .with_flips([false, true]);

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(out.contains("(bit_flags)"), "Expected bit_flags:\n{out}");
    assert!(
        out.contains("enum EnumAlpha : byte (bit_flags)"),
        "Expected enum declaration:\n{out}"
    );
}

#[test]
fn scripted_enum_no_bit_flags() {
    let config = GenConfig {
        max_enums: 1,
        ..minimal_config()
    };
    // Picks: enum_count=1, underlying=short(idx 2) -- interleaved after enum count
    let chooser = ScriptedChooser::new().with_defaults().with_picks([1, 2]); // 1 enum, underlying=short(SCALAR_INT_TYPES[2])

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(
        !out.contains("bit_flags"),
        "Should not have bit_flags:\n{out}"
    );
    assert!(
        out.contains("enum EnumAlpha : short"),
        "Expected short enum:\n{out}"
    );
}

// -- Struct branch tests --

#[test]
fn scripted_struct_force_align() {
    let config = GenConfig {
        max_structs: 1,
        ..minimal_config()
    };
    // Picks: enum=0, struct=1, then interleaved: fields(->1), type(->bool), alignment=2(->8)
    // Flips: fixed_array=false, doc_comment=false, force_align=TRUE
    let chooser = ScriptedChooser::new()
        .with_defaults()
        .with_picks([0, 1, 0, 0, 2]) // enum=0, struct=1, fields, type, alignment=2(->8)
        .with_flips([false, false, true]); // fixed_array=no, doc=no, force_align=YES

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(
        out.contains("(force_align: 8)"),
        "Expected force_align: 8:\n{out}"
    );
}

#[test]
fn scripted_struct_fixed_array() {
    let config = GenConfig {
        max_structs: 1,
        max_fixed_array_len: 4,
        ..minimal_config()
    };
    // Picks: enum=0, struct=1, then interleaved: fields(->1), type=int(5), array_len=3
    // Flips: fixed_array=TRUE
    let chooser = ScriptedChooser::new()
        .with_defaults()
        .with_picks([0, 1, 0, 5, 3]) // enum=0, struct=1, fields, type=int(5), len=3
        .with_flips([true]); // fixed_array=YES

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(
        out.contains("[int:3]"),
        "Expected [int:3] fixed array:\n{out}"
    );
}

// -- Table field type weighted selection --

#[test]
fn scripted_table_field_string() {
    let config = minimal_config();
    // Weighted: string bucket (category 1)
    let chooser = ScriptedChooser::new().with_defaults().with_weighted([1]); // STRING

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(out.contains(": string;"), "Expected string field:\n{out}");
}

#[test]
fn scripted_table_field_vector() {
    let config = minimal_config();
    // Weighted: vector bucket (5), then scalar element (0)
    let chooser = ScriptedChooser::new().with_defaults().with_weighted([5, 0]); // VECTOR, then scalar element

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(
        out.contains("[bool]"),
        "Expected [bool] vector field:\n{out}"
    );
}

// -- Default value branch tests --

#[test]
fn scripted_default_null() {
    let config = minimal_config();
    // Flips: field_ids=no, doc=no, deprecated=no, key=no, default=YES, null=YES
    let chooser = ScriptedChooser::new()
        .with_defaults()
        .with_flips([false, false, false, false, true, true]);

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(out.contains("= null"), "Expected '= null' default:\n{out}");
}

#[test]
fn scripted_default_bool_true() {
    let config = minimal_config();
    // Flips: field_ids=no, doc=no, deprecated=no, key=no, default=YES, null=no, bool=true
    let chooser = ScriptedChooser::new()
        .with_defaults()
        .with_flips([false, false, false, false, true, false, true]);

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(out.contains("= true"), "Expected '= true' default:\n{out}");
}

#[test]
fn scripted_default_float_nan() {
    let config = minimal_config();
    // Need float field type: ALL_SCALAR_TYPES[9] = "float"
    // Picks: pad to position 5 for scalar type
    // Flips: field_ids=no, doc=no, deprecated=no, key=no, default=YES, null=no, nan=YES
    let chooser = ScriptedChooser::new()
        .with_defaults()
        .with_picks([0, 0, 0, 0, 0, 9]) // field type = float
        .with_flips([false, false, false, false, true, false, true]);

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(out.contains("= nan"), "Expected '= nan' default:\n{out}");
}

// -- Attribute branch tests --

#[test]
fn scripted_field_required() {
    let config = minimal_config();
    // String field (ref type) with required
    // Flips: field_ids=no, doc=no, required=YES
    let chooser = ScriptedChooser::new()
        .with_defaults()
        .with_weighted([1]) // string
        .with_flips([false, false, true]); // required=YES

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(
        out.contains("(required)"),
        "Expected (required) attribute:\n{out}"
    );
}

#[test]
fn scripted_field_deprecated() {
    let config = minimal_config();
    // Scalar field (no required flip), then deprecated
    // Flips: field_ids=no, doc=no, deprecated=YES
    let chooser = ScriptedChooser::new()
        .with_defaults()
        .with_flips([false, false, true]); // deprecated=YES

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(
        out.contains("(deprecated)"),
        "Expected (deprecated) attribute:\n{out}"
    );
}

#[test]
fn scripted_field_key() {
    let config = minimal_config();
    // Scalar field, not deprecated, then key
    // Flips: field_ids=no, doc=no, deprecated=no, key=YES
    let chooser = ScriptedChooser::new()
        .with_defaults()
        .with_flips([false, false, false, true]); // key=YES

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(out.contains("(key)"), "Expected (key) attribute:\n{out}");
}

// -- File identifier branch test --

#[test]
fn scripted_file_identifier() {
    let config = GenConfig {
        use_file_ident: true,
        prob_file_ident: 1.0,
        use_root_type: true,
        ..minimal_config()
    };
    // Flips: table stuff all false (6 flips), then file_ident=YES
    // Picks: table stuff defaults, root_type idx, then 4 char picks
    let chooser = ScriptedChooser::new()
        .with_defaults()
        .with_picks([
            0,
            0,
            0,
            0,
            0,
            0, // section + field + scalar
            0, // root_type index
            b'T' as usize,
            b'E' as usize,
            b'S' as usize,
            b'T' as usize, // "TEST"
        ])
        .with_flips([
            false, false, false, false, false, false, // table generation
            true,  // file_ident = YES
        ]);

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(
        out.contains("file_identifier \"TEST\""),
        "Expected file_identifier \"TEST\":\n{out}"
    );
}

// -- RPC service branch test --

#[test]
fn scripted_rpc_service() {
    let config = GenConfig {
        prob_rpc_service: 1.0,
        use_root_type: false,
        use_file_ident: false,
        ..minimal_config()
    };
    // Flips: table stuff (6), file_ext=no, rpc=YES, then rpc internals
    let chooser = ScriptedChooser::new().with_defaults().with_flips([
        false, false, false, false, false, false, // table generation
        false, // file_extension
        true,  // rpc_service = YES
    ]);

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(
        out.contains("rpc_service"),
        "Expected rpc_service block:\n{out}"
    );
    assert!(out.contains("Get("), "Expected Get method:\n{out}");
}

#[test]
fn scripted_rpc_streaming() {
    let config = GenConfig {
        prob_rpc_service: 1.0,
        use_root_type: false,
        use_file_ident: false,
        ..minimal_config()
    };
    // Flips: table(6), file_ext=no, rpc=YES, service_doc=no, streaming=YES
    // Picks: table defaults, then rpc: num_methods, req, resp, streaming_mode
    let chooser = ScriptedChooser::new()
        .with_defaults()
        .with_picks([
            0, 0, 0, 0, 0, 0, // section + field + scalar
            0, 0, 0, // rpc: num_methods(clamped to 1), req, resp
            1, // streaming_modes[1] = "server"
        ])
        .with_flips([
            false, false, false, false, false, false, // table generation
            false, // file_extension
            true,  // rpc_service = YES
            false, // service doc
            true,  // streaming = YES
        ]);

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(
        out.contains("streaming: \"server\""),
        "Expected streaming: \"server\":\n{out}"
    );
}

// -- Doc comment branch test --

#[test]
fn scripted_doc_comment_on_table() {
    let config = minimal_config();
    // Flips: field_ids=no, doc_field=YES, deprecated=no, key=no, default=no, doc_table=YES
    let chooser = ScriptedChooser::new()
        .with_defaults()
        .with_flips([false, true, false, false, false, true]);

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(
        out.contains("/// Table"),
        "Expected doc comment on table:\n{out}"
    );
    assert!(
        out.contains("/// Field"),
        "Expected doc comment on field:\n{out}"
    );
}

// -- Type alias branch test --

#[test]
fn scripted_enum_type_alias() {
    let config = GenConfig {
        max_enums: 1,
        ..minimal_config()
    };
    // Flips: type_alias=YES (first flip in gen_and_emit_enum)
    let chooser = ScriptedChooser::new()
        .with_defaults()
        .with_picks([1]) // 1 enum
        .with_flips([true]); // type_alias = YES -> byte becomes int8

    let out = SchemaBuilder::with_chooser(chooser, config).build();
    assert!(
        out.contains("enum EnumAlpha : int8"),
        "Expected int8 type alias:\n{out}"
    );
}
