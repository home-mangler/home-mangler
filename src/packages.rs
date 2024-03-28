use std::collections::BTreeSet;

use camino::Utf8Path;
use camino::Utf8PathBuf;
use command_error::CommandExt;
use miette::Context;
use miette::IntoDiagnostic;

use crate::flake::Flake;
use crate::format_bulleted_list;
use crate::nix::Nix;
use crate::nix::ProfileList;
use crate::nix::ResolvedFlake;

pub fn ensure_packages(
    nix: &Nix,
    flake: &Flake,
    hostname: &str,
    update: bool,
) -> miette::Result<()> {
    if update {
        nix.flake_update(flake)
            .wrap_err_with(|| format!("Failed to update `flake.lock` for {flake}"))?;
    }

    let flake_attr = format!("home-mangler.{hostname}.packages");
    let package_installable = format!("{flake}#{flake_attr}");

    // TODO: We have a few things we could run in separate threads here.
    let resolved = nix.resolve(flake.clone())?;

    tracing::info!("Building packages for install");
    let package_out_paths = nix.build(&package_installable)?;
    let profile = nix.profile_list()?;
    let missing_paths = profile.missing_paths(&package_out_paths)?;
    if !missing_paths.is_empty() {
        let removed_paths = profile.remove_old_packages(nix, &resolved, &flake_attr)?;
        tracing::info!(
            "Installing new packages to `nix profile`:\n{}",
            format_bulleted_list(&missing_paths)
        );
        install_new_packages(nix, &package_installable)?;

        let removed_paths = removed_paths.difference(&missing_paths).copied().collect();
        let added_paths = missing_paths;

        let diff = crate::diff_trees::diff_trees(&removed_paths, &added_paths)?;

        tracing::info!("Updated `nix profile`:\n{diff}");
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
            ProfileList::V3(packages) => {
                for package in packages.values() {
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
        nix: &Nix,
        flake: &ResolvedFlake,
        attr_path: &str,
    ) -> miette::Result<BTreeSet<&Utf8Path>> {
        let mut elements_to_remove = vec![];
        let mut paths_to_remove = BTreeSet::new();
        match &self {
            ProfileList::V2(packages) => {
                for (i, package) in packages.iter().enumerate() {
                    if package.attr_path.as_deref() == Some(attr_path)
                        && package.original_url.as_deref()
                            == Some(flake.metadata.original_url.as_str())
                    {
                        elements_to_remove.push(i.to_string());
                        paths_to_remove.extend(package.store_paths.iter().map(|p| p.as_path()));
                    }
                }
            }
            ProfileList::V3(packages) => {
                for (name, package) in packages {
                    if package.attr_path.as_deref() == Some(attr_path)
                        && package.original_url.as_deref()
                            == Some(flake.metadata.original_url.as_str())
                    {
                        elements_to_remove.push(name.clone());
                        paths_to_remove.extend(package.store_paths.iter().map(|p| p.as_path()));
                    }
                }
            }
        }

        if !elements_to_remove.is_empty() {
            // TODO: Confirm before removing.
            tracing::info!(
                "Removing old packages from `nix profile`:\n{}",
                format_bulleted_list(&paths_to_remove)
            );
            nix.command(&["profile", "remove"])
                .args(elements_to_remove.iter().map(|element| element.to_string()))
                .status_checked()
                .into_diagnostic()?;
        }

        Ok(paths_to_remove)
    }
}

fn install_new_packages(nix: &Nix, flake_ref: &str) -> miette::Result<()> {
    nix.command(&["profile", "install"])
        .args(["--print-build-logs", flake_ref])
        .status_checked()
        .into_diagnostic()
        .map(|_| ())
}
