//! Time and duration utilities.

use chrono::{DateTime, Duration, Utc};
use std::fmt;

/// Calculate the duration between two timestamps.
#[must_use]
pub fn duration_between(start: DateTime<Utc>, end: DateTime<Utc>) -> Duration {
    end - start
}

/// Format a duration in a human-readable format.
#[must_use]
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.num_seconds();

    if total_seconds < 60 {
        format!("{}s", total_seconds)
    } else if total_seconds < 3600 {
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{}m {}s", minutes, seconds)
    } else {
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        format!("{}h {}m", hours, minutes)
    }
}

/// Format a timestamp in ISO 8601 format.
#[must_use]
pub fn format_timestamp(ts: DateTime<Utc>) -> String {
    ts.to_rfc3339()
}

/// Get the current timestamp in ISO 8601 format.
#[must_use]
pub fn now_iso8601() -> String {
    format_timestamp(Utc::now())
}

/// A duration with display formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayDuration(pub Duration);

impl fmt::Display for DisplayDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_duration(self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_seconds() {
        let duration = Duration::seconds(45);
        assert_eq!(format_duration(duration), "45s");
    }

    #[test]
    fn test_format_duration_minutes() {
        let duration = Duration::minutes(5) + Duration::seconds(30);
        assert_eq!(format_duration(duration), "5m 30s");
    }

    #[test]
    fn test_format_duration_hours() {
        let duration = Duration::hours(2) + Duration::minutes(15) + Duration::seconds(45);
        assert_eq!(format_duration(duration), "2h 15m");
    }

    #[test]
    fn test_now_iso8601() {
        let ts = now_iso8601();
        // ISO 8601 format: 2024-01-15T10:30:00+00:00
        assert!(ts.contains('T'));
        assert!(ts.contains("+00:00") || ts.contains("Z"));
    }
}
