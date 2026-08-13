use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use super::model::{JobRecord, Settings, now_millis};

const HISTORY_LIMIT: usize = 100;

pub struct JobStore {
    connection: Connection,
}

impl JobStore {
    pub fn open(path: &Path) -> Result<Self, String> {
        let connection = Connection::open(path).map_err(display_error)?;
        let store = Self { connection };
        store.migrate()?;
        Ok(store)
    }

    #[cfg(test)]
    pub fn in_memory() -> Result<Self, String> {
        let connection = Connection::open_in_memory().map_err(display_error)?;
        let store = Self { connection };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), String> {
        self.connection
            .execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA foreign_keys = ON;
                 CREATE TABLE IF NOT EXISTS job_records (
                    id TEXT PRIMARY KEY NOT NULL,
                    state TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    record_json TEXT NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS job_records_state_created
                    ON job_records(state, created_at DESC);
                 CREATE TABLE IF NOT EXISTS settings (
                    key TEXT PRIMARY KEY NOT NULL,
                    value_json TEXT NOT NULL
                 );",
            )
            .map_err(display_error)
    }

    pub fn save_job(&self, record: &JobRecord) -> Result<(), String> {
        let json = serde_json::to_string(record).map_err(display_error)?;
        self.connection
            .execute(
                "INSERT INTO job_records(id, state, created_at, updated_at, record_json)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(id) DO UPDATE SET
                    state = excluded.state,
                    updated_at = excluded.updated_at,
                    record_json = excluded.record_json",
                params![
                    record.id,
                    record.state.as_database_str(),
                    to_i64(record.created_at),
                    to_i64(now_millis()),
                    json
                ],
            )
            .map_err(display_error)?;
        self.prune_history()
    }

    pub fn delete_job(&self, id: &str) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM job_records WHERE id = ?1", [id])
            .map_err(display_error)?;
        Ok(())
    }

    pub fn load_all(&self) -> Result<Vec<JobRecord>, String> {
        let mut statement = self
            .connection
            .prepare("SELECT record_json FROM job_records ORDER BY created_at ASC")
            .map_err(display_error)?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(display_error)?;
        rows.map(|row| {
            let json = row.map_err(display_error)?;
            serde_json::from_str(&json).map_err(display_error)
        })
        .collect()
    }

    pub fn load_settings(&self) -> Result<Settings, String> {
        let json = self
            .connection
            .query_row(
                "SELECT value_json FROM settings WHERE key = 'preferences'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(display_error)?;
        json.map(|value| serde_json::from_str(&value).map_err(display_error))
            .unwrap_or_else(|| Ok(Settings::default()))
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        let json = serde_json::to_string(settings).map_err(display_error)?;
        self.connection
            .execute(
                "INSERT INTO settings(key, value_json) VALUES ('preferences', ?1)
                 ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
                [json],
            )
            .map_err(display_error)?;
        Ok(())
    }

    fn prune_history(&self) -> Result<(), String> {
        self.connection
            .execute(
                "DELETE FROM job_records
                 WHERE state IN ('completed', 'failed', 'cancelled', 'interrupted')
                   AND id NOT IN (
                     SELECT id FROM job_records
                     WHERE state IN ('completed', 'failed', 'cancelled', 'interrupted')
                     ORDER BY updated_at DESC
                     LIMIT ?1
                   )",
                [HISTORY_LIMIT],
            )
            .map_err(display_error)?;
        Ok(())
    }
}

fn to_i64(value: u64) -> i64 {
    value.try_into().unwrap_or(i64::MAX)
}

fn display_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::domain::{MediaKind, Operation, SourceFormat, SourceSet};

    use super::*;
    use crate::jobs::model::{JobOptions, JobSpec, JobState};

    fn record(state: JobState) -> JobRecord {
        let mut record = JobRecord::queued(JobSpec {
            source: SourceSet {
                primary_file: PathBuf::from("disc.iso"),
                referenced_files: vec![],
                format: SourceFormat::Iso,
                media_kind: MediaKind::UnknownOptical,
                tracks: vec![],
                total_size: 4,
                validation_problems: vec![],
            },
            operation: Operation::CreateDvd,
            destination: Some(PathBuf::from("disc.chd")),
            options: JobOptions::default(),
        });
        record.state = state;
        record
    }

    #[test]
    fn round_trips_jobs_and_settings() {
        let store = JobStore::in_memory().unwrap();
        let job = record(JobState::Queued);
        store.save_job(&job).unwrap();
        store
            .save_settings(&Settings {
                destination_directory: Some("output".into()),
            })
            .unwrap();

        assert_eq!(store.load_all().unwrap()[0].id, job.id);
        assert_eq!(
            store.load_settings().unwrap().destination_directory,
            Some(PathBuf::from("output"))
        );
    }

    #[test]
    fn keeps_only_latest_one_hundred_terminal_records() {
        let store = JobStore::in_memory().unwrap();
        for _ in 0..105 {
            store.save_job(&record(JobState::Completed)).unwrap();
        }

        assert_eq!(store.load_all().unwrap().len(), 100);
    }
}
