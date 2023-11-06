use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Display;

use blake3::Hash;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use miette::IntoDiagnostic;
use owo_colors::OwoColorize;
use owo_colors::Stream;

use crate::symlink::SymlinkChain;

pub fn diff_trees(
    removed_paths: &BTreeSet<&Utf8Path>,
    added_paths: &BTreeSet<&Utf8Path>,
) -> miette::Result<String> {
    let removed_tree = walk_trees(removed_paths)?;
    let mut added_tree = walk_trees(added_paths)?;

    let mut diff = Tree::new();

    for (path, removed_belongs_to) in removed_tree {
        if let Some(added_belongs_to) = added_tree.remove(&path) {
            // Path was unchanged or updated.
            let entry = diff_path(&removed_belongs_to, &added_belongs_to, &path)?;
            diff.insert(path, entry);
        } else {
            // Path was removed.
            diff.insert(path, Entry::Removed);
        }
    }

    // We removed paths from `removed_tree` before, so all these paths have been
    // added.
    diff.extend(added_tree.into_keys().map(|path| (path, Entry::Added)));

    display_diff(diff)
}

fn display_diff(diff: Tree) -> miette::Result<String> {
    let mut ret = String::new();

    for (path, entry) in diff {
        match entry {
            Entry::Same => {
                tracing::debug!("Path unchanged: {path}");
            }
            Entry::Added => {
                ret.push_str(
                    &"+ "
                        .if_supports_color(Stream::Stdout, |text| text.green())
                        .to_string(),
                );
                ret.push('\n');
            }
            Entry::Removed => {
                ret.push_str(
                    &"- "
                        .if_supports_color(Stream::Stdout, |text| text.red())
                        .to_string(),
                );
                ret.push('\n');
            }
            Entry::Updated { old, new } => match (old, new) {
                (
                    PathState::Symlink {
                        chain: old_chain,
                        hash: old_hash,
                    },
                    PathState::Symlink {
                        chain: new_chain,
                        hash: new_hash,
                    },
                ) => todo!(),
                (
                    PathState::Symlink {
                        chain: old_chain,
                        hash: old_hash,
                    },
                    PathState::File { hash: new_hash },
                ) => todo!(),
                (
                    PathState::File { hash: old_hash },
                    PathState::Symlink {
                        chain: new_chain,
                        hash: new_hash,
                    },
                ) => todo!(),
                (PathState::File { .. }, PathState::File { .. }) => {
                    ret.push_str(
                        &"~ "
                            .if_supports_color(Stream::Stdout, |text| text.yellow())
                            .to_string(),
                    );
                    ret.push('\n');
                }
            },
        }
    }

    Ok(ret)
}

type Tree = BTreeMap<Utf8PathBuf, Entry>;

enum Entry {
    Added,
    Removed,
    Updated { old: PathState, new: PathState },
    Same,
}

#[derive(PartialEq, Eq)]
enum PathState {
    Symlink { chain: SymlinkChain, hash: Hash },
    File { hash: Hash },
}

impl PathState {
    pub fn hash(&self) -> Hash {
        match &self {
            PathState::Symlink { chain: _, hash } => *hash,
            PathState::File { hash } => *hash,
        }
    }

    fn new(path: &Utf8Path) -> miette::Result<Self> {
        let metadata = path.symlink_metadata().into_diagnostic()?;
        Ok(if metadata.is_symlink() {
            let chain = crate::read_symlink(path.to_path_buf())?;
            let hash = hash_file(&chain.to)?;
            Self::Symlink { chain, hash }
        } else {
            Self::File {
                hash: hash_file(path)?,
            }
        })
    }
}

fn diff_path(
    removed_base: &Utf8Path,
    added_base: &Utf8Path,
    relative: &Utf8Path,
) -> miette::Result<Entry> {
    let removed = removed_base.join(relative);
    let added = added_base.join(relative);

    let removed_state = PathState::new(&removed)?;
    let added_state = PathState::new(&added)?;

    Ok(if removed_state == added_state {
        Entry::Same
    } else if removed_state.hash() == added_state.hash() {
        tracing::debug!(
            "Path updated: {relative}\n\
            - {removed_state}\n\
            + {added_state}",
        );
        Entry::Same
    } else {
        Entry::Updated {
            old: removed_state,
            new: added_state,
        }
    })
}

/// Determine if two files are the same by hashing them.
fn files_are_same(old: &Utf8Path, new: &Utf8Path) -> miette::Result<bool> {
    let old_hash = hash_file(old)?;
    let new_hash = hash_file(new)?;

    Ok(old_hash == new_hash)
}

fn hash_file(path: &Utf8Path) -> miette::Result<Hash> {
    Ok(blake3::Hasher::new()
        .update_mmap(path)
        .into_diagnostic()?
        .finalize())
}

fn walk_trees(
    dirs: impl IntoIterator<Item = impl AsRef<Utf8Path>>,
) -> miette::Result<BTreeMap<Utf8PathBuf, Utf8PathBuf>> {
    let mut tree = BTreeMap::new();
    for path in dirs {
        walk_subtree(&mut tree, path.as_ref(), Utf8Path::new(""))?;
    }
    Ok(tree)
}

fn walk_subtree(
    results: &mut BTreeMap<Utf8PathBuf, Utf8PathBuf>,
    base: &Utf8Path,
    relative: &Utf8Path,
) -> miette::Result<()> {
    let path = base.join(relative);
    let metadata = path.symlink_metadata().into_diagnostic()?;
    if !metadata.is_dir() {
        // TODO: Handle multiple `base`s providing the same path.
        results
            .entry(relative.to_path_buf())
            .or_insert_with(|| base.to_path_buf());
    } else {
        for entry in path.read_dir_utf8().into_diagnostic()? {
            let entry = entry.into_diagnostic()?;
            walk_subtree(results, base, &relative.join(entry.file_name()))?;
        }
    }

    Ok(())
}
