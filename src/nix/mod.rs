use std::process::Command;

mod profile_list;
use camino::Utf8PathBuf;
use miette::Context;
use miette::IntoDiagnostic;
pub use profile_list::ProfileList;

mod flake_metadata;
pub use flake_metadata::ResolvedFlake;
use tap::TryConv;

mod build;
mod flake_update;

#[derive(Debug, Clone)]
pub struct Nix {
    /// Path to the `nix` binary.
    program: Utf8PathBuf,
    /// Path to the current profile.
    profile: Option<Utf8PathBuf>,
}

impl Nix {
    pub fn new() -> miette::Result<Self> {
        let mut program = which::which_global("nix")
            .into_diagnostic()
            .wrap_err("Could not find `nix` executable")?
            .try_conv::<Utf8PathBuf>()
            .into_diagnostic()?;
        if program.is_symlink() {
            tracing::debug!(path = %program, "`nix` is symlink");
            program = program
                .read_link_utf8()
                .into_diagnostic()
                .wrap_err_with(|| format!("Failed to read `nix` symlink: {program:?}"))?;
        }
        tracing::debug!(path = %program, "Found `nix`");
        Ok(Self {
            program,
            profile: None,
        })
    }

    pub fn with_profile(mut self, profile: Option<Utf8PathBuf>) -> Self {
        self.profile = profile;
        self
    }

    pub fn command(&self, subcommand: &[&str]) -> Command {
        // TODO: Should run in `sh` after sourcing the Nix profile.
        let mut command = Command::new(&self.program);
        command.args(["--extra-experimental-features", "nix-command flakes"]);
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
