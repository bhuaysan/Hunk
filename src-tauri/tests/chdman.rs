use hunk_lib::chdman::{
    ChdmanErrorKind, classify_error, parse_info, parse_progress, parse_verification,
};
use hunk_lib::domain::{JobPhase, MediaKind, TrackKind};

const INFO_CD: &str = include_str!("fixtures/chdman/info_cd.txt");
const INFO_DVD: &str = include_str!("fixtures/chdman/info_dvd.txt");
const VERIFY_SUCCESS: &str = include_str!("fixtures/chdman/verify_success.stdout.txt");
const VERIFY_MISMATCH: &str = include_str!("fixtures/chdman/verify_mismatch.stderr.txt");
const EXISTING_OUTPUT: &str = include_str!("fixtures/chdman/errors.stderr.txt");

#[test]
fn parses_cd_information_and_track_metadata() {
    let info = parse_info(INFO_CD).unwrap();

    assert_eq!(info.format_version, 5);
    assert_eq!(info.media_kind, MediaKind::Cd);
    assert_eq!(info.logical_size, 734_003_200);
    assert_eq!(info.compressed_size, 449_742_848);
    assert_eq!(info.ratio, Some(61.3));
    assert_eq!(info.codecs, ["cdlz", "cdzl", "cdfl"]);
    assert_eq!(info.tracks.len(), 2);
    assert_eq!(info.tracks[0].kind, TrackKind::Data);
    assert_eq!(info.tracks[1].kind, TrackKind::Audio);
    assert_eq!(info.tracks[1].pregap, Some(150));
    assert_eq!(info.metadata[0].tag, "CHT2");
}

#[test]
fn parses_dvd_information() {
    let info = parse_info(INFO_DVD).unwrap();

    assert_eq!(info.media_kind, MediaKind::Dvd);
    assert_eq!(info.unit_size, 2_048);
    assert_eq!(info.hashes.parent_sha1, None);
    assert!(info.tracks.is_empty());
}

#[test]
fn parses_progress_and_preserves_unknown_output_as_indeterminate() {
    let compressing = parse_progress("Compressing, 73.4% complete... (ratio=61.0%)\r").unwrap();
    assert_eq!(compressing.phase, JobPhase::Compressing);
    assert_eq!(compressing.percentage, Some(73.4));

    let unknown = parse_progress("Rebuilding codec table").unwrap();
    assert_eq!(unknown.phase, JobPhase::Unknown);
    assert_eq!(unknown.percentage, None);
}

#[test]
fn verification_requires_both_hashes_and_no_mismatch() {
    let success = parse_verification(VERIFY_SUCCESS, "", Some(0));
    assert!(success.passed);
    assert!(success.raw_sha1_verified);
    assert!(success.overall_sha1_verified);

    let mismatch = parse_verification("", VERIFY_MISMATCH, Some(0));
    assert!(!mismatch.passed);
    assert_eq!(
        mismatch.error.unwrap().kind,
        ChdmanErrorKind::ChecksumMismatch
    );
}

#[test]
fn classifies_golden_error_output() {
    let error = classify_error(EXISTING_OUTPUT, Some(1));

    assert_eq!(error.kind, ChdmanErrorKind::OutputExists);
    assert_eq!(error.exit_code, Some(1));
}
