use camino::Utf8Path;
use camino::Utf8PathBuf;
use miette::miette;
use miette::Context;
use miette::IntoDiagnostic;

use crate::cli::Args;
use crate::format_bulleted_list;
use crate::ProjectPaths;

#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Step {
    Packages,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum LogFilter {
    One(String),
    Many(Vec<String>),
}

/// Configuration loaded from a file.
#[derive(serde::Deserialize, Default)]
pub struct ConfigFile {
    #[serde(alias = "log_filters")]
    log_filter: Option<LogFilter>,

    flake: Option<String>,

    update: Option<bool>,
}

impl ConfigFile {
    pub fn from_path(path: &Utf8Path) -> miette::Result<Self> {
        tracing::debug!("Loading config from {path}");
        let contents = std::fs::read_to_string(path)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to read {path}"))?;

        toml::from_str(&contents).into_diagnostic()
    }
}

pub struct Config {
    path: Option<Utf8PathBuf>,
    project_paths: ProjectPaths,
    file: ConfigFile,
    args: Args,
}

impl Config {
    pub fn from_args(args: Args) -> miette::Result<Self> {
        let project_paths = ProjectPaths::new()?;
        let paths = args.config_paths(&project_paths)?;

        tracing::trace!(?paths, "Looking for configuration file");
        for path in paths {
            if path
                .try_exists()
                .into_diagnostic()
                .wrap_err_with(|| format!("Failed to check if configuration path exists: {path}"))?
            {
                tracing::debug!(%path, "Loading configuration");
                let file = ConfigFile::from_path(&path)?;
                return Ok(Self {
                    path: Some(path),
                    project_paths,
                    file,
                    args,
                });
            }
        }

        tracing::debug!("No configuration file found");
        Ok(Self {
            path: None,
            project_paths,
            file: Default::default(),
            args,
        })
    }

    pub fn log_filter(&self) -> String {
        let mut ret = String::new();
        match &self.file.log_filter {
            Some(LogFilter::One(filter)) => {
                ret.push_str(filter);
            }
            Some(LogFilter::Many(filters)) => {
                ret.push_str(&filters.join(","));
            }
            None => {}
        }

        if let Some(filter) = &self.args.log_filter() {
            ret.push(',');
            ret.push_str(filter);
        }

        if ret.is_empty() {
            ret.push_str(crate::tracing::DEFAULT_FILTER);
        }

        ret
    }

    pub fn update(&self) -> bool {
        if self.args.update {
            return true;
        }
        self.file.update.unwrap_or(false)
    }

    pub fn flake(&self) -> miette::Result<String> {
        if let Some(flake) = &self.args.flake {
            return Ok(flake.clone());
        }

        if let Some(flake) = &self.file.flake {
            return Ok(flake.clone());
        }

        let mut paths = self.project_paths.flake_paths()?;

        if let Some(path) = &self.path {
            paths.push(
                path.parent()
                    .ok_or_else(|| miette!("Configuration file has no parent directory: {path}"))?
                    .to_path_buf(),
            );
        }

        for path in &paths {
            if path.try_exists().into_diagnostic()? {
                return Ok(fix_flake_path(path)?.into_string());
            }
        }

        Err(miette!(
            "Unable to find home-mangler `flake.nix`. I looked in these paths:\n{}",
            format_bulleted_list(&paths)
        ))
    }

    pub fn hostname(&self) -> miette::Result<String> {
        match &self.args.hostname {
            Some(hostname) => Ok(hostname.clone()),
            None => gethostname::gethostname()
                .into_string()
                .map_err(|s| miette!("Hostname is not UTF-8: {s:?}")),
        }
    }
}

/// We resolve symlinks to work around Nix.
/// See: <https://github.com/NixOS/nix/issues/9253>
fn fix_flake_path(path: &Utf8Path) -> miette::Result<Utf8PathBuf> {
    let mut path = path
        .parent()
        .ok_or_else(|| miette!("Path has no parent: {path}"))?
        .to_path_buf();

    if path
        .symlink_metadata()
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to query metadata for {path}"))?
        .is_symlink()
    {
        // TODO: What if there's multiple layers of link?
        path = path
            .read_link_utf8()
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to read link: {path}"))?;
    }

    let mut flake_file = path.clone();
    flake_file.push("flake.nix");

    if flake_file
        .symlink_metadata()
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to query metadata for {flake_file}"))?
        .is_symlink()
    {
        // TODO: What if there's multiple layers of link?
        path = flake_file
            .read_link_utf8()
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to read link: {flake_file}"))?;
        path = path
            .parent()
            .ok_or_else(|| miette!("Path has no parent directory: {path}"))?
            .to_owned();
    }

    Ok(path)
}
