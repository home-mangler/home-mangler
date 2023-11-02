use crate::command_ext::CommandExt;

/// Update a flake lockfile.
pub fn flake_update(flake: &str) -> miette::Result<()> {
    super::nix_command()
        .args(["flake", "update", flake])
        .status_checked()
}
