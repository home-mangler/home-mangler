use std::process::Output;

use assert_cmd::Command;
use assert_fs::fixture::PathCopy;
use assert_fs::TempDir;

/// Test data flake path, relative the directory containing `Cargo.toml`.
const TEST_DATA_PATH: &str = "test-data/";
const TEST_ENV_VAR: &str = "HOME_MANGLER_NIXOS_INTEGRATION_TEST";

pub struct Session {
    pub temp: TempDir,
    pub cmd: Command,
}

impl Session {
    pub fn new() -> Self {
        if std::env::var(TEST_ENV_VAR).is_err() {
            panic!("${TEST_ENV_VAR} is not set; `home-mangler` integration tests can only be run in a NixOS VM");
        }

        let temp = assert_fs::fixture::TempDir::new().unwrap();
        temp.copy_from(TEST_DATA_PATH, &["**/*"]).unwrap();

        let temp = temp.into_persistent();

        let mut cmd = Command::new("home-mangler");
        cmd.current_dir(&temp);
        Session { temp, cmd }
    }
}

pub fn session() -> Session {
    Session::new()
}

pub trait OutputExt {
    fn stdout_utf8(&self) -> String;
    fn stderr_utf8(&self) -> String;
}

impl OutputExt for Output {
    fn stdout_utf8(&self) -> String {
        String::from_utf8(self.stdout.clone()).unwrap()
    }

    fn stderr_utf8(&self) -> String {
        String::from_utf8(self.stderr.clone()).unwrap()
    }
}
