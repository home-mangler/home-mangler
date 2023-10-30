use std::collections::BTreeSet;

use camino::Utf8Path;
use camino::Utf8PathBuf;
use itertools::Itertools;
use owo_colors::OwoColorize;

use crate::command_ext::CommandExt;
use crate::format_bulleted_list;
use crate::nix;
use crate::nix::ProfileList;
use crate::nix::ResolvedFlake;

pub fn ensure_packages(flake: &str, hostname: &str) -> miette::Result<()> {
    let flake_attr = format!("home-mangler.{hostname}.packages");
    let package_installable = format!("{flake}#{flake_attr}");

    // TODO: We have a few things we could run in separate threads here.
    let resolved = nix::resolve(flake.to_owned())?;

    let package_out_paths = nix::build(&package_installable)?;
    let profile = nix::profile_list()?;
    let missing_paths = profile.missing_paths(&package_out_paths)?;
    if !missing_paths.is_empty() {
        let removed_paths = profile.remove_old_packages(&resolved, &flake_attr)?;
        install_new_packages(&package_installable)?;

        let mut diff = String::new();
        let removed_display = (removed_paths.difference(&missing_paths))
            .map(|p| format!("- {p}"))
            .join("\n");
        if !removed_display.is_empty() {
            diff.push('\n');
            diff.push_str(&removed_display.red().to_string());
        }

        let added_display = missing_paths.iter().map(|p| format!("+ {p}")).join("\n");
        if !added_display.is_empty() {
            diff.push('\n');
            diff.push_str(&added_display.green().to_string());
        }

        tracing::info!("Updated `nix profile`:{diff}");
    } else {
        tracing::info!(
            "Already up to date:\n{}",
            format_bulleted_list(&package_out_paths)
        );
    }
    Ok(())
}

impl ProfileList {
    /// Find store paths that aren't installed in the profile.
    pub fn missing_paths<'p>(
        &self,
        out_paths: &'p BTreeSet<Utf8PathBuf>,
    ) -> miette::Result<BTreeSet<&'p Utf8Path>> {
        let mut uninstalled_paths: BTreeSet<&Utf8Path> =
            out_paths.iter().map(|p| p.as_path()).collect();

        match &self {
            ProfileList::V2(packages) => {
                for package in packages {
                    for store_path in &package.store_paths {
                        uninstalled_paths.remove(store_path.as_path());
                    }
                }
            }
        }

        Ok(uninstalled_paths)
    }

    pub fn remove_old_packages(
        &self,
        flake: &ResolvedFlake,
        attr_path: &str,
    ) -> miette::Result<BTreeSet<&Utf8Path>> {
        let mut indices_to_remove = vec![];
        let mut paths_to_remove = BTreeSet::new();
        match &self {
            ProfileList::V2(packages) => {
                for (i, package) in packages.iter().enumerate() {
                    if package.attr_path.as_deref() == Some(attr_path)
                        && package.original_url.as_deref()
                            == Some(flake.metadata.original_url.as_str())
                    {
                        indices_to_remove.push(i);
                        paths_to_remove.extend(package.store_paths.iter().map(|p| p.as_path()));
                    }
                }
            }
        }

        if !indices_to_remove.is_empty() {
            // TODO: Confirm before removing.
            tracing::info!(
                "Removing old packages from `nix profile`:\n{}",
                format_bulleted_list(&paths_to_remove)
            );
            nix::nix_command()
                .args(["profile", "remove"])
                .args(indices_to_remove.iter().map(|i| i.to_string()))
                .status_checked()?;
        }

        Ok(paths_to_remove)
    }
}

fn install_new_packages(flake_ref: &str) -> miette::Result<()> {
    tracing::info!("Installing new packages");
    nix::nix_command()
        .args(["profile", "install", "--print-build-logs", flake_ref])
        .status_checked()
}
