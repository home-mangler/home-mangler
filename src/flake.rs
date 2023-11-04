use miette::miette;
use miette::Context;
use std::fmt::Display;
use std::str::FromStr;

use camino::Utf8Path;
use camino::Utf8PathBuf;
use miette::IntoDiagnostic;

#[derive(Debug, Clone)]
pub enum Flake {
    Url(String),
    Path(PathFlake),
}

impl Flake {
    pub fn set_use_path_flake(mut self, use_path_flake: bool) -> Self {
        if let Flake::Path(flake) = &mut self {
            flake.use_path_flake = use_path_flake;
        }
        self
    }
}

impl Display for Flake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Flake::Url(url) => write!(f, "{url}"),
            Flake::Path(path) => write!(f, "{path}"),
        }
    }
}

impl TryFrom<&Utf8Path> for Flake {
    type Error = miette::Report;

    fn try_from(path: &Utf8Path) -> Result<Self, Self::Error> {
        Ok(Self::Path(PathFlake::new(path)?))
    }
}

impl FromStr for Flake {
    type Err = miette::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = Utf8Path::new(s);
        Ok(if path.try_exists().into_diagnostic()? {
            Self::Path(PathFlake::new(path)?)
        } else {
            Self::Url(s.to_owned())
        })
    }
}

#[derive(Debug, Clone)]
pub struct PathFlake {
    path: Utf8PathBuf,
    use_path_flake: bool,
}

impl Display for PathFlake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.use_path_flake {
            write!(f, "path:")?;
        }
        write!(f, "{}", self.path)
    }
}

impl PathFlake {
    /// We resolve symlinks to work around Nix.
    /// See: <https://github.com/NixOS/nix/issues/9253>
    fn new(path: &Utf8Path) -> miette::Result<Self> {
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

        Ok(Self {
            path,
            use_path_flake: false,
        })
    }
}
