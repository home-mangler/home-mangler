use std::collections::BTreeSet;
use std::process::Stdio;

use camino::Utf8PathBuf;
use command_error::CommandExt;
use miette::IntoDiagnostic;

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
            .output_checked_utf8()
            .into_diagnostic()?
            .stdout;

        Ok(stdout.lines().map(Utf8PathBuf::from).collect())
    }
}
