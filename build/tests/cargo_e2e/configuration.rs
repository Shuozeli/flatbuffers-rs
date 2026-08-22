use crate::harness::{CargoProject, Fixture};

#[test]
fn reports_duplicate_generated_output_names() {
    // Arrange
    let project = CargoProject::new("output-collision", Fixture::Collision);

    // Act
    let output = project.check();

    // Assert
    output.assert_error("OutputCollision", &["schema_generated.rs"]);
}
