use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use filetime::{set_file_mtime, FileTime};

const BASIC_BUILD_RS: &str = include_str!("../fixtures/basic/build.rs");
const BASIC_LIB_RS: &str = include_str!("../fixtures/basic/src/lib.rs");
const BASIC_GAME_FBS: &str = include_str!("../fixtures/basic/schemas/game.fbs");
const BASIC_SHARED_FBS: &str = include_str!("../fixtures/basic/schemas/shared.fbs");
const COLLISION_BUILD_RS: &str = include_str!("../fixtures/collision/build.rs");
const COLLISION_LIB_RS: &str = include_str!("../fixtures/collision/src/lib.rs");
const COLLISION_FIRST_FBS: &str = include_str!("../fixtures/collision/schemas/first/schema.fbs");
const COLLISION_SECOND_FBS: &str = include_str!("../fixtures/collision/schemas/second/schema.fbs");
const CHECKPOINT_MTIME_SECONDS: i64 = 1_600_000_000;
const FIXTURE_OUT_DIR_ENV: &str = "FLATC_RS_E2E_OUT_DIR";
const OUT_DIR_POINTER: &str = ".flatc-rs-build-out-dir";
const RUN_COUNTER: &str = "flatc-rs-build-runs";

static NEXT_PROJECT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Copy)]
pub enum Fixture {
    Basic,
    Collision,
}

pub struct CargoProject {
    directory: Option<tempfile::TempDir>,
    root: PathBuf,
    target: PathBuf,
    fixture: Fixture,
    environment: BTreeMap<OsString, OsString>,
}

impl CargoProject {
    pub fn new(name: &str, fixture: Fixture) -> Self {
        let directory = tempfile::Builder::new()
            .prefix(&format!("flatc-rs-build-e2e-{name}-"))
            .tempdir()
            .unwrap();
        let root = directory.path().join("consumer");
        let target = if env_flag("FLATC_RS_E2E_ISOLATED_TARGET") {
            directory.path().join("target")
        } else {
            shared_target_dir()
        };
        let project_id = NEXT_PROJECT_ID.fetch_add(1, Ordering::Relaxed);
        let process_id = std::process::id();
        let package_name = format!(
            "flatc-rs-build-e2e-{}-{process_id}-{project_id}",
            sanitize_name(name)
        );

        materialize_fixture(&root, fixture, &package_name);

        Self {
            directory: Some(directory),
            root,
            target,
            fixture,
            environment: BTreeMap::new(),
        }
    }

    pub fn check(&self) -> CargoRun {
        let mut command = Command::new(env!("CARGO"));
        command
            .args(["check", "--offline", "--quiet"])
            .current_dir(&self.root)
            .env("CARGO_TARGET_DIR", &self.target)
            .env("CARGO_TERM_COLOR", "never")
            .env_remove(FIXTURE_OUT_DIR_ENV)
            .envs(&self.environment);
        let output = command.output().unwrap();
        let build_runs = self
            .out_dir()
            .and_then(|out_dir| fs::read_to_string(out_dir.join(RUN_COUNTER)).ok())
            .and_then(|value| value.parse().ok());

        CargoRun {
            context: self.root.clone(),
            output,
            build_runs,
        }
    }

    pub fn write_schema(&self, relative: &str, contents: &str) {
        self.write_file(Path::new("schemas").join(relative), contents);
    }

    pub fn restore_schema(&self, relative: &str) {
        let contents = match (self.fixture, relative) {
            (Fixture::Basic, "game.fbs") => BASIC_GAME_FBS,
            (Fixture::Basic, "shared.fbs") => BASIC_SHARED_FBS,
            _ => panic!("fixture has no restorable schema {relative}"),
        };
        self.write_schema(relative, contents);
    }

    pub fn write_file(&self, relative: impl AsRef<Path>, contents: &str) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    pub fn create_directory(&self, relative: impl AsRef<Path>) {
        fs::create_dir_all(self.root.join(relative)).unwrap();
    }

    pub fn set_path_read_only(&self, relative: impl AsRef<Path>, read_only: bool) {
        set_read_only(&self.root.join(relative), read_only);
    }

    pub fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.root.join(relative)
    }

    pub fn set_env(&mut self, name: impl Into<OsString>, value: impl Into<OsString>) {
        self.environment.insert(name.into(), value.into());
    }

    pub fn remove_env(&mut self, name: impl AsRef<OsStr>) {
        self.environment.remove(name.as_ref());
    }

    pub fn remove_schema(&self, relative: &str) {
        fs::remove_file(self.root.join("schemas").join(relative)).unwrap();
    }

    pub fn create_schema_directory(&self, relative: &str) {
        fs::create_dir(self.root.join("schemas").join(relative)).unwrap();
    }

    pub fn remove_schema_directory(&self, relative: &str) {
        fs::remove_dir(self.root.join("schemas").join(relative)).unwrap();
    }

    pub fn generated(&self, filename: &str) -> GeneratedArtifact {
        let out_dir = self.out_dir().unwrap_or_else(|| {
            panic!(
                "build script did not record OUT_DIR for {}",
                self.root.display()
            )
        });
        GeneratedArtifact {
            path: out_dir.join(filename),
        }
    }

    fn out_dir(&self) -> Option<PathBuf> {
        fs::read_to_string(self.root.join(OUT_DIR_POINTER))
            .ok()
            .map(|path| PathBuf::from(path.trim()))
    }
}

impl Drop for CargoProject {
    fn drop(&mut self) {
        let keep = std::thread::panicking() || env_flag("FLATC_RS_E2E_KEEP");
        if keep {
            let path = self.directory.take().unwrap().keep();
            eprintln!("preserved E2E project at {}", path.display());
        }
    }
}

pub struct CargoRun {
    context: PathBuf,
    output: Output,
    build_runs: Option<u32>,
}

impl CargoRun {
    pub fn assert_success(&self) -> &Self {
        assert!(
            self.output.status.success(),
            "cargo check failed in {}\nstdout:\n{}\nstderr:\n{}",
            self.context.display(),
            String::from_utf8_lossy(&self.output.stdout),
            String::from_utf8_lossy(&self.output.stderr)
        );
        self
    }

    pub fn assert_error(&self, variant: &str, details: &[&str]) -> &Self {
        let combined = self.combined_output();
        assert!(
            !self.output.status.success(),
            "cargo check unexpectedly succeeded in {}\n{combined}",
            self.context.display()
        );
        for expected in std::iter::once(variant).chain(details.iter().copied()) {
            assert!(
                combined.contains(expected),
                "cargo error in {} did not contain {expected:?}\n{combined}",
                self.context.display()
            );
        }
        self
    }

    pub fn assert_build_runs(&self, expected: u32) -> &Self {
        assert_eq!(
            self.build_runs,
            Some(expected),
            "unexpected build-script run count in {}",
            self.context.display()
        );
        self
    }

    fn combined_output(&self) -> String {
        format!(
            "{}\n{}",
            String::from_utf8_lossy(&self.output.stdout),
            String::from_utf8_lossy(&self.output.stderr)
        )
    }
}

pub struct GeneratedArtifact {
    path: PathBuf,
}

impl GeneratedArtifact {
    pub fn assert_contains(&self, expected: &str) {
        let contents = fs::read_to_string(&self.path).unwrap();
        assert!(
            contents.contains(expected),
            "{} did not contain {expected:?}",
            self.path.display()
        );
    }

    pub fn checkpoint(&self) -> ArtifactCheckpoint {
        let mtime = FileTime::from_unix_time(CHECKPOINT_MTIME_SECONDS, 0);
        set_file_mtime(&self.path, mtime).unwrap();
        ArtifactCheckpoint {
            path: self.path.clone(),
            bytes: fs::read(&self.path).unwrap(),
            mtime,
        }
    }

    pub fn replace_with_directory(&self) {
        fs::remove_file(&self.path).unwrap();
        fs::create_dir(&self.path).unwrap();
    }

    pub fn remove_directory(&self) {
        fs::remove_dir(&self.path).unwrap();
    }

    pub fn set_read_only(&self, read_only: bool) {
        set_read_only(&self.path, read_only);
    }
}

pub struct ArtifactCheckpoint {
    path: PathBuf,
    bytes: Vec<u8>,
    mtime: FileTime,
}

impl ArtifactCheckpoint {
    pub fn assert_unchanged(&self) {
        assert_eq!(
            fs::read(&self.path).unwrap(),
            self.bytes,
            "generated contents changed at {}",
            self.path.display()
        );
        let current_mtime =
            FileTime::from_last_modification_time(&fs::metadata(&self.path).unwrap());
        assert_eq!(
            current_mtime,
            self.mtime,
            "generated file was rewritten at {}",
            self.path.display()
        );
    }

    pub fn assert_changed(&self) {
        assert_ne!(
            fs::read(&self.path).unwrap(),
            self.bytes,
            "generated contents did not change at {}",
            self.path.display()
        );
    }
}

fn materialize_fixture(root: &Path, fixture: Fixture, package_name: &str) {
    match fixture {
        Fixture::Basic => {
            write(root.join("build.rs"), BASIC_BUILD_RS);
            write(root.join("src/lib.rs"), BASIC_LIB_RS);
            write(root.join("schemas/game.fbs"), BASIC_GAME_FBS);
            write(root.join("schemas/shared.fbs"), BASIC_SHARED_FBS);
            write(root.join("Cargo.toml"), &manifest(package_name, true));
        }
        Fixture::Collision => {
            write(root.join("build.rs"), COLLISION_BUILD_RS);
            write(root.join("src/lib.rs"), COLLISION_LIB_RS);
            write(root.join("schemas/first/schema.fbs"), COLLISION_FIRST_FBS);
            write(root.join("schemas/second/schema.fbs"), COLLISION_SECOND_FBS);
            write(root.join("Cargo.toml"), &manifest(package_name, false));
        }
    }
}

fn manifest(package_name: &str, needs_runtime: bool) -> String {
    let build_crate = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let build_crate = build_crate.to_string_lossy().replace('\\', "/");
    let dependencies = if needs_runtime {
        "[dependencies]\nflatbuffers = \"25.12.19\"\n\n"
    } else {
        ""
    };
    format!(
        "[package]\nname = \"{package_name}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n\n{dependencies}[build-dependencies]\nflatc-rs-build = {{ path = \"{build_crate}\" }}\n"
    )
}

fn write(path: PathBuf, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn set_read_only(path: &Path, read_only: bool) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_readonly(read_only);
    fs::set_permissions(path, permissions).unwrap();
}

fn shared_target_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target/flatc-rs-build-e2e")
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn env_flag(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| value == "1")
}
