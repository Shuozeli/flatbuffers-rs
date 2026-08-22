use crate::harness::{CargoProject, Fixture};

const OUTPUT_DIR_ENV: &str = "FLATC_RS_E2E_OUT_DIR";

#[test]
fn reports_output_read_failures_and_recovers() {
    // Arrange
    let project = CargoProject::new("output-read-failure", Fixture::Basic);
    project.check().assert_success().assert_build_runs(1);
    let generated = project.generated("game_generated.rs");
    generated.replace_with_directory();
    project.write_schema("shared.fbs", "table Shared { value:int; label:string; }\n");

    // Act
    let unreadable = project.check();

    // Assert
    unreadable
        .assert_error("Io {", &["operation: \"read\"", "game_generated.rs"])
        .assert_build_runs(2);

    // Act
    generated.remove_directory();
    let repaired = project.check();

    // Assert
    repaired.assert_success().assert_build_runs(3);
    generated.assert_contains("pub fn label(");
}

#[cfg(unix)]
#[test]
fn reports_output_directory_creation_failures_preserves_output_and_recovers() {
    // Arrange
    let mut project = CargoProject::new("output-directory-failure", Fixture::Basic);
    project.check().assert_success().assert_build_runs(1);
    let generated = project.generated("game_generated.rs");
    let initial = generated.checkpoint();
    project.create_directory("blocked-parent");
    project.set_path_read_only("blocked-parent", true);
    let blocked_output = project.path("blocked-parent/generated");
    project.set_env(OUTPUT_DIR_ENV, blocked_output.as_os_str());

    // Act
    let blocked = project.check();

    // Assert
    blocked
        .assert_error(
            "Io {",
            &["operation: \"create directory\"", "blocked-parent"],
        )
        .assert_build_runs(2);
    initial.assert_unchanged();

    // Act
    project.set_path_read_only("blocked-parent", false);
    project.remove_env(OUTPUT_DIR_ENV);
    let repaired = project.check();

    // Assert
    repaired.assert_success().assert_build_runs(3);
    initial.assert_unchanged();
}

#[cfg(unix)]
#[test]
fn reports_output_write_failures_preserves_output_and_recovers() {
    // Arrange
    let project = CargoProject::new("output-write-failure", Fixture::Basic);
    project.check().assert_success().assert_build_runs(1);
    let generated = project.generated("game_generated.rs");
    let initial = generated.checkpoint();
    generated.set_read_only(true);
    project.write_schema("shared.fbs", "table Shared { value:int; label:string; }\n");

    // Act
    let unwritable = project.check();

    // Assert
    unwritable
        .assert_error("Io {", &["operation: \"write\"", "game_generated.rs"])
        .assert_build_runs(2);
    initial.assert_unchanged();

    // Act
    generated.set_read_only(false);
    let repaired = project.check();

    // Assert
    repaired.assert_success().assert_build_runs(3);
    initial.assert_changed();
    generated.assert_contains("pub fn label(");
}
