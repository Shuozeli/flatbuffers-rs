//! Cross-compatibility tests: verify flatc-rs codegen produces binary-compatible
//! output with the official C++ flatc.
//!
//! Strategy: build a FlatBuffer with one implementation's API, read it with
//! the other's types, and assert all field values match.

extern crate alloc;

// ========== Official generated modules (multi-file, #[path]) ==========

#[allow(
    dead_code,
    unused_imports,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::all
)]
#[path = "../../testdata/reference/official_generated/more_defaults/mod.rs"]
mod off_more_defaults;

#[allow(
    dead_code,
    unused_imports,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::all
)]
#[path = "../../testdata/reference/official_generated/optional_scalars/mod.rs"]
mod off_optional_scalars;

#[allow(
    dead_code,
    unused_imports,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::all
)]
#[path = "../../testdata/reference/official_generated/monster_test/mod.rs"]
mod off_monster_test;

// ========== Our generated modules (single-file, include!) ==========

#[allow(
    dead_code,
    unused_imports,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::all
)]
mod our_more_defaults {
    extern crate flatbuffers;
    include!("../../testdata/reference/flatc_rs_generated/more_defaults.rs");
}

#[allow(
    dead_code,
    unused_imports,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::all
)]
mod our_optional_scalars {
    extern crate flatbuffers;
    include!("../../testdata/reference/flatc_rs_generated/optional_scalars.rs");
    pub use optional_scalars::*;
}

#[allow(
    dead_code,
    unused_imports,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::all
)]
mod our_monster_test {
    extern crate flatbuffers;
    include!("../../testdata/reference/flatc_rs_generated/monster_test.rs");
    pub use my_game::example::*;
}

// ==========================================================================
// more_defaults: flat namespace, string/vector defaults
// ==========================================================================

#[test]
fn cross_more_defaults_off_build_our_read() {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let ints = fbb.create_vector(&[1i32, 2, 3]);
    let floats = fbb.create_vector(&[1.5f32, 2.5]);
    let empty_str = fbb.create_string("hello");
    let some_str = fbb.create_string("world");
    let abcs = fbb.create_vector(&[off_more_defaults::ABC::B, off_more_defaults::ABC::C]);
    let bools = fbb.create_vector(&[true, false, true]);
    let offset = off_more_defaults::MoreDefaults::create(
        &mut fbb,
        &off_more_defaults::MoreDefaultsArgs {
            ints: Some(ints),
            floats: Some(floats),
            empty_string: Some(empty_str),
            some_string: Some(some_str),
            abcs: Some(abcs),
            bools: Some(bools),
        },
    );
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();

    let m = flatbuffers::root::<our_more_defaults::MoreDefaults>(buf).unwrap();
    assert_eq!(m.ints().iter().collect::<Vec<_>>(), vec![1, 2, 3]);
    assert_eq!(m.floats().iter().collect::<Vec<_>>(), vec![1.5, 2.5]);
    assert_eq!(m.empty_string(), "hello");
    assert_eq!(m.some_string(), "world");
    assert_eq!(
        m.abcs().iter().map(|a| a.0).collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(m.bools().iter().collect::<Vec<_>>(), vec![true, false, true]);
}

#[test]
fn cross_more_defaults_our_build_off_read() {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let ints = fbb.create_vector(&[10i32, 20]);
    let floats = fbb.create_vector(&[3.14f32]);
    let empty_str = fbb.create_string("test");
    let some_str = fbb.create_string("data");
    let abcs = fbb.create_vector(&[our_more_defaults::ABC::A]);
    let bools = fbb.create_vector(&[false]);
    let offset = our_more_defaults::createMoreDefaults(
        &mut fbb,
        &our_more_defaults::MoreDefaultsArgs {
            ints: Some(ints),
            floats: Some(floats),
            empty_string: Some(empty_str),
            some_string: Some(some_str),
            abcs: Some(abcs),
            bools: Some(bools),
        },
    );
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();

    let m = flatbuffers::root::<off_more_defaults::MoreDefaults>(buf).unwrap();
    assert_eq!(m.ints().iter().collect::<Vec<_>>(), vec![10, 20]);
    assert_eq!(m.floats().iter().collect::<Vec<_>>(), vec![3.14]);
    assert_eq!(m.empty_string(), "test");
    assert_eq!(m.some_string(), "data");
    assert_eq!(
        m.abcs().iter().map(|a| a.0).collect::<Vec<_>>(),
        vec![0]
    );
    assert_eq!(m.bools().iter().collect::<Vec<_>>(), vec![false]);
}

#[test]
fn cross_more_defaults_empty_defaults() {
    let buf: &[u8] = &[0; 4];
    let off = flatbuffers::root::<off_more_defaults::MoreDefaults>(buf).unwrap();
    let our = flatbuffers::root::<our_more_defaults::MoreDefaults>(buf).unwrap();

    assert_eq!(off.ints().len(), our.ints().len());
    assert_eq!(off.floats().len(), our.floats().len());
    assert_eq!(off.empty_string(), our.empty_string());
    assert_eq!(off.some_string(), our.some_string());
    assert_eq!(off.abcs().len(), our.abcs().len());
    assert_eq!(off.bools().len(), our.bools().len());
}

#[test]
fn cross_more_defaults_object_api_defaults() {
    let off_t = off_more_defaults::MoreDefaultsT::default();
    let our_t = our_more_defaults::MoreDefaultsT::default();

    assert_eq!(off_t.ints, our_t.ints);
    assert_eq!(off_t.floats, our_t.floats);
    assert_eq!(off_t.empty_string, our_t.empty_string);
    assert_eq!(off_t.some_string, our_t.some_string);
    assert_eq!(
        off_t.abcs.iter().map(|a| a.0).collect::<Vec<_>>(),
        our_t.abcs.iter().map(|a| a.0).collect::<Vec<_>>()
    );
    assert_eq!(off_t.bools, our_t.bools);
}

#[test]
fn cross_more_defaults_object_api_cross_pack() {
    // Pack with ours, unpack with official
    let our_t = our_more_defaults::MoreDefaultsT {
        ints: vec![1, 2, 3],
        floats: vec![1.5, 2.5],
        empty_string: "hello".to_string(),
        some_string: "world".to_string(),
        abcs: vec![our_more_defaults::ABC::B],
        bools: vec![true, false],
    };
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let offset = our_t.pack(&mut fbb);
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();

    let off_m = flatbuffers::root::<off_more_defaults::MoreDefaults>(buf).unwrap();
    let off_t = off_m.unpack();
    assert_eq!(off_t.ints, vec![1, 2, 3]);
    assert_eq!(off_t.floats, vec![1.5, 2.5]);
    assert_eq!(off_t.empty_string, "hello");
    assert_eq!(off_t.some_string, "world");
    assert_eq!(
        off_t.abcs.iter().map(|a| a.0).collect::<Vec<_>>(),
        vec![1]
    );
    assert_eq!(off_t.bools, vec![true, false]);
}

#[test]
fn cross_more_defaults_vtable_offsets() {
    use off_more_defaults::MoreDefaults as Off;
    use our_more_defaults::MoreDefaults as Our;
    assert_eq!(Off::VT_INTS, Our::VT_INTS);
    assert_eq!(Off::VT_FLOATS, Our::VT_FLOATS);
    assert_eq!(Off::VT_EMPTY_STRING, Our::VT_EMPTY_STRING);
    assert_eq!(Off::VT_SOME_STRING, Our::VT_SOME_STRING);
    assert_eq!(Off::VT_ABCS, Our::VT_ABCS);
    assert_eq!(Off::VT_BOOLS, Our::VT_BOOLS);
}

// ==========================================================================
// optional_scalars: namespaced, optional/default/just scalar fields
// ==========================================================================

#[test]
fn cross_optional_scalars_off_build_our_read() {
    use off_optional_scalars::optional_scalars as off_ns;
    use our_optional_scalars::optional_scalars as our_ns;

    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let offset = off_ns::ScalarStuff::create(
        &mut fbb,
        &off_ns::ScalarStuffArgs {
            just_i8: 5,
            maybe_i8: Some(10),
            default_i8: 99,
            just_u16: 1000,
            maybe_u16: Some(2000),
            default_u16: 3000,
            just_f32: 1.5,
            maybe_f32: Some(2.5),
            just_bool: true,
            maybe_bool: Some(false),
            default_bool: false,
            just_enum: off_ns::OptionalByte::One,
            maybe_enum: Some(off_ns::OptionalByte::Two),
            ..Default::default()
        },
    );
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();

    let s = flatbuffers::root::<our_ns::ScalarStuff>(buf).unwrap();
    assert_eq!(s.just_i8(), 5);
    assert_eq!(s.maybe_i8(), Some(10));
    assert_eq!(s.default_i8(), 99);
    assert_eq!(s.just_u16(), 1000);
    assert_eq!(s.maybe_u16(), Some(2000));
    assert_eq!(s.default_u16(), 3000);
    assert!((s.just_f32() - 1.5_f32).abs() < f32::EPSILON);
    assert_eq!(s.maybe_f32(), Some(2.5));
    assert_eq!(s.just_bool(), true);
    assert_eq!(s.maybe_bool(), Some(false));
    assert_eq!(s.default_bool(), false);
    assert_eq!(s.just_enum().0, 1); // OptionalByte::One
    assert_eq!(s.maybe_enum().map(|e| e.0), Some(2)); // OptionalByte::Two
}

#[test]
fn cross_optional_scalars_our_build_off_read() {
    use off_optional_scalars::optional_scalars as off_ns;
    use our_optional_scalars::optional_scalars as our_ns;

    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let offset = our_ns::createScalarStuff(
        &mut fbb,
        &our_ns::ScalarStuffArgs {
            just_i8: -128,
            maybe_i8: None,
            default_i8: 42,
            just_i64: i64::MAX,
            maybe_i64: Some(i64::MIN),
            just_f64: core::f64::consts::PI,
            maybe_bool: None,
            default_bool: true,
            ..Default::default()
        },
    );
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();

    let s = flatbuffers::root::<off_ns::ScalarStuff>(buf).unwrap();
    assert_eq!(s.just_i8(), -128);
    assert_eq!(s.maybe_i8(), None);
    assert_eq!(s.default_i8(), 42);
    assert_eq!(s.just_i64(), i64::MAX);
    assert_eq!(s.maybe_i64(), Some(i64::MIN));
    assert!((s.just_f64() - core::f64::consts::PI).abs() < f64::EPSILON);
    assert_eq!(s.maybe_bool(), None);
    assert_eq!(s.default_bool(), true);
}

#[test]
fn cross_optional_scalars_empty_defaults() {
    use off_optional_scalars::optional_scalars as off_ns;
    use our_optional_scalars::optional_scalars as our_ns;

    // Minimal valid table: 4 bytes with soffset pointing to an empty vtable
    let buf: &[u8] = &[0; 8];
    let off = flatbuffers::root::<off_ns::ScalarStuff>(buf).unwrap();
    let our = flatbuffers::root::<our_ns::ScalarStuff>(buf).unwrap();

    // just_* fields default to 0
    assert_eq!(off.just_i8(), our.just_i8());
    assert_eq!(off.just_u8(), our.just_u8());
    assert_eq!(off.just_i16(), our.just_i16());
    assert_eq!(off.just_u16(), our.just_u16());
    assert_eq!(off.just_i32(), our.just_i32());
    assert_eq!(off.just_u32(), our.just_u32());
    assert_eq!(off.just_i64(), our.just_i64());
    assert_eq!(off.just_u64(), our.just_u64());
    assert_eq!(off.just_f32(), our.just_f32());
    assert_eq!(off.just_f64(), our.just_f64());
    assert_eq!(off.just_bool(), our.just_bool());

    // maybe_* fields are None
    assert_eq!(off.maybe_i8(), our.maybe_i8());
    assert_eq!(off.maybe_u8(), our.maybe_u8());
    assert_eq!(off.maybe_i16(), our.maybe_i16());
    assert_eq!(off.maybe_u16(), our.maybe_u16());
    assert_eq!(off.maybe_i32(), our.maybe_i32());
    assert_eq!(off.maybe_u32(), our.maybe_u32());
    assert_eq!(off.maybe_i64(), our.maybe_i64());
    assert_eq!(off.maybe_u64(), our.maybe_u64());
    assert_eq!(off.maybe_f32(), our.maybe_f32());
    assert_eq!(off.maybe_f64(), our.maybe_f64());
    assert_eq!(off.maybe_bool(), our.maybe_bool());

    // default_* fields have schema defaults (42, true, One)
    assert_eq!(off.default_i8(), our.default_i8());
    assert_eq!(off.default_u8(), our.default_u8());
    assert_eq!(off.default_bool(), our.default_bool());
    assert_eq!(off.default_enum().0, our.default_enum().0);
}

#[test]
fn cross_optional_scalars_object_api_defaults() {
    use off_optional_scalars::optional_scalars as off_ns;
    use our_optional_scalars::optional_scalars as our_ns;

    let off_t = off_ns::ScalarStuffT::default();
    let our_t = our_ns::ScalarStuffT::default();

    assert_eq!(off_t.just_i8, our_t.just_i8);
    assert_eq!(off_t.maybe_i8, our_t.maybe_i8);
    assert_eq!(off_t.default_i8, our_t.default_i8);
    assert_eq!(off_t.just_u8, our_t.just_u8);
    assert_eq!(off_t.maybe_u8, our_t.maybe_u8);
    assert_eq!(off_t.default_u8, our_t.default_u8);
    assert_eq!(off_t.just_bool, our_t.just_bool);
    assert_eq!(off_t.maybe_bool, our_t.maybe_bool);
    assert_eq!(off_t.default_bool, our_t.default_bool);
    assert_eq!(off_t.just_enum.0, our_t.just_enum.0);
    assert_eq!(
        off_t.maybe_enum.map(|e| e.0),
        our_t.maybe_enum.map(|e| e.0)
    );
    assert_eq!(off_t.default_enum.0, our_t.default_enum.0);
}

#[test]
fn cross_optional_scalars_vtable_offsets() {
    use off_optional_scalars::optional_scalars as off_ns;
    use our_optional_scalars::optional_scalars as our_ns;

    assert_eq!(off_ns::ScalarStuff::VT_JUST_I8, our_ns::ScalarStuff::VT_JUST_I8);
    assert_eq!(off_ns::ScalarStuff::VT_MAYBE_I8, our_ns::ScalarStuff::VT_MAYBE_I8);
    assert_eq!(off_ns::ScalarStuff::VT_DEFAULT_I8, our_ns::ScalarStuff::VT_DEFAULT_I8);
    assert_eq!(off_ns::ScalarStuff::VT_JUST_BOOL, our_ns::ScalarStuff::VT_JUST_BOOL);
    assert_eq!(off_ns::ScalarStuff::VT_MAYBE_BOOL, our_ns::ScalarStuff::VT_MAYBE_BOOL);
    assert_eq!(off_ns::ScalarStuff::VT_DEFAULT_BOOL, our_ns::ScalarStuff::VT_DEFAULT_BOOL);
    assert_eq!(off_ns::ScalarStuff::VT_JUST_ENUM, our_ns::ScalarStuff::VT_JUST_ENUM);
    assert_eq!(off_ns::ScalarStuff::VT_MAYBE_ENUM, our_ns::ScalarStuff::VT_MAYBE_ENUM);
    assert_eq!(off_ns::ScalarStuff::VT_DEFAULT_ENUM, our_ns::ScalarStuff::VT_DEFAULT_ENUM);
}

// ==========================================================================
// monster_test: comprehensive schema with structs, enums, unions, namespaces
// ==========================================================================

#[test]
fn cross_monster_off_build_our_read() {
    use off_monster_test::my_game::example as off_ns;

    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("Orc");
    let inventory = fbb.create_vector(&[0u8, 1, 2, 3, 4]);

    let offset = off_ns::Monster::create(
        &mut fbb,
        &off_ns::MonsterArgs {
            hp: 300,
            mana: 150,
            name: Some(name),
            pos: Some(&off_ns::Vec3::new(1.0, 2.0, 3.0, 4.0, off_ns::Color::Green, &off_ns::Test::new(10, 20))),
            color: off_ns::Color::Blue,
            inventory: Some(inventory),
            ..Default::default()
        },
    );
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();

    let m = flatbuffers::root::<our_monster_test::Monster>(buf).unwrap();
    assert_eq!(m.hp(), 300);
    assert_eq!(m.mana(), 150);
    assert_eq!(m.name(), Some("Orc"));

    let pos = m.pos().unwrap();
    assert!((pos.x() - 1.0).abs() < f32::EPSILON);
    assert!((pos.y() - 2.0).abs() < f32::EPSILON);
    assert!((pos.z() - 3.0).abs() < f32::EPSILON);

    assert_eq!(m.color(), our_monster_test::Color::Blue);
    assert_eq!(
        m.inventory().unwrap().iter().collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4]
    );
}

#[test]
fn cross_monster_our_build_off_read() {
    use off_monster_test::my_game::example as off_ns;

    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("Goblin");
    let s1 = fbb.create_string("hello");
    let s2 = fbb.create_string("world");
    let testarrayofstring = fbb.create_vector(&[s1, s2]);

    let offset = our_monster_test::createMonster(
        &mut fbb,
        &our_monster_test::MonsterArgs {
            hp: 100,
            mana: 50,
            name: Some(name),
            pos: Some(&our_monster_test::Vec3::new(10.0, 20.0, 30.0, 0.0, our_monster_test::Color::Red, &our_monster_test::Test::new(5, 15))),
            testarrayofstring: Some(testarrayofstring),
            ..Default::default()
        },
    );
    fbb.finish_minimal(offset);
    let buf = fbb.finished_data();

    let m = flatbuffers::root::<off_ns::Monster>(buf).unwrap();
    assert_eq!(m.hp(), 100);
    assert_eq!(m.mana(), 50);
    assert_eq!(m.name(), "Goblin");

    let pos = m.pos().unwrap();
    assert!((pos.x() - 10.0).abs() < f32::EPSILON);
    assert!((pos.y() - 20.0).abs() < f32::EPSILON);
    assert!((pos.z() - 30.0).abs() < f32::EPSILON);

    let strings = m.testarrayofstring().unwrap();
    assert_eq!(strings.len(), 2);
    assert_eq!(strings.get(0), "hello");
    assert_eq!(strings.get(1), "world");
}

#[test]
fn cross_monster_enum_values() {
    use off_monster_test::my_game::example as off_ns;

    // Color bitflags: Red=1, Green=2, Blue=8
    assert_eq!(off_ns::Color::Red.bits(), our_monster_test::Color::Red.bits());
    assert_eq!(off_ns::Color::Green.bits(), our_monster_test::Color::Green.bits());
    assert_eq!(off_ns::Color::Blue.bits(), our_monster_test::Color::Blue.bits());

    // Race enum values
    assert_eq!(off_ns::Race::None.0, our_monster_test::Race::None.0);
    assert_eq!(off_ns::Race::Human.0, our_monster_test::Race::Human.0);
    assert_eq!(off_ns::Race::Dwarf.0, our_monster_test::Race::Dwarf.0);
    assert_eq!(off_ns::Race::Elf.0, our_monster_test::Race::Elf.0);

    // Any union discriminant values
    assert_eq!(off_ns::Any::NONE.0, our_monster_test::Any::NONE.0);
    assert_eq!(off_ns::Any::Monster.0, our_monster_test::Any::Monster.0);
    assert_eq!(
        off_ns::Any::TestSimpleTableWithEnum.0,
        our_monster_test::Any::TestSimpleTableWithEnum.0
    );
    assert_eq!(
        off_ns::Any::MyGame_Example2_Monster.0,
        our_monster_test::Any::MyGame_Example2_Monster.0
    );
}

#[test]
fn cross_monster_struct_layout() {
    use off_monster_test::my_game::example as off_ns;

    // Vec3: 3 floats + double + Color(u8) + pad(1) + Test(2 bytes) + pad(2) = 32 bytes
    assert_eq!(
        core::mem::size_of::<off_ns::Vec3>(),
        core::mem::size_of::<our_monster_test::Vec3>()
    );

    // Test: i16 + i8 + pad(1) = 4 bytes
    assert_eq!(
        core::mem::size_of::<off_ns::Test>(),
        core::mem::size_of::<our_monster_test::Test>()
    );

    // Ability: u32 + u32 = 8 bytes
    assert_eq!(
        core::mem::size_of::<off_ns::Ability>(),
        core::mem::size_of::<our_monster_test::Ability>()
    );
}

#[test]
fn cross_monster_file_identifier() {
    use off_monster_test::my_game::example as off_ns;

    assert_eq!(off_ns::MONSTER_IDENTIFIER, our_monster_test::MONSTER_IDENTIFIER);

    // Build with official, verify identifier with ours
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let name = fbb.create_string("Test");
    let offset = off_ns::Monster::create(
        &mut fbb,
        &off_ns::MonsterArgs {
            name: Some(name),
            ..Default::default()
        },
    );
    off_ns::finish_monster_buffer(&mut fbb, offset);
    let buf = fbb.finished_data();

    assert!(our_monster_test::monster_buffer_has_identifier(buf));
    assert!(off_ns::monster_buffer_has_identifier(buf));
}

#[test]
fn cross_monster_vtable_offsets() {
    use off_monster_test::my_game::example as off_ns;

    assert_eq!(off_ns::Monster::VT_POS, our_monster_test::Monster::VT_POS);
    assert_eq!(off_ns::Monster::VT_MANA, our_monster_test::Monster::VT_MANA);
    assert_eq!(off_ns::Monster::VT_HP, our_monster_test::Monster::VT_HP);
    assert_eq!(off_ns::Monster::VT_NAME, our_monster_test::Monster::VT_NAME);
    assert_eq!(off_ns::Monster::VT_INVENTORY, our_monster_test::Monster::VT_INVENTORY);
    assert_eq!(off_ns::Monster::VT_COLOR, our_monster_test::Monster::VT_COLOR);
    assert_eq!(off_ns::Monster::VT_TEST_TYPE, our_monster_test::Monster::VT_TEST_TYPE);
    assert_eq!(off_ns::Monster::VT_TEST, our_monster_test::Monster::VT_TEST);
    assert_eq!(off_ns::Monster::VT_ENEMY, our_monster_test::Monster::VT_ENEMY);
    assert_eq!(off_ns::Monster::VT_TESTNESTEDFLATBUFFER, our_monster_test::Monster::VT_TESTNESTEDFLATBUFFER);
}
