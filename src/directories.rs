use camino::Utf8PathBuf;
use directories::BaseDirs;
use directories::ProjectDirs;
use miette::miette;
use miette::IntoDiagnostic;
use tap::TryConv;

fn base_dirs() -> miette::Result<BaseDirs> {
    BaseDirs::new().ok_or_else(|| miette!("Unable to find home directory"))
}

fn project_dirs() -> miette::Result<ProjectDirs> {
    // The documentation says you can just leave these blank...?
    ProjectDirs::from("", "", "home-mangler")
        .ok_or_else(|| miette!("Unable to find home directory"))
}

pub struct ProjectPaths {
    base_dirs: BaseDirs,
    project_dirs: ProjectDirs,
}

impl ProjectPaths {
    pub fn new() -> miette::Result<Self> {
        let base_dirs = base_dirs()?;
        let project_dirs = project_dirs()?;
        Ok(Self {
            base_dirs,
            project_dirs,
        })
    }

    pub fn config_dirs(&self) -> miette::Result<Vec<Utf8PathBuf>> {
        let mut ret = Vec::new();

        ret.push({
            self.project_dirs
                .config_dir()
                .to_path_buf()
                .try_conv::<Utf8PathBuf>()
                .into_diagnostic()?
        });

        ret.push({
            let mut config = self
                .base_dirs
                .config_dir()
                .to_path_buf()
                .try_conv::<Utf8PathBuf>()
                .into_diagnostic()?;

            config.push("home-mangler");
            config
        });

        ret.push({
            let mut home = self
                .base_dirs
                .home_dir()
                .to_path_buf()
                .try_conv::<Utf8PathBuf>()
                .into_diagnostic()?;

            home.push(".config");
            home.push("home-mangler");
            home
        });

        Ok(ret)
    }

    pub fn config_paths(&self) -> miette::Result<Vec<Utf8PathBuf>> {
        let mut ret = self.config_dirs()?;

        for path in &mut ret {
            path.push("config.toml");
        }

        Ok(ret)
    }

    pub fn flake_paths(&self) -> miette::Result<Vec<Utf8PathBuf>> {
        let mut ret = self.config_dirs()?;

        for path in &mut ret {
            path.push("flake.nix");
        }

        Ok(ret)
    }
}
