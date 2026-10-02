use std::time::{SystemTime, UNIX_EPOCH};

use channel::dnp3::app::measurement::Time;

/// Current wall-clock time as a synchronized DNP3 timestamp.
pub fn now() -> Time {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    Time::synchronized(ms)
}