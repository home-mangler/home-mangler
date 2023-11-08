use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs::Metadata;
use std::os::unix::prelude::MetadataExt;
use std::path::Path;

use camino::Utf8Path;
use camino::Utf8PathBuf;
use miette::Context;
use miette::IntoDiagnostic;
use owo_colors::OwoColorize;
use owo_colors::Stream;
use tap::TryConv;
use walkdir::WalkDir;

pub fn diff_trees(
    removed_paths: &BTreeSet<&Utf8Path>,
    added_paths: &BTreeSet<&Utf8Path>,
) -> miette::Result<String> {
    let diff = walk_trees(removed_paths, added_paths)?;

    Ok(display_diff(&diff))
}

fn display_diff(diff: &Diff) -> String {
    let mut ret = String::new();
    let mut changed_entries = 0;

    for (path, entry) in diff {
        match entry.kind {
            DiffKind::Same => {}
            DiffKind::Added => {
                ret.push_str(
                    &format!("+ {path}")
                        .if_supports_color(Stream::Stdout, |text| text.green())
                        .to_string(),
                );
                ret.push('\n');
                changed_entries += 1;
            }
            DiffKind::Removed => {
                ret.push_str(
                    &format!("- {path}")
                        .if_supports_color(Stream::Stdout, |text| text.red())
                        .to_string(),
                );
                ret.push('\n');
                changed_entries += 1;
            }
            DiffKind::Changed => {
                if entry
                    .new
                    .as_ref()
                    .map(|info| info.metadata.is_dir())
                    .unwrap_or(false)
                {
                    continue;
                }

                ret.push_str(
                    &format!("~ {path}")
                        .if_supports_color(Stream::Stdout, |text| text.yellow())
                        .to_string(),
                );
                ret.push('\n');
                changed_entries += 1;
            }
        }
    }

    tracing::debug!("{changed_entries} entries updated");

    ret
}

type Diff<'a> = BTreeMap<Utf8PathBuf, FullDiffEntry<'a>>;

#[derive(Debug)]
enum DiffKind {
    Same,
    Removed,
    Changed,
    Added,
}

#[derive(Debug)]
struct PathInfo<'a> {
    metadata: Metadata,
    #[allow(dead_code)]
    base: &'a Utf8Path,
}

#[derive(Debug)]
struct FullDiffEntry<'a> {
    kind: DiffKind,
    old: Option<PathInfo<'a>>,
    new: Option<PathInfo<'a>>,
}

fn walk_trees<'a>(
    removed_paths: &'a BTreeSet<&'a Utf8Path>,
    added_paths: &'a BTreeSet<&'a Utf8Path>,
) -> miette::Result<Diff<'a>> {
    let mut diff = BTreeMap::new();

    walk_removed_trees(&mut diff, removed_paths, added_paths)?;
    walk_added_trees(&mut diff, added_paths)?;

    Ok(diff)
}

fn walk_removed_trees<'a>(
    diff: &mut Diff<'a>,
    removed_paths: &'a BTreeSet<&'a Utf8Path>,
    added_paths: &'a BTreeSet<&'a Utf8Path>,
) -> miette::Result<()> {
    for removed_base in removed_paths {
        walk_removed_tree(diff, removed_base, added_paths)?;
    }

    Ok(())
}

fn walk_removed_tree<'a>(
    diff: &mut Diff<'a>,
    removed_base: &'a Utf8Path,
    added_paths: &'a BTreeSet<&'a Utf8Path>,
) -> miette::Result<()> {
    let walker = WalkDir::new(removed_base).follow_links(true);
    let mut iterator = walker.into_iter();

    loop {
        let removed_entry = match iterator.next() {
            Some(entry) => entry
                .into_diagnostic()
                .wrap_err_with(|| format!("Failed to traverse {removed_base}")),
            None => break,
        }?;

        if removed_entry.depth() == 0 {
            continue;
        }

        let relative = strip_prefix(removed_entry.path(), removed_base)?;

        let removed_metadata = removed_entry
            .metadata()
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to query metadata for {:?}", removed_entry.path()))?;

        let mut entry = FullDiffEntry {
            kind: DiffKind::Removed,
            old: None,
            new: None,
        };
        for added_base in added_paths {
            let candidate = added_base.join(&relative);
            let candidate_metadata = match candidate.metadata() {
                Ok(metadata) => metadata,
                Err(err) => {
                    tracing::debug!(
                        "Failed to read metadata for candidate path {candidate}: {err}"
                    );
                    continue;
                }
            };

            entry.kind = candidate_is_same(
                removed_entry.path(),
                &removed_metadata,
                candidate.as_std_path(),
                &candidate_metadata,
            )?;
            entry.new = Some(PathInfo {
                metadata: candidate_metadata,
                base: added_base,
            });

            break;
        }

        if removed_entry.file_type().is_dir() {
            if let DiffKind::Removed = entry.kind {
                // Don't recurse if a directory has been removed.
                iterator.skip_current_dir();
            }
        }

        entry.old = Some(PathInfo {
            metadata: removed_metadata,
            base: removed_base,
        });
        diff.insert(relative, entry);
    }
    Ok(())
}

fn walk_added_trees<'a>(
    diff: &mut Diff<'a>,
    added_paths: &'a BTreeSet<&'a Utf8Path>,
) -> miette::Result<()> {
    for added_base in added_paths {
        walk_added_tree(diff, added_base)?;
    }

    Ok(())
}

fn walk_added_tree<'a>(diff: &mut Diff<'a>, added_base: &'a Utf8Path) -> miette::Result<()> {
    let walker = WalkDir::new(added_base).follow_links(true);
    let mut iterator = walker.into_iter();

    loop {
        let added_entry = match iterator.next() {
            Some(entry) => entry
                .into_diagnostic()
                .wrap_err_with(|| format!("Failed to traverse {added_base}")),
            None => break,
        }?;

        if added_entry.depth() == 0 {
            continue;
        }

        let relative = strip_prefix(added_entry.path(), added_base)?;

        if let Some(diff_entry) = diff.get(&relative) {
            if let DiffKind::Removed = diff_entry.kind {
                // Don't recurse if a directory has been removed.
                iterator.skip_current_dir();
                continue;
            }
        } else {
            if added_entry.file_type().is_dir() {
                iterator.skip_current_dir();
            }

            diff.insert(
                relative,
                FullDiffEntry {
                    kind: DiffKind::Added,
                    old: None,
                    new: Some(PathInfo {
                        metadata: added_entry
                            .path()
                            .metadata()
                            .into_diagnostic()
                            .wrap_err_with(|| {
                                format!("Failed to query metadata for {:?}", added_entry.path())
                            })?,
                        base: added_base,
                    }),
                },
            );
        }
    }
    Ok(())
}

fn candidate_is_same(
    removed_path: &Path,
    removed_metadata: &Metadata,
    candidate_path: &Path,
    candidate_metadata: &Metadata,
) -> miette::Result<DiffKind> {
    Ok(
        if (candidate_metadata.dev(), candidate_metadata.ino())
            == (removed_metadata.dev(), removed_metadata.ino())
        {
            DiffKind::Same
        } else if removed_metadata.is_dir()
            || candidate_metadata.is_dir()
            || candidate_metadata.len() != removed_metadata.len()
            || hash_file(removed_path)? != hash_file(candidate_path)?
        {
            DiffKind::Changed
        } else {
            DiffKind::Same
        },
    )
}

fn hash_file(path: impl AsRef<Path>) -> miette::Result<blake3::Hash> {
    let path = path.as_ref();
    tracing::debug!("Hashing {path:?}");
    Ok(blake3::Hasher::new()
        .update_mmap(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to hash {path:?}"))?
        .finalize())
}

fn strip_prefix(path: &Path, prefix: impl AsRef<Path>) -> miette::Result<Utf8PathBuf> {
    let prefix = prefix.as_ref();
    path.strip_prefix(prefix)
        .into_diagnostic()
        .wrap_err_with(|| format!("Path {path:?} doesn't start with {prefix:?}",))?
        .to_path_buf()
        .try_conv::<Utf8PathBuf>()
        .into_diagnostic()
}
