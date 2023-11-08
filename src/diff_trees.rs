use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs::File;
use std::fs::Metadata;
use std::io::BufReader;
use std::io::Read;
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
            }
            DiffKind::Removed => {
                ret.push_str(
                    &format!("- {path}")
                        .if_supports_color(Stream::Stdout, |text| text.red())
                        .to_string(),
                );
                ret.push('\n');
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
            }
        }
    }

    ret
}

type Diff<'a> = BTreeMap<Utf8PathBuf, FullDiffEntry<'a>>;

enum DiffKind {
    Same,
    Removed,
    Changed,
    Added,
}

struct PathInfo<'a> {
    metadata: Metadata,
    base: &'a Utf8Path,
}

struct FullDiffEntry<'a> {
    kind: DiffKind,
    old: Option<PathInfo<'a>>,
    new: Option<PathInfo<'a>>,
}

fn walk_trees<'a>(
    removed_paths: &'a BTreeSet<&'a Utf8Path>,
    added_paths: &'a BTreeSet<&'a Utf8Path>,
) -> miette::Result<Diff<'a>> {
    let mut tree = BTreeMap::new();

    for removed_base in removed_paths {
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

            let relative = removed_entry
                .path()
                .strip_prefix(removed_base)
                .into_diagnostic()
                .wrap_err_with(|| {
                    format!(
                        "Path {:?} doesn't start with {}",
                        removed_entry.path(),
                        removed_base
                    )
                })?
                .to_path_buf()
                .try_conv::<Utf8PathBuf>()
                .into_diagnostic()?;

            let removed_metadata =
                removed_entry
                    .metadata()
                    .into_diagnostic()
                    .wrap_err_with(|| {
                        format!("Failed to query metadata for {:?}", removed_entry.path())
                    })?;

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

                entry.kind = if (candidate_metadata.dev(), candidate_metadata.ino())
                    == (removed_metadata.dev(), removed_metadata.ino())
                {
                    DiffKind::Same
                } else if removed_metadata.is_dir()
                    || candidate_metadata.is_dir()
                    || candidate_metadata.len() != removed_metadata.len()
                    || !file_contents_same(removed_entry.path(), &candidate).wrap_err_with(
                        || {
                            format!(
                                "Failed to check file contents for {:?} and {:?}",
                                removed_entry.path(),
                                candidate
                            )
                        },
                    )?
                {
                    DiffKind::Changed
                } else {
                    DiffKind::Same
                };
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
            tree.insert(relative, entry);
        }
    }

    for added_base in added_paths {
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

            let relative = added_entry
                .path()
                .strip_prefix(added_base)
                .into_diagnostic()
                .wrap_err_with(|| {
                    format!(
                        "Path {:?} doesn't start with {}",
                        added_entry.path(),
                        added_base
                    )
                })?
                .to_path_buf()
                .try_conv::<Utf8PathBuf>()
                .into_diagnostic()?;

            if let Some(diff_entry) = tree.get(&relative) {
                if let DiffKind::Removed = diff_entry.kind {
                    // Don't recurse if a directory has been removed.
                    iterator.skip_current_dir();
                    continue;
                }
            } else {
                tree.insert(
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
    }

    Ok(tree)
}

fn file_contents_same(path1: impl AsRef<Path>, path2: impl AsRef<Path>) -> miette::Result<bool> {
    let path1 = path1.as_ref();
    let path2 = path2.as_ref();
    tracing::debug!(?path1, ?path2, "Comparing file contents");

    let reader1 = BufReader::new(
        File::open(path1)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to open {path1:?}"))?,
    );
    let reader2 = BufReader::new(
        File::open(path2)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to open {path2:?}"))?,
    );

    for (byte1, byte2) in reader1.bytes().zip(reader2.bytes()) {
        let byte1 = byte1.into_diagnostic()?;
        let byte2 = byte2.into_diagnostic()?;
        if byte1 != byte2 {
            return Ok(false);
        }
    }

    Ok(true)
}
