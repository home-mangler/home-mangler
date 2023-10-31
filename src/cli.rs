use camino::Utf8PathBuf;

use crate::ProjectPaths;

#[derive(clap::Parser)]
#[allow(rustdoc::bare_urls)]
pub struct Args {
    /// Tracing log filter.
    ///
    /// See: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives
    #[arg(long, env = "HOME_MANGLER_LOG")]
    pub log_filter: Option<String>,

    /// Alias for `--log-filter=trace`.
    #[arg(long)]
    pub debug: bool,

    /// Alias for `--log-filter=debug`.
    #[arg(short, long)]
    pub verbose: bool,

    /// Path to the configuration file to use.
    ///
    /// Defaults to `~/.config/home-mangler/config.toml`.
    #[arg(long)]
    pub config: Option<Utf8PathBuf>,

    /// Flake containing home-mangler configuration.
    ///
    /// Defaults to the `--config` directory.
    #[arg(long)]
    pub flake: Option<String>,

    /// The hostname to build the configuration for.
    ///
    /// This corresponds to the `home-mangler.${hostname}` output attribute in your flake.
    #[arg(long, alias = "host", env = "HOSTNAME")]
    pub hostname: Option<String>,
}

impl Args {
    pub fn config_paths(&self, project_paths: &ProjectPaths) -> miette::Result<Vec<Utf8PathBuf>> {
        if let Some(path) = &self.config {
            return Ok(vec![path.clone()]);
        }

        project_paths.config_paths()
    }

    pub fn log_filter(&self) -> Option<String> {
        let mut ret = String::new();

        if let Some(filter) = &self.log_filter {
            ret.push_str(filter);
        }

        if self.debug {
            ret.push_str(",trace");
        } else if self.verbose {
            ret.push_str(",debug");
        }

        if ret.is_empty() {
            None
        } else {
            Some(ret)
        }
    }
}
