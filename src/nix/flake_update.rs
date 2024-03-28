use command_error::CommandExt;
use miette::IntoDiagnostic;

use super::Nix;
use crate::flake::Flake;

impl Nix {
    /// Update a flake lockfile.
    pub fn flake_update(&self, flake: &Flake) -> miette::Result<()> {
        tracing::info!("Updating flake inputs");
        self.command(&["flake", "update", "--flake"])
            .arg(&flake.to_string())
            .status_checked()
            .into_diagnostic()
            .map(|_| ())
    }
}
