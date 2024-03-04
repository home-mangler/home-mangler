use std::process::Command;

mod profile_list;
use camino::Utf8PathBuf;
pub use profile_list::ProfileList;

mod flake_metadata;
pub use flake_metadata::ResolvedFlake;

mod build;
mod flake_update;

#[derive(Debug, Default, Clone)]
pub struct Nix {
    profile: Option<Utf8PathBuf>,
}

impl Nix {
    pub fn with_profile(mut self, profile: Option<Utf8PathBuf>) -> Self {
        self.profile = profile;
        self
    }

    pub fn command(&self, subcommand: &[&str]) -> Command {
        // TODO: Should run in `sh` after sourcing the Nix profile.
        let mut command = Command::new("nix");
        command.args(["--extra-experimental-features", "nix-command"]);
        command.args(subcommand);
        #[allow(clippy::single_match)]
        match subcommand {
            ["profile", _] => {
                if let Some(profile) = &self.profile {
                    command.args(["--profile", profile.as_str()]);
                }
            }
            _ => {}
        }

        command
    }
}
