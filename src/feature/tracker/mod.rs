use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use error_stack::Result;

mod flatfile;
mod reporter;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StartTime(DateTime<Utc>);

impl StartTime {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn timestamp_millis(&self) -> i64 {
        self.0.timestamp_millis()
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EndTime(DateTime<Utc>);

impl EndTime {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn timestamp_millis(&self) -> i64 {
        self.0.timestamp_millis()
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeRecord {
    pub start: StartTime,
    pub end: EndTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartupStatus {
    Started,
    Running
}

#[derive(Debug, thiserror::Error)]
#[error("tracker error")]
pub struct TrackerError;

pub trait Tracker {
    /// Tracker trait for generic time tracking implementations on various storage backends.
    fn start(&mut self) -> Result<StartupStatus, TrackerError>;

    fn is_running(&self) -> bool;

    fn stop(&mut self) -> Result<(), TrackerError>;

    fn records(&self) -> Result<impl Iterator<Item = TimeRecord>, TrackerError>;
}

#[cfg(test)]
pub mod tlib {
    use super::*;

    #[derive(Debug, Default)]
    pub struct FakeTracker {
        tracking: Option<StartTime>,
        records: Vec<TimeRecord>,
    }

    impl Tracker for FakeTracker {
        fn start(&mut self) -> Result<StartupStatus, TrackerError> {
            if self.tracking.is_some() {
                Ok(StartupStatus::Running)
            } else {
                self.tracking = Some(StartTime::now());
                Ok(StartupStatus::Started)
            }
        }

        fn is_running(&self) -> bool {
            self.tracking.is_some()
        }

        fn stop(&mut self) -> Result<(), TrackerError> {
            let start_time: StartTime = self.tracking.take().ok_or(TrackerError)?;
            let end_time = EndTime::now();
            self.records.push(TimeRecord {
                start: start_time,
                end: end_time,
            });
            Ok(())
        }

        fn records(&self) -> Result<impl Iterator<Item = TimeRecord>, TrackerError> {
            Ok(self.records.clone().into_iter())
        }
    }
}