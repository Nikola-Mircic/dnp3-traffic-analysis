use chrono::Utc;
use std::{fs::File, io::Write};

pub struct Logger(File);

impl Logger {
    pub fn open(path: &str) -> std::io::Result<Self> {
        Ok(Self(File::create(path)?))
    }

    pub fn log(&mut self, kind: &str, msg: impl std::fmt::Display) {
        let now = Utc::now();
        // epoch_ms joins directly against pcap timestamps; ISO is for humans
        let _ = writeln!(
            self.0,
            "{}\t{}\t{}\t{}",
            now.timestamp_millis(),
            now.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
            kind,
            msg
        );
        let _ = self.0.flush(); // flush every line so a crash loses nothing
    }
}