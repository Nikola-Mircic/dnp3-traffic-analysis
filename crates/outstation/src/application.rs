use channel::dnp3::app::*;
use channel::dnp3::outstation::*;

use std::sync::{Arc, Mutex};

/// Application IIN bits, shared with the CLI (`iin set|clear`).
pub type SharedIin = Arc<Mutex<ApplicationIin>>;

pub struct SimulatorApplication {
    iin: SharedIin,
}

impl SimulatorApplication {
    pub fn new(iin: SharedIin) -> Self {
        Self { iin }
    }
}

impl OutstationApplication for SimulatorApplication {
    fn get_application_iin(&self) -> ApplicationIin {
        *self.iin.lock().unwrap()
    }

    // SupportsWriteAbsoluteTime() == True in the Python version
    fn write_absolute_time(&mut self, time: Timestamp) -> Result<(), RequestError> {
        tracing::info!("master wrote absolute time: {:?}", time);
        self.iin.lock().unwrap().need_time = false; // a time write satisfies NEED_TIME
        Ok(())
    }

    // SUPPORTED_DELAY_COARSE, 5 seconds
    fn cold_restart(&mut self) -> Option<RestartDelay> {
        tracing::info!("restarting [COLD]...");
        Some(RestartDelay::Seconds(5))
    }

    // SUPPORTED_DELAY_FINE, 4500 ms
    fn warm_restart(&mut self) -> Option<RestartDelay> {
        tracing::info!("restarting [WARM]...");
        Some(RestartDelay::Milliseconds(4500))
    }
}

/// Python had no equivalent; the Rust API just requires one. All defaults.
pub struct SimulatorInformation;

impl OutstationInformation for SimulatorInformation {}