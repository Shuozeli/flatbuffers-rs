use crate::harness::{CargoProject, Fixture};

#[test]
fn reports_transitive_failures_preserves_output_and_recovers() {
    // Arrange
    let project = CargoProject::new("transitive-failures", Fixture::Basic);
    project.check().assert_success().assert_build_runs(1);
    let generated = project.generated("game_generated.rs");
    let initial = generated.checkpoint();

    // Act
    project.write_schema("shared.fbs", "table Shared { value:; }\n");
    let malformed = project.check();

    // Assert
    malformed
        .assert_error("ParseError", &["shared.fbs"])
        .assert_build_runs(2);
    initial.assert_unchanged();

    // Act
    project.write_schema("shared.fbs", "table Shared { value:int; label:string; }\n");
    let repaired = project.check();

    // Assert
    repaired.assert_success().assert_build_runs(3);
    initial.assert_changed();
    generated.assert_contains("pub fn label(");
    let repaired_output = generated.checkpoint();

    // Act
    project.remove_schema("shared.fbs");
    let missing = project.check();

    // Assert
    missing
        .assert_error("IncludeNotFound", &["shared.fbs"])
        .assert_build_runs(4);
    repaired_output.assert_unchanged();

    // Act
    project.write_schema("shared.fbs", "table Shared { value:int; label:string; }\n");
    let restored = project.check();

    // Assert
    restored.assert_success().assert_build_runs(5);
    repaired_output.assert_unchanged();
}

#[test]
fn reports_include_cycles_preserves_output_and_recovers() {
    // Arrange
    let project = CargoProject::new("include-cycle", Fixture::Basic);
    project.check().assert_success().assert_build_runs(1);
    let generated = project.generated("game_generated.rs");
    let initial = generated.checkpoint();

    // Act
    project.write_schema(
        "shared.fbs",
        "include \"game.fbs\";\ntable Shared { value:int; }\n",
    );
    let cyclic = project.check();

    // Assert
    cyclic
        .assert_error("IncludeCycle", &["game.fbs"])
        .assert_build_runs(2);
    initial.assert_unchanged();

    // Act
    project.restore_schema("shared.fbs");
    let repaired = project.check();

    // Assert
    repaired.assert_success().assert_build_runs(3);
    initial.assert_unchanged();
}
