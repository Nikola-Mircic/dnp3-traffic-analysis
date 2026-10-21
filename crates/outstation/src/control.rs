use std::time::Duration;

use channel::dnp3::app::control::*;
use channel::dnp3::app::measurement::*;
use channel::dnp3::outstation::database::*;
use channel::dnp3::outstation::*;

use crate::util::now;

pub struct SimulatorControlHandler {
    binary_outputs: u16,
    analog_outputs: u16,
}

enum CrobAction {
    /// Maintained state (latch on/off, or a trip pulse whose resulting position persists).
    Latch(bool),
    /// Output goes active for the given time, then reverts to false.
    Pulse(Duration),
}

impl SimulatorControlHandler {
    /// Number of configured binary / analog output points, used to reject out-of-range indices.
    pub fn new(binary_outputs: u16, analog_outputs: u16) -> Self {
        Self {
            binary_outputs,
            analog_outputs,
        }
    }

    fn classify(control: &Group12Var1) -> Option<CrobAction> {
        match (control.code.op_type, control.code.tcc) {
            (OpType::LatchOn, _) => Some(CrobAction::Latch(true)),
            (OpType::LatchOff, _) => Some(CrobAction::Latch(false)),
            // TRIP_PULSE_ON: breaker ends up open and stays open after the pulse
            (OpType::PulseOn, TripCloseCode::Trip) => Some(CrobAction::Latch(false)),
            // PULSE_ON / CLOSE_PULSE_ON: active for on_time, then reverts
            (OpType::PulseOn, _) => Some(CrobAction::Pulse(Duration::from_millis(
                control.on_time as u64,
            ))),
            // PULSE_OFF and NUL are not supported
            _ => None,
        }
    }

    fn set_binary_output(database: &mut DatabaseHandle, index: u16, value: bool) {
        database.transaction(|db| {
            db.update(
                index,
                &BinaryOutputStatus::new(value, Flags::ONLINE, now()),
                UpdateOptions::detect_event(),
            );
        });
    }

    fn select_analog(&self, index: u16) -> CommandStatus {
        if index < self.analog_outputs {
            CommandStatus::Success
        } else {
            CommandStatus::OutOfRange
        }
    }

    fn operate_analog(
        &self,
        database: &mut DatabaseHandle,
        index: u16,
        value: f64,
    ) -> CommandStatus {
        if index >= self.analog_outputs {
            return CommandStatus::OutOfRange;
        }
        tracing::info!("analog output index {} -> {}", index, value);
        database.transaction(|db| {
            db.update(
                index,
                &AnalogOutputStatus::new(value, Flags::ONLINE, now()),
                UpdateOptions::detect_event(),
            );
        });
        CommandStatus::Success
    }
}

// ---------- CROB (group 12 var 1) ----------

impl ControlSupport<Group12Var1> for SimulatorControlHandler {
    // Select is a feasibility check only: nothing is actuated.
    fn select(
        &mut self,
        control: Group12Var1,
        index: u16,
        _database: &mut DatabaseHandle,
    ) -> CommandStatus {
        if index >= self.binary_outputs {
            return CommandStatus::OutOfRange;
        }
        match Self::classify(&control) {
            Some(_) => CommandStatus::Success,
            None => CommandStatus::NotSupported,
        }
    }

    fn operate(
        &mut self,
        control: Group12Var1,
        index: u16,
        _op_type: OperateType,
        database: &mut DatabaseHandle,
    ) -> CommandStatus {
        if index >= self.binary_outputs {
            return CommandStatus::OutOfRange;
        }
        match Self::classify(&control) {
            Some(CrobAction::Latch(value)) => {
                tracing::info!("binary output index {} -> {} (maintained)", index, value);
                Self::set_binary_output(database, index, value);
                CommandStatus::Success
            }
            Some(CrobAction::Pulse(duration)) => {
                tracing::info!(
                    "binary output index {} -> ON (pulse, {} ms)",
                    index,
                    duration.as_millis()
                );
                Self::set_binary_output(database, index, true);

                // revert after on_time (replaces threading.Timer)
                let mut db = database.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(duration).await;
                    Self::set_binary_output(&mut db, index, false);
                });
                CommandStatus::Success
            }
            None => {
                tracing::warn!("unsupported control code for index {}", index);
                CommandStatus::NotSupported
            }
        }
    }
}

// ---------- Analog outputs (group 41 var 1..4: i32, i16, f32, f64) ----------

macro_rules! impl_analog_control {
    ($($ty:ty),+) => {
        $(
            impl ControlSupport<$ty> for SimulatorControlHandler {
                fn select(
                    &mut self,
                    _control: $ty,
                    index: u16,
                    _database: &mut DatabaseHandle,
                ) -> CommandStatus {
                    self.select_analog(index)
                }

                fn operate(
                    &mut self,
                    control: $ty,
                    index: u16,
                    _op_type: OperateType,
                    database: &mut DatabaseHandle,
                ) -> CommandStatus {
                    self.operate_analog(database, index, control.value as f64)
                }
            }
        )+
    };
}

impl_analog_control!(Group41Var1, Group41Var2, Group41Var3, Group41Var4);

// Marker trait: satisfied because all ControlSupport<T> impls exist above.
impl ControlHandler for SimulatorControlHandler {}