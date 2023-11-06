use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Display;
use std::fs::Metadata;

use blake3::Hash;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use miette::Context;
use miette::IntoDiagnostic;
use owo_colors::OwoColorize;
use owo_colors::Stream;

use crate::symlink::SymlinkChain;

pub fn diff_trees(
    removed_paths: &BTreeSet<&Utf8Path>,
    added_paths: &BTreeSet<&Utf8Path>,
) -> miette::Result<String> {
    let removed_tree = walk_trees(removed_paths)?;
    let added_tree = walk_trees(added_paths)?;

    let diff = diff_walked_trees(removed_tree, added_tree)?;

    display_diff(&diff)
}

fn display_diff(diff: &Tree) -> miette::Result<String> {
    let mut ret = String::new();

    let mut added_dirs = BTreeSet::<&Utf8Path>::new();
    let mut removed_dirs = BTreeSet::<&Utf8Path>::new();

    fn is_in_dirs(dirs: &BTreeSet<&Utf8Path>, path: &Utf8Path) -> bool {
        dirs.iter().any(|dir| path.starts_with(dir))
    }

    for (path, entry) in diff {
        match entry {
            Entry::Same => {
                tracing::debug!("Path unchanged: {path}");
            }
            Entry::Added(info) => {
                if is_in_dirs(&added_dirs, path) {
                    continue;
                }

                if info.metadata.is_dir() {
                    added_dirs.insert(path);
                }

                ret.push_str(
                    &format!("+ {path}")
                        .if_supports_color(Stream::Stdout, |text| text.green())
                        .to_string(),
                );
                ret.push('\n');
            }
            Entry::Removed(info) => {
                if is_in_dirs(&removed_dirs, path) {
                    continue;
                }

                if info.metadata.is_dir() {
                    removed_dirs.insert(path);
                }

                ret.push_str(
                    &format!("- {path}")
                        .if_supports_color(Stream::Stdout, |text| text.red())
                        .to_string(),
                );
                ret.push('\n');
            }
            Entry::Updated { old, new } => {
                tracing::debug!(
                    "Path updated:\n\
                    - {old}\n\
                    + {new}"
                );
                ret.push_str(
                    &format!("~ {path}")
                        .if_supports_color(Stream::Stdout, |text| text.yellow())
                        .to_string(),
                );
                ret.push('\n');
            }
        }
    }

    Ok(ret)
}

type Tree = BTreeMap<Utf8PathBuf, Entry>;

enum Entry {
    Added(FullPathState),
    Removed(FullPathState),
    Updated {
        old: Box<FullPathState>,
        new: Box<FullPathState>,
    },
    Same,
}

fn diff_walked_trees(
    removed_tree: BTreeMap<Utf8PathBuf, FullPathState>,
    mut added_tree: BTreeMap<Utf8PathBuf, FullPathState>,
) -> miette::Result<Tree> {
    let mut diff = Tree::new();

    for (path, removed_info) in removed_tree {
        if let Some(added_info) = added_tree.remove(&path) {
            // Path was unchanged or updated.
            let entry = diff_path(&removed_info.belongs_to, &added_info.belongs_to, &path)
                .wrap_err_with(|| {
                    format!(
                        "Failed to diff path {path} between {} and {}",
                        removed_info.belongs_to, added_info.belongs_to
                    )
                })?;
            diff.insert(path, entry);
        } else {
            // Path was removed.
            diff.insert(path, Entry::Removed(removed_info));
        }
    }

    // We removed paths from `removed_tree` before, so all these paths have been
    // added.
    diff.extend(
        added_tree
            .into_iter()
            .map(|(path, info)| (path, Entry::Added(info))),
    );

    Ok(diff)
}

fn diff_path(
    removed_base: &Utf8Path,
    added_base: &Utf8Path,
    relative: &Utf8Path,
) -> miette::Result<Entry> {
    let removed = removed_base.join(relative);
    let added = added_base.join(relative);

    let removed_state = PathState::new(&removed)?.with_path(removed)?;
    let added_state = PathState::new(&added)?.with_path(added)?;

    let removed_hash = removed_state.state.hash();
    let added_hash = added_state.state.hash();

    Ok(if removed_state.state == added_state.state {
        Entry::Same
    } else if removed_hash.is_some() && removed_hash == added_hash {
        // If the destination hashes are the same, the contents haven't changed but symlinks have.
        tracing::debug!(
            "Path updated: {relative}\n\
            - {removed_state}\n\
            + {added_state}",
        );
        Entry::Same
    } else {
        Entry::Updated {
            old: Box::new(removed_state),
            new: Box::new(added_state),
        }
    })
}

#[derive(PartialEq, Eq)]
enum PathState {
    Symlink {
        chain: SymlinkChain,
        hash: Option<Hash>,
    },
    File {
        hash: Hash,
    },
    Dir,
}

impl PathState {
    pub fn hash(&self) -> Option<Hash> {
        match &self {
            PathState::Symlink { chain: _, hash } => *hash,
            PathState::File { hash } => Some(*hash),
            PathState::Dir => None,
        }
    }

    fn new(path: &Utf8Path) -> miette::Result<Self> {
        let metadata = path.symlink_metadata().into_diagnostic()?;
        Self::from_symlink_metadata(path, &metadata)
    }

    fn from_symlink_metadata(path: &Utf8Path, metadata: &Metadata) -> miette::Result<Self> {
        Ok(if metadata.is_symlink() {
            let chain = crate::read_symlink(path.to_path_buf())?;
            let hash = if chain
                .to
                .metadata()
                .into_diagnostic()
                .wrap_err_with(|| format!("Failed to read metadata for {}", chain.to))?
                .is_file()
            {
                Some(hash_file(&chain.to)?)
            } else {
                None
            };
            Self::Symlink { chain, hash }
        } else if metadata.is_dir() {
            Self::Dir
        } else {
            Self::File {
                hash: hash_file(path)?,
            }
        })
    }

    fn with_path(self, path: Utf8PathBuf) -> miette::Result<FullPathState> {
        let metadata = path
            .symlink_metadata()
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to query metadata for {path}"))?;
        Ok(FullPathState {
            belongs_to: path,
            state: self,
            metadata,
        })
    }
}

struct FullPathState {
    belongs_to: Utf8PathBuf,
    metadata: Metadata,
    state: PathState,
}

impl Display for FullPathState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let PathState::Symlink { chain, .. } = &self.state {
            write!(f, " -> {chain}")?;
        }
        Ok(())
    }
}

fn hash_file(path: &Utf8Path) -> miette::Result<Hash> {
    tracing::debug!("Hashing {path}");
    Ok(blake3::Hasher::new()
        .update_mmap(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to hash {path}"))?
        .finalize())
}

fn walk_trees(
    dirs: impl IntoIterator<Item = impl AsRef<Utf8Path>>,
) -> miette::Result<BTreeMap<Utf8PathBuf, FullPathState>> {
    let mut tree = BTreeMap::new();
    for path in dirs {
        let path = path.as_ref();
        walk_subtree(&mut tree, path, Utf8Path::new(""))
            .wrap_err_with(|| format!("Failed to read tree at {path}"))?;
    }
    Ok(tree)
}

fn walk_subtree(
    results: &mut BTreeMap<Utf8PathBuf, FullPathState>,
    base: &Utf8Path,
    relative: &Utf8Path,
) -> miette::Result<()> {
    let path = base.join(relative);
    let metadata = path
        .symlink_metadata()
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read metadata for {path}"))?;
    let is_dir = metadata.is_dir();
    let state = PathState::from_symlink_metadata(&path, &metadata)
        .wrap_err_with(|| format!("Failed to get path state for {path}"))?;
    // TODO: Handle multiple `base`s providing the same path.
    results
        .entry(relative.to_path_buf())
        .or_insert_with(|| FullPathState {
            belongs_to: base.to_path_buf(),
            metadata,
            state,
        });
    if is_dir {
        for entry in path
            .read_dir_utf8()
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to read directory contents for {path}"))?
        {
            let entry = entry
                .into_diagnostic()
                .wrap_err_with(|| format!("Failed to read directory contents for {path}"))?;
            let subtree = relative.join(entry.file_name());
            walk_subtree(results, base, &subtree)
                .wrap_err_with(|| format!("Failed to read tree at {base}/{subtree}"))?;
        }
    }

    Ok(())
}
