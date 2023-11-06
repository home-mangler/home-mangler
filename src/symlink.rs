use std::collections::BTreeSet;
use std::fmt::Display;

use camino::Utf8Path;
use camino::Utf8PathBuf;
use miette::miette;
use miette::IntoDiagnostic;

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
        path = path.read_link_utf8().into_diagnostic()?;
        intermediate.push(path.clone());
    }

    intermediate.pop();
    let to = path;

    Ok(SymlinkChain { intermediate, to })
}
