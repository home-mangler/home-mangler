use std::collections::BTreeSet;

use camino::Utf8PathBuf;

use crate::command_ext::CommandExt;

/// Build an installable and return the out paths.
pub fn build(installable: &str) -> miette::Result<BTreeSet<Utf8PathBuf>> {
    let stdout = super::nix_command()
        .args([
            "build",
            "--print-build-logs",
            "--no-link",
            "--print-out-paths",
            installable,
        ])
        .stdout_checked_utf8()?;

    Ok(stdout.lines().map(Utf8PathBuf::from).collect())
}
