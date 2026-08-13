use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::domain::{ChdInfo, JobProgress, Operation, SourceSet};

pub const MAX_LOG_LINES: usize = 400;
pub const MAX_LOG_BYTES: usize = 128 * 1024;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobOptions {
    pub split_bin: bool,
    pub processors: Option<u16>,
    pub hunk_size: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobSpec {
    pub source: SourceSet,
    pub operation: Operation,
    pub destination: Option<PathBuf>,
    #[serde(default)]
    pub options: JobOptions,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JobState {
    Queued,
    Preflight,
    Running,
    Verifying,
    Completed,
    Failed,
    Cancelled,
    Interrupted,
    Blocked,
}

impl JobState {
    pub fn is_active(self) -> bool {
        matches!(self, Self::Preflight | Self::Running | Self::Verifying)
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::Interrupted
        )
    }

    pub fn as_database_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Preflight => "preflight",
            Self::Running => "running",
            Self::Verifying => "verifying",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Interrupted => "interrupted",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobRecord {
    pub id: String,
    pub spec: JobSpec,
    pub state: JobState,
    pub progress: Option<JobProgress>,
    pub message: String,
    pub error: Option<String>,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    pub input_size: u64,
    pub output_size: Option<u64>,
    pub log: Vec<String>,
    pub chd_info: Option<ChdInfo>,
    #[serde(default)]
    pub temporary_paths: Vec<PathBuf>,
}

impl JobRecord {
    pub fn queued(spec: JobSpec) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            input_size: spec.source.total_size,
            spec,
            state: JobState::Queued,
            progress: None,
            message: "Waiting in the serial queue".to_owned(),
            error: None,
            created_at: now_millis(),
            started_at: None,
            finished_at: None,
            output_size: None,
            log: Vec::new(),
            chd_info: None,
            temporary_paths: Vec::new(),
        }
    }

    pub fn append_log(&mut self, line: impl Into<String>) {
        let line = line.into();
        if line.is_empty() {
            return;
        }
        self.log.push(line);
        while self.log.len() > MAX_LOG_LINES
            || self.log.iter().map(String::len).sum::<usize>() > MAX_LOG_BYTES
        {
            self.log.remove(0);
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueSnapshot {
    pub paused: bool,
    pub active_job_id: Option<String>,
    pub jobs: Vec<JobRecord>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub destination_directory: Option<PathBuf>,
    pub locale: Option<Locale>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    En,
    De,
}

pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
