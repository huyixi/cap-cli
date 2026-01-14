use chrono::{DateTime, Local};

pub(crate) fn format_display_time(value: &str) -> String {
    match DateTime::parse_from_rfc3339(value) {
        Ok(timestamp) => timestamp
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        Err(_) => value.to_string(),
    }
}
