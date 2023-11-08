use std::collections::BTreeSet;

use camino::Utf8Path;
use clap::Parser;

mod cli;
mod command_ext;
mod config;
mod diff_trees;
mod directories;
mod flake;
mod format_bulleted_list;
mod nix;
mod packages;
mod symlink;
mod tracing;

use config::Config;

pub use directories::ProjectPaths;
pub use format_bulleted_list::format_bulleted_list;
pub use symlink::read_symlink;

fn main() -> miette::Result<()> {
    let opts = cli::Args::parse();
    let filter_reload = tracing::install_tracing(
        opts.log_filter()
            .as_deref()
            .unwrap_or(tracing::DEFAULT_FILTER),
    )?;
    let config = Config::from_args(opts)?;
    tracing::update_log_filters(&filter_reload, &config.log_filter())?;

    // let removed = "/nix/store/1w39p07mws3zv6skf3p40ilw8bma7f5h-home-mangler-packages";
    // let added = "/nix/store/rgy242kgmadxi607qkq3iij8ppbckzc0-home-mangler-packages";

    let removed = "/nix/store/lq7wg8zi6qxs1plj00pgdhc4lblbhc1m-home-mangler-packages";
    let added = "/nix/store/3qzz3990l72789sspp275cb6acjb06wm-home-mangler-packages";

    println!(
        "{}",
        diff_trees::diff_trees(
            &BTreeSet::from([Utf8Path::new(removed)]),
            &BTreeSet::from([Utf8Path::new(added)])
        )?
    );
    return Ok(());

    let flake = config.flake()?;
    let hostname = config.hostname()?;
    ::tracing::debug!(%flake, %hostname, "Resolved configuration");
    packages::ensure_packages(&flake, &hostname, config.update())?;

    Ok(())
}
