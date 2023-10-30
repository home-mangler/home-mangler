use camino::Utf8Path;
use camino::Utf8PathBuf;
use dirs::home_dir;
use miette::miette;
use miette::IntoDiagnostic;
use tap::TryConv;

use crate::nix;

pub fn ensure_files(flake: &str, hostname: &str) -> miette::Result<()> {
    let flake_attr = format!("home-mangler.{hostname}.files");
    let files_installable = format!("{flake}#{flake_attr}");

    // TODO: Recover gracefully if attribute doesn't exist.
    let files_out_paths = nix::build(&files_installable)?;
    for out_path in files_out_paths {
        ensure_files_from_out_path(&out_path)?;
    }

    Ok(())
}

fn ensure_files_from_out_path(out_path: &Utf8Path) -> miette::Result<()> {
    let home_dir = home_dir()
        .ok_or_else(|| miette!("Unable to find home directory"))?
        .try_conv::<Utf8PathBuf>()
        .into_diagnostic()?;
    ensure_files_from_dir(&home_dir, out_path, Utf8Path::new(""))?;
    Ok(())
}

fn ensure_files_from_dir(
    home_dir: &Utf8Path,
    out_path: &Utf8Path,
    base_dir: &Utf8Path,
) -> miette::Result<()> {
    let dir = out_path.join(base_dir);
    for entry in dir.read_dir_utf8().into_diagnostic()? {
        match entry {
            Ok(entry) => {
                let file_type = entry.file_type().into_diagnostic()?;

                if file_type.is_dir() {
                    ensure_files_from_dir(home_dir, out_path, &base_dir.join(entry.file_name()))?;
                } else if file_type.is_file() {
                    ensure_path(
                        &home_dir.join(base_dir).join(entry.file_name()),
                        entry.path(),
                    )?;
                } else if file_type.is_symlink() {
                    tracing::warn!("Don't know what to do with symlinks yet: {}", entry.path());
                } else {
                    tracing::error!("Unknown file type {file_type:?} for path: {}", entry.path());
                }
            }
            Err(err) => {
                tracing::error!("Failed to read directory entry: {err}");
            }
        }
    }

    Ok(())
}

fn ensure_path(from: &Utf8Path, to: &Utf8Path) -> miette::Result<()> {
    tracing::info!("{from} -> {to}");
    Ok(())
}
