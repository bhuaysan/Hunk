use std::fmt;

use crate::domain::{
    ChdHashes, ChdInfo, ChdMetadata, ChdTrack, JobPhase, JobProgress, MediaKind, TrackKind,
};

const MAX_MESSAGE_CHARS: usize = 2_048;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChdmanErrorKind {
    InputNotFound,
    OutputExists,
    PermissionDenied,
    InvalidData,
    ChecksumMismatch,
    UnsupportedFormat,
    InvalidArguments,
    Io,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChdmanError {
    pub kind: ChdmanErrorKind,
    pub exit_code: Option<i32>,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InfoParseError {
    MissingField(&'static str),
    InvalidField { field: &'static str, value: String },
    InvalidMetadata(String),
}

impl fmt::Display for InfoParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField(field) => write!(formatter, "missing chdman info field: {field}"),
            Self::InvalidField { field, value } => {
                write!(formatter, "invalid chdman info field {field}: {value}")
            }
            Self::InvalidMetadata(value) => write!(formatter, "invalid chdman metadata: {value}"),
        }
    }
}

impl std::error::Error for InfoParseError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationResult {
    pub passed: bool,
    pub raw_sha1_verified: bool,
    pub overall_sha1_verified: bool,
    pub error: Option<ChdmanError>,
}

pub fn parse_info(output: &str) -> Result<ChdInfo, InfoParseError> {
    let mut format_version = None;
    let mut logical_size = None;
    let mut compressed_size = None;
    let mut ratio = None;
    let mut hunk_size = None;
    let mut total_hunks = None;
    let mut unit_size = None;
    let mut total_units = None;
    let mut codecs = Vec::new();
    let mut hashes = ChdHashes {
        sha1: None,
        data_sha1: None,
        parent_sha1: None,
    };
    let mut metadata = Vec::new();
    let mut pending_metadata: Option<(String, u32, u64)> = None;

    for raw_line in output.lines() {
        let line = raw_line.trim_end_matches('\r');
        if let Some((tag, index, length)) = pending_metadata.take() {
            if raw_line.chars().next().is_some_and(char::is_whitespace) && !line.trim().is_empty() {
                metadata.push(ChdMetadata {
                    tag,
                    index,
                    length,
                    value: line.trim().to_owned(),
                });
                continue;
            }
            metadata.push(ChdMetadata {
                tag,
                index,
                length,
                value: String::new(),
            });
        }

        let Some((label, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match label.trim() {
            "File Version" => format_version = Some(parse_u32("File Version", value)?),
            "Logical size" => logical_size = Some(parse_bytes("Logical size", value)?),
            "CHD size" => compressed_size = Some(parse_bytes("CHD size", value)?),
            "Ratio" => ratio = Some(parse_ratio(value)?),
            "Hunk Size" => hunk_size = Some(parse_bytes("Hunk Size", value)?),
            "Total Hunks" => total_hunks = Some(parse_u64("Total Hunks", value)?),
            "Unit Size" => unit_size = Some(parse_bytes("Unit Size", value)?),
            "Total Units" => total_units = Some(parse_u64("Total Units", value)?),
            "Compression" => codecs = parse_codecs(value),
            "SHA1" => hashes.sha1 = nonempty_hash(value),
            "Data SHA1" => hashes.data_sha1 = nonempty_hash(value),
            "Parent SHA1" => hashes.parent_sha1 = nonempty_hash(value),
            "Metadata" => pending_metadata = Some(parse_metadata_header(value)?),
            _ => {}
        }
    }

    if let Some((tag, index, length)) = pending_metadata {
        metadata.push(ChdMetadata {
            tag,
            index,
            length,
            value: String::new(),
        });
    }

    let media_kind = infer_media_kind(&metadata);
    let tracks = metadata.iter().filter_map(parse_track).collect();

    Ok(ChdInfo {
        format_version: required(format_version, "File Version")?,
        media_kind,
        codecs,
        logical_size: required(logical_size, "Logical size")?,
        compressed_size: required(compressed_size, "CHD size")?,
        ratio,
        hunk_size: required(hunk_size, "Hunk Size")?,
        total_hunks: required(total_hunks, "Total Hunks")?,
        unit_size: required(unit_size, "Unit Size")?,
        total_units: required(total_units, "Total Units")?,
        hashes,
        tracks,
        metadata,
    })
}

pub fn parse_progress(line: &str) -> Option<JobProgress> {
    let message = line
        .split('\r')
        .rev()
        .find(|part| !part.trim().is_empty())?
        .trim();
    let (phase, percentage) = if message.starts_with("Compressing,") {
        (JobPhase::Compressing, parse_percentage(message))
    } else if message.starts_with("Extracting,") {
        (JobPhase::Extracting, parse_percentage(message))
    } else if message.starts_with("Verifying,") {
        (JobPhase::Verifying, parse_percentage(message))
    } else if message.starts_with("Compression complete")
        || message.starts_with("Extraction complete")
        || message.starts_with("Verification complete")
    {
        (JobPhase::Complete, Some(100.0))
    } else {
        (JobPhase::Unknown, None)
    };

    Some(JobProgress {
        phase,
        percentage,
        processed_bytes: None,
        elapsed_millis: None,
        message: bounded(message),
    })
}

pub fn parse_verification(
    stdout: &str,
    stderr: &str,
    exit_code: Option<i32>,
) -> VerificationResult {
    let raw_sha1_verified = stdout.contains("Raw SHA1 verification successful!");
    let overall_sha1_verified = stdout.contains("Overall SHA1 verification successful!");
    let mismatch = stderr.contains("SHA1 in header") || stderr.contains("actual SHA1");
    let passed = exit_code == Some(0) && raw_sha1_verified && overall_sha1_verified && !mismatch;
    let error = (!passed).then(|| classify_error(stderr, exit_code));

    VerificationResult {
        passed,
        raw_sha1_verified,
        overall_sha1_verified,
        error,
    }
}

pub fn classify_error(stderr: &str, exit_code: Option<i32>) -> ChdmanError {
    let lower = stderr.to_ascii_lowercase();
    let kind = if lower.contains("sha1 in header") || lower.contains("checksum") {
        ChdmanErrorKind::ChecksumMismatch
    } else if lower.contains("already exists") || lower.contains("file exists") {
        ChdmanErrorKind::OutputExists
    } else if lower.contains("permission denied") || lower.contains("access is denied") {
        ChdmanErrorKind::PermissionDenied
    } else if lower.contains("no such file") || lower.contains("not found") {
        ChdmanErrorKind::InputNotFound
    } else if lower.contains("invalid compressor")
        || lower.contains("required parameter")
        || lower.contains("unknown option")
        || lower.contains("must be specified")
    {
        ChdmanErrorKind::InvalidArguments
    } else if lower.contains("unsupported") || lower.contains("unrecognized") {
        ChdmanErrorKind::UnsupportedFormat
    } else if lower.contains("invalid")
        || lower.contains("corrupt")
        || lower.contains("error reading metadata")
    {
        ChdmanErrorKind::InvalidData
    } else if lower.contains("i/o error")
        || lower.contains("error reading")
        || lower.contains("error writing")
    {
        ChdmanErrorKind::Io
    } else {
        ChdmanErrorKind::Unknown
    };

    ChdmanError {
        kind,
        exit_code,
        message: actionable_message(stderr),
    }
}

fn parse_percentage(message: &str) -> Option<f32> {
    let before_percent = message.split_once('%')?.0;
    let value = before_percent.rsplit_once(' ')?.1.parse::<f32>().ok()?;
    value.is_finite().then_some(value.clamp(0.0, 100.0))
}

fn parse_codecs(value: &str) -> Vec<String> {
    if value == "none" {
        return vec!["none".to_owned()];
    }
    value
        .split(',')
        .filter_map(|entry| entry.split_whitespace().next())
        .filter(|entry| !entry.is_empty())
        .map(str::to_owned)
        .collect()
}

fn parse_metadata_header(value: &str) -> Result<(String, u32, u64), InfoParseError> {
    let tag = if let Some(tag_marker) = value.find("Tag='") {
        let tag_start = tag_marker + 5;
        let tag_end = value[tag_start..]
            .find('\'')
            .map(|offset| tag_start + offset)
            .ok_or_else(|| InfoParseError::InvalidMetadata(value.to_owned()))?;
        value[tag_start..tag_end].to_owned()
    } else {
        let tag_start = value
            .find("Tag=")
            .ok_or_else(|| InfoParseError::InvalidMetadata(value.to_owned()))?
            + 4;
        value[tag_start..]
            .split_whitespace()
            .next()
            .filter(|tag| {
                tag.len() == 8 && tag.chars().all(|character| character.is_ascii_hexdigit())
            })
            .ok_or_else(|| InfoParseError::InvalidMetadata(value.to_owned()))?
            .to_owned()
    };
    let index = token_number(value, "Index=")?;
    let length = token_number(value, "Length=")?;
    Ok((tag, index as u32, length))
}

fn token_number(value: &str, prefix: &str) -> Result<u64, InfoParseError> {
    let start = value
        .find(prefix)
        .ok_or_else(|| InfoParseError::InvalidMetadata(value.to_owned()))?
        + prefix.len();
    let number = value[start..]
        .split_whitespace()
        .next()
        .ok_or_else(|| InfoParseError::InvalidMetadata(value.to_owned()))?;
    number
        .parse()
        .map_err(|_| InfoParseError::InvalidMetadata(value.to_owned()))
}

fn parse_track(metadata: &ChdMetadata) -> Option<ChdTrack> {
    if !matches!(
        metadata.tag.as_str(),
        "CHCD" | "CHTR" | "CHT2" | "CHGD" | "CHGT"
    ) {
        return None;
    }
    let number = metadata_token(&metadata.value, "TRACK")?.parse().ok()?;
    let track_type = metadata_token(&metadata.value, "TYPE").unwrap_or_default();
    let kind = if track_type.eq_ignore_ascii_case("AUDIO") {
        TrackKind::Audio
    } else if track_type.is_empty() {
        TrackKind::Unknown
    } else {
        TrackKind::Data
    };
    Some(ChdTrack {
        number,
        kind,
        frames: metadata_number(&metadata.value, "FRAMES"),
        pregap: metadata_number(&metadata.value, "PREGAP"),
        postgap: metadata_number(&metadata.value, "POSTGAP"),
    })
}

fn metadata_token<'a>(value: &'a str, key: &str) -> Option<&'a str> {
    value.split_whitespace().find_map(|token| {
        let (candidate, value) = token.split_once(':')?;
        (candidate == key).then_some(value)
    })
}

fn metadata_number(value: &str, key: &str) -> Option<u64> {
    metadata_token(value, key)?.parse().ok()
}

fn infer_media_kind(metadata: &[ChdMetadata]) -> MediaKind {
    if metadata.iter().any(|entry| entry.tag == "DVD ") {
        MediaKind::Dvd
    } else if metadata.iter().any(|entry| {
        matches!(
            entry.tag.as_str(),
            "CHCD" | "CHTR" | "CHT2" | "CHGD" | "CHGT"
        )
    }) {
        MediaKind::Cd
    } else {
        MediaKind::UnknownOptical
    }
}

fn parse_bytes(field: &'static str, value: &str) -> Result<u64, InfoParseError> {
    let number = value.strip_suffix(" bytes").unwrap_or(value);
    parse_u64(field, number)
}

fn parse_u64(field: &'static str, value: &str) -> Result<u64, InfoParseError> {
    value
        .replace([',', '.'], "")
        .parse()
        .map_err(|_| InfoParseError::InvalidField {
            field,
            value: value.to_owned(),
        })
}

fn parse_u32(field: &'static str, value: &str) -> Result<u32, InfoParseError> {
    value.parse().map_err(|_| InfoParseError::InvalidField {
        field,
        value: value.to_owned(),
    })
}

fn parse_ratio(value: &str) -> Result<f64, InfoParseError> {
    value
        .strip_suffix('%')
        .unwrap_or(value)
        .trim()
        .parse()
        .map_err(|_| InfoParseError::InvalidField {
            field: "Ratio",
            value: value.to_owned(),
        })
}

fn required<T>(value: Option<T>, field: &'static str) -> Result<T, InfoParseError> {
    value.ok_or(InfoParseError::MissingField(field))
}

fn nonempty_hash(value: &str) -> Option<String> {
    (!value.is_empty() && value != "(none)").then(|| value.to_owned())
}

fn actionable_message(stderr: &str) -> String {
    let message = stderr
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with("Error:"))
        .or_else(|| stderr.lines().rev().find(|line| !line.trim().is_empty()))
        .unwrap_or("chdman failed without an error message")
        .trim();
    bounded(message)
}

fn bounded(value: &str) -> String {
    value.chars().take(MAX_MESSAGE_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_progress_is_indeterminate() {
        let progress = parse_progress("Preparing codec tables...").unwrap();

        assert_eq!(progress.phase, JobPhase::Unknown);
        assert_eq!(progress.percentage, None);
    }

    #[test]
    fn progress_percentage_is_parsed_and_clamped() {
        let progress = parse_progress("Compressing, 42.5% complete... (ratio=61.2%)\r").unwrap();

        assert_eq!(progress.phase, JobPhase::Compressing);
        assert_eq!(progress.percentage, Some(42.5));
    }

    #[test]
    fn classifies_common_actionable_errors() {
        assert_eq!(
            classify_error("Error: output file already exists", Some(1)).kind,
            ChdmanErrorKind::OutputExists
        );
        assert_eq!(
            classify_error("Error: permission denied", Some(1)).kind,
            ChdmanErrorKind::PermissionDenied
        );
        assert_eq!(
            classify_error("Error: Raw SHA1 in header = abc", Some(0)).kind,
            ChdmanErrorKind::ChecksumMismatch
        );
    }
}
