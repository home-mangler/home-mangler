use camino::Utf8PathBuf;
use miette::miette;
use miette::Context;
use miette::IntoDiagnostic;
use tap::TryConv;

#[derive(clap::Parser)]
#[allow(rustdoc::bare_urls)]
pub struct Opts {
    /// Tracing log filter.
    ///
    /// See: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives
    #[arg(long, env = "HOME_MANGLER_LOG", default_value = "info")]
    pub log_filter: String,

    /// The hostname to build the configuration for.
    ///
    /// This corresponds to the `home-mangler.${hostname}` output attribute in your flake.
    #[arg(long, alias = "host", env = "HOSTNAME")]
    hostname: Option<String>,
}

impl Opts {
    pub fn config_dir(&self) -> miette::Result<Utf8PathBuf> {
        let mut home = dirs::home_dir()
            .ok_or_else(|| miette!("Unable to find home directory"))?
            .try_conv::<Utf8PathBuf>()
            .into_diagnostic()?;

        // TODO: Support those `$XDG` environment variables that nobody uses.
        home.push(".config");
        home.push("home-mangler");

        Ok(home)
    }

    pub fn flake_directory(&self) -> miette::Result<Utf8PathBuf> {
        // We resolve symlinks to work around Nix.
        // See: https://github.com/NixOS/nix/issues/9253
        let mut flake_dir = self
            .config_dir()
            .wrap_err("Could not find home-mangler config directory")?;
        if flake_dir
            .symlink_metadata()
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to query metadata for {flake_dir}"))?
            .is_symlink()
        {
            // TODO: What if there's multiple layers of link?
            flake_dir = flake_dir
                .read_link_utf8()
                .into_diagnostic()
                .wrap_err_with(|| format!("Failed to read link: {flake_dir}"))?;
        }

        let mut flake_file = flake_dir.clone();
        flake_file.push("flake.nix");

        if flake_file
            .symlink_metadata()
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to query metadata for {flake_file}"))?
            .is_symlink()
        {
            // TODO: What if there's multiple layers of link?
            flake_dir = flake_file
                .read_link_utf8()
                .into_diagnostic()
                .wrap_err_with(|| format!("Failed to read link: {flake_file}"))?;
            flake_dir = flake_dir
                .parent()
                .ok_or_else(|| miette!("Path has no parent directory: {flake_dir}"))?
                .to_owned();
        }

        Ok(flake_dir)
    }

    pub fn hostname(&self) -> miette::Result<String> {
        match &self.hostname {
            Some(hostname) => Ok(hostname.clone()),
            None => gethostname::gethostname()
                .into_string()
                .map_err(|s| miette!("Hostname is not UTF-8: {s:?}")),
        }
    }
}
