pub mod chdman;
pub mod discovery;
pub mod domain;

use std::path::PathBuf;

use domain::DiscoveryReport;

#[tauri::command]
fn discover_sources(paths: Vec<PathBuf>) -> DiscoveryReport {
    discovery::discover_sources(&paths)
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![discover_sources])
        .run(tauri::generate_context!())
        .expect("failed to run Hunk");
}
