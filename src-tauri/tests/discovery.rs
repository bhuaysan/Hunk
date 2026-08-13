use std::fs;
use std::path::Path;

use hunk_lib::discovery::discover_sources;
use hunk_lib::domain::{
    DiscoveryIssueKind, MediaKind, SourceFormat, SourceSet, TrackKind, ValidationProblemKind,
};
use tempfile::tempdir;

#[test]
fn recursively_discovers_and_deduplicates_supported_source_sets() {
    let temporary = tempdir().unwrap();
    let root = temporary.path();
    let disc = root.join("Sammlung [日本]").join("Spiel mit Leerzeichen");
    fs::create_dir_all(&disc).unwrap();

    let data_track = disc.join("Track [01] データ.bin");
    let audio_track = disc.join("Track 02 音声.bin");
    write_bytes(&data_track, &[1, 2, 3, 4]);
    write_bytes(&audio_track, &[5, 6, 7]);
    let cue = disc.join("Mehrspur Disc.cue");
    write_text(
        &cue,
        "FILE \"Track [01] データ.bin\" BINARY\n  TRACK 01 MODE2/2352\nFILE \"Track 02 音声.bin\" BINARY\n  TRACK 02 AUDIO\n",
    );
    write_bytes(&root.join("standalone.ISO"), &[8, 9]);
    write_bytes(&root.join("archive.ChD"), &[10]);

    let report = discover_sources(&[root.to_path_buf(), cue.clone()]);

    assert!(report.issues.is_empty());
    assert_eq!(report.source_sets.len(), 3);
    let cue_set = source_named(&report.source_sets, "Mehrspur Disc.cue");
    assert_eq!(cue_set.format, SourceFormat::Cue);
    assert_eq!(cue_set.media_kind, MediaKind::Cd);
    assert_eq!(cue_set.referenced_files.len(), 2);
    assert_eq!(cue_set.tracks.len(), 2);
    assert_eq!(cue_set.tracks[0].kind, TrackKind::Data);
    assert_eq!(cue_set.tracks[0].sector_size, Some(2352));
    assert_eq!(cue_set.tracks[1].kind, TrackKind::Audio);
    assert_eq!(cue_set.total_size, file_size(&cue) + 7);
    assert!(cue_set.is_valid());

    let iso = source_named(&report.source_sets, "standalone.ISO");
    assert_eq!(iso.format, SourceFormat::Iso);
    assert_eq!(iso.media_kind, MediaKind::UnknownOptical);
    assert!(source_named(&report.source_sets, "archive.ChD").is_valid());
}

#[test]
fn parses_quoted_gdi_tracks_with_spaces_and_mixed_media() {
    let temporary = tempdir().unwrap();
    let root = temporary.path();
    write_bytes(&root.join("track 01.bin"), &[1]);
    write_bytes(&root.join("track [02].raw"), &[2, 3]);
    write_bytes(&root.join("track03.bin"), &[4, 5, 6]);
    let gdi = root.join("Dreamcast [日本].gdi");
    write_text(
        &gdi,
        "3\n1 0 4 2352 \"track 01.bin\" 0\n2 450 0 2352 \"track [02].raw\" 0\n3 900 4 2352 track03.bin 0\n",
    );

    let report = discover_sources(&[gdi]);
    let source = &report.source_sets[0];

    assert!(report.issues.is_empty());
    assert!(source.is_valid());
    assert_eq!(source.format, SourceFormat::Gdi);
    assert_eq!(source.referenced_files.len(), 3);
    assert_eq!(source.tracks[0].kind, TrackKind::Data);
    assert_eq!(source.tracks[1].kind, TrackKind::Audio);
    assert_eq!(source.tracks[1].start_lba, Some(450));
    assert_eq!(source.tracks[2].kind, TrackKind::Data);
}

#[test]
fn does_not_create_a_separate_job_for_a_referenced_primary_format() {
    let temporary = tempdir().unwrap();
    let root = temporary.path();
    write_bytes(&root.join("track.iso"), &[1, 2, 3]);
    write_text(
        &root.join("disc.cue"),
        "FILE \"track.iso\" BINARY\n  TRACK 01 MODE1/2048\n",
    );

    let report = discover_sources(&[root.to_path_buf()]);

    assert_eq!(report.source_sets.len(), 1);
    assert_eq!(report.source_sets[0].format, SourceFormat::Cue);
}

#[test]
fn accepts_backslash_separators_without_shell_interpretation() {
    let temporary = tempdir().unwrap();
    let root = temporary.path();
    fs::create_dir(root.join("tracks")).unwrap();
    write_bytes(&root.join("tracks").join("Track [01].bin"), &[1]);
    write_text(
        &root.join("disc.cue"),
        "FILE \"tracks\\Track [01].bin\" BINARY\n  TRACK 01 MODE1/2352\n",
    );

    let report = discover_sources(&[root.to_path_buf()]);

    assert!(report.source_sets[0].is_valid());
    assert_eq!(report.source_sets[0].referenced_files.len(), 1);
}

#[test]
fn reports_missing_duplicate_escaping_and_malformed_references() {
    let temporary = tempdir().unwrap();
    let root = temporary.path();
    fs::create_dir(root.join("disc")).unwrap();
    write_bytes(&root.join("outside.bin"), &[1, 2, 3]);
    write_bytes(&root.join("disc").join("same.bin"), &[4]);
    write_text(
        &root.join("disc").join("invalid.cue"),
        "FILE \"same.bin\" BINARY\n  TRACK 01 MODE2/2352\nFILE \"./same.bin\" BINARY\n  TRACK 02 AUDIO\nFILE \"missing.bin\" BINARY\n  TRACK 03 AUDIO\nFILE \"../outside.bin\" BINARY\n  TRACK 04 AUDIO\nFILE \"unterminated.bin BINARY\n",
    );

    let report = discover_sources(&[root.join("disc")]);
    let source = &report.source_sets[0];

    assert_problem(source, ValidationProblemKind::DuplicateReference);
    assert_problem(source, ValidationProblemKind::MissingReference);
    assert_problem(source, ValidationProblemKind::EscapingReference);
    assert_problem(source, ValidationProblemKind::MalformedDescriptor);
    assert_eq!(source.referenced_files.len(), 2);
}

#[test]
fn reports_malformed_gdi_and_track_count_mismatch() {
    let temporary = tempdir().unwrap();
    let gdi = temporary.path().join("broken.gdi");
    write_text(&gdi, "2\n1 0 4 2352 \"unterminated.bin 0\n");

    let report = discover_sources(&[gdi]);
    let source = &report.source_sets[0];

    assert_problem(source, ValidationProblemKind::TrackCountMismatch);
    assert_problem(source, ValidationProblemKind::MalformedDescriptor);
}

#[test]
fn reports_non_utf8_descriptor_as_malformed() {
    let temporary = tempdir().unwrap();
    let cue = temporary.path().join("invalid-encoding.cue");
    write_bytes(&cue, &[0xff, 0xfe, 0xfd]);

    let report = discover_sources(&[cue]);

    assert_problem(
        &report.source_sets[0],
        ValidationProblemKind::MalformedDescriptor,
    );
}

#[test]
fn reports_missing_and_unsupported_explicit_inputs() {
    let temporary = tempdir().unwrap();
    let missing = temporary.path().join("missing.cue");
    let unsupported = temporary.path().join("notes.txt");
    write_text(&unsupported, "not an image");

    let report = discover_sources(&[missing.clone(), unsupported.clone()]);

    assert!(report.source_sets.is_empty());
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.path == missing && issue.kind == DiscoveryIssueKind::InputNotFound)
    );
    assert!(report.issues.iter().any(|issue| {
        issue.path == unsupported && issue.kind == DiscoveryIssueKind::UnsupportedInput
    }));
}

#[cfg(unix)]
#[test]
fn reports_symlink_escape_and_unreadable_reference_without_modifying_sources() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let temporary = tempdir().unwrap();
    let root = temporary.path();
    let disc = root.join("disc");
    fs::create_dir(&disc).unwrap();
    let outside = root.join("outside.bin");
    let unreadable = disc.join("private.bin");
    write_bytes(&outside, &[1, 2, 3]);
    write_bytes(&unreadable, &[4, 5, 6]);
    symlink(&outside, disc.join("linked.bin")).unwrap();
    let original_permissions = fs::metadata(&unreadable).unwrap().permissions();
    let mut blocked_permissions = original_permissions.clone();
    blocked_permissions.set_mode(0o000);
    fs::set_permissions(&unreadable, blocked_permissions).unwrap();
    write_text(
        &disc.join("disc.cue"),
        "FILE \"linked.bin\" BINARY\n  TRACK 01 MODE1/2352\nFILE \"private.bin\" BINARY\n  TRACK 02 AUDIO\n",
    );

    let report = discover_sources(&[disc]);
    fs::set_permissions(&unreadable, original_permissions).unwrap();
    let source = &report.source_sets[0];

    assert_problem(source, ValidationProblemKind::EscapingReference);
    assert_problem(source, ValidationProblemKind::UnreadableReference);
    assert_eq!(fs::read(&outside).unwrap(), [1, 2, 3]);
    assert_eq!(fs::read(&unreadable).unwrap(), [4, 5, 6]);
}

fn write_text(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap();
}

fn write_bytes(path: &Path, contents: &[u8]) {
    fs::write(path, contents).unwrap();
}

fn source_named<'a>(sources: &'a [SourceSet], name: &str) -> &'a SourceSet {
    sources
        .iter()
        .find(|source| source.primary_file.file_name().unwrap() == name)
        .unwrap()
}

fn file_size(path: &Path) -> u64 {
    fs::metadata(path).unwrap().len()
}

fn assert_problem(source: &SourceSet, kind: ValidationProblemKind) {
    assert!(
        source
            .validation_problems
            .iter()
            .any(|problem| problem.kind == kind),
        "expected {kind:?}, got {:?}",
        source.validation_problems
    );
}
