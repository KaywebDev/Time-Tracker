use std::time::Duration;

use chrono::Utc;
use error_stack::{Result, ResultExt};

use crate::feature::tracker::Tracker;

#[derive(Debug, thiserror::Error)]
#[error("reporter error")]
pub struct ReporterError;

#[derive(Debug, Clone, Copy)]
pub enum ReportTimespan {
    Last(Duration),
}

pub trait Reporter: Tracker {
    fn total_duration(&self, timespan: ReportTimespan) -> Result<Duration, ReporterError> {
        match timespan {
            ReportTimespan::Last(duration) => {
                let target: i64 = (Utc::now() - duration).timestamp_millis();
                let total_ms: i64 = self.records().change_context(ReporterError).attach_printable("Failed to query records")?
                    .filter_map(|record| {
                        if record.start.timestamp_millis() >= target {
                            let ms: i64 = record.end.timestamp_millis() - record.start.timestamp_millis();
                            Some(ms)
                        } else {
                            None
                        }
                    })
                    .sum::<i64>();
                Ok(Duration::from_millis(total_ms as u64))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::feature::tracker::tlib::FakeTracker;

use super::*;

    impl Reporter for FakeTracker {}

    #[test]
    fn calculates_correct_duration_when_there_are_no_records() {
        let tracker = FakeTracker::default(); // Using fake tracker for development purposes
        let duration = tracker.total_duration(ReportTimespan::Last(Duration::from_secs(1))); // This way allows for expandability

        assert_eq!(duration.unwrap(), Duration::from_millis(0));
    }
    #[test]
    fn calculate_duration_when_there_are_two_records() {
        let mut tracker = FakeTracker::default();
        tracker.start().unwrap();
        std::thread::sleep(Duration::from_millis(500)); // Simulate some work between start and stop
        tracker.stop().unwrap();
        tracker.start().unwrap();
        std::thread::sleep(Duration::from_millis(500)); // Simulate some work between start and stop
        tracker.stop().unwrap();

        let duration: Result<Duration, ReporterError> = tracker.total_duration(ReportTimespan::Last(Duration::from_secs(1)));
        assert!(duration.unwrap() >= Duration::from_millis(1000));
    }
}