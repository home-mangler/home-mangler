use std::collections::BTreeSet;
use std::process::Stdio;

use camino::Utf8PathBuf;

use crate::command_ext::CommandExt;

use super::Nix;

impl Nix {
    /// Build an installable and return the out paths.
    pub fn build(&self, installable: &str) -> miette::Result<BTreeSet<Utf8PathBuf>> {
        let stdout = self
            .command(&["build"])
            .args([
                "--print-build-logs",
                "--no-link",
                "--print-out-paths",
                installable,
            ])
            .stderr(Stdio::inherit())
            .stdout_checked_utf8()?;

        Ok(stdout.lines().map(Utf8PathBuf::from).collect())
    }
}
