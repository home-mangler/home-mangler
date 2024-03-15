use camino::Utf8PathBuf;
use miette::miette;
use miette::IntoDiagnostic;
use serde_json::Value as Json;

use crate::command_ext::CommandExt;

use super::Nix;

impl Nix {
    pub fn profile_list(&self) -> miette::Result<ProfileList> {
        let json_output = self
            .command(&["profile", "list"])
            .arg("--json")
            .stdout_checked_utf8()?;

        let data: ProfileListUnknown = serde_json::from_str(&json_output).into_diagnostic()?;

        match data.version {
        1..=3 => {
            let data: ProfileListV3 = serde_json::from_value(data.rest).into_diagnostic()?;
            Ok(ProfileList::V3(data.elements))
        }
        version => {
            Err(miette!("Unknown `nix profile list --json` output version {version}; I only know how to interpret output for version 2"))
        }
    }
    }
}

pub enum ProfileList {
    /// Versions 1-3.
    V3(Vec<ProfileListV3Element>),
}

#[derive(serde::Deserialize)]
struct ProfileListUnknown {
    version: u8,
    #[serde(flatten)]
    rest: Json,
}

#[derive(serde::Deserialize)]
struct ProfileListV3 {
    elements: Vec<ProfileListV3Element>,
}

/// `nix profile list --json` element for versions 1-3.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileListV3Element {
    /// How is an element 'deactivated'?
    pub active: bool,

    /// 5
    pub priority: u16,

    /// `["/nix/store/dccm0y9xpz85sm9gsfb0n7rs07cp4l7p-home-mangler-packages"]`.
    pub store_paths: Vec<Utf8PathBuf>,

    /// `git+file:///Users/wiggles/.dotfiles?dir=config/home-mangler`
    #[serde(alias = "uri")]
    pub url: Option<String>,

    /// `home-mangler.grandiflora.packages`
    ///
    /// Why is there no `original_attr_path`?
    pub attr_path: Option<String>,

    /// `git+file:///Users/wiggles/.dotfiles?dir=config/home-mangler`
    ///
    /// TODO: When does this differ from `url`?
    #[serde(alias = "originalUri")]
    pub original_url: Option<String>,

    /// `null` or `["man"]`
    ///
    /// Doesn't seem to include the default output `out`. Or maybe that's only if it's `null`?
    pub outputs: Option<Vec<String>>,
}
