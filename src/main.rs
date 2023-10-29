use clap::Parser;

mod cli;
mod command_ext;
mod format_bulleted_list;
mod nix;
mod packages;
mod tracing;

pub use format_bulleted_list::format_bulleted_list;

fn main() -> miette::Result<()> {
    let opts = cli::Opts::parse();
    tracing::install_tracing(&opts.log_filter)?;

    let flake_dir = opts.flake_directory()?;
    let hostname = opts.hostname()?;
    packages::ensure_packages(flake_dir.as_str(), &hostname)?;

    Ok(())
}
