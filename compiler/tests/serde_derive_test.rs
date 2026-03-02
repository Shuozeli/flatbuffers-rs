extern crate flatbuffers;
extern crate serde;
extern crate serde_json;

#[allow(dead_code, unused_imports, non_camel_case_types, non_snake_case)]
mod serde_test_generated {
    include!("../testdata/serde_codegen_golden/serde_test.expected");
}

#[test]
fn test_enum_serialization() {
    use serde_test_generated::serde_test::Color;
    let c = Color::Red;
    let json = serde_json::to_string(&c).unwrap();
    assert_eq!(json, r#""Red""#);

    let c2: Color = serde_json::from_str(r#""Green""#).unwrap();
    assert_eq!(c2, Color::Green);

    // Should also support numeric deserialization
    let c3: Color = serde_json::from_str("8").unwrap();
    assert_eq!(c3, Color::Blue);
}

#[test]
fn test_bitflags_serialization() {
    use serde_test_generated::serde_test::Flags;
    let f = Flags::A | Flags::C;
    let json = serde_json::to_string(&f).unwrap();
    assert_eq!(json, "5");

    let f2: Flags = serde_json::from_str("5").unwrap();
    assert_eq!(f2, Flags::A | Flags::C);
}

#[test]
fn test_struct_serialization() {
    use serde_test_generated::serde_test::Vec3;
    let v = Vec3::new(1.0, 2.0, 3.0);
    let json = serde_json::to_string(&v).unwrap();
    assert!(json.contains(r#""x":1.0"#));
    assert!(json.contains(r#""y":2.0"#));
    assert!(json.contains(r#""z":3.0"#));
}

#[test]
fn test_object_api_serialization() {
    use serde_test_generated::serde_test::{Color, Flags, MonsterT, Vec3T};
    let m = MonsterT {
        pos: Some(Vec3T {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        }),
        hp: 42,
        name: Some("SerdeMonster".to_string()),
        color: Color::Blue,
        inventory: Some(vec![1, 2, 3]),
        flags: Flags::A | Flags::B,
    };
    let json = serde_json::to_string(&m).unwrap();
    assert!(json.contains(r#""name":"SerdeMonster""#));
    assert!(json.contains(r#""hp":42"#));
    assert!(json.contains(r#""color":"Blue""#));
    assert!(json.contains(r#""flags":3"#));

    let m2: MonsterT = serde_json::from_str(&json).unwrap();
    assert_eq!(m, m2);
}
