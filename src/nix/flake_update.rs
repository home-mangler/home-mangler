use crate::command_ext::CommandExt;
use crate::flake::Flake;

use super::Nix;

impl Nix {
    /// Update a flake lockfile.
    pub fn flake_update(&self, flake: &Flake) -> miette::Result<()> {
        self.command(&["flake", "update"])
            .arg(&flake.to_string())
            .status_checked()
    }
}
