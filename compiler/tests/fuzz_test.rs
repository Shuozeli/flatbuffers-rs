/// Quickcheck-based property tests that verify generated FlatBuffer code
/// correctly round-trips random inputs through build -> serialize -> deserialize.

#[allow(unused_imports, dead_code, non_camel_case_types, non_snake_case)]
mod fuzz_struct {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/struct_simple.expected");
}

#[allow(unused_imports, dead_code, non_camel_case_types, non_snake_case)]
mod fuzz_table_scalar {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/table_scalar_fields.expected");
}

#[allow(unused_imports, dead_code, non_camel_case_types, non_snake_case)]
mod fuzz_table_string {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/table_string_field.expected");
}

#[allow(unused_imports, dead_code, non_camel_case_types, non_snake_case)]
mod fuzz_table_vector {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/table_vector_fields.expected");
}

#[allow(unused_imports, dead_code, non_upper_case_globals, non_camel_case_types, non_snake_case)]
mod fuzz_table_enum {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/table_enum_field.expected");
}

#[allow(unused_imports, dead_code, non_upper_case_globals, non_camel_case_types)]
mod fuzz_struct_key {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/struct_key.expected");
}

#[allow(unused_imports, dead_code, non_upper_case_globals, non_camel_case_types)]
mod fuzz_struct_array {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/struct_array.expected");
}

// ==========================================================================
// Struct round-trip fuzz
// ==========================================================================

#[cfg(not(miri))]
mod struct_fuzz {
    use quickcheck::quickcheck;

    quickcheck! {
        fn vec3_roundtrip(x: f32, y: f32, z: f32) -> bool {
            if x.is_nan() || y.is_nan() || z.is_nan() { return true; }
            let v = super::fuzz_struct::Vec3::new(x, y, z);
            v.x() == x && v.y() == y && v.z() == z
        }
    }

    quickcheck! {
        fn vec3_set_roundtrip(x: f32, y: f32, z: f32) -> bool {
            if x.is_nan() || y.is_nan() || z.is_nan() { return true; }
            let mut v = super::fuzz_struct::Vec3::default();
            v.set_x(x);
            v.set_y(y);
            v.set_z(z);
            v.x() == x && v.y() == y && v.z() == z
        }
    }
}

// ==========================================================================
// Struct key comparison consistency fuzz
// ==========================================================================

#[cfg(not(miri))]
mod struct_key_fuzz {
    use quickcheck::quickcheck;

    quickcheck! {
        fn key_compare_consistency(id_a: u32, id_b: u32) -> bool {
            let a = super::fuzz_struct_key::Ability::new(id_a, 0);
            let b = super::fuzz_struct_key::Ability::new(id_b, 0);
            let less = a.key_compare_less_than(&b);
            let cmp = a.key_compare_with_value(id_b);
            match cmp {
                ::core::cmp::Ordering::Less => less,
                ::core::cmp::Ordering::Equal => !less,
                ::core::cmp::Ordering::Greater => !less,
            }
        }
    }

    quickcheck! {
        fn key_compare_transitivity(a_id: u32, b_id: u32, c_id: u32) -> bool {
            let a = super::fuzz_struct_key::Ability::new(a_id, 0);
            let b = super::fuzz_struct_key::Ability::new(b_id, 0);
            let c = super::fuzz_struct_key::Ability::new(c_id, 0);
            if a.key_compare_less_than(&b) && b.key_compare_less_than(&c) {
                a.key_compare_less_than(&c)
            } else {
                true
            }
        }
    }
}

// ==========================================================================
// Struct array round-trip fuzz
// ==========================================================================

#[cfg(not(miri))]
mod struct_array_fuzz {
    use quickcheck::quickcheck;

    quickcheck! {
        fn inner_struct_roundtrip(x: i32, y: i32) -> bool {
            let s = super::fuzz_struct_array::Inner::new(x, y);
            s.x() == x && s.y() == y
        }
    }
}

// ==========================================================================
// Table scalar round-trip fuzz
// ==========================================================================

#[cfg(not(miri))]
mod table_scalar_fuzz {
    use quickcheck::quickcheck;

    quickcheck! {
        fn scalar_roundtrip(hp: i32, mana: i16, speed: f32, active: bool) -> bool {
            if speed.is_nan() { return true; }
            let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
            let args = super::fuzz_table_scalar::StatsArgs { hp, mana, speed, active };
            let offset = super::fuzz_table_scalar::createStats(&mut fbb, &args);
            fbb.finish_minimal(offset);
            let buf = fbb.finished_data();
            let stats = ::flatbuffers::root::<super::fuzz_table_scalar::Stats>(buf).unwrap();
            stats.hp() == hp && stats.mana() == mana && stats.speed() == speed && stats.active() == active
        }
    }
}

// ==========================================================================
// Table string round-trip fuzz
// ==========================================================================

#[cfg(not(miri))]
mod table_string_fuzz {
    use quickcheck::quickcheck;

    quickcheck! {
        fn string_roundtrip(name: String) -> bool {
            let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
            let name_off = fbb.create_string(&name);
            let args = super::fuzz_table_string::GreetingArgs {
                message: None,
                name: Some(name_off),
            };
            let offset = super::fuzz_table_string::createGreeting(&mut fbb, &args);
            fbb.finish_minimal(offset);
            let buf = fbb.finished_data();
            let greeting = ::flatbuffers::root::<super::fuzz_table_string::Greeting>(buf).unwrap();
            greeting.name() == name.as_str()
        }
    }

    quickcheck! {
        fn two_strings_roundtrip(msg: String, name: String) -> bool {
            let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
            let msg_off = fbb.create_string(&msg);
            let name_off = fbb.create_string(&name);
            let args = super::fuzz_table_string::GreetingArgs {
                message: Some(msg_off),
                name: Some(name_off),
            };
            let offset = super::fuzz_table_string::createGreeting(&mut fbb, &args);
            fbb.finish_minimal(offset);
            let buf = fbb.finished_data();
            let greeting = ::flatbuffers::root::<super::fuzz_table_string::Greeting>(buf).unwrap();
            greeting.name() == name.as_str() && greeting.message() == Some(msg.as_str())
        }
    }
}

// ==========================================================================
// Table vector round-trip fuzz
// ==========================================================================

#[cfg(not(miri))]
mod table_vector_fuzz {
    use quickcheck::quickcheck;

    quickcheck! {
        fn ubyte_vector_roundtrip(data: Vec<u8>) -> bool {
            let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
            let slots = fbb.create_vector(&data);
            let args = super::fuzz_table_vector::InventoryArgs {
                slots: Some(slots),
                names: None,
                scores: None,
            };
            let offset = super::fuzz_table_vector::createInventory(&mut fbb, &args);
            fbb.finish_minimal(offset);
            let buf = fbb.finished_data();
            let inv = ::flatbuffers::root::<super::fuzz_table_vector::Inventory>(buf).unwrap();
            let s = inv.slots().unwrap();
            s.len() == data.len() && (0..data.len()).all(|i| s.get(i) == data[i])
        }
    }

    quickcheck! {
        fn int_vector_roundtrip(data: Vec<i32>) -> bool {
            let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
            let scores = fbb.create_vector(&data);
            let args = super::fuzz_table_vector::InventoryArgs {
                slots: None,
                names: None,
                scores: Some(scores),
            };
            let offset = super::fuzz_table_vector::createInventory(&mut fbb, &args);
            fbb.finish_minimal(offset);
            let buf = fbb.finished_data();
            let inv = ::flatbuffers::root::<super::fuzz_table_vector::Inventory>(buf).unwrap();
            let sc = inv.scores().unwrap();
            sc.len() == data.len() && (0..data.len()).all(|i| sc.get(i) == data[i])
        }
    }

    quickcheck! {
        fn string_vector_roundtrip(data: Vec<String>) -> bool {
            let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
            let str_offsets: Vec<_> = data.iter()
                .map(|s| fbb.create_string(s))
                .collect();
            let names = fbb.create_vector(&str_offsets);
            let args = super::fuzz_table_vector::InventoryArgs {
                slots: None,
                names: Some(names),
                scores: None,
            };
            let offset = super::fuzz_table_vector::createInventory(&mut fbb, &args);
            fbb.finish_minimal(offset);
            let buf = fbb.finished_data();
            let inv = ::flatbuffers::root::<super::fuzz_table_vector::Inventory>(buf).unwrap();
            let n = inv.names().unwrap();
            n.len() == data.len() && (0..data.len()).all(|i| n.get(i) == data[i].as_str())
        }
    }
}

// ==========================================================================
// Table enum round-trip fuzz
// ==========================================================================

#[cfg(not(miri))]
mod table_enum_fuzz {
    use quickcheck::quickcheck;

    quickcheck! {
        fn enum_field_roundtrip(color_raw: i8, damage: i16) -> bool {
            let color = super::fuzz_table_enum::Color(color_raw);
            let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
            let args = super::fuzz_table_enum::WeaponArgs {
                name: None,
                color,
                damage,
            };
            let offset = super::fuzz_table_enum::createWeapon(&mut fbb, &args);
            fbb.finish_minimal(offset);
            let buf = fbb.finished_data();
            let weapon = ::flatbuffers::root::<super::fuzz_table_enum::Weapon>(buf).unwrap();
            weapon.color().0 == color_raw && weapon.damage() == damage
        }
    }
}

// ==========================================================================
// Verifier fuzz: random byte mutations never panic
// ==========================================================================

#[cfg(not(miri))]
mod verifier_fuzz {
    use quickcheck::quickcheck;

    quickcheck! {
        fn random_bytes_never_panic(data: Vec<u8>) -> bool {
            let _ = ::flatbuffers::root::<super::fuzz_table_scalar::Stats>(&data);
            true
        }
    }

    quickcheck! {
        fn corrupted_scalar_table_never_panic(hp: i32, flip_pos: usize, flip_val: u8) -> bool {
            let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
            let args = super::fuzz_table_scalar::StatsArgs { hp, ..Default::default() };
            let offset = super::fuzz_table_scalar::createStats(&mut fbb, &args);
            fbb.finish_minimal(offset);
            let mut buf = fbb.finished_data().to_vec();
            if !buf.is_empty() {
                let len = buf.len();
                buf[flip_pos % len] ^= flip_val;
            }
            let _ = ::flatbuffers::root::<super::fuzz_table_scalar::Stats>(&buf);
            true
        }
    }
}
