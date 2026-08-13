pub mod chdman;
pub mod discovery;
pub mod domain;
pub mod jobs;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use domain::DiscoveryReport;
use jobs::{EventSink, JobEngine, JobRecord, JobSpec, JobStore, QueueSnapshot, Settings};
use tauri::{Emitter, Manager};

#[tauri::command]
fn discover_sources(paths: Vec<PathBuf>) -> DiscoveryReport {
    discovery::discover_sources(&paths)
}

#[tauri::command]
fn enqueue_job(
    job_spec: JobSpec,
    engine: tauri::State<'_, Arc<JobEngine>>,
) -> Result<JobRecord, String> {
    engine.enqueue(job_spec)
}

#[tauri::command]
fn get_queue(engine: tauri::State<'_, Arc<JobEngine>>) -> QueueSnapshot {
    engine.snapshot()
}

#[tauri::command]
fn set_queue_paused(paused: bool, engine: tauri::State<'_, Arc<JobEngine>>) -> QueueSnapshot {
    engine.set_paused(paused)
}

#[tauri::command]
fn cancel_job(id: String, engine: tauri::State<'_, Arc<JobEngine>>) -> Result<JobRecord, String> {
    engine.cancel(&id)
}

#[tauri::command]
fn retry_job(id: String, engine: tauri::State<'_, Arc<JobEngine>>) -> Result<JobRecord, String> {
    engine.retry(&id)
}

#[tauri::command]
fn remove_job(id: String, engine: tauri::State<'_, Arc<JobEngine>>) -> Result<(), String> {
    engine.remove(&id)
}

#[tauri::command]
fn get_history(engine: tauri::State<'_, Arc<JobEngine>>) -> Vec<JobRecord> {
    engine.history()
}

#[tauri::command]
fn get_settings(engine: tauri::State<'_, Arc<JobEngine>>) -> Result<Settings, String> {
    engine.settings()
}

#[tauri::command]
fn update_settings(
    settings: Settings,
    engine: tauri::State<'_, Arc<JobEngine>>,
) -> Result<Settings, String> {
    engine.update_settings(settings)
}

#[tauri::command]
fn confirm_close(window: tauri::Window, engine: tauri::State<'_, Arc<JobEngine>>) {
    engine.shutdown();
    let _ = window.close();
}

struct AppEventSink {
    app: tauri::AppHandle,
}

impl EventSink for AppEventSink {
    fn job_changed(&self, record: &JobRecord) {
        let _ = self.app.emit("job-state", record);
    }

    fn progress_changed(&self, id: &str, progress: &domain::JobProgress) {
        let _ = self.app.emit(
            "job-progress",
            serde_json::json!({ "id": id, "progress": progress }),
        );
    }

    fn queue_changed(&self, snapshot: &QueueSnapshot) {
        let _ = self.app.emit("queue-state", snapshot);
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_directory)?;
            let store = JobStore::open(&data_directory.join("hunk.sqlite3"))
                .map_err(std::io::Error::other)?;
            let program = resolve_chdman(app.handle());
            let events = Arc::new(AppEventSink {
                app: app.handle().clone(),
            });
            let engine = JobEngine::new(store, program, events).map_err(std::io::Error::other)?;
            app.manage(engine);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let engine = window.state::<Arc<JobEngine>>();
                if engine.has_active_job() {
                    api.prevent_close();
                    let _ = window.emit("close-requested", ());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            discover_sources,
            enqueue_job,
            get_queue,
            set_queue_paused,
            cancel_job,
            retry_job,
            remove_job,
            get_history,
            get_settings,
            update_settings,
            confirm_close,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Hunk");
}

fn resolve_chdman(app: &tauri::AppHandle) -> PathBuf {
    if let Some(path) = std::env::var_os("HUNK_CHDMAN") {
        return PathBuf::from(path);
    }

    let target_file_name = if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "chdman-x86_64-unknown-linux-gnu"
    } else if cfg!(target_os = "windows") {
        "chdman.exe"
    } else {
        "chdman"
    };
    let mut candidates = Vec::new();
    if let Ok(resource_directory) = app.path().resource_dir() {
        add_chdman_candidates(&mut candidates, &resource_directory, target_file_name);
    }
    if let Ok(executable) = std::env::current_exe()
        && let Some(executable_directory) = executable.parent()
    {
        add_chdman_candidates(&mut candidates, executable_directory, target_file_name);
    }
    candidates.push(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join(target_file_name),
    );
    candidates
        .into_iter()
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| PathBuf::from("chdman"))
}

fn add_chdman_candidates(candidates: &mut Vec<PathBuf>, directory: &Path, target_file_name: &str) {
    candidates.push(directory.join("chdman"));
    candidates.push(directory.join(target_file_name));
    candidates.push(directory.join("binaries").join(target_file_name));
    candidates.push(directory.join("lib").join("hunk").join("chdman"));
}

#[cfg(test)]
mod tests {
    use super::add_chdman_candidates;
    use std::path::{Path, PathBuf};

    #[test]
    fn packaged_sidecar_candidates_cover_tauri_and_flatpak_layouts() {
        let mut candidates = Vec::new();

        add_chdman_candidates(
            &mut candidates,
            Path::new("/app"),
            "chdman-x86_64-unknown-linux-gnu",
        );

        assert_eq!(
            candidates,
            vec![
                PathBuf::from("/app/chdman"),
                PathBuf::from("/app/chdman-x86_64-unknown-linux-gnu"),
                PathBuf::from("/app/binaries/chdman-x86_64-unknown-linux-gnu"),
                PathBuf::from("/app/lib/hunk/chdman"),
            ]
        );
    }
}
