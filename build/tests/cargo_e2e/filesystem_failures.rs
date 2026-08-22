use crate::harness::{CargoProject, Fixture};

#[test]
fn rejects_include_path_traversal_preserves_output_and_recovers() {
    // Arrange
    let project = CargoProject::new("path-traversal", Fixture::Basic);
    project.write_file("outside.fbs", "table Outside { secret:string; }\n");
    project.check().assert_success().assert_build_runs(1);
    let initial = project.generated("game_generated.rs").checkpoint();

    // Act
    project.write_schema(
        "shared.fbs",
        "include \"../outside.fbs\";\ntable Shared { value:int; }\n",
    );
    let escaping = project.check();

    // Assert
    escaping
        .assert_error("PathTraversal", &["../outside.fbs", "shared.fbs"])
        .assert_build_runs(2);
    initial.assert_unchanged();

    // Act
    project.restore_schema("shared.fbs");
    let repaired = project.check();

    // Assert
    repaired.assert_success().assert_build_runs(3);
    initial.assert_unchanged();
}

#[test]
fn reports_include_io_failures_preserves_output_and_recovers() {
    // Arrange
    let project = CargoProject::new("include-io-failure", Fixture::Basic);
    project.check().assert_success().assert_build_runs(1);
    let initial = project.generated("game_generated.rs").checkpoint();

    // Act
    project.create_schema_directory("unreadable.fbs");
    project.write_schema(
        "shared.fbs",
        "include \"unreadable.fbs\";\ntable Shared { value:int; }\n",
    );
    let unreadable = project.check();

    // Assert
    unreadable
        .assert_error("IoError", &["unreadable.fbs"])
        .assert_build_runs(2);
    initial.assert_unchanged();

    // Act
    project.remove_schema_directory("unreadable.fbs");
    project.restore_schema("shared.fbs");
    let repaired = project.check();

    // Assert
    repaired.assert_success().assert_build_runs(3);
    initial.assert_unchanged();
}
