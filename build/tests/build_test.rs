use std::fs;

use flatc_rs_build::{Builder, Error};
use flatc_rs_compiler::CompilerError;

#[test]
fn compile_tracks_transitive_sources_and_preserves_unchanged_output() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    let schemas = directory.path().join("schemas");
    let output = directory.path().join("out");
    fs::create_dir(&schemas).unwrap();
    let shared = schemas.join("shared.fbs");
    let game = schemas.join("game.fbs");
    fs::write(&shared, "struct Vec3 { x:float; y:float; z:float; }\n").unwrap();
    fs::write(
        &game,
        "include \"shared.fbs\";\ntable Monster { pos:Vec3; }\nroot_type Monster;\n",
    )
    .unwrap();
    let builder = Builder::new()
        .schema(&game)
        .out_dir(&output)
        .rerun_if_env_changed("FLATC_RS_OPTIONS");

    // Act
    let mut first_directives = Vec::new();
    let first = builder
        .clone()
        .compile_with_writer(&mut first_directives)
        .unwrap();
    let mut second_directives = Vec::new();
    let second = builder.compile_with_writer(&mut second_directives).unwrap();

    // Assert
    let generated = output.join("game_generated.rs");
    assert_eq!(first.generated_files, vec![generated.clone()]);
    assert_eq!(first.updated_files, vec![generated.clone()]);
    assert_eq!(second.generated_files, vec![generated.clone()]);
    assert!(second.updated_files.is_empty());
    assert_eq!(first.source_files, second.source_files);
    assert_eq!(first_directives, second_directives);
    let directives = String::from_utf8(first_directives).unwrap();
    assert!(directives.contains(&format!(
        "cargo::rerun-if-changed={}",
        fs::canonicalize(shared).unwrap().display()
    )));
    assert!(directives.contains(&format!(
        "cargo::rerun-if-changed={}",
        fs::canonicalize(game).unwrap().display()
    )));
    assert!(directives.contains("cargo::rerun-if-env-changed=FLATC_RS_OPTIONS"));
    assert!(fs::read_to_string(generated)
        .unwrap()
        .contains("pub struct Monster"));
}

#[test]
fn compile_updates_gen_all_output_when_an_include_changes() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("out");
    let shared = directory.path().join("shared.fbs");
    let game = directory.path().join("game.fbs");
    fs::write(&shared, "table Shared { value:int; }\n").unwrap();
    fs::write(
        &game,
        "include \"shared.fbs\";\ntable Game { shared:Shared; }\nroot_type Game;\n",
    )
    .unwrap();
    let builder = Builder::new().schema(&game).out_dir(&output).gen_all();
    builder
        .clone()
        .compile_with_writer(&mut Vec::new())
        .unwrap();
    fs::write(&shared, "table Shared { value:int; label:string; }\n").unwrap();

    // Act
    let result = builder.compile_with_writer(&mut Vec::new()).unwrap();

    // Assert
    assert_eq!(result.updated_files, result.generated_files);
    let generated = fs::read_to_string(&result.generated_files[0]).unwrap();
    assert!(generated.contains("pub fn label("));
}

#[test]
fn compile_rejects_duplicate_output_paths() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    let first_dir = directory.path().join("first");
    let second_dir = directory.path().join("second");
    fs::create_dir(&first_dir).unwrap();
    fs::create_dir(&second_dir).unwrap();
    let first = first_dir.join("schema.fbs");
    let second = second_dir.join("schema.fbs");
    fs::write(&first, "table First { value:int; }\n").unwrap();
    fs::write(&second, "table Second { value:int; }\n").unwrap();

    // Act
    let error = Builder::new()
        .schemas([first, second])
        .out_dir(directory.path().join("out"))
        .compile_with_writer(&mut Vec::new())
        .unwrap_err();

    // Assert
    assert!(matches!(error, Error::OutputCollision { .. }));
}

#[test]
fn compile_requires_at_least_one_schema() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();

    // Act
    let error = Builder::new()
        .out_dir(directory.path())
        .compile_with_writer(&mut Vec::new())
        .unwrap_err();

    // Assert
    assert!(matches!(error, Error::NoSchemas));
}

#[test]
fn compile_generates_multiple_schemas_with_a_custom_suffix() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first.fbs");
    let second = directory.path().join("second.fbs");
    let output = directory.path().join("out");
    fs::write(&first, "table First { value:int; }\n").unwrap();
    fs::write(&second, "table Second { value:string; }\n").unwrap();

    // Act
    let result = Builder::new()
        .schemas([&first, &second])
        .out_dir(&output)
        .filename_suffix("_fbs")
        .compile_with_writer(&mut Vec::new())
        .unwrap();

    // Assert
    assert_eq!(
        result.generated_files,
        vec![output.join("first_fbs.rs"), output.join("second_fbs.rs")]
    );
    assert_eq!(result.updated_files, result.generated_files);
    assert!(fs::read_to_string(&result.generated_files[0])
        .unwrap()
        .contains("pub struct First"));
    assert!(fs::read_to_string(&result.generated_files[1])
        .unwrap()
        .contains("pub struct Second"));
}

#[test]
fn compile_reports_a_missing_transitive_include() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    let schema = directory.path().join("game.fbs");
    fs::write(
        &schema,
        "include \"missing.fbs\";\ntable Game { value:int; }\n",
    )
    .unwrap();

    // Act
    let error = Builder::new()
        .schema(schema)
        .out_dir(directory.path().join("out"))
        .compile_with_writer(&mut Vec::new())
        .unwrap_err();

    // Assert
    assert!(matches!(
        error,
        Error::Compiler(CompilerError::IncludeNotFound { include, .. })
            if include == "missing.fbs"
    ));
}

#[test]
fn no_leak_private_rejects_a_public_type_exposing_a_private_type() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    let schema = directory.path().join("private.fbs");
    fs::write(
        &schema,
        "table PrivateInner (private) { value:int; }\ntable PublicOuter { inner:PrivateInner; }\n",
    )
    .unwrap();

    // Act
    let error = Builder::new()
        .schema(schema)
        .out_dir(directory.path().join("out"))
        .no_leak_private()
        .compile_with_writer(&mut Vec::new())
        .unwrap_err();

    // Assert
    assert!(matches!(
        error,
        Error::Compiler(CompilerError::AnalyzeError(_))
    ));
}
