use camino::Utf8Path;
use camino::Utf8PathBuf;
use miette::miette;
use miette::Context;
use miette::IntoDiagnostic;

use crate::cli::Args;
use crate::flake::Flake;
use crate::format_bulleted_list;
use crate::nix::Nix;
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
    use_path_flake: Option<bool>,
    profile: Option<Utf8PathBuf>,
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

    fn use_path_flake(&self) -> bool {
        self.args.use_path_flake || self.file.use_path_flake.unwrap_or(false)
    }

    pub fn flake(&self) -> miette::Result<Flake> {
        Ok(self
            .flake_unconfigured()?
            .set_use_path_flake(self.use_path_flake()))
    }

    fn flake_unconfigured(&self) -> miette::Result<Flake> {
        if let Some(flake) = &self.args.flake {
            return flake.parse();
        }

        if let Some(flake) = &self.file.flake {
            return flake.parse();
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
                return path.as_path().try_into();
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

    pub fn nix(&self) -> Nix {
        Nix::default().with_profile(
            self.args
                .profile
                .clone()
                .or_else(|| self.file.profile.clone()),
        )
    }
}
