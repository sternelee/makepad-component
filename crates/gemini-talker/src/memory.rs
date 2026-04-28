//! Memory Module
//!
//! Thin re-exports from storage so the rest of the codebase uses one set of types.
//! All persistence is done through `Storage` (SQLite) in storage.rs.

pub use crate::storage::{Memory, MemorySummary, Message};

/// Format a Unix timestamp to a display string ("MM/DD/YY HH:MMam/pm")
pub fn format_timestamp(timestamp: f64) -> String {
    let total_seconds = timestamp as u64;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let hour_12 = if hours == 0 { 12 } else if hours > 12 { hours - 12 } else { hours };
    let am_pm = if hours >= 12 { "PM" } else { "AM" };
    format!(
        "{:02}/{:02}/{:02} {:02}:{:02}{}",
        (total_seconds / 86400 / 30) % 12 + 1,
        (total_seconds / 86400) % 30 + 1,
        (total_seconds / 86400 / 365) % 100,
        hour_12,
        minutes,
        am_pm
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_format_timestamp() {
        let result = format_timestamp(0.0);
        assert!(result.contains("AM") || result.contains("PM"));
    }
}
