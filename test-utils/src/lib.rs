use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Options controlling golden test behavior.
#[derive(Debug, Clone)]
pub struct GoldenTestOptions {
    /// When true, overwrite .expected files with actual results.
    pub update: bool,
}

impl GoldenTestOptions {
    pub fn check_only() -> Self {
        Self { update: false }
    }

    pub fn update_golden() -> Self {
        Self { update: true }
    }

    /// Read mode from UPDATE_GOLDEN environment variable.
    pub fn from_env() -> Self {
        if std::env::var("UPDATE_GOLDEN").is_ok() {
            Self::update_golden()
        } else {
            Self::check_only()
        }
    }
}

/// A single discovered test case: a .fbs input file with a paired .expected file.
#[derive(Debug, Clone)]
pub struct GoldenTestCase {
    /// Test name derived from file stem (e.g., "empty_table").
    pub name: String,
    /// Path to the .fbs input file.
    pub input_path: PathBuf,
    /// Path to the .expected output file (may not exist yet).
    pub expected_path: PathBuf,
}

/// Discover all golden test cases in a directory.
///
/// Scans `dir` for `*.fbs` files. For each, the expected output file
/// is the same path with `.fbs` replaced by `.expected`.
/// Returns cases sorted by name for deterministic ordering.
pub fn discover_golden_tests(dir: &Path) -> io::Result<Vec<GoldenTestCase>> {
    let mut cases = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("fbs") {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "non-UTF8 file name"))?
                .to_string();
            let expected_path = path.with_extension("expected");
            cases.push(GoldenTestCase {
                name,
                input_path: path,
                expected_path,
            });
        }
    }
    cases.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(cases)
}

/// Run a single golden test case.
///
/// Reads the input .fbs file, applies `transform`, and compares
/// against the .expected file. In update mode, writes the actual
/// output to the .expected file.
pub fn run_golden_test<F>(
    case: &GoldenTestCase,
    transform: &F,
    options: &GoldenTestOptions,
) -> io::Result<()>
where
    F: Fn(&str) -> String,
{
    let input = fs::read_to_string(&case.input_path)?;
    let actual = transform(&input);
    let expected = fs::read_to_string(&case.expected_path).unwrap_or_default();

    if actual != expected {
        if options.update {
            fs::write(&case.expected_path, &actual)?;
            return Ok(());
        }
        panic!(
            "golden test '{}' failed.\n\
             Input: {}\n\
             Expected: {}\n\
             Set UPDATE_GOLDEN=1 to regenerate.\n\
             --- expected ---\n{expected}\n\
             --- actual ---\n{actual}",
            case.name,
            case.input_path.display(),
            case.expected_path.display(),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_discover_golden_tests() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("alpha.fbs"), "table A {}").unwrap();
        fs::write(dir.path().join("alpha.expected"), "output A").unwrap();
        fs::write(dir.path().join("beta.fbs"), "table B {}").unwrap();
        fs::write(dir.path().join("beta.expected"), "output B").unwrap();
        fs::write(dir.path().join("not_a_test.txt"), "ignore").unwrap();

        let cases = discover_golden_tests(dir.path()).unwrap();
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].name, "alpha");
        assert_eq!(cases[1].name, "beta");
    }

    #[test]
    fn test_run_golden_test_pass() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.fbs"), "input").unwrap();
        fs::write(dir.path().join("test.expected"), "INPUT").unwrap();

        let case = GoldenTestCase {
            name: "test".into(),
            input_path: dir.path().join("test.fbs"),
            expected_path: dir.path().join("test.expected"),
        };

        run_golden_test(&case, &|s: &str| s.to_uppercase(), &GoldenTestOptions::check_only())
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "golden test 'test' failed")]
    fn test_run_golden_test_fail() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.fbs"), "input").unwrap();
        fs::write(dir.path().join("test.expected"), "wrong").unwrap();

        let case = GoldenTestCase {
            name: "test".into(),
            input_path: dir.path().join("test.fbs"),
            expected_path: dir.path().join("test.expected"),
        };

        run_golden_test(&case, &|s: &str| s.to_uppercase(), &GoldenTestOptions::check_only())
            .unwrap();
    }

    #[test]
    fn test_run_golden_test_update() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.fbs"), "input").unwrap();
        fs::write(dir.path().join("test.expected"), "old").unwrap();

        let case = GoldenTestCase {
            name: "test".into(),
            input_path: dir.path().join("test.fbs"),
            expected_path: dir.path().join("test.expected"),
        };

        run_golden_test(
            &case,
            &|s: &str| s.to_uppercase(),
            &GoldenTestOptions::update_golden(),
        )
        .unwrap();

        let updated = fs::read_to_string(dir.path().join("test.expected")).unwrap();
        assert_eq!(updated, "INPUT");
    }
}
