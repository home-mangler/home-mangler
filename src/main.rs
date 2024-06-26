use clap::Parser;

mod cli;
mod config;
mod diff_trees;
mod directories;
mod flake;
mod format_bulleted_list;
mod nix;
mod packages;
mod tracing;

use config::Config;

pub use directories::ProjectPaths;
pub use format_bulleted_list::format_bulleted_list;

fn main() -> miette::Result<()> {
    let opts = cli::Args::parse();
    let filter_reload = tracing::install_tracing(
        opts.log_filter()
            .as_deref()
            .unwrap_or(tracing::DEFAULT_FILTER),
    )?;
    let config = Config::from_args(opts)?;
    tracing::update_log_filters(&filter_reload, &config.log_filter())?;

    let nix = config.nix()?;
    let flake = config.flake()?;
    let hostname = config.hostname()?;
    ::tracing::debug!(%flake, %hostname, "Resolved configuration");
    packages::ensure_packages(&nix, &flake, &hostname, config.update())?;

    Ok(())
}
