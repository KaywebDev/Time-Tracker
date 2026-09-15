//! A filesystem tracker

// flat file tracker
//
// 2 files:
// - "lockfile": Tracker is running
// - "database file": JSON doc (records)

use std::{fs::OpenOptions, io::{Read, Write}, path::{Path, PathBuf}};
use error_stack::{Result, ResultExt};
use serde::{Deserialize, Serialize};

use crate::feature::tracker::{EndTime, StartTime, StartupStatus, TimeRecord, Tracker, TrackerError};

#[derive(Debug, thiserror::Error)]
#[error("filesystem tracker error")]
pub struct FlatFileTrackerError;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LockFileData {
    start_time: StartTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FlatFileDatabase {
    records: Vec<TimeRecord>,
}

impl FlatFileDatabase {
    pub fn push(&mut self, value: TimeRecord) {
        self.records.push(value)
    }
}

fn read_lockfile<P>(lock_file: P) -> Result<LockFileData, FlatFileTrackerError>
where
    P: AsRef<Path>,
{
    let file = OpenOptions::new()
        .read(true)
        .open(lock_file.as_ref())
        .change_context(FlatFileTrackerError)
        .attach_printable("Unable to open lockfile for reading")?;
    
    Ok(serde_json::from_reader(file)
        .change_context(FlatFileTrackerError)
        .attach_printable("Unable to deserialize lockfile data")?)
}

fn load_database<P>(db: P) -> Result<FlatFileDatabase, FlatFileTrackerError>
where
    P: AsRef<Path>,
{
    let mut db_buf = String::default();
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(db.as_ref())
        .change_context(FlatFileTrackerError)
        .attach_printable("Unable to open database file for reading")?
        .read_to_string(&mut db_buf)
        .change_context(FlatFileTrackerError)
        .attach_printable("Unable to read database file")?;

    if db_buf.is_empty() {
        Ok(FlatFileDatabase::default())
    } else {
        Ok(serde_json::from_str(&db_buf)
        .change_context(FlatFileTrackerError)
        .attach_printable("Unable to deserialize database file")?)
    }
}

fn save_database<P>(db: P, data: &FlatFileDatabase) -> Result<(), FlatFileTrackerError>
where
    P: AsRef<Path>,
{
    let db_buf = serde_json::to_string(data)
        .change_context(FlatFileTrackerError)
        .attach_printable("Unable to serialize database data")?;
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(db.as_ref())
        .change_context(FlatFileTrackerError)
        .attach_printable("Unable to open database file for writing")?
        .write_all(db_buf.as_bytes())
        .change_context(FlatFileTrackerError)
        .attach_printable("Unable to write database file")?;

    Ok(())
}

struct FlatFileTracker {
    db_file: PathBuf,
    lock_file: PathBuf
}

impl FlatFileTracker {
     fn new<D, L>(db_file: D, lock_file: L) -> Self
    where 
        D: Into<PathBuf>,
        L: Into<PathBuf>,
    {
        Self {
            db_file: db_file.into(),
            lock_file: lock_file.into()
        }
    }
    
    fn start_impl(&mut self) -> Result<StartupStatus, FlatFileTrackerError> {
        // Two scenarios: either the tracker is starting for the first time, or it is already running.
        if self.is_running() {
            return Ok(StartupStatus::Running);
        }
    
        let lockfile_data: String = {
            let data: LockFileData = LockFileData { start_time: StartTime::now() };
            serde_json::to_string(&data)
                .change_context(FlatFileTrackerError)
                .attach_printable("Unable to serialize lockfile data")?
        };
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.lock_file)
            .change_context(FlatFileTrackerError)
            .attach_printable("Unable to create new lockfile when starting tracker")?
            .write_all(lockfile_data.as_bytes())
            .change_context(FlatFileTrackerError)
            .attach_printable("Unable to write lockfile data when starting tracker")?;
    
        Ok(StartupStatus::Started)
    }
    
    fn stop_impl(&mut self) -> Result<(), FlatFileTrackerError> {
        let start_time: StartTime = read_lockfile(&self.lock_file)?.start_time;
        let end_time: EndTime = EndTime::now();
        let time_record: TimeRecord = TimeRecord { start: start_time, end: end_time };
        let mut db = load_database(&self.db_file)?;
        
        db.push(time_record);
        save_database(&self.db_file, &db)
            .change_context(FlatFileTrackerError)
            .attach_printable("Unable to save database when stopping tracker")?;
    
        std::fs::remove_file(&self.lock_file)
            .change_context(FlatFileTrackerError)
            .attach_printable("Unable to remove lockfile when stopping tracker")?;
        Ok(())
    }
}

impl Tracker for FlatFileTracker {
    fn start(&mut self) -> Result<StartupStatus, TrackerError> {
        self.start_impl().change_context(TrackerError)
    }

    fn is_running(&self) -> bool {
        self.lock_file.exists()
    }

    fn stop(&mut self) -> Result<(), TrackerError> {
        self.stop_impl().change_context(TrackerError)
    }

    fn records(&self) -> Result<impl Iterator<Item = TimeRecord>, TrackerError> {
        let db = load_database(&self.db_file)
            .change_context(TrackerError)
            .attach_printable("Unable to load database when fetching records")?;
        Ok(db.records.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use assert_fs::fixture::ChildPath;

    use super::*;

    fn tracking_paths() -> (TempDir, ChildPath, ChildPath) {
        let temp_dir: TempDir = TempDir::new().unwrap(); // this needs to be returned so it doesn't get dropped immediately
        let db_file = temp_dir.child("db.json");
        let lock_file = temp_dir.child("lockfile");
        (temp_dir, db_file, lock_file)
    }

    fn new_flatfile_tracker(db_file: &ChildPath, lock_file: &ChildPath) -> FlatFileTracker {
        FlatFileTracker::new(db_file.to_path_buf(), lock_file.to_path_buf())
    }

    #[test]
    fn starts_tracking_with_default_tracker() {
        let (_temp_dir, db_file, lock_file) = tracking_paths();
        // Given a new tracker
        let mut tracker: FlatFileTracker = new_flatfile_tracker(&db_file, &lock_file);

        // When we start tracking
        tracker.start().unwrap();

        // Then it should be running
        assert!(tracker.is_running());
    }
    #[test]
    fn is_running_returns_true_when_started() {
        let (_temp_dir, db_file, lock_file) = tracking_paths();
        // Given a new tracker
        let mut tracker: FlatFileTracker = new_flatfile_tracker(&db_file, &lock_file);

        // When we start tracking
        tracker.start().unwrap();

        // Then it should be running
        assert!(tracker.is_running());
    }
    #[test]
    fn is_running_returns_false_when_not_started() {
        let (_temp_dir, db_file, lock_file) = tracking_paths();
        // Given a new tracker
        let tracker: FlatFileTracker = new_flatfile_tracker(&db_file, &lock_file);

        // When we check if it's running
        let running: bool = tracker.is_running();

        // Then it should return false
        assert!(!running);
    }
    #[test]
    fn is_running_returns_false_after_stopping() {
        let (_temp_dir, db_file, lock_file) = tracking_paths();
        // Given a tracker that is running
        let mut tracker: FlatFileTracker = new_flatfile_tracker(&db_file, &lock_file);
        tracker.start().unwrap();

        // When the tracker is stopped
        tracker.stop().unwrap();

        // Then it should not be running
        assert!(!tracker.is_running());
    }
    #[test]
    fn time_record_created_when_tracking_stops() {
        let (_temp_dir, db_file, lock_file) = tracking_paths();
        // Given a tracker that is running
        let mut tracker: FlatFileTracker = new_flatfile_tracker(&db_file, &lock_file);
        tracker.start().unwrap();

        // sleep for a short time to simulate tracking
        std::thread::sleep(std::time::Duration::from_millis(20));

        // When the tracker is stopped
        tracker.stop().unwrap();

        // Then a record is saved to the database file
        assert!(tracker.records().unwrap().next().is_some());
    }
    #[test]
    fn inital_start_returns_started_status() {
        let (_temp_dir, db_file, lock_file) = tracking_paths();
        // Given a new tracker
        let mut tracker: FlatFileTracker = new_flatfile_tracker(&db_file, &lock_file);

        // When the tracker starts for the first time
        let started = tracker.start().unwrap();

        // Then it should return a started status
        assert_eq!(started, StartupStatus::Started);
    }
    #[test]
    fn multiple_starts_returns_already_running_state() {
        let (_temp_dir, db_file, lock_file) = tracking_paths();
        // Given a new tracker thats running
        let mut tracker: FlatFileTracker = new_flatfile_tracker(&db_file, &lock_file);
        tracker.start().unwrap();

        // When the tracker starts again
        let started = tracker.start().unwrap();
        

        // Then starting again should return an already running state
        assert_eq!(started, StartupStatus::Running);
    }
}