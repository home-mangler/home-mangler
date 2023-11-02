use std::process::Command;

mod profile_list;
pub use profile_list::profile_list;
pub use profile_list::ProfileList;
pub use profile_list::ProfileListV2Element;

mod flake_metadata;
pub use flake_metadata::flake_metadata;
pub use flake_metadata::resolve;
pub use flake_metadata::FlakeMetadata;
pub use flake_metadata::ResolvedFlake;

mod flake_update;
pub use flake_update::flake_update;

mod build;
pub use build::build;

pub fn nix_command() -> Command {
    // TODO: Should run in `sh` after sourcing the Nix profile.
    let mut command = Command::new("nix");
    command.args(["--extra-experimental-features", "nix-command"]);
    command
}
