/// Integration tests that verify generated Rust code compiles against the
/// flatbuffers runtime crate and produces functionally correct results.
///
/// Each test module includes a pre-generated `.expected` file from the codegen
/// golden tests. If it compiles and the runtime operations work, the generated
/// code is correct.

// ==========================================================================
// Enum: build + read + debug
// ==========================================================================

#[allow(
    unused_imports,
    dead_code,
    non_upper_case_globals,
    non_camel_case_types
)]
mod enum_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/enum_basic.expected");
}

#[test]
fn enum_variant_values() {
    use enum_runtime::*;
    assert_eq!(Color::Red.0, 1);
    assert_eq!(Color::Green.0, 2);
    assert_eq!(Color::Blue.0, 8);
}

#[test]
fn enum_min_max() {
    use enum_runtime::*;
    assert_eq!(Color::ENUM_MIN, 1);
    assert_eq!(Color::ENUM_MAX, 8);
}

#[test]
fn enum_variant_name() {
    use enum_runtime::*;
    assert_eq!(Color::Red.variant_name(), Some("Red"));
    assert_eq!(Color::Green.variant_name(), Some("Green"));
    assert_eq!(Color::Blue.variant_name(), Some("Blue"));
    assert_eq!(Color(99).variant_name(), None);
}

#[test]
fn enum_debug() {
    use enum_runtime::*;
    assert_eq!(format!("{:?}", Color::Red), "Red");
    assert_eq!(format!("{:?}", Color(99)), "<UNKNOWN 99>");
}

#[test]
fn enum_default() {
    use enum_runtime::*;
    let c = Color::default();
    assert_eq!(c.0, 0);
}

// ==========================================================================
// Table with scalars: build + read + verify
// ==========================================================================

#[allow(unused_imports, dead_code, non_camel_case_types, non_snake_case)]
mod table_scalar_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/table_scalar_fields.expected");
}

#[test]
fn table_build_and_read_defaults() {
    use table_scalar_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = StatsArgs::default();
    let offset = createStats(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let stats = ::flatbuffers::root::<Stats>(buf).unwrap();
    assert_eq!(stats.hp(), 100);
    assert_eq!(stats.mana(), 50);
    assert!((stats.speed() - 1.5_f32).abs() < f32::EPSILON);
    assert!(stats.active());
}

#[test]
fn table_build_and_read_custom_values() {
    use table_scalar_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = StatsArgs {
        hp: 42,
        mana: -10,
        speed: 3.14_f32,
        active: false,
    };
    let offset = createStats(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let stats = ::flatbuffers::root::<Stats>(buf).unwrap();
    assert_eq!(stats.hp(), 42);
    assert_eq!(stats.mana(), -10);
    assert!((stats.speed() - 3.14_f32).abs() < f32::EPSILON);
    assert!(!stats.active());
}

#[test]
fn table_verify_valid() {
    use table_scalar_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let offset = createStats(&mut fbb, &StatsArgs::default());
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    assert!(::flatbuffers::root::<Stats>(buf).is_ok());
}

#[test]
fn table_debug_output() {
    use table_scalar_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let offset = createStats(&mut fbb, &StatsArgs::default());
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let stats = ::flatbuffers::root::<Stats>(buf).unwrap();
    let dbg = format!("{:?}", stats);
    assert!(dbg.contains("hp: 100"));
    assert!(dbg.contains("mana: 50"));
}

// ==========================================================================
// Table with string: build + read
// Schema: message (optional string), name (required string)
// ==========================================================================

#[allow(unused_imports, dead_code, non_camel_case_types, non_snake_case)]
mod table_string_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/table_string_field.expected");
}

#[test]
fn table_with_required_string() {
    use table_string_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let name_offset = fbb.create_string("World");
    let args = GreetingArgs {
        message: None,
        name: Some(name_offset),
    };
    let offset = createGreeting(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let greeting = ::flatbuffers::root::<Greeting>(buf).unwrap();
    assert_eq!(greeting.name(), "World");
    assert_eq!(greeting.message(), None);
}

#[test]
fn table_with_both_strings() {
    use table_string_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let msg = fbb.create_string("Hello");
    let name = fbb.create_string("World");
    let args = GreetingArgs {
        message: Some(msg),
        name: Some(name),
    };
    let offset = createGreeting(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let greeting = ::flatbuffers::root::<Greeting>(buf).unwrap();
    assert_eq!(greeting.name(), "World");
    assert_eq!(greeting.message(), Some("Hello"));
}

// ==========================================================================
// String builder: edge cases
// ==========================================================================

#[test]
fn string_empty_roundtrip() {
    use table_string_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("");
    let args = GreetingArgs {
        message: None,
        name: Some(name),
    };
    let offset = createGreeting(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let greeting = ::flatbuffers::root::<Greeting>(buf).unwrap();
    assert_eq!(greeting.name(), "");
}

#[test]
fn string_unicode_roundtrip() {
    use table_string_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("hello");
    let msg = fbb.create_string("CJK: \u{4e16}\u{754c}");
    let args = GreetingArgs {
        message: Some(msg),
        name: Some(name),
    };
    let offset = createGreeting(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let greeting = ::flatbuffers::root::<Greeting>(buf).unwrap();
    assert_eq!(greeting.message(), Some("CJK: \u{4e16}\u{754c}"));
}

#[test]
fn string_long_roundtrip() {
    use table_string_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let long_str: String = "x".repeat(10_000);
    let name = fbb.create_string(&long_str);
    let args = GreetingArgs {
        message: None,
        name: Some(name),
    };
    let offset = createGreeting(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let greeting = ::flatbuffers::root::<Greeting>(buf).unwrap();
    assert_eq!(greeting.name().len(), 10_000);
}

#[test]
fn string_vector_multiple_items() {
    use table_vector_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let strs: Vec<_> = (0..100)
        .map(|i| fbb.create_string(&format!("item_{i}")))
        .collect();
    let names = fbb.create_vector(&strs);
    let args = InventoryArgs {
        slots: None,
        names: Some(names),
        scores: None,
    };
    let offset = createInventory(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let inv = ::flatbuffers::root::<Inventory>(buf).unwrap();
    let n = inv.names().unwrap();
    assert_eq!(n.len(), 100);
    assert_eq!(n.get(0), "item_0");
    assert_eq!(n.get(99), "item_99");
}

// ==========================================================================
// Struct: new + read fields
// ==========================================================================

#[allow(unused_imports, dead_code, non_camel_case_types)]
mod struct_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/struct_simple.expected");
}

#[test]
fn struct_new_and_read() {
    use struct_runtime::*;
    let v = Vec3::new(1.0, 2.0, 3.0);
    assert!((v.x() - 1.0_f32).abs() < f32::EPSILON);
    assert!((v.y() - 2.0_f32).abs() < f32::EPSILON);
    assert!((v.z() - 3.0_f32).abs() < f32::EPSILON);
}

#[test]
fn struct_set_and_read() {
    use struct_runtime::*;
    let mut v = Vec3::new(0.0, 0.0, 0.0);
    v.set_x(10.0);
    v.set_y(20.0);
    v.set_z(30.0);
    assert!((v.x() - 10.0_f32).abs() < f32::EPSILON);
    assert!((v.y() - 20.0_f32).abs() < f32::EPSILON);
    assert!((v.z() - 30.0_f32).abs() < f32::EPSILON);
}

#[test]
fn struct_default_is_zero() {
    use struct_runtime::*;
    let v = Vec3::default();
    assert_eq!(v.x(), 0.0);
    assert_eq!(v.y(), 0.0);
    assert_eq!(v.z(), 0.0);
}

#[test]
fn struct_debug() {
    use struct_runtime::*;
    let v = Vec3::new(1.0, 2.0, 3.0);
    let dbg = format!("{:?}", v);
    assert!(dbg.contains("Vec3"));
}

// ==========================================================================
// Table with enum field: build + read + default enum value
// ==========================================================================

#[allow(
    unused_imports,
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case
)]
mod table_enum_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/table_enum_field.expected");
}

#[test]
fn table_enum_field_default() {
    use table_enum_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = WeaponArgs::default();
    let offset = createWeapon(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let weapon = ::flatbuffers::root::<Weapon>(buf).unwrap();
    // Default color is Green (2)
    assert_eq!(weapon.color(), Color::Green);
    assert_eq!(weapon.color().0, 2);
    assert_eq!(weapon.damage(), 0);
    assert_eq!(weapon.name(), None);
}

#[test]
fn table_enum_field_custom() {
    use table_enum_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("Excalibur");
    let args = WeaponArgs {
        name: Some(name),
        color: Color::Blue,
        damage: 999,
    };
    let offset = createWeapon(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let weapon = ::flatbuffers::root::<Weapon>(buf).unwrap();
    assert_eq!(weapon.name(), Some("Excalibur"));
    assert_eq!(weapon.color(), Color::Blue);
    assert_eq!(weapon.damage(), 999);
}

// ==========================================================================
// Table with vector fields: build + read vectors of scalars and strings
// ==========================================================================

#[allow(unused_imports, dead_code, non_camel_case_types, non_snake_case)]
mod table_vector_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/table_vector_fields.expected");
}

#[test]
fn table_vector_empty() {
    use table_vector_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = InventoryArgs::default();
    let offset = createInventory(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let inv = ::flatbuffers::root::<Inventory>(buf).unwrap();
    assert!(inv.slots().is_none());
    assert!(inv.names().is_none());
    assert!(inv.scores().is_none());
}

#[test]
fn table_vector_with_data() {
    use table_vector_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let slots = fbb.create_vector(&[1u8, 2, 3, 4, 5]);
    let name_a = fbb.create_string("Alice");
    let name_b = fbb.create_string("Bob");
    let names = fbb.create_vector(&[name_a, name_b]);
    let scores = fbb.create_vector(&[100i32, -50, 0]);
    let args = InventoryArgs {
        slots: Some(slots),
        names: Some(names),
        scores: Some(scores),
    };
    let offset = createInventory(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let inv = ::flatbuffers::root::<Inventory>(buf).unwrap();

    let s = inv.slots().unwrap();
    assert_eq!(s.len(), 5);
    assert_eq!(s.get(0), 1);
    assert_eq!(s.get(4), 5);

    let n = inv.names().unwrap();
    assert_eq!(n.len(), 2);
    assert_eq!(n.get(0), "Alice");
    assert_eq!(n.get(1), "Bob");

    let sc = inv.scores().unwrap();
    assert_eq!(sc.len(), 3);
    assert_eq!(sc.get(0), 100);
    assert_eq!(sc.get(1), -50);
}

// ==========================================================================
// Table with struct field: struct embedded in table
// ==========================================================================

#[allow(unused_imports, dead_code, non_camel_case_types, non_snake_case)]
mod table_struct_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/table_struct_field.expected");
}

#[test]
fn table_with_struct_field() {
    use table_struct_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let pos = Vec2::new(3.0, 4.0);
    let name = fbb.create_string("hero");
    let args = SpriteArgs {
        pos: Some(&pos),
        name: Some(name),
    };
    let offset = createSprite(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let sprite = ::flatbuffers::root::<Sprite>(buf).unwrap();
    assert_eq!(sprite.name(), Some("hero"));
    let p = sprite.pos().unwrap();
    assert!((p.x() - 3.0_f32).abs() < f32::EPSILON);
    assert!((p.y() - 4.0_f32).abs() < f32::EPSILON);
}

// ==========================================================================
// Nested table: table containing another table
// ==========================================================================

#[allow(unused_imports, dead_code, non_camel_case_types, non_snake_case)]
mod nested_table_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/table_table_field.expected");
}

#[test]
fn table_with_nested_table() {
    use nested_table_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let child_offset = createChild(&mut fbb, &ChildArgs { value: 42 });
    let label = fbb.create_string("parent_label");
    let args = ParentArgs {
        child: Some(child_offset),
        label: Some(label),
    };
    let offset = createParent(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let parent = ::flatbuffers::root::<Parent>(buf).unwrap();
    assert_eq!(parent.label(), Some("parent_label"));
    let child = parent.child().unwrap();
    assert_eq!(child.value(), 42);
}

#[test]
fn table_with_no_nested_table() {
    use nested_table_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = ParentArgs::default();
    let offset = createParent(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let parent = ::flatbuffers::root::<Parent>(buf).unwrap();
    assert!(parent.child().is_none());
    assert!(parent.label().is_none());
}

// ==========================================================================
// Root type with file_identifier
// ==========================================================================

#[allow(unused_imports, dead_code, non_camel_case_types, non_snake_case)]
mod root_type_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/root_type_with_ident.expected");
}

#[test]
fn root_type_build_and_read() {
    use root_type_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("Orc");
    let args = MonsterArgs {
        hp: 300,
        name: Some(name),
    };
    let offset = createMonster(&mut fbb, &args);
    finish_monster_buffer(&mut fbb, offset);
    let buf = fbb.finished_data();

    // Verify identifier
    assert!(monster_buffer_has_identifier(buf));

    // Read back
    let monster = root_as_monster(buf).unwrap();
    assert_eq!(monster.hp(), 300);
    assert_eq!(monster.name(), "Orc");
}

#[test]
fn root_type_size_prefixed() {
    use root_type_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("Goblin");
    let args = MonsterArgs {
        hp: 50,
        name: Some(name),
    };
    let offset = createMonster(&mut fbb, &args);
    finish_size_prefixed_monster_buffer(&mut fbb, offset);
    let buf = fbb.finished_data();

    let monster = size_prefixed_root_as_monster(buf).unwrap();
    assert_eq!(monster.hp(), 50);
    assert_eq!(monster.name(), "Goblin");
}

#[test]
fn root_type_identifier_constant() {
    use root_type_runtime::*;
    assert_eq!(MONSTER_IDENTIFIER, "MONS");
}

// ==========================================================================
// Bitflags enum: bitwise ops + build + read
// ==========================================================================

#[allow(
    unused_imports,
    dead_code,
    non_upper_case_globals,
    non_camel_case_types
)]
mod bitflags_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/enum_bitflags.expected");
}

#[test]
fn bitflags_individual_values() {
    use bitflags_runtime::*;
    assert_eq!(Color::Red.bits(), 1);
    assert_eq!(Color::Green.bits(), 2);
    assert_eq!(Color::Blue.bits(), 8);
}

#[test]
fn bitflags_bitwise_or() {
    use bitflags_runtime::*;
    let combined = Color::Red | Color::Green;
    assert_eq!(combined.bits(), 3);
    assert!(combined.contains(Color::Red));
    assert!(combined.contains(Color::Green));
    assert!(!combined.contains(Color::Blue));
}

#[test]
fn bitflags_bitwise_and() {
    use bitflags_runtime::*;
    let combined = Color::Red | Color::Green | Color::Blue;
    let masked = combined & Color::Green;
    assert_eq!(masked, Color::Green);
}

#[test]
fn bitflags_default_is_empty() {
    use bitflags_runtime::*;
    let c = Color::default();
    assert!(c.is_empty());
    assert_eq!(c.bits(), 0);
}

#[test]
fn bitflags_size_is_1_byte() {
    use bitflags_runtime::*;
    assert_eq!(::core::mem::size_of::<Color>(), 1);
}

#[test]
fn bitflags_debug() {
    use bitflags_runtime::*;
    let dbg = format!("{:?}", Color::Red);
    assert!(dbg.contains("Red"));
    let combined = format!("{:?}", Color::Red | Color::Green);
    assert!(combined.contains("Red"));
    assert!(combined.contains("Green"));
}

// ==========================================================================
// Union: build + read with type-safe accessors
// ==========================================================================

#[allow(
    unused_imports,
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case
)]
mod union_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/union_basic.expected");
}

#[test]
fn union_build_and_read_sword() {
    use union_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();

    // Build a Sword
    let sword = createSword(&mut fbb, &SwordArgs { damage: 42 });

    // Build a Hero with Sword as weapon
    let name = fbb.create_string("Link");
    let args = HeroArgs {
        name: Some(name),
        weapon_type: Equipment::Sword,
        weapon: Some(sword.as_union_value()),
    };
    let hero = createHero(&mut fbb, &args);
    fbb.finish_minimal(hero);

    let buf = fbb.finished_data();
    let h = ::flatbuffers::root::<Hero>(buf).unwrap();
    assert_eq!(h.name(), Some("Link"));
    assert_eq!(h.weapon_type(), Equipment::Sword);
    let sword = h.weapon_as_sword().unwrap();
    assert_eq!(sword.damage(), 42);
    assert!(h.weapon_as_shield().is_none());
}

#[test]
fn union_build_and_read_shield() {
    use union_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();

    let shield = createShield(&mut fbb, &ShieldArgs { armor: 100 });
    let name = fbb.create_string("Guard");
    let args = HeroArgs {
        name: Some(name),
        weapon_type: Equipment::Shield,
        weapon: Some(shield.as_union_value()),
    };
    let hero = createHero(&mut fbb, &args);
    fbb.finish_minimal(hero);

    let buf = fbb.finished_data();
    let h = ::flatbuffers::root::<Hero>(buf).unwrap();
    assert_eq!(h.weapon_type(), Equipment::Shield);
    let shield = h.weapon_as_shield().unwrap();
    assert_eq!(shield.armor(), 100);
    assert!(h.weapon_as_sword().is_none());
}

#[test]
fn union_default_is_none() {
    use union_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = HeroArgs::default();
    let hero = createHero(&mut fbb, &args);
    fbb.finish_minimal(hero);

    let buf = fbb.finished_data();
    let h = ::flatbuffers::root::<Hero>(buf).unwrap();
    assert_eq!(h.weapon_type(), Equipment::NONE);
    assert!(h.weapon().is_none());
    assert!(h.weapon_as_sword().is_none());
    assert!(h.weapon_as_shield().is_none());
}

#[test]
fn union_enum_size() {
    use union_runtime::*;
    assert_eq!(::core::mem::size_of::<Equipment>(), 1);
    assert_eq!(::core::mem::align_of::<Equipment>(), 1);
}

// ==========================================================================
// Struct with fixed-size arrays: new + read + set
// ==========================================================================

#[allow(
    unused_imports,
    dead_code,
    non_upper_case_globals,
    non_camel_case_types
)]
mod struct_array_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/struct_array.expected");
}

#[test]
fn struct_array_scalar_roundtrip() {
    use struct_array_runtime::*;
    let s = Outer::new(
        &[10, 20, 30],
        TestEnum::B,
        &[TestEnum::A, TestEnum::C],
        &[Inner::new(1, 2), Inner::new(3, 4)],
        42.0,
    );

    // Scalar array
    let a = s.a();
    assert_eq!(a.len(), 3);
    assert_eq!(a.get(0), 10);
    assert_eq!(a.get(1), 20);
    assert_eq!(a.get(2), 30);
}

#[test]
fn struct_array_enum_roundtrip() {
    use struct_array_runtime::*;
    let s = Outer::new(
        &[0, 0, 0],
        TestEnum::A,
        &[TestEnum::B, TestEnum::C],
        &[Inner::default(), Inner::default()],
        0.0,
    );

    // Enum array
    let c = s.c();
    assert_eq!(c.len(), 2);
    assert_eq!(c.get(0), TestEnum::B);
    assert_eq!(c.get(1), TestEnum::C);
}

#[test]
fn struct_array_nested_struct_roundtrip() {
    use struct_array_runtime::*;
    let inner1 = Inner::new(100, 200);
    let inner2 = Inner::new(300, 400);
    let s = Outer::new(
        &[0, 0, 0],
        TestEnum::A,
        &[TestEnum::A, TestEnum::A],
        &[inner1, inner2],
        0.0,
    );

    // Struct array
    let d = s.d();
    assert_eq!(d.len(), 2);
    assert_eq!(d.get(0).x(), 100);
    assert_eq!(d.get(0).y(), 200);
    assert_eq!(d.get(1).x(), 300);
    assert_eq!(d.get(1).y(), 400);
}

#[test]
fn struct_array_default_is_zero() {
    use struct_array_runtime::*;
    let s = Outer::default();

    let a = s.a();
    assert_eq!(a.get(0), 0);
    assert_eq!(a.get(1), 0);
    assert_eq!(a.get(2), 0);

    assert_eq!(s.b(), TestEnum::A);
    assert_eq!(s.e(), 0.0);
}

#[test]
fn struct_array_size_and_align() {
    use struct_array_runtime::*;
    // Inner: 2 x int = 8 bytes, aligned to 4
    assert_eq!(::core::mem::size_of::<Inner>(), 8);
    // Outer: a:[int:3]=12 + b:byte=1 + c:[byte:2]=2 + pad=1 + d:[Inner:2]=16 + e:float=4 = 36
    // With alignment padding the total should be 36 bytes, aligned to 4
    assert_eq!(::core::mem::size_of::<Outer>(), 36);
}

#[test]
fn struct_array_set_scalar_array() {
    use struct_array_runtime::*;
    let mut s = Outer::default();
    s.set_a(&[7, 8, 9]);
    let a = s.a();
    assert_eq!(a.get(0), 7);
    assert_eq!(a.get(1), 8);
    assert_eq!(a.get(2), 9);
}

// ==========================================================================
// Struct layout: size_of and align_of validation
// ==========================================================================

#[test]
fn struct_vec3_is_12_bytes() {
    assert_eq!(::core::mem::size_of::<struct_runtime::Vec3>(), 12);
    assert_eq!(::core::mem::align_of::<struct_runtime::Vec3>(), 1);
}

#[test]
fn struct_inner_is_8_bytes() {
    use struct_array_runtime::*;
    assert_eq!(::core::mem::size_of::<Inner>(), 8);
    assert_eq!(::core::mem::align_of::<Inner>(), 1);
}

#[test]
fn enum_basic_is_1_byte() {
    use enum_runtime::*;
    assert_eq!(::core::mem::size_of::<Color>(), 1);
    assert_eq!(::core::mem::align_of::<Color>(), 1);
}

#[test]
fn bitflags_enum_is_1_byte() {
    use bitflags_runtime::*;
    assert_eq!(::core::mem::size_of::<Color>(), 1);
    assert_eq!(::core::mem::align_of::<Color>(), 1);
}

// ==========================================================================
// Additional builder round-trip tests
// ==========================================================================

#[test]
fn table_scalar_defaults_without_setting() {
    use table_scalar_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = StatsArgs::default();
    let offset = createStats(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let stats = ::flatbuffers::root::<Stats>(buf).unwrap();
    // Verify all defaults are read correctly
    assert_eq!(stats.hp(), 100);
    assert_eq!(stats.mana(), 50);
    assert!((stats.speed() - 1.5_f32).abs() < f32::EPSILON);
    assert_eq!(stats.active(), true);
}

#[test]
fn table_scalar_explicit_zeros() {
    use table_scalar_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = StatsArgs {
        hp: 0,
        mana: 0,
        speed: 0.0,
        active: false,
    };
    let offset = createStats(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let stats = ::flatbuffers::root::<Stats>(buf).unwrap();
    assert_eq!(stats.hp(), 0);
    assert_eq!(stats.mana(), 0);
    assert_eq!(stats.speed(), 0.0);
    assert_eq!(stats.active(), false);
}

#[test]
fn table_vector_ubyte_roundtrip() {
    use table_vector_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let data: Vec<u8> = (0..=255).collect();
    let slots = fbb.create_vector(&data);
    let args = InventoryArgs {
        slots: Some(slots),
        names: None,
        scores: None,
    };
    let offset = createInventory(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let inv = ::flatbuffers::root::<Inventory>(buf).unwrap();
    let s = inv.slots().unwrap();
    assert_eq!(s.len(), 256);
    for i in 0..256 {
        assert_eq!(s.get(i), i as u8);
    }
}

#[test]
fn table_vector_string_empty_vector() {
    use table_vector_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let names = fbb.create_vector::<::flatbuffers::ForwardsUOffset<&str>>(&[]);
    let args = InventoryArgs {
        slots: None,
        names: Some(names),
        scores: None,
    };
    let offset = createInventory(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let inv = ::flatbuffers::root::<Inventory>(buf).unwrap();
    let n = inv.names().unwrap();
    assert_eq!(n.len(), 0);
}

#[test]
fn table_nested_table_roundtrip() {
    use nested_table_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let child_args = ChildArgs { value: 42 };
    let child = createChild(&mut fbb, &child_args);
    let label = fbb.create_string("parent_label");
    let args = ParentArgs {
        child: Some(child),
        label: Some(label),
    };
    let offset = createParent(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let parent = ::flatbuffers::root::<Parent>(buf).unwrap();
    assert_eq!(parent.label(), Some("parent_label"));
    let child = parent.child().unwrap();
    assert_eq!(child.value(), 42);
}

#[test]
fn table_nested_table_none() {
    use nested_table_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = ParentArgs::default();
    let offset = createParent(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let parent = ::flatbuffers::root::<Parent>(buf).unwrap();
    assert!(parent.child().is_none());
    assert!(parent.label().is_none());
}

#[test]
fn table_struct_field_none() {
    use table_struct_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = SpriteArgs::default();
    let offset = createSprite(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let sprite = ::flatbuffers::root::<Sprite>(buf).unwrap();
    assert!(sprite.pos().is_none());
    assert!(sprite.name().is_none());
}

// ==========================================================================
// Verifier edge case tests
// ==========================================================================

#[test]
fn verifier_empty_buffer_fails() {
    use table_scalar_runtime::*;
    let buf: &[u8] = &[];
    assert!(::flatbuffers::root::<Stats>(buf).is_err());
}

#[test]
fn verifier_too_short_buffer_fails() {
    use table_scalar_runtime::*;
    let buf: &[u8] = &[0, 0, 0];
    assert!(::flatbuffers::root::<Stats>(buf).is_err());
}

#[test]
fn verifier_truncated_buffer_fails() {
    use table_scalar_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let offset = createStats(&mut fbb, &StatsArgs::default());
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    // Truncate valid buffer to various lengths
    for len in 1..buf.len() {
        assert!(
            ::flatbuffers::root::<Stats>(&buf[..len]).is_err(),
            "Expected verification failure at truncation length {len}"
        );
    }
}

#[test]
fn verifier_one_byte_flip_no_crash() {
    use table_scalar_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = StatsArgs {
        hp: 42,
        mana: 10,
        speed: 3.14,
        active: true,
    };
    let offset = createStats(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let original = fbb.finished_data().to_vec();
    // Flip each byte and verify it either succeeds or returns Err (but never panics)
    for i in 0..original.len() {
        let mut corrupted = original.clone();
        corrupted[i] ^= 0xFF;
        let _ = ::flatbuffers::root::<Stats>(&corrupted);
    }
}

#[test]
fn verifier_string_table_one_byte_flip_no_crash() {
    use table_string_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let msg = fbb.create_string("hello world");
    let name = fbb.create_string("test_name");
    let args = GreetingArgs {
        message: Some(msg),
        name: Some(name),
    };
    let offset = createGreeting(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let original = fbb.finished_data().to_vec();
    for i in 0..original.len() {
        let mut corrupted = original.clone();
        corrupted[i] ^= 0xFF;
        let _ = ::flatbuffers::root::<Greeting>(&corrupted);
    }
}

#[test]
fn verifier_nested_table_one_byte_flip_no_crash() {
    use nested_table_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let child = createChild(&mut fbb, &ChildArgs { value: 99 });
    let label = fbb.create_string("lbl");
    let args = ParentArgs {
        child: Some(child),
        label: Some(label),
    };
    let offset = createParent(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let original = fbb.finished_data().to_vec();
    for i in 0..original.len() {
        let mut corrupted = original.clone();
        corrupted[i] ^= 0xFF;
        let _ = ::flatbuffers::root::<Parent>(&corrupted);
    }
}

// ==========================================================================
// Keyword escaping: Rust keywords as field/variant names
// ==========================================================================

#[allow(
    unused_imports,
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case
)]
mod keyword_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/keyword_escape.expected");
}

#[test]
fn keyword_enum_variants() {
    use keyword_runtime::*;
    assert_eq!(TestKW::None.0, 0);
    assert_eq!(TestKW::where_.0, 1);
    assert_eq!(TestKW::match_.0, 2);
}

#[test]
fn keyword_enum_variant_names() {
    use keyword_runtime::*;
    assert_eq!(TestKW::None.variant_name(), Some("None"));
    assert_eq!(TestKW::where_.variant_name(), Some("where"));
    assert_eq!(TestKW::match_.variant_name(), Some("match"));
}

#[test]
fn keyword_table_build_and_read() {
    use keyword_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let is_str = fbb.create_string("hello");
    let args = KWTableArgs {
        type_: 42,
        is: Some(is_str),
        where_: TestKW::match_,
    };
    let offset = createKWTable(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let t = ::flatbuffers::root::<KWTable>(buf).unwrap();
    assert_eq!(t.type_(), 42);
    assert_eq!(t.is(), Some("hello"));
    assert_eq!(t.where_(), TestKW::match_);
}

#[test]
fn keyword_table_defaults() {
    use keyword_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = KWTableArgs::default();
    let offset = createKWTable(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let t = ::flatbuffers::root::<KWTable>(buf).unwrap();
    assert_eq!(t.type_(), 0);
    assert_eq!(t.is(), None);
    assert_eq!(t.where_(), TestKW::None);
}

// ==========================================================================
// Namespace: single namespace build + read
// ==========================================================================

#[allow(
    unused_imports,
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case
)]
mod namespace_simple_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/namespace_simple.expected");
}

#[test]
fn namespace_simple_build_and_read() {
    use namespace_simple_runtime::my_game::example::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = MonsterArgs {
        hp: 200,
        color: Color::Green,
    };
    let offset = createMonster(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let m = ::flatbuffers::root::<Monster>(buf).unwrap();
    assert_eq!(m.hp(), 200);
    assert_eq!(m.color(), Color::Green);
}

#[test]
fn namespace_simple_defaults() {
    use namespace_simple_runtime::my_game::example::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = MonsterArgs::default();
    let offset = createMonster(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let m = ::flatbuffers::root::<Monster>(buf).unwrap();
    assert_eq!(m.hp(), 100);
    assert_eq!(m.color(), Color::Blue);
}

// ==========================================================================
// Namespace: multi-namespace types
// ==========================================================================

#[allow(
    unused_imports,
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case
)]
mod namespace_multi_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/namespace_multi.expected");
}

#[test]
fn namespace_multi_build_root() {
    use namespace_multi_runtime::my_game::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("world");
    let args = RootArgs { name: Some(name) };
    let offset = createRoot(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let root = ::flatbuffers::root::<Root>(buf).unwrap();
    assert_eq!(root.name(), Some("world"));
}

#[test]
fn namespace_multi_build_helper() {
    use namespace_multi_runtime::my_game::example2::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = HelperArgs { count: 42 };
    let offset = createHelper(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let helper = ::flatbuffers::root::<Helper>(buf).unwrap();
    assert_eq!(helper.count(), 42);
}

#[test]
fn namespace_multi_enum_from_sibling() {
    use namespace_multi_runtime::my_game::example::*;
    assert_eq!(Color::Red.0, 1);
    assert_eq!(Color::Green.0, 2);
    assert_eq!(Color::Red.variant_name(), Some("Red"));
}

// ==========================================================================
// Namespace: cross-namespace table references
// ==========================================================================

#[allow(
    unused_imports,
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case
)]
mod namespace_cross_ref_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/namespace_cross_ref.expected");
}

#[test]
fn namespace_cross_ref_item_roundtrip() {
    use namespace_cross_ref_runtime::game::items::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("Sword");
    let stats = ItemStats::new(25, 10);
    let args = ItemArgs {
        name: Some(name),
        rarity: Rarity::Epic,
        stats: Some(&stats),
    };
    let offset = createItem(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let item = ::flatbuffers::root::<Item>(buf).unwrap();
    assert_eq!(item.name(), Some("Sword"));
    assert_eq!(item.rarity(), Rarity::Epic);
    let s = item.stats().unwrap();
    assert_eq!(s.attack(), 25);
    assert_eq!(s.defense(), 10);
}

#[test]
fn namespace_cross_ref_inventory_roundtrip() {
    use namespace_cross_ref_runtime::game::items::*;
    use namespace_cross_ref_runtime::game::player::*;
    // Build two items
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let name1 = fbb.create_string("Shield");
    let item1_args = ItemArgs {
        name: Some(name1),
        rarity: Rarity::Rare,
        stats: None,
    };
    let item1 = createItem(&mut fbb, &item1_args);
    let name2 = fbb.create_string("Potion");
    let item2_args = ItemArgs {
        name: Some(name2),
        rarity: Rarity::Common,
        stats: None,
    };
    let item2 = createItem(&mut fbb, &item2_args);
    // Build inventory with vector of items and a favorite
    let items_vec = fbb.create_vector(&[item1, item2]);
    let name3 = fbb.create_string("Staff");
    let fav_args = ItemArgs {
        name: Some(name3),
        rarity: Rarity::Epic,
        stats: None,
    };
    let fav = createItem(&mut fbb, &fav_args);
    let owner = fbb.create_string("Alice");
    let inv_args = InventoryArgs {
        owner: Some(owner),
        items: Some(items_vec),
        favorite: Some(fav),
    };
    let inv_offset = createInventory(&mut fbb, &inv_args);
    fbb.finish_minimal(inv_offset);
    let buf = fbb.finished_data();
    let inv = ::flatbuffers::root::<Inventory>(buf).unwrap();
    assert_eq!(inv.owner(), Some("Alice"));
    // Verify items vector
    let items = inv.items().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items.get(0).name(), Some("Shield"));
    assert_eq!(items.get(0).rarity(), Rarity::Rare);
    assert_eq!(items.get(1).name(), Some("Potion"));
    assert_eq!(items.get(1).rarity(), Rarity::Common);
    // Verify favorite
    let fav_read = inv.favorite().unwrap();
    assert_eq!(fav_read.name(), Some("Staff"));
    assert_eq!(fav_read.rarity(), Rarity::Epic);
}

#[test]
fn namespace_cross_ref_inventory_defaults() {
    use namespace_cross_ref_runtime::game::player::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = InventoryArgs::default();
    let offset = createInventory(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let inv = ::flatbuffers::root::<Inventory>(buf).unwrap();
    assert_eq!(inv.owner(), None);
    assert!(inv.items().is_none());
    assert!(inv.favorite().is_none());
}

// ==========================================================================
// Nested flatbuffer: build + typed read
// ==========================================================================

#[allow(
    unused_imports,
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case
)]
mod nested_flatbuffer_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/nested_flatbuffer.expected");
}

#[test]
fn nested_flatbuffer_roundtrip() {
    use nested_flatbuffer_runtime::*;
    // First, build the inner table as a separate flatbuffer
    let mut inner_fbb = ::flatbuffers::FlatBufferBuilder::new();
    let name = inner_fbb.create_string("hero");
    let inner_args = InnerArgs {
        hp: 250,
        name: Some(name),
    };
    let inner_offset = createInner(&mut inner_fbb, &inner_args);
    inner_fbb.finish_minimal(inner_offset);
    let inner_bytes = inner_fbb.finished_data().to_vec();

    // Build the outer table with the inner flatbuffer as a [ubyte] field
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let label = fbb.create_string("container");
    let data = fbb.create_vector(&inner_bytes);
    let outer_args = OuterArgs {
        label: Some(label),
        data: Some(data),
    };
    let outer_offset = createOuter(&mut fbb, &outer_args);
    fbb.finish_minimal(outer_offset);
    let buf = fbb.finished_data();

    // Read back and verify
    let outer = ::flatbuffers::root::<Outer>(buf).unwrap();
    assert_eq!(outer.label(), Some("container"));

    // Access raw bytes
    let raw_data = outer.data().unwrap();
    assert_eq!(raw_data.len(), inner_bytes.len());

    // Access typed nested flatbuffer
    let inner = outer.data_nested_flatbuffer().unwrap();
    assert_eq!(inner.hp(), 250);
    assert_eq!(inner.name(), Some("hero"));
}

#[test]
fn nested_flatbuffer_default_is_none() {
    use nested_flatbuffer_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let args = OuterArgs::default();
    let offset = createOuter(&mut fbb, &args);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let outer = ::flatbuffers::root::<Outer>(buf).unwrap();
    assert!(outer.data().is_none());
    assert!(outer.data_nested_flatbuffer().is_none());
}

#[test]
fn nested_flatbuffer_inner_defaults() {
    use nested_flatbuffer_runtime::*;
    // Build an inner with default values
    let mut inner_fbb = ::flatbuffers::FlatBufferBuilder::new();
    let inner_args = InnerArgs::default();
    let inner_offset = createInner(&mut inner_fbb, &inner_args);
    inner_fbb.finish_minimal(inner_offset);
    let inner_bytes = inner_fbb.finished_data().to_vec();

    // Wrap in outer
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let data = fbb.create_vector(&inner_bytes);
    let outer_args = OuterArgs {
        label: None,
        data: Some(data),
    };
    let outer_offset = createOuter(&mut fbb, &outer_args);
    fbb.finish_minimal(outer_offset);
    let buf = fbb.finished_data();

    let outer = ::flatbuffers::root::<Outer>(buf).unwrap();
    let inner = outer.data_nested_flatbuffer().unwrap();
    assert_eq!(inner.hp(), 100); // default
    assert_eq!(inner.name(), None);
}

// ==========================================================================
// Key comparison methods
// ==========================================================================

#[allow(
    unused_imports,
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case
)]
mod key_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/table_key.expected");
}

#[test]
fn key_string_compare_less_than() {
    use key_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let name_a = fbb.create_string("Alice");
    let a = createMonster(
        &mut fbb,
        &MonsterArgs {
            name: Some(name_a),
            hp: 100,
        },
    );
    fbb.finish_minimal(a);
    let buf_a = fbb.finished_data().to_vec();

    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let name_b = fbb.create_string("Bob");
    let b = createMonster(
        &mut fbb,
        &MonsterArgs {
            name: Some(name_b),
            hp: 50,
        },
    );
    fbb.finish_minimal(b);
    let buf_b = fbb.finished_data().to_vec();

    let ma = ::flatbuffers::root::<Monster>(&buf_a).unwrap();
    let mb = ::flatbuffers::root::<Monster>(&buf_b).unwrap();
    assert!(ma.key_compare_less_than(&mb));
    assert!(!mb.key_compare_less_than(&ma));
    assert!(!ma.key_compare_less_than(&ma));
}

#[test]
fn key_string_compare_with_value() {
    use key_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("Monster");
    let offset = createMonster(
        &mut fbb,
        &MonsterArgs {
            name: Some(name),
            hp: 100,
        },
    );
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let m = ::flatbuffers::root::<Monster>(buf).unwrap();
    assert_eq!(
        m.key_compare_with_value("Monster"),
        ::core::cmp::Ordering::Equal
    );
    assert_eq!(
        m.key_compare_with_value("Alpha"),
        ::core::cmp::Ordering::Greater
    );
    assert_eq!(
        m.key_compare_with_value("Zombie"),
        ::core::cmp::Ordering::Less
    );
}

#[test]
fn key_scalar_compare_less_than() {
    use key_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let id = fbb.create_string("a");
    let a = createStat(
        &mut fbb,
        &StatArgs {
            id: Some(id),
            count: 10,
        },
    );
    fbb.finish_minimal(a);
    let buf_a = fbb.finished_data().to_vec();

    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let id = fbb.create_string("b");
    let b = createStat(
        &mut fbb,
        &StatArgs {
            id: Some(id),
            count: 20,
        },
    );
    fbb.finish_minimal(b);
    let buf_b = fbb.finished_data().to_vec();

    let sa = ::flatbuffers::root::<Stat>(&buf_a).unwrap();
    let sb = ::flatbuffers::root::<Stat>(&buf_b).unwrap();
    assert!(sa.key_compare_less_than(&sb));
    assert!(!sb.key_compare_less_than(&sa));
}

#[test]
fn key_scalar_compare_with_value() {
    use key_runtime::*;
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let id = fbb.create_string("test");
    let offset = createStat(
        &mut fbb,
        &StatArgs {
            id: Some(id),
            count: 42,
        },
    );
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let s = ::flatbuffers::root::<Stat>(buf).unwrap();
    assert_eq!(s.key_compare_with_value(42), ::core::cmp::Ordering::Equal);
    assert_eq!(s.key_compare_with_value(10), ::core::cmp::Ordering::Greater);
    assert_eq!(s.key_compare_with_value(100), ::core::cmp::Ordering::Less);
}

// ==========================================================================
// Struct key comparison methods
// ==========================================================================

#[allow(
    unused_imports,
    dead_code,
    non_upper_case_globals,
    non_camel_case_types
)]
mod struct_key_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/struct_key.expected");
}

#[test]
fn struct_key_compare_less_than() {
    use struct_key_runtime::*;
    let a = Ability::new(10, 100);
    let b = Ability::new(20, 50);
    assert!(a.key_compare_less_than(&b));
    assert!(!b.key_compare_less_than(&a));
    assert!(!a.key_compare_less_than(&a));
}

#[test]
fn struct_key_compare_with_value() {
    use struct_key_runtime::*;
    let a = Ability::new(42, 100);
    assert_eq!(a.key_compare_with_value(42), ::core::cmp::Ordering::Equal);
    assert_eq!(a.key_compare_with_value(10), ::core::cmp::Ordering::Greater);
    assert_eq!(a.key_compare_with_value(100), ::core::cmp::Ordering::Less);
}

// ==========================================================================
// Object API: struct T pack/unpack
// ==========================================================================

#[allow(unused_imports, dead_code, non_camel_case_types)]
mod object_api_struct_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/object_api_struct.expected");
}

#[test]
fn object_api_struct_default() {
    use object_api_struct_runtime::*;
    let v = Vec3T::default();
    assert_eq!(v.x, 0.0);
    assert_eq!(v.y, 0.0);
    assert_eq!(v.z, 0.0);
}

#[test]
fn object_api_struct_pack_unpack_roundtrip() {
    use object_api_struct_runtime::*;
    let orig = Vec3T {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let packed = orig.pack();
    let unpacked = packed.unpack();
    assert_eq!(orig, unpacked);
}

#[test]
fn object_api_struct_unpack_from_constructor() {
    use object_api_struct_runtime::*;
    let v = Vec3::new(10.0, 20.0, 30.0);
    let t = v.unpack();
    assert_eq!(t.x, 10.0);
    assert_eq!(t.y, 20.0);
    assert_eq!(t.z, 30.0);
}

// ==========================================================================
// Object API: table T pack/unpack
// ==========================================================================

#[allow(unused_imports, dead_code, non_camel_case_types, non_snake_case)]
mod object_api_table_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/object_api_table.expected");
}

#[test]
fn object_api_table_default() {
    use object_api_table_runtime::*;
    let m = MonsterT::default();
    assert_eq!(m.hp, 100);
    assert_eq!(m.name, "");
    assert!(m.pos.is_none());
    assert!(m.inventory.is_none());
    assert!(m.tags.is_none());
}

#[test]
fn object_api_table_pack_unpack_scalars() {
    use object_api_table_runtime::*;
    let orig = MonsterT {
        hp: 42,
        name: "hero".to_string(),
        pos: None,
        inventory: None,
        tags: None,
    };
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let offset = orig.pack(&mut fbb);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let m = ::flatbuffers::root::<Monster>(buf).unwrap();
    assert_eq!(m.hp(), 42);
    assert_eq!(m.name(), "hero");
    let unpacked = m.unpack();
    assert_eq!(orig, unpacked);
}

#[test]
fn object_api_table_pack_unpack_struct_field() {
    use object_api_table_runtime::*;
    let orig = MonsterT {
        hp: 100,
        name: "test".to_string(),
        pos: Some(Vec2T { x: 3.0, y: 4.0 }),
        inventory: None,
        tags: None,
    };
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let offset = orig.pack(&mut fbb);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let m = ::flatbuffers::root::<Monster>(buf).unwrap();
    let unpacked = m.unpack();
    assert_eq!(orig, unpacked);
}

#[test]
fn object_api_table_pack_unpack_vectors() {
    use object_api_table_runtime::*;
    let orig = MonsterT {
        hp: 100,
        name: "bags".to_string(),
        pos: None,
        inventory: Some(vec![1, 2, 3, 4, 5]),
        tags: Some(vec!["alpha".to_string(), "beta".to_string()]),
    };
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let offset = orig.pack(&mut fbb);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let m = ::flatbuffers::root::<Monster>(buf).unwrap();
    let unpacked = m.unpack();
    assert_eq!(orig, unpacked);
}

#[test]
fn object_api_table_pack_defaults() {
    use object_api_table_runtime::*;
    let orig = MonsterT {
        name: "minimal".to_string(),
        ..Default::default()
    };
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let offset = orig.pack(&mut fbb);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let m = ::flatbuffers::root::<Monster>(buf).unwrap();
    assert_eq!(m.hp(), 100);
    assert_eq!(m.name(), "minimal");
}

// ==========================================================================
// Object API: union T pack/unpack
// ==========================================================================

#[allow(
    unused_imports,
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case
)]
mod object_api_union_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/object_api_union.expected");
}

#[test]
fn object_api_union_default_is_none() {
    use object_api_union_runtime::*;
    let h = HeroT::default();
    assert_eq!(h.weapon, EquipmentT::NONE);
    assert_eq!(h.name, None);
}

#[test]
fn object_api_union_pack_unpack_sword() {
    use object_api_union_runtime::*;
    let orig = HeroT {
        name: Some("Link".to_string()),
        weapon: EquipmentT::Sword(Box::new(SwordT { damage: 42 })),
    };
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let offset = orig.pack(&mut fbb);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let h = ::flatbuffers::root::<Hero>(buf).unwrap();
    assert_eq!(h.name(), Some("Link"));
    assert_eq!(h.weapon_type(), Equipment::Sword);
    let sword = h.weapon_as_sword().unwrap();
    assert_eq!(sword.damage(), 42);
    let unpacked = h.unpack();
    assert_eq!(orig, unpacked);
}

#[test]
fn object_api_union_pack_unpack_shield() {
    use object_api_union_runtime::*;
    let orig = HeroT {
        name: Some("Guard".to_string()),
        weapon: EquipmentT::Shield(Box::new(ShieldT { armor: 100 })),
    };
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let offset = orig.pack(&mut fbb);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let h = ::flatbuffers::root::<Hero>(buf).unwrap();
    let unpacked = h.unpack();
    assert_eq!(orig, unpacked);
}

#[test]
fn object_api_union_pack_none() {
    use object_api_union_runtime::*;
    let orig = HeroT {
        name: Some("Unarmed".to_string()),
        weapon: EquipmentT::NONE,
    };
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let offset = orig.pack(&mut fbb);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let h = ::flatbuffers::root::<Hero>(buf).unwrap();
    assert_eq!(h.weapon_type(), Equipment::NONE);
    assert!(h.weapon().is_none());
    let unpacked = h.unpack();
    assert_eq!(orig, unpacked);
}

// ==========================================================================
// Optional scalars: build + read + object API
// ==========================================================================

#[allow(
    unused_imports,
    dead_code,
    non_upper_case_globals,
    non_camel_case_types
)]
mod optional_scalars_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/optional_scalars.expected");
}

#[test]
fn optional_scalars_defaults() {
    // Reading an empty buffer should return defaults
    use optional_scalars_runtime::*;
    let s = ::flatbuffers::root::<ScalarStuff>(&[0; 8]).unwrap();
    assert_eq!(s.just_i8(), 0);
    assert_eq!(s.maybe_i8(), None);
    assert_eq!(s.default_i8(), 42);
    assert_eq!(s.just_bool(), false);
    assert_eq!(s.maybe_bool(), None);
    assert_eq!(s.default_bool(), true);
}

#[test]
fn optional_scalars_set_and_read() {
    use optional_scalars_runtime::*;
    let mut builder = ::flatbuffers::FlatBufferBuilder::new();
    let ss = createScalarStuff(
        &mut builder,
        &ScalarStuffArgs {
            just_i8: 5,
            maybe_i8: Some(5),
            default_i8: 5,
            just_bool: true,
            maybe_bool: Some(true),
            default_bool: false,
        },
    );
    builder.finish_minimal(ss);
    let buf = builder.finished_data();
    let s = ::flatbuffers::root::<ScalarStuff>(buf).unwrap();
    assert_eq!(s.just_i8(), 5);
    assert_eq!(s.maybe_i8(), Some(5));
    assert_eq!(s.default_i8(), 5);
    assert_eq!(s.just_bool(), true);
    assert_eq!(s.maybe_bool(), Some(true));
    assert_eq!(s.default_bool(), false);
}

#[test]
fn optional_scalars_object_api_defaults() {
    use optional_scalars_runtime::*;
    let t = ScalarStuffT::default();
    assert_eq!(t.just_i8, 0);
    assert_eq!(t.maybe_i8, None);
    assert_eq!(t.default_i8, 42);
    assert_eq!(t.just_bool, false);
    assert_eq!(t.maybe_bool, None);
    assert_eq!(t.default_bool, true);
}

#[test]
fn optional_scalars_object_api_pack_unpack() {
    use optional_scalars_runtime::*;
    let orig = ScalarStuffT {
        just_i8: 5,
        maybe_i8: Some(5),
        default_i8: 5,
        just_bool: true,
        maybe_bool: Some(true),
        default_bool: false,
    };
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let offset = orig.pack(&mut fbb);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let s = ::flatbuffers::root::<ScalarStuff>(buf).unwrap();
    assert_eq!(s.just_i8(), 5);
    assert_eq!(s.maybe_i8(), Some(5));
    assert_eq!(s.default_i8(), 5);
    assert_eq!(s.just_bool(), true);
    assert_eq!(s.maybe_bool(), Some(true));
    assert_eq!(s.default_bool(), false);
    let unpacked = s.unpack();
    assert_eq!(orig, unpacked);
}

#[test]
fn optional_scalars_object_api_none_roundtrip() {
    use optional_scalars_runtime::*;
    let orig = ScalarStuffT::default();
    let mut fbb = ::flatbuffers::FlatBufferBuilder::new();
    let offset = orig.pack(&mut fbb);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let s = ::flatbuffers::root::<ScalarStuff>(buf).unwrap();
    assert_eq!(s.maybe_i8(), None);
    assert_eq!(s.maybe_bool(), None);
    let unpacked = s.unpack();
    assert_eq!(unpacked.maybe_i8, None);
    assert_eq!(unpacked.maybe_bool, None);
    assert_eq!(unpacked.default_i8, 42);
    assert_eq!(unpacked.default_bool, true);
}

// ---------------------------------------------------------------------------
// more_defaults: string and vector defaults
// ---------------------------------------------------------------------------

#[allow(
    unused_imports,
    dead_code,
    non_upper_case_globals,
    non_camel_case_types
)]
mod more_defaults_runtime {
    extern crate flatbuffers;
    include!("../testdata/codegen_golden/more_defaults.expected");
}

#[test]
fn more_defaults_nonpresent_values() {
    use more_defaults_runtime::*;
    let m = flatbuffers::root::<MoreDefaults>(&[0; 4]).unwrap();
    assert_eq!(m.ints().len(), 0);
    assert_eq!(m.floats().len(), 0);
    assert_eq!(m.abcs().len(), 0);
    assert_eq!(m.bools().len(), 0);
    assert_eq!(m.empty_string(), "");
    assert_eq!(m.some_string(), "some");
}

#[test]
fn more_defaults_object_api_defaults() {
    use more_defaults_runtime::*;
    let d = MoreDefaultsT::default();
    assert!(d.ints.is_empty());
    assert!(d.floats.is_empty());
    assert_eq!(d.empty_string, "");
    assert_eq!(d.some_string, "some");
    assert!(d.abcs.is_empty());
    assert!(d.bools.is_empty());
}

#[test]
fn more_defaults_object_api_pack_unpack() {
    use more_defaults_runtime::*;
    let orig = MoreDefaultsT {
        ints: vec![1, 2, 3],
        floats: vec![1.0, 2.5],
        empty_string: "hello".to_string(),
        some_string: "world".to_string(),
        abcs: vec![ABC::B, ABC::C],
        bools: vec![true, false, true],
    };
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let offset = orig.pack(&mut fbb);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();
    let m = flatbuffers::root::<MoreDefaults>(buf).unwrap();
    assert_eq!(m.ints().iter().collect::<Vec<_>>(), vec![1, 2, 3]);
    assert_eq!(m.floats().iter().collect::<Vec<_>>(), vec![1.0, 2.5]);
    assert_eq!(m.empty_string(), "hello");
    assert_eq!(m.some_string(), "world");
    assert_eq!(m.abcs().iter().collect::<Vec<_>>(), vec![ABC::B, ABC::C]);
    assert_eq!(
        m.bools().iter().collect::<Vec<_>>(),
        vec![true, false, true]
    );
    let unpacked = m.unpack();
    assert_eq!(orig, unpacked);
}
