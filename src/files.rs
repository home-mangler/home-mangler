use camino::Utf8Path;

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
    Ok(())
}

fn ensure_path(from: &Utf8Path, to: &Utf8Path) -> miette::Result<()> {
    Ok(())
}
