use assert_cmd::Command;
use assert_fs::fixture::PathCopy;
use assert_fs::TempDir;

/// Test data flake path, relative the directory containing `Cargo.toml`.
const TEST_DATA_PATH: &str = "test-data/";

pub struct Session {
    pub temp: TempDir,
    pub cmd: Command,
}

impl Session {
    pub fn new() -> Self {
        let temp = assert_fs::fixture::TempDir::new().unwrap();
        temp.copy_from(TEST_DATA_PATH, &["flake.nix", "flake.lock"])
            .unwrap();
        let mut cmd = Command::from_std(test_bin::get_test_bin("home-mangler"));
        cmd.current_dir(&temp);
        Session { temp, cmd }
    }
}

pub fn session() -> Session {
    Session::new()
}
