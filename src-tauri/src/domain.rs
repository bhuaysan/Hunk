use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaKind {
    Cd,
    Dvd,
    UnknownOptical,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    CreateCd,
    CreateDvd,
    ExtractCd,
    ExtractDvd,
    Verify,
    Info,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JobPhase {
    Inspecting,
    Compressing,
    Extracting,
    Verifying,
    Complete,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgress {
    pub phase: JobPhase,
    pub percentage: Option<f32>,
    pub processed_bytes: Option<u64>,
    pub elapsed_millis: Option<u64>,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChdHashes {
    pub sha1: Option<String>,
    pub data_sha1: Option<String>,
    pub parent_sha1: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChdTrack {
    pub number: u32,
    pub kind: TrackKind,
    pub frames: Option<u64>,
    pub pregap: Option<u64>,
    pub postgap: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChdMetadata {
    pub tag: String,
    pub index: u32,
    pub length: u64,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChdInfo {
    pub format_version: u32,
    pub media_kind: MediaKind,
    pub codecs: Vec<String>,
    pub logical_size: u64,
    pub compressed_size: u64,
    pub ratio: Option<f64>,
    pub hunk_size: u64,
    pub total_hunks: u64,
    pub unit_size: u64,
    pub total_units: u64,
    pub hashes: ChdHashes,
    pub tracks: Vec<ChdTrack>,
    pub metadata: Vec<ChdMetadata>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceFormat {
    Cue,
    Gdi,
    Iso,
    Chd,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TrackKind {
    Data,
    Audio,
    Subchannel,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub number: u32,
    pub kind: TrackKind,
    pub source_file: String,
    pub start_lba: Option<u64>,
    pub sector_size: Option<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DiscoveryIssueKind {
    InputNotFound,
    InputUnreadable,
    UnsupportedInput,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryIssue {
    pub kind: DiscoveryIssueKind,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryReport {
    pub source_sets: Vec<SourceSet>,
    pub issues: Vec<DiscoveryIssue>,
}
