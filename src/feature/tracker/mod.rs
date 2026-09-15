use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use error_stack::Result;

mod flatfile;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StartTime(DateTime<Utc>);

impl StartTime {
    pub fn now() -> Self {
        Self(Utc::now())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EndTime(DateTime<Utc>);

impl EndTime {
    pub fn now() -> Self {
        Self(Utc::now())
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