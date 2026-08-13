mod cue;
mod gdi;

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use crate::domain::{
    DiscoveryIssue, DiscoveryIssueKind, DiscoveryReport, MediaKind, SourceFormat, SourceSet, Track,
    TrackKind, ValidationProblem, ValidationProblemKind,
};

#[derive(Debug, Default)]
struct ParsedDescriptor {
    references: Vec<ParsedReference>,
    tracks: Vec<ParsedTrack>,
    problems: Vec<ValidationProblem>,
}

#[derive(Debug)]
struct ParsedReference {
    raw: String,
    line: usize,
}

#[derive(Debug)]
struct ParsedTrack {
    number: u32,
    kind: TrackKind,
    source_reference: String,
    start_lba: Option<u64>,
    sector_size: Option<u32>,
}

struct BuiltSource {
    identity: PathBuf,
    source_set: SourceSet,
    referenced_identities: BTreeSet<PathBuf>,
}

pub fn discover_sources(inputs: &[PathBuf]) -> DiscoveryReport {
    let mut candidates = BTreeMap::new();
    let mut issues = Vec::new();
    let mut visited_directories = BTreeSet::new();

    for input in inputs {
        collect_input(
            input,
            &mut candidates,
            &mut visited_directories,
            &mut issues,
        );
    }

    let built_sources: Vec<_> = candidates
        .into_iter()
        .map(|(path, format)| build_source(path, format))
        .collect();
    let referenced_identities: BTreeSet<_> = built_sources
        .iter()
        .flat_map(|source| source.referenced_identities.iter().cloned())
        .collect();

    let mut source_sets: Vec<_> = built_sources
        .into_iter()
        .filter(|source| !referenced_identities.contains(&source.identity))
        .map(|source| source.source_set)
        .collect();
    source_sets.sort_by(|left, right| left.primary_file.cmp(&right.primary_file));
    issues.sort_by(|left, right| left.path.cmp(&right.path));
    issues.dedup();

    DiscoveryReport {
        source_sets,
        issues,
    }
}

fn collect_input(
    input: &Path,
    candidates: &mut BTreeMap<PathBuf, SourceFormat>,
    visited_directories: &mut BTreeSet<PathBuf>,
    issues: &mut Vec<DiscoveryIssue>,
) {
    let absolute = absolute_path(input);
    let metadata = match fs::metadata(&absolute) {
        Ok(metadata) => metadata,
        Err(error) => {
            issues.push(DiscoveryIssue {
                kind: issue_kind(&error),
                path: absolute,
            });
            return;
        }
    };

    if metadata.is_dir() {
        walk_directory(&absolute, candidates, visited_directories, issues);
    } else if metadata.is_file() {
        if let Some(format) = source_format(&absolute) {
            insert_candidate(&absolute, format, candidates);
        } else {
            issues.push(DiscoveryIssue {
                kind: DiscoveryIssueKind::UnsupportedInput,
                path: absolute,
            });
        }
    } else {
        issues.push(DiscoveryIssue {
            kind: DiscoveryIssueKind::UnsupportedInput,
            path: absolute,
        });
    }
}

fn walk_directory(
    directory: &Path,
    candidates: &mut BTreeMap<PathBuf, SourceFormat>,
    visited_directories: &mut BTreeSet<PathBuf>,
    issues: &mut Vec<DiscoveryIssue>,
) {
    let identity = fs::canonicalize(directory).unwrap_or_else(|_| directory.to_path_buf());
    if !visited_directories.insert(identity) {
        return;
    }

    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => {
            issues.push(DiscoveryIssue {
                kind: DiscoveryIssueKind::InputUnreadable,
                path: directory.to_path_buf(),
            });
            return;
        }
    };
    let mut entries: Vec<_> = entries.collect();
    entries.sort_by_key(|entry| entry.as_ref().ok().map(|entry| entry.path()));

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                issues.push(DiscoveryIssue {
                    kind: DiscoveryIssueKind::InputUnreadable,
                    path: directory.to_path_buf(),
                });
                continue;
            }
        };
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => {
                issues.push(DiscoveryIssue {
                    kind: DiscoveryIssueKind::InputUnreadable,
                    path,
                });
                continue;
            }
        };

        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            walk_directory(&path, candidates, visited_directories, issues);
        } else if file_type.is_file()
            && let Some(format) = source_format(&path)
        {
            insert_candidate(&path, format, candidates);
        }
    }
}

fn insert_candidate(
    path: &Path,
    format: SourceFormat,
    candidates: &mut BTreeMap<PathBuf, SourceFormat>,
) {
    let identity = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    candidates.entry(identity).or_insert(format);
}

fn build_source(primary_file: PathBuf, format: SourceFormat) -> BuiltSource {
    let mut source_set = SourceSet {
        total_size: fs::metadata(&primary_file)
            .map(|metadata| metadata.len())
            .unwrap_or(0),
        primary_file: primary_file.clone(),
        referenced_files: Vec::new(),
        format,
        media_kind: match format {
            SourceFormat::Cue | SourceFormat::Gdi => MediaKind::Cd,
            SourceFormat::Iso | SourceFormat::Chd => MediaKind::UnknownOptical,
        },
        tracks: Vec::new(),
        validation_problems: Vec::new(),
    };
    let mut referenced_identities = BTreeSet::new();

    match format {
        SourceFormat::Cue | SourceFormat::Gdi => match fs::read_to_string(&primary_file) {
            Ok(contents) => {
                let parsed = if format == SourceFormat::Cue {
                    cue::parse(&contents)
                } else {
                    gdi::parse(&contents)
                };
                source_set.validation_problems.extend(parsed.problems);
                source_set.tracks = parsed
                    .tracks
                    .into_iter()
                    .map(|track| Track {
                        number: track.number,
                        kind: track.kind,
                        source_file: track.source_reference,
                        start_lba: track.start_lba,
                        sector_size: track.sector_size,
                    })
                    .collect();
                validate_references(
                    &primary_file,
                    parsed.references,
                    &mut source_set,
                    &mut referenced_identities,
                );
            }
            Err(error) => {
                let kind = if error.kind() == io::ErrorKind::InvalidData {
                    ValidationProblemKind::MalformedDescriptor
                } else {
                    ValidationProblemKind::UnreadablePrimary
                };
                source_set
                    .validation_problems
                    .push(ValidationProblem::new(kind, None, None));
            }
        },
        SourceFormat::Iso | SourceFormat::Chd => {
            if File::open(&primary_file).is_err() {
                source_set.validation_problems.push(ValidationProblem::new(
                    ValidationProblemKind::UnreadablePrimary,
                    None,
                    None,
                ));
            }
        }
    }

    BuiltSource {
        identity: primary_file,
        source_set,
        referenced_identities,
    }
}

fn validate_references(
    primary_file: &Path,
    references: Vec<ParsedReference>,
    source_set: &mut SourceSet,
    referenced_identities: &mut BTreeSet<PathBuf>,
) {
    let base = primary_file.parent().unwrap_or_else(|| Path::new("."));
    let base_identity = fs::canonicalize(base).unwrap_or_else(|_| base.to_path_buf());
    let mut seen = BTreeSet::new();

    for reference in references {
        let resolved = match resolve_reference(base, &reference.raw) {
            Ok(path) => path,
            Err(ReferencePathError::Escaping) => {
                source_set.validation_problems.push(ValidationProblem::new(
                    ValidationProblemKind::EscapingReference,
                    Some(reference.line),
                    Some(reference.raw),
                ));
                continue;
            }
            Err(ReferencePathError::Malformed) => {
                source_set.validation_problems.push(ValidationProblem::new(
                    ValidationProblemKind::MalformedDescriptor,
                    Some(reference.line),
                    Some(reference.raw),
                ));
                continue;
            }
        };

        if !seen.insert(resolved.clone()) {
            source_set.validation_problems.push(ValidationProblem::new(
                ValidationProblemKind::DuplicateReference,
                Some(reference.line),
                Some(reference.raw),
            ));
            continue;
        }
        source_set.referenced_files.push(resolved.clone());

        let identity = match fs::canonicalize(&resolved) {
            Ok(identity) => identity,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                source_set.validation_problems.push(ValidationProblem::new(
                    ValidationProblemKind::MissingReference,
                    Some(reference.line),
                    Some(reference.raw),
                ));
                continue;
            }
            Err(_) => {
                source_set.validation_problems.push(ValidationProblem::new(
                    ValidationProblemKind::UnreadableReference,
                    Some(reference.line),
                    Some(reference.raw),
                ));
                continue;
            }
        };

        if !identity.starts_with(&base_identity) {
            source_set.validation_problems.push(ValidationProblem::new(
                ValidationProblemKind::EscapingReference,
                Some(reference.line),
                Some(reference.raw),
            ));
            continue;
        }
        referenced_identities.insert(identity.clone());

        let metadata = match fs::metadata(&identity) {
            Ok(metadata) if metadata.is_file() && File::open(&identity).is_ok() => metadata,
            _ => {
                source_set.validation_problems.push(ValidationProblem::new(
                    ValidationProblemKind::UnreadableReference,
                    Some(reference.line),
                    Some(reference.raw),
                ));
                continue;
            }
        };
        source_set.total_size = source_set.total_size.saturating_add(metadata.len());
    }

    source_set.referenced_files.sort();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReferencePathError {
    Escaping,
    Malformed,
}

fn resolve_reference(base: &Path, raw: &str) -> Result<PathBuf, ReferencePathError> {
    if raw.is_empty() || raw.contains('\0') {
        return Err(ReferencePathError::Malformed);
    }

    let normalized = raw.replace('\\', "/");
    let bytes = normalized.as_bytes();
    let has_windows_drive = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    if normalized.starts_with('/') || has_windows_drive {
        return Err(ReferencePathError::Escaping);
    }

    let mut components = Vec::new();
    for component in normalized.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if components.pop().is_none() {
                    return Err(ReferencePathError::Escaping);
                }
            }
            component => components.push(component),
        }
    }
    if components.is_empty() {
        return Err(ReferencePathError::Malformed);
    }

    let mut resolved = base.to_path_buf();
    for component in components {
        resolved.push(component);
    }
    Ok(resolved)
}

fn source_format(path: &Path) -> Option<SourceFormat> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "cue" => Some(SourceFormat::Cue),
        "gdi" => Some(SourceFormat::Gdi),
        "iso" => Some(SourceFormat::Iso),
        "chd" => Some(SourceFormat::Chd),
        _ => None,
    }
}

fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn issue_kind(error: &io::Error) -> DiscoveryIssueKind {
    if error.kind() == io::ErrorKind::NotFound {
        DiscoveryIssueKind::InputNotFound
    } else {
        DiscoveryIssueKind::InputUnreadable
    }
}
