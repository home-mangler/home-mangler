use std::collections::HashMap;

use camino::Utf8PathBuf;
use miette::IntoDiagnostic;

use crate::command_ext::CommandExt;
use crate::flake::Flake;

pub struct ResolvedFlake {
    /// The original user input string. May not match the `original_url`, e.g. `.` is parsed
    /// into an absolute path, `nixpkgs` is resolved to `flake:nixpkgs`, etc.
    pub original: Flake,
    /// The resolved and parsed URL.
    pub metadata: FlakeMetadata,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlakeMetadata {
    /// Source store path.
    pub path: Utf8PathBuf,
    pub original: HashMap<String, String>,
    pub original_url: String,
    pub resolved: HashMap<String, String>,
    pub resolved_url: String,
    // Other fields: `locked`, `locks`, `lastModified`, `dirtyRevision`, `description`,
    // `revCount`, `revision`.
}

pub fn flake_metadata(flake: &Flake) -> miette::Result<FlakeMetadata> {
    let json_output = super::nix_command()
        .args(["flake", "metadata", "--json", &flake.to_string()])
        .stdout_checked_utf8()?;

    serde_json::from_str(&json_output).into_diagnostic()
}

pub fn resolve(flake: Flake) -> miette::Result<ResolvedFlake> {
    let metadata = flake_metadata(&flake)?;
    Ok(ResolvedFlake {
        original: flake,
        metadata,
    })
}
