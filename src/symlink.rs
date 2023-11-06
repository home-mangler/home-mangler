use std::collections::BTreeSet;
use std::fmt::Display;

use camino::Utf8Path;
use camino::Utf8PathBuf;
use miette::miette;
use miette::Context;
use miette::IntoDiagnostic;
use path_absolutize::Absolutize;

use crate::format_bulleted_list;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymlinkChain {
    pub intermediate: Vec<Utf8PathBuf>,
    pub to: Utf8PathBuf,
}

impl SymlinkChain {
    pub fn destinations(&self) -> impl Iterator<Item = &Utf8Path> {
        self.intermediate
            .iter()
            .map(|path| path.as_path())
            .chain(std::iter::once(self.to.as_path()))
    }
}

impl Display for SymlinkChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for path in &self.intermediate {
            write!(f, "{path} -> ")?;
        }
        write!(f, "{}", self.to)
    }
}

pub fn read_symlink(mut path: Utf8PathBuf) -> miette::Result<SymlinkChain> {
    let orig = path.clone();
    tracing::debug!("Resolving symlink {orig}");
    let mut seen = BTreeSet::new();
    let mut intermediate = Vec::new();

    while path.is_symlink() {
        if seen.contains(&path) {
            return Err(miette!(
                "Detected symlink loop:\n{}",
                format_bulleted_list(&seen)
            ));
        }

        seen.insert(path.clone());
        let parent = path.parent().map(|p| p.to_path_buf());
        path = path.read_link_utf8().into_diagnostic().wrap_err_with(|| {
            format!("Failed to read link {path} while resolving symlink {orig}")
        })?;
        if let Some(parent) = parent {
            path = path
                .as_std_path()
                .absolutize_from(parent.as_std_path())
                .into_diagnostic()
                .wrap_err_with(|| format!("Failed to join {parent} and {path}"))?
                .into_owned()
                .try_into()
                .into_diagnostic()?;
        }
        tracing::debug!("Resolved symlink to {path}");
        intermediate.push(path.clone());
    }

    intermediate.pop();
    let to = path;

    Ok(SymlinkChain { intermediate, to })
}
