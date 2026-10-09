use channel::dnp3::app::*;
use channel::dnp3::outstation::*;

pub struct SimApplication;

impl OutstationApplication for SimApplication {
    // SupportsWriteAbsoluteTime() == True in the Python version
    fn write_absolute_time(&mut self, time: Timestamp) -> Result<(), RequestError> {
        tracing::info!("master wrote absolute time: {:?}", time);
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
pub struct SimInformation;

impl OutstationInformation for SimInformation {}