#![cfg(unix)]

use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use hunk_lib::discovery::discover_sources;
use hunk_lib::domain::{MediaKind, Operation, SourceFormat, SourceSet};
use hunk_lib::jobs::{JobEngine, JobOptions, JobRecord, JobSpec, JobState, NoopEventSink};
use tempfile::{TempDir, tempdir};

const FAKE_TIMEOUT: Duration = Duration::from_secs(10);
const REAL_TIMEOUT: Duration = Duration::from_secs(180);
const LOCAL_TIMEOUT: Duration = Duration::from_secs(20 * 60);

struct SourceSnapshot {
    path: PathBuf,
    length: u64,
    digest: u64,
}

impl SourceSnapshot {
    fn capture(source: &SourceSet) -> Vec<Self> {
        std::iter::once(&source.primary_file)
            .chain(&source.referenced_files)
            .map(|path| Self {
                path: path.clone(),
                length: fs::metadata(path).unwrap().len(),
                digest: file_digest(path),
            })
            .collect()
    }

    fn assert_unchanged(snapshots: &[Self]) {
        for snapshot in snapshots {
            assert_eq!(
                fs::metadata(&snapshot.path).unwrap().len(),
                snapshot.length,
                "source length changed: {}",
                snapshot.path.display()
            );
            assert_eq!(
                file_digest(&snapshot.path),
                snapshot.digest,
                "source contents changed: {}",
                snapshot.path.display()
            );
        }
    }
}

struct Harness {
    _directory: TempDir,
    engine: Arc<JobEngine>,
}

impl Harness {
    fn fake() -> Self {
        let directory = tempdir().unwrap();
        let program = fake_chdman(directory.path());
        let engine = JobEngine::open(
            &directory.path().join("jobs.sqlite3"),
            program,
            Arc::new(NoopEventSink),
        )
        .unwrap();
        Self {
            _directory: directory,
            engine,
        }
    }

    fn with_program(program: PathBuf) -> Self {
        let directory = tempdir().unwrap();
        let engine = JobEngine::open(
            &directory.path().join("jobs.sqlite3"),
            program,
            Arc::new(NoopEventSink),
        )
        .unwrap();
        Self {
            _directory: directory,
            engine,
        }
    }

    fn enqueue(
        &self,
        source: SourceSet,
        operation: Operation,
        destination: Option<PathBuf>,
    ) -> JobRecord {
        self.engine
            .enqueue(JobSpec {
                source,
                operation,
                destination,
                options: JobOptions::default(),
            })
            .unwrap()
    }

    fn wait_for(&self, id: &str, states: &[JobState], timeout: Duration) -> JobRecord {
        wait_for(&self.engine, id, states, timeout)
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.engine.shutdown();
    }
}

#[test]
fn generated_sources_complete_create_extract_verify_and_info_flows() {
    let workspace = tempdir().unwrap();
    let fixtures = generate_fixtures(workspace.path());
    let harness = Harness::fake();

    let cd = source_with_format(&fixtures, SourceFormat::Cue);
    let cd_snapshot = SourceSnapshot::capture(&cd);
    let cd_output = workspace.path().join("generated CD.chd");
    let created_cd = harness.enqueue(cd, Operation::CreateCd, Some(cd_output.clone()));
    harness.wait_for(&created_cd.id, &[JobState::Completed], FAKE_TIMEOUT);
    SourceSnapshot::assert_unchanged(&cd_snapshot);

    let chd = discover_one(&cd_output);
    let info = harness.enqueue(chd.clone(), Operation::Info, None);
    let info = harness.wait_for(&info.id, &[JobState::Completed], FAKE_TIMEOUT);
    assert_eq!(info.chd_info.unwrap().media_kind, MediaKind::Cd);
    let verify = harness.enqueue(chd.clone(), Operation::Verify, None);
    harness.wait_for(&verify.id, &[JobState::Completed], FAKE_TIMEOUT);

    let extracted_cue = workspace.path().join("round trip CD.cue");
    let extract = harness.enqueue(chd, Operation::ExtractCd, Some(extracted_cue.clone()));
    harness.wait_for(&extract.id, &[JobState::Completed], FAKE_TIMEOUT);
    let extracted = discover_one(&extracted_cue);
    let recreated = harness.enqueue(
        extracted,
        Operation::CreateCd,
        Some(workspace.path().join("recreated CD.chd")),
    );
    harness.wait_for(&recreated.id, &[JobState::Completed], FAKE_TIMEOUT);

    let dvd = source_with_format(&fixtures, SourceFormat::Iso);
    let dvd_snapshot = SourceSnapshot::capture(&dvd);
    let dvd_output = workspace.path().join("generated DVD.chd");
    let created_dvd = harness.enqueue(dvd, Operation::CreateDvd, Some(dvd_output.clone()));
    harness.wait_for(&created_dvd.id, &[JobState::Completed], FAKE_TIMEOUT);
    SourceSnapshot::assert_unchanged(&dvd_snapshot);
    let info = harness.enqueue(discover_one(&dvd_output), Operation::Info, None);
    let info = harness.wait_for(&info.id, &[JobState::Completed], FAKE_TIMEOUT);
    assert_eq!(info.chd_info.unwrap().media_kind, MediaKind::Dvd);
}

#[test]
fn cancellation_and_sidecar_failures_remove_only_owned_temporary_outputs() {
    let workspace = tempdir().unwrap();
    let fixtures = generate_fixtures(workspace.path());
    let harness = Harness::fake();

    let slow_path = workspace.path().join("slow source.iso");
    fs::copy(
        &source_with_format(&fixtures, SourceFormat::Iso).primary_file,
        &slow_path,
    )
    .unwrap();
    let slow = discover_one(&slow_path);
    let snapshot = SourceSnapshot::capture(&slow);
    let destination = workspace.path().join("cancelled.chd");
    let job = harness.enqueue(slow, Operation::CreateDvd, Some(destination.clone()));
    harness.wait_for(&job.id, &[JobState::Running], FAKE_TIMEOUT);
    harness.engine.cancel(&job.id).unwrap();
    harness.wait_for(&job.id, &[JobState::Cancelled], FAKE_TIMEOUT);
    assert!(!destination.exists());
    SourceSnapshot::assert_unchanged(&snapshot);
    assert_no_hunk_temporaries(workspace.path());

    let corrupt_path = workspace.path().join("corrupt source.iso");
    fs::write(&corrupt_path, deterministic_bytes(8 * 2_048)).unwrap();
    let corrupt = discover_one(&corrupt_path);
    let snapshot = SourceSnapshot::capture(&corrupt);
    let destination = workspace.path().join("corrupt output.chd");
    let job = harness.enqueue(corrupt, Operation::CreateDvd, Some(destination.clone()));
    let failed = harness.wait_for(&job.id, &[JobState::Failed], FAKE_TIMEOUT);
    assert!(
        failed
            .error
            .unwrap()
            .to_ascii_lowercase()
            .contains("corrupt")
    );
    assert!(!destination.exists());
    SourceSnapshot::assert_unchanged(&snapshot);
    assert_no_hunk_temporaries(workspace.path());

    let valid = discover_one(&workspace.path().join("generated.iso"));
    let snapshot = SourceSnapshot::capture(&valid);
    let destination = workspace.path().join("bad verification.chd");
    let job = harness.enqueue(valid, Operation::CreateDvd, Some(destination.clone()));
    harness.wait_for(&job.id, &[JobState::Failed], FAKE_TIMEOUT);
    assert!(!destination.exists());
    SourceSnapshot::assert_unchanged(&snapshot);
    assert_no_hunk_temporaries(workspace.path());
}

#[test]
fn preflight_blocks_low_space_permissions_and_collisions_without_starting_conversion() {
    let workspace = tempdir().unwrap();
    let fixtures = generate_fixtures(workspace.path());
    let harness = Harness::fake();

    let mut too_large = source_with_format(&fixtures, SourceFormat::Iso);
    too_large.total_size = u64::MAX;
    let snapshot = SourceSnapshot::capture(&too_large);
    let low_space_output = workspace.path().join("too-large.chd");
    let job = harness.enqueue(
        too_large,
        Operation::CreateDvd,
        Some(low_space_output.clone()),
    );
    let blocked = harness.wait_for(&job.id, &[JobState::Blocked], FAKE_TIMEOUT);
    assert!(blocked.error.unwrap().contains("Not enough free space"));
    assert!(!low_space_output.exists());
    SourceSnapshot::assert_unchanged(&snapshot);

    let source = source_with_format(&fixtures, SourceFormat::Cue);
    let snapshot = SourceSnapshot::capture(&source);
    let collision = workspace.path().join("existing.chd");
    fs::write(&collision, b"pre-existing output").unwrap();
    let job = harness.enqueue(source, Operation::CreateCd, Some(collision.clone()));
    let blocked = harness.wait_for(&job.id, &[JobState::Blocked], FAKE_TIMEOUT);
    assert!(blocked.error.unwrap().contains("Output already exists"));
    assert_eq!(fs::read(&collision).unwrap(), b"pre-existing output");
    SourceSnapshot::assert_unchanged(&snapshot);

    #[cfg(target_os = "linux")]
    {
        let source = discover_one(&workspace.path().join("generated.iso"));
        let snapshot = SourceSnapshot::capture(&source);
        let job = harness.enqueue(
            source,
            Operation::CreateDvd,
            Some(PathBuf::from("/proc/hunk-permission-test.chd")),
        );
        let blocked = harness.wait_for(&job.id, &[JobState::Blocked], FAKE_TIMEOUT);
        assert!(
            blocked
                .error
                .unwrap()
                .contains("Destination folder is not writable")
        );
        SourceSnapshot::assert_unchanged(&snapshot);
    }

    assert_no_hunk_temporaries(workspace.path());
}

#[test]
fn malformed_sidecar_information_fails_without_modifying_input() {
    let workspace = tempdir().unwrap();
    let input = workspace.path().join("malformed info.chd");
    fs::write(&input, b"FAKE_CD\n").unwrap();
    let source = discover_one(&input);
    let snapshot = SourceSnapshot::capture(&source);
    let harness = Harness::fake();

    let job = harness.enqueue(source, Operation::Info, None);
    let failed = harness.wait_for(&job.id, &[JobState::Failed], FAKE_TIMEOUT);

    assert!(failed.error.unwrap().contains("incomplete information"));
    SourceSnapshot::assert_unchanged(&snapshot);
}

#[test]
#[ignore = "requires HUNK_CHDMAN pointing to the approved real sidecar"]
fn generated_fixture_round_trip_with_real_chdman() {
    let program = required_chdman();
    let workspace = tempdir().unwrap();
    let fixtures = generate_fixtures(workspace.path());
    let harness = Harness::with_program(program);

    for (format, operation, name) in [
        (SourceFormat::Cue, Operation::CreateCd, "fixture-cd.chd"),
        (SourceFormat::Iso, Operation::CreateDvd, "fixture-dvd.chd"),
    ] {
        let source = source_with_format(&fixtures, format);
        let snapshot = SourceSnapshot::capture(&source);
        let output = workspace.path().join(name);
        let create = harness.enqueue(source, operation, Some(output.clone()));
        harness.wait_for(&create.id, &[JobState::Completed], REAL_TIMEOUT);
        SourceSnapshot::assert_unchanged(&snapshot);

        let chd = discover_one(&output);
        let verify = harness.enqueue(chd.clone(), Operation::Verify, None);
        harness.wait_for(&verify.id, &[JobState::Completed], REAL_TIMEOUT);
        let info = harness.enqueue(chd.clone(), Operation::Info, None);
        harness.wait_for(&info.id, &[JobState::Completed], REAL_TIMEOUT);

        let (extract_operation, extract_path, recreate_operation, recreate_path) = match format {
            SourceFormat::Cue => (
                Operation::ExtractCd,
                workspace.path().join("real-extracted.cue"),
                Operation::CreateCd,
                workspace.path().join("real-recreated-cd.chd"),
            ),
            SourceFormat::Iso => (
                Operation::ExtractDvd,
                workspace.path().join("real-extracted.iso"),
                Operation::CreateDvd,
                workspace.path().join("real-recreated-dvd.chd"),
            ),
            _ => unreachable!(),
        };
        let extract = harness.enqueue(chd, extract_operation, Some(extract_path.clone()));
        harness.wait_for(&extract.id, &[JobState::Completed], REAL_TIMEOUT);
        let extracted = discover_one(&extract_path);
        let recreate = harness.enqueue(extracted, recreate_operation, Some(recreate_path));
        harness.wait_for(&recreate.id, &[JobState::Completed], REAL_TIMEOUT);
    }
}

#[test]
#[ignore = "uses ignored local Test/ media; invoke through scripts/test-local-media.sh"]
fn local_representative_media() {
    let program = required_chdman();
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let test_directory = repository.join("Test");
    assert!(test_directory.is_dir(), "local Test/ directory is missing");
    let report = discover_sources(&[test_directory]);
    assert!(
        report.issues.is_empty(),
        "discovery issues: {:?}",
        report.issues
    );
    assert_eq!(
        report.source_sets.len(),
        3,
        "expected exactly three source sets"
    );
    assert!(report.source_sets.iter().all(SourceSet::is_valid));

    let snapshots = report
        .source_sets
        .iter()
        .map(SourceSnapshot::capture)
        .collect::<Vec<_>>();
    let output = tempdir().unwrap();
    let harness = Harness::with_program(program);
    let mut created = Vec::new();
    for (index, source) in report.source_sets.iter().cloned().enumerate() {
        let destination = output.path().join(format!("disc-{}.chd", index + 1));
        let job = harness.enqueue(source, Operation::CreateCd, Some(destination.clone()));
        harness.wait_for(&job.id, &[JobState::Completed], LOCAL_TIMEOUT);
        created.push(destination);
    }

    let mixed_index = report
        .source_sets
        .iter()
        .position(|source| source.tracks.len() > 1)
        .expect("expected one mixed-mode source");
    let extracted_cue = output.path().join("mixed-mode-extracted.cue");
    let extract = harness.enqueue(
        discover_one(&created[mixed_index]),
        Operation::ExtractCd,
        Some(extracted_cue.clone()),
    );
    harness.wait_for(&extract.id, &[JobState::Completed], LOCAL_TIMEOUT);
    let recreate = harness.enqueue(
        discover_one(&extracted_cue),
        Operation::CreateCd,
        Some(output.path().join("mixed-mode-recreated.chd")),
    );
    harness.wait_for(&recreate.id, &[JobState::Completed], LOCAL_TIMEOUT);

    for snapshot in &snapshots {
        SourceSnapshot::assert_unchanged(snapshot);
    }
}

fn generate_fixtures(directory: &Path) -> Vec<SourceSet> {
    let bin = directory.join("generated track.bin");
    fs::write(&bin, deterministic_bytes(16 * 2_352)).unwrap();
    fs::write(
        directory.join("generated.cue"),
        "FILE \"generated track.bin\" BINARY\n  TRACK 01 MODE1/2352\n    INDEX 01 00:00:00\n",
    )
    .unwrap();
    fs::write(
        directory.join("generated.iso"),
        deterministic_bytes(32 * 2_048),
    )
    .unwrap();

    let report = discover_sources(&[directory.to_path_buf()]);
    assert!(report.issues.is_empty());
    assert_eq!(report.source_sets.len(), 2);
    assert!(report.source_sets.iter().all(SourceSet::is_valid));
    report.source_sets
}

fn deterministic_bytes(length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| ((index.wrapping_mul(31).wrapping_add(17)) % 251) as u8)
        .collect()
}

fn discover_one(path: &Path) -> SourceSet {
    let report = discover_sources(&[path.to_path_buf()]);
    assert!(
        report.issues.is_empty(),
        "discovery issues: {:?}",
        report.issues
    );
    assert_eq!(report.source_sets.len(), 1);
    assert!(report.source_sets[0].is_valid());
    report.source_sets.into_iter().next().unwrap()
}

fn source_with_format(sources: &[SourceSet], format: SourceFormat) -> SourceSet {
    sources
        .iter()
        .find(|source| source.format == format)
        .unwrap()
        .clone()
}

fn wait_for(engine: &JobEngine, id: &str, states: &[JobState], timeout: Duration) -> JobRecord {
    let started = Instant::now();
    loop {
        let record = engine
            .snapshot()
            .jobs
            .into_iter()
            .chain(engine.history())
            .find(|record| record.id == id);
        if let Some(record) = record {
            if states.contains(&record.state) {
                return record;
            }
            assert!(
                !matches!(
                    record.state,
                    JobState::Failed | JobState::Cancelled | JobState::Interrupted
                ),
                "job reached unexpected state {:?}: {:?}",
                record.state,
                record.error
            );
        }
        assert!(started.elapsed() < timeout, "job {id} timed out");
        thread::sleep(Duration::from_millis(20));
    }
}

fn assert_no_hunk_temporaries(directory: &Path) {
    assert!(
        fs::read_dir(directory)
            .unwrap()
            .filter_map(Result::ok)
            .all(|entry| !entry.file_name().to_string_lossy().starts_with(".hunk-")),
        "Hunk-owned temporary output was not cleaned"
    );
}

fn file_digest(path: &Path) -> u64 {
    let mut file = File::open(path).unwrap();
    let mut digest = 0xcbf29ce484222325_u64;
    let mut buffer = [0_u8; 64 * 1_024];
    loop {
        let count = file.read(&mut buffer).unwrap();
        if count == 0 {
            return digest;
        }
        for byte in &buffer[..count] {
            digest ^= u64::from(*byte);
            digest = digest.wrapping_mul(0x100000001b3);
        }
    }
}

fn required_chdman() -> PathBuf {
    let path = std::env::var_os("HUNK_CHDMAN")
        .map(PathBuf::from)
        .expect("set HUNK_CHDMAN to the approved chdman executable");
    assert!(path.is_file(), "HUNK_CHDMAN does not point to a file");
    path
}

fn fake_chdman(directory: &Path) -> PathBuf {
    let path = directory.join("fake chdman");
    let mut file = File::create(&path).unwrap();
    file.write_all(
        br#"#!/bin/sh
if [ "$#" -eq 0 ]; then
  echo "chdman - MAME Compressed Hunks of Data (CHD) manager 0.289"
  echo "info: displays information"
  echo "verify: verifies integrity"
  echo "createcd: creates"
  echo "createdvd: creates"
  echo "extractcd: extracts"
  echo "extractdvd: extracts"
  exit 0
fi

command="$1"
shift
while [ "$#" -gt 0 ]; do
  case "$1" in
    -i) shift; input="$1" ;;
    -o) shift; output="$1" ;;
    -ob) shift; output_bin="$1" ;;
  esac
  shift
done

case "$command" in
  createcd|createdvd)
    case "$input" in
      *corrupt*) echo "Error: corrupt input data" >&2; exit 1 ;;
      *slow*) printf 'partial output' > "$output"; echo "Compressing, 1.0% complete"; exec sleep 30 ;;
    esac
    case "$command" in createcd) printf 'FAKE_CD\n' > "$output" ;; *) printf 'FAKE_DVD\n' > "$output" ;; esac
    echo "Compressing, 100.0% complete"
    echo "Compression complete"
    ;;
  verify)
    echo "Raw SHA1 verification successful!"
    case "$input" in *bad\ verification*) exit 0 ;; esac
    echo "Overall SHA1 verification successful!"
    ;;
  info)
    case "$input" in *malformed\ info*) echo "File Version: not-a-number"; exit 0 ;; esac
    echo "File Version: 5"
    echo "Logical size: 65,536 bytes"
    echo "CHD size: 8 bytes"
    echo "Compression: zlib"
    echo "Hunk Size: 2,048 bytes"
    echo "Total Hunks: 32"
    echo "Unit Size: 2,048 bytes"
    echo "Total Units: 32"
    if grep -q FAKE_CD "$input"; then
      echo "Metadata: Tag='CHT2' Index=0 Length=80 bytes"
      echo "          TRACK:1 TYPE:MODE1 SUBTYPE:NONE FRAMES:16 PREGAP:0 PGTYPE:MODE1 PGSUB:NONE POSTGAP:0"
    else
      echo "Metadata: Tag='DVD ' Index=0 Length=0 bytes"
    fi
    ;;
  extractcd)
    printf 'track bytes' > "$output_bin"
    printf 'FILE "%s" BINARY\n  TRACK 01 MODE1/2352\n' "$output_bin" > "$output"
    echo "Extraction complete"
    ;;
  extractdvd)
    printf 'dvd bytes' > "$output"
    echo "Extraction complete"
    ;;
esac
"#,
    )
    .unwrap();
    drop(file);
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).unwrap();
    path
}
