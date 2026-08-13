use std::path::PathBuf;

use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaKind {
    Cd,
    Dvd,
    UnknownOptical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceFormat {
    Cue,
    Gdi,
    Iso,
    Chd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TrackKind {
    Data,
    Audio,
    Subchannel,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub number: u32,
    pub kind: TrackKind,
    pub source_file: String,
    pub start_lba: Option<u64>,
    pub sector_size: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ValidationProblemKind {
    MissingReference,
    DuplicateReference,
    EscapingReference,
    UnreadableReference,
    UnreadablePrimary,
    MalformedDescriptor,
    DuplicateTrack,
    TrackCountMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationProblem {
    pub kind: ValidationProblemKind,
    pub line: Option<usize>,
    pub reference: Option<String>,
}

impl ValidationProblem {
    pub(crate) fn new(
        kind: ValidationProblemKind,
        line: Option<usize>,
        reference: Option<String>,
    ) -> Self {
        Self {
            kind,
            line,
            reference,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSet {
    pub primary_file: PathBuf,
    pub referenced_files: Vec<PathBuf>,
    pub format: SourceFormat,
    pub media_kind: MediaKind,
    pub tracks: Vec<Track>,
    pub total_size: u64,
    pub validation_problems: Vec<ValidationProblem>,
}

impl SourceSet {
    pub fn is_valid(&self) -> bool {
        self.validation_problems.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DiscoveryIssueKind {
    InputNotFound,
    InputUnreadable,
    UnsupportedInput,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryIssue {
    pub kind: DiscoveryIssueKind,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryReport {
    pub source_sets: Vec<SourceSet>,
    pub issues: Vec<DiscoveryIssue>,
}
