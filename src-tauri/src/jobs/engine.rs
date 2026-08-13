use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, ExitStatus, Stdio};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use crate::chdman::{
    ChdmanRequest, CreateOptions, build_command, check_capabilities, classify_error, parse_info,
    parse_progress, parse_verification,
};
use crate::domain::{JobProgress, Operation, SourceFormat};

use super::model::{JobRecord, JobSpec, JobState, QueueSnapshot, Settings, now_millis};
use super::store::JobStore;

const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(50);
const MAX_CAPTURE_BYTES: usize = 2 * 1024 * 1024;

pub trait EventSink: Send + Sync + 'static {
    fn job_changed(&self, record: &JobRecord);
    fn progress_changed(&self, id: &str, progress: &JobProgress);
    fn queue_changed(&self, snapshot: &QueueSnapshot);
}

pub struct NoopEventSink;

impl EventSink for NoopEventSink {
    fn job_changed(&self, _record: &JobRecord) {}
    fn progress_changed(&self, _id: &str, _progress: &JobProgress) {}
    fn queue_changed(&self, _snapshot: &QueueSnapshot) {}
}

pub struct JobEngine {
    shared: Arc<Shared>,
}

struct Shared {
    state: Mutex<EngineState>,
    wake: Condvar,
    store: Mutex<JobStore>,
    program: PathBuf,
    events: Arc<dyn EventSink>,
}

struct EngineState {
    jobs: Vec<JobRecord>,
    paused: bool,
    active_job_id: Option<String>,
    active_child: Option<Arc<Mutex<Child>>>,
    cancel_requested: HashSet<String>,
    capabilities_checked: bool,
    shutting_down: bool,
}

struct PreparedJob {
    request: ChdmanRequest,
    temporary_paths: Vec<PathBuf>,
    publications: Vec<(PathBuf, PathBuf)>,
    split_bin: Option<SplitBinPublication>,
}

struct SplitBinPublication {
    directory: PathBuf,
    temporary_prefix: String,
    final_stem: String,
}

struct ProcessOutput {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

#[derive(Debug)]
enum ExecutionError {
    Blocked(String),
    Failed(String),
    Cancelled,
}

impl JobEngine {
    pub fn new(
        store: JobStore,
        program: PathBuf,
        events: Arc<dyn EventSink>,
    ) -> Result<Arc<Self>, String> {
        let mut jobs = store.load_all()?;
        for record in &mut jobs {
            if record.state.is_active() {
                cleanup_owned_temporary_paths(record);
                record.state = JobState::Interrupted;
                record.message = "Interrupted when Hunk last stopped".to_owned();
                record.error = Some("The previous Hunk session ended during this job.".to_owned());
                record.finished_at = Some(now_millis());
                record.temporary_paths.clear();
                store.save_job(record)?;
            }
        }

        let engine = Arc::new(Self {
            shared: Arc::new(Shared {
                state: Mutex::new(EngineState {
                    jobs,
                    paused: false,
                    active_job_id: None,
                    active_child: None,
                    cancel_requested: HashSet::new(),
                    capabilities_checked: false,
                    shutting_down: false,
                }),
                wake: Condvar::new(),
                store: Mutex::new(store),
                program,
                events,
            }),
        });
        let worker_engine = Arc::clone(&engine);
        thread::Builder::new()
            .name("hunk-serial-job-worker".to_owned())
            .spawn(move || worker_engine.worker_loop())
            .map_err(|error| error.to_string())?;
        Ok(engine)
    }

    pub fn enqueue(&self, spec: JobSpec) -> Result<JobRecord, String> {
        validate_spec_shape(&spec)?;
        let record = JobRecord::queued(spec);
        self.shared.store.lock().unwrap().save_job(&record)?;
        {
            let mut state = self.shared.state.lock().unwrap();
            state.jobs.push(record.clone());
        }
        self.emit_job_and_queue(&record);
        self.shared.wake.notify_one();
        Ok(record)
    }

    pub fn snapshot(&self) -> QueueSnapshot {
        snapshot_from_state(&self.shared.state.lock().unwrap())
    }

    pub fn history(&self) -> Vec<JobRecord> {
        let state = self.shared.state.lock().unwrap();
        let mut records = state
            .jobs
            .iter()
            .filter(|record| record.state.is_terminal())
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by_key(|record| std::cmp::Reverse(record.finished_at.unwrap_or(0)));
        records.truncate(100);
        records
    }

    pub fn set_paused(&self, paused: bool) -> QueueSnapshot {
        let snapshot = {
            let mut state = self.shared.state.lock().unwrap();
            state.paused = paused;
            snapshot_from_state(&state)
        };
        self.shared.events.queue_changed(&snapshot);
        if !paused {
            self.shared.wake.notify_one();
        }
        snapshot
    }

    pub fn cancel(&self, id: &str) -> Result<JobRecord, String> {
        let (record, child) = {
            let mut state = self.shared.state.lock().unwrap();
            let position = state
                .jobs
                .iter()
                .position(|record| record.id == id)
                .ok_or_else(|| "Job not found".to_owned())?;
            match state.jobs[position].state {
                JobState::Queued | JobState::Blocked => {
                    let record = &mut state.jobs[position];
                    record.state = JobState::Cancelled;
                    record.message = "Cancelled before processing".to_owned();
                    record.finished_at = Some(now_millis());
                    (record.clone(), None)
                }
                current if current.is_active() => {
                    state.cancel_requested.insert(id.to_owned());
                    (
                        state.jobs[position].clone(),
                        state.active_child.as_ref().map(Arc::clone),
                    )
                }
                _ => return Err("Only queued, blocked, or active jobs can be cancelled".to_owned()),
            }
        };
        if let Some(child) = child {
            let _ = child.lock().unwrap().kill();
        }
        if record.state == JobState::Cancelled {
            self.shared.store.lock().unwrap().save_job(&record)?;
            self.emit_job_and_queue(&record);
        }
        self.shared.wake.notify_one();
        Ok(record)
    }

    pub fn retry(&self, id: &str) -> Result<JobRecord, String> {
        let spec = {
            let state = self.shared.state.lock().unwrap();
            let record = state
                .jobs
                .iter()
                .find(|record| record.id == id)
                .ok_or_else(|| "Job not found".to_owned())?;
            if !record.state.is_terminal() && record.state != JobState::Blocked {
                return Err("Only finished, interrupted, or blocked jobs can be retried".to_owned());
            }
            record.spec.clone()
        };
        self.enqueue(spec)
    }

    pub fn remove(&self, id: &str) -> Result<(), String> {
        {
            let mut state = self.shared.state.lock().unwrap();
            let record = state
                .jobs
                .iter()
                .find(|record| record.id == id)
                .ok_or_else(|| "Job not found".to_owned())?;
            if record.state.is_active() {
                return Err("An active job must be cancelled before it can be removed".to_owned());
            }
            state.jobs.retain(|record| record.id != id);
        }
        self.shared.store.lock().unwrap().delete_job(id)?;
        self.shared.events.queue_changed(&self.snapshot());
        Ok(())
    }

    pub fn settings(&self) -> Result<Settings, String> {
        self.shared.store.lock().unwrap().load_settings()
    }

    pub fn update_settings(&self, settings: Settings) -> Result<Settings, String> {
        self.shared.store.lock().unwrap().save_settings(&settings)?;
        Ok(settings)
    }

    pub fn has_active_job(&self) -> bool {
        let state = self.shared.state.lock().unwrap();
        state.active_job_id.is_some() && !state.shutting_down
    }

    pub fn shutdown(&self) {
        let child = {
            let mut state = self.shared.state.lock().unwrap();
            state.shutting_down = true;
            if let Some(id) = state.active_job_id.clone() {
                state.cancel_requested.insert(id);
            }
            state.active_child.as_ref().map(Arc::clone)
        };
        if let Some(child) = child {
            let _ = child.lock().unwrap().kill();
        }
        self.shared.wake.notify_all();
    }

    fn worker_loop(&self) {
        loop {
            let id = {
                let mut state = self.shared.state.lock().unwrap();
                while !state.shutting_down
                    && (state.paused
                        || state.active_job_id.is_some()
                        || !state
                            .jobs
                            .iter()
                            .any(|record| record.state == JobState::Queued))
                {
                    state = self.shared.wake.wait(state).unwrap();
                }
                if state.shutting_down {
                    return;
                }
                let Some(position) = state
                    .jobs
                    .iter()
                    .position(|record| record.state == JobState::Queued)
                else {
                    continue;
                };
                let id = state.jobs[position].id.clone();
                let record = &mut state.jobs[position];
                record.state = JobState::Preflight;
                record.started_at = Some(now_millis());
                record.message = "Checking source and destination".to_owned();
                record.error = None;
                state.active_job_id = Some(id.clone());
                id
            };
            self.persist_and_emit(&id);
            let result = self.execute(&id);
            self.finish(&id, result);
        }
    }

    fn execute(
        &self,
        id: &str,
    ) -> Result<(Option<u64>, Option<crate::domain::ChdInfo>), ExecutionError> {
        let spec = self
            .record(id)
            .ok_or_else(|| ExecutionError::Failed("Job disappeared".to_owned()))?
            .spec;
        validate_source_set(&spec)?;
        self.ensure_capabilities()?;
        let inspected_info =
            if matches!(spec.operation, Operation::ExtractCd | Operation::ExtractDvd) {
                let output = self.run_request(
                    id,
                    &ChdmanRequest::Info {
                        input: spec.source.primary_file.clone(),
                    },
                )?;
                let info = parse_info(&output.stdout).map_err(|error| {
                    ExecutionError::Blocked(format!(
                        "Could not determine the extracted size from this CHD: {error}"
                    ))
                })?;
                let expected_media = if spec.operation == Operation::ExtractCd {
                    crate::domain::MediaKind::Cd
                } else {
                    crate::domain::MediaKind::Dvd
                };
                if info.media_kind != expected_media {
                    return Err(ExecutionError::Blocked(format!(
                        "This CHD is {:?}; the selected extraction expects {:?}",
                        info.media_kind, expected_media
                    )));
                }
                self.update_record(id, |record| record.chd_info = Some(info.clone()));
                Some(info)
            } else {
                None
            };
        let required_space = inspected_info
            .as_ref()
            .map(|info| info.logical_size)
            .unwrap_or(spec.source.total_size);
        let prepared = preflight(&spec, id, &self.shared.program, required_space)?;
        self.update_record(id, |record| {
            record.temporary_paths = prepared.temporary_paths.clone();
            record.state = JobState::Running;
            record.message = operation_running_message(record.spec.operation).to_owned();
        });
        self.persist_and_emit(id);

        let output = self.run_request(id, &prepared.request)?;
        let operation = spec.operation;
        let info = if operation == Operation::Info {
            Some(parse_info(&output.stdout).map_err(|error| {
                ExecutionError::Failed(format!("chdman returned incomplete information: {error}"))
            })?)
        } else {
            inspected_info
        };
        if operation == Operation::Verify {
            let verification =
                parse_verification(&output.stdout, &output.stderr, output.status.code());
            if !verification.passed {
                return Err(ExecutionError::Failed(
                    verification
                        .error
                        .map(|error| error.message)
                        .unwrap_or_else(|| "CHD verification did not pass".to_owned()),
                ));
            }
        }

        if matches!(operation, Operation::CreateCd | Operation::CreateDvd) {
            self.update_record(id, |record| {
                record.state = JobState::Verifying;
                record.message = "Fully verifying the temporary CHD".to_owned();
            });
            self.persist_and_emit(id);
            let verify_request = ChdmanRequest::Verify {
                input: prepared.temporary_paths[0].clone(),
            };
            let verify_output = self.run_request(id, &verify_request)?;
            let verification = parse_verification(
                &verify_output.stdout,
                &verify_output.stderr,
                verify_output.status.code(),
            );
            if !verification.passed {
                return Err(ExecutionError::Failed(
                    verification
                        .error
                        .map(|error| error.message)
                        .unwrap_or_else(|| {
                            "The newly created CHD failed full verification".to_owned()
                        }),
                ));
            }
        }

        let cue_to_rewrite = (operation == Operation::ExtractCd)
            .then(|| prepared.temporary_paths.first().cloned())
            .flatten();
        let publications = if let Some(split) = prepared.split_bin {
            let mut split_tracks = discover_split_bin_publications(&split)?;
            // Publish track dependencies first and the CUE descriptor last, so a visible
            // descriptor never points at an output that has not been published yet.
            split_tracks.extend(prepared.publications);
            split_tracks
        } else {
            prepared.publications
        };
        if let Some(cue_path) = cue_to_rewrite {
            rewrite_extracted_cue(&cue_path, &publications)?;
        }
        publish_without_overwrite(&publications)?;
        let output_size = publications
            .iter()
            .filter_map(|(_, final_path)| {
                fs::metadata(final_path).ok().map(|metadata| metadata.len())
            })
            .sum::<u64>();
        Ok(((!publications.is_empty()).then_some(output_size), info))
    }

    fn ensure_capabilities(&self) -> Result<(), ExecutionError> {
        if self.shared.state.lock().unwrap().capabilities_checked {
            return Ok(());
        }
        check_capabilities(&self.shared.program)
            .map_err(|error| ExecutionError::Blocked(error.to_string()))?;
        self.shared.state.lock().unwrap().capabilities_checked = true;
        Ok(())
    }

    fn run_request(
        &self,
        id: &str,
        request: &ChdmanRequest,
    ) -> Result<ProcessOutput, ExecutionError> {
        let command = build_command(&self.shared.program, request)
            .map_err(|error| ExecutionError::Blocked(error.to_string()))?;
        let mut process = command.process();
        process.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = process
            .spawn()
            .map_err(|error| ExecutionError::Failed(format!("Could not start chdman: {error}")))?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let child = Arc::new(Mutex::new(child));
        self.shared.state.lock().unwrap().active_child = Some(Arc::clone(&child));

        let (sender, receiver) = mpsc::channel();
        if let Some(stdout) = stdout {
            spawn_reader(stdout, false, sender.clone());
        }
        if let Some(stderr) = stderr {
            spawn_reader(stderr, true, sender.clone());
        }
        drop(sender);
        let started = Instant::now();
        let mut stdout_capture = String::new();
        let mut stderr_capture = String::new();

        let status = loop {
            while let Ok((is_stderr, line)) = receiver.try_recv() {
                self.capture_line(
                    id,
                    is_stderr,
                    &line,
                    started,
                    &mut stdout_capture,
                    &mut stderr_capture,
                );
            }
            if self.is_cancel_requested(id) {
                let _ = child.lock().unwrap().kill();
            }
            if let Some(status) = child.lock().unwrap().try_wait().map_err(|error| {
                ExecutionError::Failed(format!("Could not wait for chdman: {error}"))
            })? {
                break status;
            }
            thread::sleep(PROCESS_POLL_INTERVAL);
        };
        for (is_stderr, line) in receiver {
            self.capture_line(
                id,
                is_stderr,
                &line,
                started,
                &mut stdout_capture,
                &mut stderr_capture,
            );
        }
        self.shared.state.lock().unwrap().active_child = None;

        if self.is_cancel_requested(id) {
            return Err(ExecutionError::Cancelled);
        }
        if !status.success() {
            let error = classify_error(&stderr_capture, status.code());
            return Err(ExecutionError::Failed(error.message));
        }
        Ok(ProcessOutput {
            status,
            stdout: stdout_capture,
            stderr: stderr_capture,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn capture_line(
        &self,
        id: &str,
        is_stderr: bool,
        line: &str,
        started: Instant,
        stdout: &mut String,
        stderr: &mut String,
    ) {
        append_capture(if is_stderr { stderr } else { stdout }, line);
        self.update_record(id, |record| {
            record.append_log(line);
            if let Some(mut progress) = parse_progress(line) {
                progress.elapsed_millis =
                    Some(started.elapsed().as_millis().try_into().unwrap_or(u64::MAX));
                record.message = progress.message.clone();
                record.progress = Some(progress);
            }
        });
        if let Some(record) = self.record(id) {
            if let Some(progress) = &record.progress {
                self.shared.events.progress_changed(id, progress);
            }
            let _ = self.shared.store.lock().unwrap().save_job(&record);
        }
    }

    fn finish(
        &self,
        id: &str,
        result: Result<(Option<u64>, Option<crate::domain::ChdInfo>), ExecutionError>,
    ) {
        let record = {
            let mut state = self.shared.state.lock().unwrap();
            let cancel_requested = state.cancel_requested.remove(id);
            state.active_job_id = None;
            state.active_child = None;
            let Some(record) = state.jobs.iter_mut().find(|record| record.id == id) else {
                self.shared.wake.notify_one();
                return;
            };
            if cancel_requested || matches!(result, Err(ExecutionError::Cancelled)) {
                record.state = JobState::Cancelled;
                record.message = "Cancelled".to_owned();
                record.error = None;
            } else {
                match result {
                    Ok((output_size, info)) => {
                        record.state = JobState::Completed;
                        record.message = "Completed".to_owned();
                        record.output_size = output_size;
                        record.chd_info = info;
                        record.progress = record.progress.take().map(|mut progress| {
                            progress.percentage = Some(100.0);
                            progress
                        });
                    }
                    Err(ExecutionError::Blocked(message)) => {
                        record.state = JobState::Blocked;
                        record.message = "Preflight blocked this job".to_owned();
                        record.error = Some(message);
                    }
                    Err(ExecutionError::Failed(message)) => {
                        record.state = JobState::Failed;
                        record.message = "chdman could not complete the job".to_owned();
                        record.error = Some(message);
                    }
                    Err(ExecutionError::Cancelled) => unreachable!(),
                }
            }
            cleanup_owned_temporary_paths(record);
            record.temporary_paths.clear();
            record.finished_at = Some(now_millis());
            record.clone()
        };
        let _ = self.shared.store.lock().unwrap().save_job(&record);
        self.emit_job_and_queue(&record);
        self.shared.wake.notify_one();
    }

    fn record(&self, id: &str) -> Option<JobRecord> {
        self.shared
            .state
            .lock()
            .unwrap()
            .jobs
            .iter()
            .find(|record| record.id == id)
            .cloned()
    }

    fn update_record(&self, id: &str, update: impl FnOnce(&mut JobRecord)) {
        if let Some(record) = self
            .shared
            .state
            .lock()
            .unwrap()
            .jobs
            .iter_mut()
            .find(|record| record.id == id)
        {
            update(record);
        }
    }

    fn persist_and_emit(&self, id: &str) {
        if let Some(record) = self.record(id) {
            let _ = self.shared.store.lock().unwrap().save_job(&record);
            self.emit_job_and_queue(&record);
        }
    }

    fn emit_job_and_queue(&self, record: &JobRecord) {
        self.shared.events.job_changed(record);
        self.shared.events.queue_changed(&self.snapshot());
    }

    fn is_cancel_requested(&self, id: &str) -> bool {
        self.shared
            .state
            .lock()
            .unwrap()
            .cancel_requested
            .contains(id)
    }
}

fn snapshot_from_state(state: &EngineState) -> QueueSnapshot {
    QueueSnapshot {
        paused: state.paused,
        active_job_id: state.active_job_id.clone(),
        jobs: state
            .jobs
            .iter()
            .filter(|record| !record.state.is_terminal())
            .cloned()
            .collect(),
    }
}

fn validate_spec_shape(spec: &JobSpec) -> Result<(), String> {
    let mutating = matches!(
        spec.operation,
        Operation::CreateCd | Operation::CreateDvd | Operation::ExtractCd | Operation::ExtractDvd
    );
    if mutating && spec.destination.is_none() {
        return Err("This operation requires a destination".to_owned());
    }
    if !mutating && spec.destination.is_some() {
        return Err("Read-only operations do not accept a destination".to_owned());
    }
    Ok(())
}

fn preflight(
    spec: &JobSpec,
    id: &str,
    program: &Path,
    required_space: u64,
) -> Result<PreparedJob, ExecutionError> {
    validate_source_set(spec)?;

    if matches!(spec.operation, Operation::Verify | Operation::Info) {
        let request = if spec.operation == Operation::Verify {
            ChdmanRequest::Verify {
                input: spec.source.primary_file.clone(),
            }
        } else {
            ChdmanRequest::Info {
                input: spec.source.primary_file.clone(),
            }
        };
        return Ok(PreparedJob {
            request,
            temporary_paths: vec![],
            publications: vec![],
            split_bin: None,
        });
    }

    let destination = spec.destination.as_ref().ok_or_else(|| {
        ExecutionError::Blocked("A destination is required for this operation".to_owned())
    })?;
    validate_final_path(destination, spec)?;
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(ExecutionError::Blocked(format!(
            "Destination folder does not exist: {}",
            parent.display()
        )));
    }
    ensure_destination_writable(parent, id)?;
    let available = fs2::available_space(parent).map_err(|error| {
        ExecutionError::Blocked(format!(
            "Could not check available destination space: {error}"
        ))
    })?;
    if available < required_space {
        return Err(ExecutionError::Blocked(format!(
            "Not enough free space: {} bytes available, at least {} bytes required",
            available, required_space
        )));
    }

    let token = id.replace('-', "");
    match spec.operation {
        Operation::CreateCd | Operation::CreateDvd => {
            let temporary = temporary_path(destination, &token, "chd");
            ensure_temporary_available(&temporary, spec)?;
            let options = CreateOptions {
                hunk_size: spec.options.hunk_size,
                compression: None,
                processors: spec.options.processors,
            };
            let request = if spec.operation == Operation::CreateCd {
                ChdmanRequest::CreateCd {
                    input: spec.source.primary_file.clone(),
                    output: temporary.clone(),
                    options,
                }
            } else {
                ChdmanRequest::CreateDvd {
                    input: spec.source.primary_file.clone(),
                    output: temporary.clone(),
                    options,
                }
            };
            Ok(PreparedJob {
                request,
                temporary_paths: vec![temporary.clone()],
                publications: vec![(temporary, destination.clone())],
                split_bin: None,
            })
        }
        Operation::ExtractDvd => {
            let temporary = temporary_path(destination, &token, "iso");
            ensure_temporary_available(&temporary, spec)?;
            Ok(PreparedJob {
                request: ChdmanRequest::ExtractDvd {
                    input: spec.source.primary_file.clone(),
                    output: temporary.clone(),
                },
                temporary_paths: vec![temporary.clone()],
                publications: vec![(temporary, destination.clone())],
                split_bin: None,
            })
        }
        Operation::ExtractCd => prepare_extract_cd(spec, destination, &token),
        Operation::Verify | Operation::Info => unreachable!(),
    }
    .and_then(|prepared| {
        build_command(program, &prepared.request)
            .map_err(|error| ExecutionError::Blocked(error.to_string()))?;
        Ok(prepared)
    })
}

fn validate_source_set(spec: &JobSpec) -> Result<(), ExecutionError> {
    if !spec.source.validation_problems.is_empty() {
        return Err(ExecutionError::Blocked(
            "The source set contains validation problems".to_owned(),
        ));
    }
    ensure_readable(&spec.source.primary_file)?;
    for dependency in &spec.source.referenced_files {
        ensure_readable(dependency)?;
    }
    validate_operation_source(spec)
}

fn prepare_extract_cd(
    spec: &JobSpec,
    destination: &Path,
    token: &str,
) -> Result<PreparedJob, ExecutionError> {
    let temporary_cue = temporary_path(destination, token, "cue");
    ensure_temporary_available(&temporary_cue, spec)?;
    let final_stem = destination
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ExecutionError::Blocked("Destination CUE name is invalid".to_owned()))?
        .to_owned();
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    if spec.options.split_bin {
        let temporary_prefix = format!(".hunk-{token}-track-");
        let temporary_bin = parent.join(format!("{temporary_prefix}%t.bin"));
        ensure_no_split_collisions(parent, &final_stem)?;
        Ok(PreparedJob {
            request: ChdmanRequest::ExtractCd {
                input: spec.source.primary_file.clone(),
                output: temporary_cue.clone(),
                output_bin: temporary_bin.clone(),
                split_bin: true,
            },
            temporary_paths: vec![temporary_cue.clone(), temporary_bin],
            publications: vec![(temporary_cue, destination.to_owned())],
            split_bin: Some(SplitBinPublication {
                directory: parent.to_owned(),
                temporary_prefix,
                final_stem,
            }),
        })
    } else {
        let final_bin = destination.with_extension("bin");
        validate_final_path(&final_bin, spec)?;
        let temporary_bin = temporary_path(&final_bin, token, "bin");
        ensure_temporary_available(&temporary_bin, spec)?;
        Ok(PreparedJob {
            request: ChdmanRequest::ExtractCd {
                input: spec.source.primary_file.clone(),
                output: temporary_cue.clone(),
                output_bin: temporary_bin.clone(),
                split_bin: false,
            },
            temporary_paths: vec![temporary_cue.clone(), temporary_bin.clone()],
            publications: vec![
                (temporary_bin, final_bin),
                (temporary_cue, destination.to_owned()),
            ],
            split_bin: None,
        })
    }
}

fn validate_operation_source(spec: &JobSpec) -> Result<(), ExecutionError> {
    let valid = match spec.operation {
        Operation::CreateCd => matches!(
            spec.source.format,
            SourceFormat::Cue | SourceFormat::Gdi | SourceFormat::Iso
        ),
        Operation::CreateDvd => spec.source.format == SourceFormat::Iso,
        Operation::ExtractCd | Operation::ExtractDvd | Operation::Verify | Operation::Info => {
            spec.source.format == SourceFormat::Chd
        }
    };
    valid.then_some(()).ok_or_else(|| {
        ExecutionError::Blocked(format!(
            "Operation {:?} is not valid for source format {:?}",
            spec.operation, spec.source.format
        ))
    })
}

fn ensure_readable(path: &Path) -> Result<(), ExecutionError> {
    fs::File::open(path).map(|_| ()).map_err(|error| {
        ExecutionError::Blocked(format!(
            "Source cannot be read: {} ({error})",
            path.display()
        ))
    })
}

fn validate_final_path(path: &Path, spec: &JobSpec) -> Result<(), ExecutionError> {
    if path.exists() {
        return Err(ExecutionError::Blocked(format!(
            "Output already exists: {}",
            path.display()
        )));
    }
    if path == spec.source.primary_file
        || spec
            .source
            .referenced_files
            .iter()
            .any(|source| source == path)
    {
        return Err(ExecutionError::Blocked(
            "Output cannot replace a source file".to_owned(),
        ));
    }
    Ok(())
}

fn ensure_destination_writable(parent: &Path, id: &str) -> Result<(), ExecutionError> {
    let probe = parent.join(format!(".hunk-{id}.write-probe"));
    let result = OpenOptions::new().write(true).create_new(true).open(&probe);
    match result {
        Ok(_) => fs::remove_file(&probe).map_err(|error| {
            ExecutionError::Blocked(format!("Could not remove Hunk write probe: {error}"))
        }),
        Err(error) => Err(ExecutionError::Blocked(format!(
            "Destination folder is not writable: {} ({error})",
            parent.display()
        ))),
    }
}

fn temporary_path(destination: &Path, token: &str, extension: &str) -> PathBuf {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let name = destination
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("output");
    parent.join(format!(".hunk-{token}-{name}.tmp.{extension}"))
}

fn ensure_temporary_available(path: &Path, spec: &JobSpec) -> Result<(), ExecutionError> {
    if path.exists()
        || path == spec.source.primary_file
        || spec
            .source
            .referenced_files
            .iter()
            .any(|source| source == path)
    {
        return Err(ExecutionError::Blocked(format!(
            "Hunk temporary output is not available: {}",
            path.display()
        )));
    }
    Ok(())
}

fn ensure_no_split_collisions(parent: &Path, final_stem: &str) -> Result<(), ExecutionError> {
    let prefix = format!("{final_stem} - Track ");
    let entries =
        fs::read_dir(parent).map_err(|error| ExecutionError::Blocked(error.to_string()))?;
    if entries.filter_map(Result::ok).any(|entry| {
        entry.file_name().to_string_lossy().starts_with(&prefix)
            && entry
                .path()
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("bin"))
    }) {
        return Err(ExecutionError::Blocked(
            "One or more split-track BIN outputs already exist".to_owned(),
        ));
    }
    Ok(())
}

fn discover_split_bin_publications(
    split: &SplitBinPublication,
) -> Result<Vec<(PathBuf, PathBuf)>, ExecutionError> {
    let mut publications = Vec::new();
    for entry in fs::read_dir(&split.directory)
        .map_err(|error| ExecutionError::Failed(error.to_string()))?
        .filter_map(Result::ok)
    {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(&split.temporary_prefix) || !name.ends_with(".bin") {
            continue;
        }
        let track = &name[split.temporary_prefix.len()..name.len() - 4];
        let final_path = split
            .directory
            .join(format!("{} - Track {track}.bin", split.final_stem));
        if final_path.exists() {
            return Err(ExecutionError::Blocked(format!(
                "Output already exists: {}",
                final_path.display()
            )));
        }
        publications.push((entry.path(), final_path));
    }
    if publications.is_empty() {
        return Err(ExecutionError::Failed(
            "chdman did not create any split-track BIN files".to_owned(),
        ));
    }
    publications.sort_by(|left, right| left.1.cmp(&right.1));
    Ok(publications)
}

fn rewrite_extracted_cue(
    cue_path: &Path,
    publications: &[(PathBuf, PathBuf)],
) -> Result<(), ExecutionError> {
    let mut contents = fs::read_to_string(cue_path).map_err(|error| {
        ExecutionError::Failed(format!(
            "Could not read Hunk's temporary extracted CUE: {error}"
        ))
    })?;
    for (temporary, final_path) in publications.iter().filter(|(temporary, _)| {
        temporary
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("bin"))
    }) {
        let temporary_name = temporary
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| {
                ExecutionError::Failed("Temporary BIN name is not valid Unicode".to_owned())
            })?;
        let final_name = final_path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| {
                ExecutionError::Blocked("Final BIN name is not valid Unicode".to_owned())
            })?;
        let full_temporary = temporary.to_string_lossy();
        let replaced_full = contents.replace(full_temporary.as_ref(), final_name);
        let replaced_name = replaced_full.replace(temporary_name, final_name);
        if replaced_name == contents {
            return Err(ExecutionError::Failed(format!(
                "Extracted CUE does not reference the expected temporary BIN: {temporary_name}"
            )));
        }
        contents = replaced_name;
    }
    fs::write(cue_path, contents).map_err(|error| {
        ExecutionError::Failed(format!(
            "Could not finalize Hunk's temporary extracted CUE: {error}"
        ))
    })
}

fn publish_without_overwrite(publications: &[(PathBuf, PathBuf)]) -> Result<(), ExecutionError> {
    let mut published: Vec<(&Path, &Path)> = Vec::new();
    for (temporary, final_path) in publications {
        if let Err(error) = fs::hard_link(temporary, final_path) {
            for (published_temporary, published_final) in published.iter().rev() {
                if same_file::is_same_file(published_temporary, published_final).unwrap_or(false) {
                    let _ = fs::remove_file(published_final);
                }
            }
            return Err(ExecutionError::Blocked(format!(
                "Could not publish output without overwriting {}: {error}",
                final_path.display()
            )));
        }
        published.push((temporary, final_path));
    }
    for (temporary, final_path) in &published {
        if !same_file::is_same_file(temporary, final_path).unwrap_or(false) {
            return Err(ExecutionError::Failed(format!(
                "Published output changed before Hunk could finalize it: {}",
                final_path.display()
            )));
        }
    }
    for (temporary, _) in publications {
        fs::remove_file(temporary).map_err(|error| {
            ExecutionError::Failed(format!(
                "Output was published but its Hunk temporary link could not be removed: {error}"
            ))
        })?;
    }
    Ok(())
}

fn cleanup_owned_temporary_paths(record: &JobRecord) {
    let token = record.id.replace('-', "");
    for path in &record.temporary_paths {
        if path == &record.spec.source.primary_file
            || record
                .spec
                .source
                .referenced_files
                .iter()
                .any(|source| source == path)
            || record.spec.destination.as_ref() == Some(path)
        {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.starts_with(&format!(".hunk-{token}-")) {
            if name.contains("%t") {
                if let Some(parent) = path.parent() {
                    if let Ok(entries) = fs::read_dir(parent) {
                        let prefix = name.split("%t").next().unwrap_or(name);
                        for entry in entries.filter_map(Result::ok) {
                            if entry.file_name().to_string_lossy().starts_with(prefix) {
                                let _ = fs::remove_file(entry.path());
                            }
                        }
                    }
                }
            } else {
                let _ = fs::remove_file(path);
            }
        }
    }
}

fn operation_running_message(operation: Operation) -> &'static str {
    match operation {
        Operation::CreateCd | Operation::CreateDvd => "Creating temporary CHD",
        Operation::ExtractCd | Operation::ExtractDvd => "Extracting to temporary output",
        Operation::Verify => "Verifying CHD integrity",
        Operation::Info => "Reading CHD information",
    }
}

fn spawn_reader(
    reader: impl Read + Send + 'static,
    is_stderr: bool,
    sender: mpsc::Sender<(bool, String)>,
) {
    thread::spawn(move || {
        let mut buffer = Vec::new();
        for byte in BufReader::new(reader).bytes().map_while(Result::ok) {
            if matches!(byte, b'\r' | b'\n') {
                if !buffer.is_empty() {
                    let line = String::from_utf8_lossy(&buffer).into_owned();
                    let _ = sender.send((is_stderr, line));
                    buffer.clear();
                }
            } else {
                buffer.push(byte);
            }
        }
        if !buffer.is_empty() {
            let _ = sender.send((is_stderr, String::from_utf8_lossy(&buffer).into_owned()));
        }
    });
}

fn append_capture(capture: &mut String, line: &str) {
    if capture.len() >= MAX_CAPTURE_BYTES {
        return;
    }
    let remaining = MAX_CAPTURE_BYTES - capture.len();
    let bounded = if line.len() <= remaining {
        line
    } else {
        let mut boundary = remaining;
        while !line.is_char_boundary(boundary) {
            boundary -= 1;
        }
        &line[..boundary]
    };
    capture.push_str(bounded);
    capture.push('\n');
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use tempfile::tempdir;

    use crate::domain::{MediaKind, SourceSet};
    use crate::jobs::model::{JobOptions, JobSpec};
    use crate::jobs::store::JobStore;

    use super::*;

    fn source(primary: PathBuf, format: SourceFormat) -> SourceSet {
        SourceSet {
            primary_file: primary,
            referenced_files: vec![],
            format,
            media_kind: MediaKind::UnknownOptical,
            tracks: vec![],
            total_size: 1,
            validation_problems: vec![],
        }
    }

    fn wait_for(engine: &JobEngine, predicate: impl Fn(&JobRecord) -> bool) -> JobRecord {
        let started = Instant::now();
        loop {
            if let Some(record) = engine
                .shared
                .state
                .lock()
                .unwrap()
                .jobs
                .iter()
                .find(|record| predicate(record))
                .cloned()
            {
                return record;
            }
            assert!(started.elapsed() < Duration::from_secs(5), "job timed out");
            thread::sleep(Duration::from_millis(20));
        }
    }

    #[cfg(unix)]
    fn fake_chdman(directory: &Path) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let path = directory.join("fake chdman");
        fs::write(
            &path,
            r#"#!/bin/sh
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
case "$1" in
  createcd|createdvd)
    shift
    while [ "$#" -gt 0 ]; do
      if [ "$1" = "-o" ]; then shift; output="$1"; fi
      shift
    done
    printf 'verified chd' > "$output"
    echo "Compressing, 100.0% complete"
    echo "Compression complete"
    ;;
  verify)
    echo "Raw SHA1 verification successful!"
    echo "Overall SHA1 verification successful!"
    ;;
  extractcd)
    shift
    while [ "$#" -gt 0 ]; do
      if [ "$1" = "-o" ]; then shift; cue="$1"; fi
      if [ "$1" = "-ob" ]; then shift; bin="$1"; fi
      shift
    done
    printf 'track bytes' > "$bin"
    printf 'FILE "%s" BINARY\n  TRACK 01 MODE1/2352\n' "$bin" > "$cue"
    echo "Extraction complete"
    ;;
  info)
    case "$3" in *slow*) sleep 3 ;; esac
    echo "File Version: 5"
    echo "Logical size: 2,048 bytes"
    echo "CHD size: 1,024 bytes"
    echo "Compression: zlib"
    echo "Hunk Size: 2,048 bytes"
    echo "Total Hunks: 1"
    echo "Unit Size: 2,048 bytes"
    echo "Total Units: 1"
    echo "Metadata: Tag='CHT2' Index=0 Length=80 bytes"
    echo "          TRACK:1 TYPE:MODE1 SUBTYPE:NONE FRAMES:1 PREGAP:0 PGTYPE:MODE1 PGSUB:NONE POSTGAP:0"
    ;;
esac
"#,
        )
        .unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[test]
    fn preflight_never_accepts_a_source_as_output() {
        let directory = tempdir().unwrap();
        let input = directory.path().join("disc.iso");
        fs::write(&input, [1]).unwrap();
        let spec = JobSpec {
            source: source(input.clone(), SourceFormat::Iso),
            operation: Operation::CreateDvd,
            destination: Some(input),
            options: JobOptions::default(),
        };

        assert!(matches!(
            preflight(
                &spec,
                "00000000-0000-0000-0000-000000000000",
                Path::new("chdman"),
                spec.source.total_size,
            ),
            Err(ExecutionError::Blocked(_))
        ));
    }

    #[test]
    fn publication_refuses_to_overwrite_and_preserves_existing_file() {
        let directory = tempdir().unwrap();
        let temporary = directory.path().join(".hunk-temp");
        let output = directory.path().join("disc.chd");
        fs::write(&temporary, b"new").unwrap();
        fs::write(&output, b"existing").unwrap();

        assert!(publish_without_overwrite(&[(temporary.clone(), output.clone())]).is_err());
        assert_eq!(fs::read(output).unwrap(), b"existing");
        assert_eq!(fs::read(temporary).unwrap(), b"new");
    }

    #[test]
    fn multi_file_publication_rolls_back_only_links_it_created() {
        let directory = tempdir().unwrap();
        let temporary_bin = directory.path().join("temporary.bin");
        let temporary_cue = directory.path().join("temporary.cue");
        let final_bin = directory.path().join("disc.bin");
        let final_cue = directory.path().join("disc.cue");
        fs::write(&temporary_bin, b"bin").unwrap();
        fs::write(&temporary_cue, b"cue").unwrap();
        fs::write(&final_cue, b"existing cue").unwrap();

        assert!(
            publish_without_overwrite(&[
                (temporary_bin.clone(), final_bin.clone()),
                (temporary_cue.clone(), final_cue.clone()),
            ])
            .is_err()
        );
        assert!(!final_bin.exists());
        assert_eq!(fs::read(final_cue).unwrap(), b"existing cue");
        assert_eq!(fs::read(temporary_bin).unwrap(), b"bin");
        assert_eq!(fs::read(temporary_cue).unwrap(), b"cue");
    }

    #[test]
    fn extracted_cue_is_rewritten_to_published_bin_names() {
        let directory = tempdir().unwrap();
        let temporary_bin = directory.path().join(".hunk-token-disc.tmp.bin");
        let final_bin = directory.path().join("Disc [日本].bin");
        let cue = directory.path().join(".hunk-token-disc.tmp.cue");
        fs::write(
            &cue,
            format!(
                "FILE \"{}\" BINARY\n  TRACK 01 MODE1/2352\n",
                temporary_bin.display()
            ),
        )
        .unwrap();

        rewrite_extracted_cue(&cue, &[(temporary_bin, final_bin)]).unwrap();

        assert_eq!(
            fs::read_to_string(cue).unwrap(),
            "FILE \"Disc [日本].bin\" BINARY\n  TRACK 01 MODE1/2352\n"
        );
    }

    #[test]
    fn cleanup_only_removes_paths_owned_by_the_record_token() {
        let directory = tempdir().unwrap();
        let source_path = directory.path().join("disc.iso");
        let existing = directory.path().join("existing.chd");
        fs::write(&source_path, b"source").unwrap();
        fs::write(&existing, b"existing").unwrap();
        let mut record = JobRecord::queued(JobSpec {
            source: source(source_path.clone(), SourceFormat::Iso),
            operation: Operation::CreateDvd,
            destination: Some(existing.clone()),
            options: JobOptions::default(),
        });
        let owned = directory
            .path()
            .join(format!(".hunk-{}-disc.tmp.chd", record.id.replace('-', "")));
        fs::write(&owned, b"temporary").unwrap();
        record.temporary_paths = vec![owned.clone(), source_path.clone(), existing.clone()];

        cleanup_owned_temporary_paths(&record);

        assert!(!owned.exists());
        assert_eq!(fs::read(source_path).unwrap(), b"source");
        assert_eq!(fs::read(existing).unwrap(), b"existing");
    }

    #[cfg(unix)]
    #[test]
    fn pause_cancel_and_retry_keep_queue_state_consistent() {
        let directory = tempdir().unwrap();
        let input = directory.path().join("disc.chd");
        fs::write(&input, b"chd").unwrap();
        let engine = JobEngine::new(
            JobStore::in_memory().unwrap(),
            fake_chdman(directory.path()),
            Arc::new(NoopEventSink),
        )
        .unwrap();
        engine.set_paused(true);
        let queued = engine
            .enqueue(JobSpec {
                source: source(input, SourceFormat::Chd),
                operation: Operation::Info,
                destination: None,
                options: JobOptions::default(),
            })
            .unwrap();
        assert_eq!(engine.snapshot().jobs[0].state, JobState::Queued);

        engine.cancel(&queued.id).unwrap();
        assert_eq!(engine.history()[0].state, JobState::Cancelled);
        let retried = engine.retry(&queued.id).unwrap();
        assert_ne!(retried.id, queued.id);
        assert_eq!(engine.snapshot().jobs[0].state, JobState::Queued);

        engine.set_paused(false);
        let completed = wait_for(&engine, |record| {
            record.id == retried.id && record.state == JobState::Completed
        });
        assert!(completed.chd_info.is_some());
        engine.shutdown();
    }

    #[cfg(unix)]
    #[test]
    fn creation_verifies_then_publishes_without_touching_source() {
        let directory = tempdir().unwrap();
        let input = directory.path().join("source [日本].iso");
        let output = directory.path().join("output [日本].chd");
        fs::write(&input, b"source bytes").unwrap();
        let mut source = source(input.clone(), SourceFormat::Iso);
        source.total_size = 12;
        let engine = JobEngine::new(
            JobStore::in_memory().unwrap(),
            fake_chdman(directory.path()),
            Arc::new(NoopEventSink),
        )
        .unwrap();
        let queued = engine
            .enqueue(JobSpec {
                source,
                operation: Operation::CreateDvd,
                destination: Some(output.clone()),
                options: JobOptions::default(),
            })
            .unwrap();

        let completed = wait_for(&engine, |record| {
            record.id == queued.id && record.state == JobState::Completed
        });
        assert_eq!(completed.output_size, Some(12));
        assert_eq!(fs::read(&input).unwrap(), b"source bytes");
        assert_eq!(fs::read(&output).unwrap(), b"verified chd");
        assert!(
            fs::read_dir(directory.path())
                .unwrap()
                .filter_map(Result::ok)
                .all(|entry| !entry.file_name().to_string_lossy().starts_with(".hunk-"))
        );
        engine.shutdown();
    }

    #[cfg(unix)]
    #[test]
    fn active_process_can_be_cancelled() {
        let directory = tempdir().unwrap();
        let input = directory.path().join("slow disc.chd");
        fs::write(&input, b"chd").unwrap();
        let engine = JobEngine::new(
            JobStore::in_memory().unwrap(),
            fake_chdman(directory.path()),
            Arc::new(NoopEventSink),
        )
        .unwrap();
        let queued = engine
            .enqueue(JobSpec {
                source: source(input, SourceFormat::Chd),
                operation: Operation::Info,
                destination: None,
                options: JobOptions::default(),
            })
            .unwrap();
        wait_for(&engine, |record| {
            record.id == queued.id && record.state == JobState::Running
        });

        engine.cancel(&queued.id).unwrap();
        wait_for(&engine, |record| {
            record.id == queued.id && record.state == JobState::Cancelled
        });
        assert!(engine.snapshot().active_job_id.is_none());
        engine.shutdown();
    }

    #[cfg(unix)]
    #[test]
    fn cd_extraction_publishes_bin_before_a_cue_with_final_references() {
        let directory = tempdir().unwrap();
        let input = directory.path().join("source.chd");
        let output_cue = directory.path().join("Extracted Disc.cue");
        let output_bin = directory.path().join("Extracted Disc.bin");
        fs::write(&input, b"source chd").unwrap();
        let engine = JobEngine::new(
            JobStore::in_memory().unwrap(),
            fake_chdman(directory.path()),
            Arc::new(NoopEventSink),
        )
        .unwrap();
        let queued = engine
            .enqueue(JobSpec {
                source: source(input.clone(), SourceFormat::Chd),
                operation: Operation::ExtractCd,
                destination: Some(output_cue.clone()),
                options: JobOptions::default(),
            })
            .unwrap();

        wait_for(&engine, |record| {
            record.id == queued.id && record.state == JobState::Completed
        });
        assert_eq!(fs::read(&input).unwrap(), b"source chd");
        assert_eq!(fs::read(&output_bin).unwrap(), b"track bytes");
        assert_eq!(
            fs::read_to_string(&output_cue).unwrap(),
            "FILE \"Extracted Disc.bin\" BINARY\n  TRACK 01 MODE1/2352\n"
        );
        engine.shutdown();
    }

    #[test]
    fn active_records_recover_as_interrupted_and_owned_temporary_files_are_cleaned() {
        let directory = tempdir().unwrap();
        let input = directory.path().join("source.iso");
        let output = directory.path().join("output.chd");
        fs::write(&input, b"source").unwrap();
        let store = JobStore::in_memory().unwrap();
        let mut active = JobRecord::queued(JobSpec {
            source: source(input.clone(), SourceFormat::Iso),
            operation: Operation::CreateDvd,
            destination: Some(output),
            options: JobOptions::default(),
        });
        active.state = JobState::Running;
        let temporary = directory.path().join(format!(
            ".hunk-{}-output.tmp.chd",
            active.id.replace('-', "")
        ));
        fs::write(&temporary, b"partial").unwrap();
        active.temporary_paths.push(temporary.clone());
        store.save_job(&active).unwrap();

        let engine = JobEngine::new(
            store,
            PathBuf::from("missing-chdman"),
            Arc::new(NoopEventSink),
        )
        .unwrap();

        assert_eq!(engine.history()[0].state, JobState::Interrupted);
        assert!(!temporary.exists());
        assert_eq!(fs::read(input).unwrap(), b"source");
        engine.shutdown();
    }
}
