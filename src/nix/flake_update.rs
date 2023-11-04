use crate::command_ext::CommandExt;
use crate::flake::Flake;

/// Update a flake lockfile.
pub fn flake_update(flake: &Flake) -> miette::Result<()> {
    super::nix_command()
        .args(["flake", "update", &flake.to_string()])
        .status_checked()
}
