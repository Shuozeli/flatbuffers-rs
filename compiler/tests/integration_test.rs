use std::fs;
use std::path::PathBuf;

use flatc_rs_compiler::{compile, compile_single, CompilerOptions};
use flatc_rs_schema::BaseType;

#[test]
fn compile_single_schema() {
    let result = compile_single(
        "table Monster { name:string; hp:short; }\nroot_type Monster;",
    )
    .unwrap();

    assert_eq!(result.schema.objects.len(), 1);
    assert_eq!(
        result.schema.objects[0].name.as_deref(),
        Some("Monster")
    );
    assert!(result.schema.root_table.is_some());
    assert_eq!(
        result.schema.root_table.as_ref().unwrap().name.as_deref(),
        Some("Monster")
    );
}

#[test]
fn compile_with_include() {
    let dir = tempfile::tempdir().unwrap();

    fs::write(
        dir.path().join("vec3.fbs"),
        "struct Vec3 { x:float; y:float; z:float; }\n",
    )
    .unwrap();

    fs::write(
        dir.path().join("monster.fbs"),
        "include \"vec3.fbs\";\ntable Monster { pos:Vec3; name:string; }\nroot_type Monster;\n",
    )
    .unwrap();

    let result = compile(
        &[dir.path().join("monster.fbs")],
        &CompilerOptions::default(),
    )
    .unwrap();

    // Should have Vec3 struct + Monster table
    assert_eq!(result.schema.objects.len(), 2);

    // Vec3 should be a struct with layout computed
    let vec3 = &result.schema.objects[0];
    assert_eq!(vec3.name.as_deref(), Some("Vec3"));
    assert_eq!(vec3.is_struct, Some(true));
    assert_eq!(vec3.byte_size, Some(12));
    assert_eq!(vec3.min_align, Some(4));

    // Monster should reference Vec3 as STRUCT with index 0
    let monster = &result.schema.objects[1];
    let pos_field = &monster.fields[0];
    let pos_type = pos_field.type_.as_ref().unwrap();
    assert_eq!(
        pos_type.base_type,
        Some(BaseType::BASE_TYPE_STRUCT)
    );
    assert_eq!(pos_type.index, Some(0));

    // Root table should be set
    assert_eq!(
        result.schema.root_table.as_ref().unwrap().name.as_deref(),
        Some("Monster")
    );
}

#[test]
fn compile_with_include_search_path() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("sub");
    fs::create_dir(&sub).unwrap();

    fs::write(
        sub.join("types.fbs"),
        "enum Color:byte { Red, Green, Blue }\n",
    )
    .unwrap();

    fs::write(
        dir.path().join("main.fbs"),
        "include \"types.fbs\";\ntable Widget { color:Color; }\n",
    )
    .unwrap();

    // Without include path, should fail
    let err = compile(
        &[dir.path().join("main.fbs")],
        &CompilerOptions::default(),
    );
    assert!(err.is_err());

    // With include path pointing to sub/, should succeed
    let options = CompilerOptions {
        include_paths: vec![sub.clone()],
    };
    let result = compile(&[dir.path().join("main.fbs")], &options).unwrap();

    assert_eq!(result.schema.enums.len(), 1);
    assert_eq!(
        result.schema.enums[0].name.as_deref(),
        Some("Color")
    );

    // Color field should be resolved to enum's underlying type
    let widget = &result.schema.objects[0];
    let color_field = &widget.fields[0];
    let color_type = color_field.type_.as_ref().unwrap();
    assert_eq!(
        color_type.base_type,
        Some(BaseType::BASE_TYPE_BYTE)
    );
    assert_eq!(color_type.index, Some(0));
}

#[test]
fn compile_circular_include() {
    let dir = tempfile::tempdir().unwrap();

    // a.fbs includes b.fbs, b.fbs includes a.fbs
    fs::write(
        dir.path().join("a.fbs"),
        "include \"b.fbs\";\ntable TableA { name:string; }\n",
    )
    .unwrap();

    fs::write(
        dir.path().join("b.fbs"),
        "include \"a.fbs\";\ntable TableB { value:int; }\n",
    )
    .unwrap();

    // Should handle circular includes gracefully (not error)
    let result = compile(
        &[dir.path().join("a.fbs")],
        &CompilerOptions::default(),
    )
    .unwrap();

    // Both tables should be in the merged schema
    assert_eq!(result.schema.objects.len(), 2);
    let names: Vec<&str> = result
        .schema
        .objects
        .iter()
        .map(|o| o.name.as_deref().unwrap_or(""))
        .collect();
    assert!(names.contains(&"TableA"));
    assert!(names.contains(&"TableB"));
}

#[test]
fn compile_duplicate_include_deduplication() {
    let dir = tempfile::tempdir().unwrap();

    fs::write(
        dir.path().join("types.fbs"),
        "struct Vec2 { x:float; y:float; }\n",
    )
    .unwrap();

    // Include same file twice
    fs::write(
        dir.path().join("main.fbs"),
        "include \"types.fbs\";\ninclude \"types.fbs\";\ntable Widget { pos:Vec2; }\n",
    )
    .unwrap();

    let result = compile(
        &[dir.path().join("main.fbs")],
        &CompilerOptions::default(),
    )
    .unwrap();

    // Vec2 should appear only once (deduplication)
    let vec2_count = result
        .schema
        .objects
        .iter()
        .filter(|o| o.name.as_deref() == Some("Vec2"))
        .count();
    assert_eq!(vec2_count, 1);
}

#[test]
fn compile_missing_include_error() {
    let dir = tempfile::tempdir().unwrap();

    fs::write(
        dir.path().join("main.fbs"),
        "include \"nonexistent.fbs\";\ntable Foo { x:int; }\n",
    )
    .unwrap();

    let err = compile(
        &[dir.path().join("main.fbs")],
        &CompilerOptions::default(),
    );

    assert!(err.is_err());
    let msg = err.unwrap_err().to_string();
    assert!(msg.contains("nonexistent.fbs"), "error: {msg}");
}

#[test]
fn compile_missing_file_error() {
    let err = compile(
        &[PathBuf::from("/nonexistent/path/foo.fbs")],
        &CompilerOptions::default(),
    );
    assert!(err.is_err());
}

#[test]
fn compile_cross_file_type_resolution() {
    let dir = tempfile::tempdir().unwrap();

    fs::write(
        dir.path().join("types.fbs"),
        "\
namespace Game;
enum Color:byte { Red, Green, Blue }
struct Vec3 { x:float; y:float; z:float; }
union Equipment { Weapon }
table Weapon { damage:short; }
",
    )
    .unwrap();

    fs::write(
        dir.path().join("monster.fbs"),
        "\
include \"types.fbs\";
namespace Game;
table Monster {
  pos:Vec3;
  color:Color;
  equipped:Equipment;
}
root_type Monster;
",
    )
    .unwrap();

    let result = compile(
        &[dir.path().join("monster.fbs")],
        &CompilerOptions::default(),
    )
    .unwrap();

    // Find Monster table
    let monster = result
        .schema
        .objects
        .iter()
        .find(|o| o.name.as_deref() == Some("Monster"))
        .unwrap();

    // pos should be resolved to STRUCT
    let pos_type = monster.fields[0].type_.as_ref().unwrap();
    assert_eq!(
        pos_type.base_type,
        Some(BaseType::BASE_TYPE_STRUCT)
    );

    // color should be resolved to BYTE (enum underlying type)
    let color_type = monster.fields[1].type_.as_ref().unwrap();
    assert_eq!(
        color_type.base_type,
        Some(BaseType::BASE_TYPE_BYTE)
    );

    // equipped_type companion field should be U_TYPE (auto-inserted for unions)
    let equipped_type_field = monster.fields[2].type_.as_ref().unwrap();
    assert_eq!(
        equipped_type_field.base_type,
        Some(BaseType::BASE_TYPE_U_TYPE)
    );
    assert_eq!(
        monster.fields[2].name.as_deref(),
        Some("equipped_type")
    );

    // equipped should be resolved to UNION
    let equipped_union = monster.fields[3].type_.as_ref().unwrap();
    assert_eq!(
        equipped_union.base_type,
        Some(BaseType::BASE_TYPE_UNION)
    );
}
