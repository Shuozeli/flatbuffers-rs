use flatc_rs_compiler::{
    analyze,
    codegen::{generate_python, PythonCodeGenOptions},
    parser::FbsParser,
};

fn generate_python_default(schema_src: &str) -> String {
    // Arrange
    let parser = FbsParser::new(schema_src).with_file_name("test.fbs".to_string());
    let parse_output = parser.parse().unwrap();
    let schema = analyze(parse_output).unwrap();
    let opts = PythonCodeGenOptions {
        gen_only_files: None,
    };

    // Act
    generate_python(&schema, &opts).unwrap()
}

#[test]
fn python_gen_struct_simple() {
    let code = generate_python_default("struct Vec3 { x: float; y: float; z: float; }");

    // Assert
    assert!(code.contains("@dataclass(slots=True)"));
    assert!(code.contains("class Vec3:"));
    assert!(code.contains("x: float = 0.0"));
    assert!(code.contains("y: float = 0.0"));
    assert!(code.contains("z: float = 0.0"));
}

#[test]
fn python_gen_table_basic() {
    let code = generate_python_default(
        "table Monster { hp: int; mana: short = 150; name: string; } root_type Monster;",
    );

    // Assert
    assert!(code.contains("class Monster:"));
    assert!(code.contains("hp: int = 0"));
    assert!(code.contains("mana: int = 150"));
    assert!(code.contains("name: str | None = None"));
}

#[test]
fn python_gen_all_scalar_types() {
    let code = generate_python_default(
        r#"
            table Scalars {
                b: bool;
                i8: byte;
                u8: ubyte;
                i16: short;
                u16: ushort;
                i32: int;
                u32: uint;
                i64: long;
                u64: ulong;
                f32: float;
                f64: double;
            }
            root_type Scalars;
        "#,
    );

    // Assert
    assert!(code.contains("b: bool = False"));
    assert!(code.contains("i8: int = 0"));
    assert!(code.contains("u8: int = 0"));
    assert!(code.contains("i16: int = 0"));
    assert!(code.contains("u16: int = 0"));
    assert!(code.contains("i32: int = 0"));
    assert!(code.contains("u32: int = 0"));
    assert!(code.contains("i64: int = 0"));
    assert!(code.contains("u64: int = 0"));
    assert!(code.contains("f32: float = 0.0"));
    assert!(code.contains("f64: float = 0.0"));
}

#[test]
fn python_gen_enum_basic_and_bitflags() {
    let code = generate_python_default(
        r#"
            enum Color: byte { Red = 1, Green = 2, Blue = 8 }
            enum Equipment: byte (bit_flags) { None = 0, Weapon = 1 }
            table Monster { color: Color; equipment: Equipment; }
            root_type Monster;
        "#,
    );

    // Assert
    assert!(code.contains("class Color(IntEnum):"));
    assert!(code.contains("RED = 1"));
    assert!(code.contains("GREEN = 2"));
    assert!(code.contains("class Equipment(IntEnum):"));
    assert!(code.contains("NONE = 0"));
    assert!(code.contains("WEAPON = 1"));
    assert!(code.contains("color: Color = 0"));
    assert!(code.contains("equipment: Equipment = 0"));
}

#[test]
fn python_gen_optional_scalars() {
    let code = generate_python_default("table Options { value: int = null; } root_type Options;");

    // Assert
    assert!(code.contains("value: int | None = None"));
}

#[test]
fn python_gen_namespace() {
    let code = generate_python_default(
        "namespace Game.Items; table Item { name: string; } root_type Item;",
    );

    // Assert
    assert!(code.contains("# namespace Game"));
    assert!(code.contains("# namespace Game.Items"));
    assert!(code.contains("class Item:"));
    assert!(code.contains("name: str | None = None"));
}

#[test]
fn python_gen_nested_struct() {
    let code = generate_python_default("struct Inner { x: int; } struct Outer { inner: Inner; }");

    // Assert
    assert!(code.contains("class Inner:"));
    assert!(code.contains("class Outer:"));
    assert!(code.contains("inner: Inner = field(default_factory=Inner)"));
}

#[test]
fn python_gen_vector_fields() {
    let code = generate_python_default(
        r#"
            table Item { name: string; }
            table Monster {
                items: [int];
                names: [string];
                children: [Item];
            }
            root_type Monster;
        "#,
    );

    // Assert
    assert!(code.contains("items: list[int] = field(default_factory=list)"));
    assert!(code.contains("names: list[str] = field(default_factory=list)"));
    assert!(code.contains("children: list[Item] = field(default_factory=list)"));
}

#[test]
fn python_gen_keyword_escape() {
    let code = generate_python_default(
        "table Keywords { type: int; class: string; match: bool; } root_type Keywords;",
    );

    // Assert
    assert!(code.contains("type_: int = 0"));
    assert!(code.contains("class_: str | None = None"));
    assert!(code.contains("match_: bool = False"));
}

#[test]
fn python_gen_union_field() {
    let code = generate_python_default(
        r#"
            table Weapon { name: string; }
            table Spell { power: int; }
            union AnyItem { Weapon, Spell }
            table Inventory { item: AnyItem; }
            root_type Inventory;
        "#,
    );

    // Assert
    assert!(code.contains("class AnyItem(IntEnum):"));
    assert!(code.contains("WEAPON = 1"));
    assert!(code.contains("SPELL = 2"));
    assert!(code.contains("item_type: AnyItem = 0"));
    assert!(code.contains("item: Any | None = None"));
}

#[test]
fn python_gen_complex_combined() {
    let code = generate_python_default(
        r#"
            namespace Game.Sample;

            enum Color: byte { Red = 1, Green = 2, Blue = 8 }
            struct Vec3 { x: float; y: float; z: float; }
            table Weapon { name: string; damage: short = 10; }
            union Equipment { Weapon }
            table Monster {
                pos: Vec3;
                mana: short = 150;
                hp: short = 100;
                name: string;
                inventory: [ubyte];
                color: Color = Blue;
                weapons: [Weapon];
                equipped: Equipment;
            }
            root_type Monster;
        "#,
    );

    // Assert
    assert!(code.contains("# namespace Game.Sample"));
    assert!(code.contains("class Color(IntEnum):"));
    assert!(code.contains("class Vec3:"));
    assert!(code.contains("class Weapon:"));
    assert!(code.contains("damage: int = 10"));
    assert!(code.contains("class Equipment(IntEnum):"));
    assert!(code.contains("class Monster:"));
    assert!(code.contains("pos: Vec3 | None = None"));
    assert!(code.contains("mana: int = 150"));
    assert!(code.contains("hp: int = 100"));
    assert!(code.contains("name: str | None = None"));
    assert!(code.contains("inventory: list[int] = field(default_factory=list)"));
    assert!(code.contains("color: Color = Color.BLUE"));
    assert!(code.contains("weapons: list[Weapon] = field(default_factory=list)"));
    assert!(code.contains("equipped: Any | None = None"));
}
