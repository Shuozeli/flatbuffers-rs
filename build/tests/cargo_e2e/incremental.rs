use crate::harness::{CargoProject, Fixture};

#[test]
fn skips_unchanged_schemas_and_rebuilds_transitive_changes() {
    // Arrange
    let project = CargoProject::new("incremental", Fixture::Basic);

    // Act
    let first = project.check();

    // Assert
    first.assert_success().assert_build_runs(1);
    let generated = project.generated("game_generated.rs");
    generated.assert_contains("pub struct Shared");

    // Act
    let second = project.check();

    // Assert
    second.assert_success().assert_build_runs(1);
    let original = generated.checkpoint();

    // Arrange
    project.write_schema("shared.fbs", "table Shared { value:int; label:string; }\n");

    // Act
    let third = project.check();

    // Assert
    third.assert_success().assert_build_runs(2);
    original.assert_changed();
    generated.assert_contains("pub fn label(");
}
