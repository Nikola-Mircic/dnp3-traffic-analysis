use std::time::Duration;

use channel::dnp3::app::measurement::*;
use channel::dnp3::outstation::database::*;
use channel::dnp3::outstation::OutstationHandle;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::util::now;

type SimulatorFn<T> = Box<dyn FnMut(u64) -> T + Send>;

pub type BinaryFn = SimulatorFn<BinaryInput>;
pub type AnalogFn = SimulatorFn<AnalogInput>;
pub type CounterFn = SimulatorFn<Counter>;
pub type FrozenCounterFn = SimulatorFn<FrozenCounter>;

type BoxError = Box<dyn std::error::Error>;

// ---------- CSV sources ----------
fn load_column(path: &str, column: &str) -> Result<Vec<Option<f64>>, BoxError> {
    let mut reader = csv::Reader::from_path(path)?;
    let idx = reader
        .headers()?
        .iter()
        .position(|h| h.trim() == column)
        .ok_or_else(|| format!("Column '{column}' not found in {path}"))?;

    let mut values = Vec::new();
    for record in reader.records() {
        let record = record?;
        let v = record
            .get(idx)
            .and_then(|s| s.trim().parse::<f64>().ok())
            .filter(|v| !v.is_nan());
        values.push(v);
    }
    if values.is_empty() {
        return Err(format!("no data rows in {path}").into());
    }
    Ok(values)
}

type CsvSource = Box<dyn FnMut(u64) -> (f64, bool) + Send>;

/// Returns (value, is_valid). Missing values repeat the last good value with is_valid = false.
fn csv_source(path: &str, column: &str) -> Result<CsvSource, BoxError> {
    tracing::info!("creating csv source for {} [{}]", column, path);
    let values = load_column(path, column)?;
    let column = column.to_string();
    let mut last = 0.0;

    Ok(Box::new(move |cycle: u64| {
        match values[(cycle as usize) % values.len()] {
            Some(v) => {
                last = v;
                (v, true)
            }
            None => {
                tracing::warn!("NaN detected at {} -> using [{}]", column, last);
                (last, false)
            }
        }
    }))
}

fn quality(valid: bool) -> Flags {
    if valid { Flags::ONLINE } else { Flags::COMM_LOST | Flags::LOCAL_FORCED}
}

pub fn csv_analog(path: &str, column: &str) -> Result<AnalogFn, BoxError> {
    let mut src = csv_source(path, column)?;
    Ok(Box::new(move |cycle| {
        let (v, valid) = src(cycle);

        AnalogInput::new(v, quality(valid), now())
    }))
}

pub fn csv_counter(path: &str, column: &str) -> Result<CounterFn, BoxError> {
    let mut src = csv_source(path, column)?;
    Ok(Box::new(move |cycle| {
        let (v, ok) = src(cycle);
        Counter::new(v as u32, quality(ok), now())
    }))
}

// ---------- Simulator ----------

pub struct PointSimulator {
    handle: OutstationHandle,
    interval: Duration,
    binary_inputs: Vec<(u16, BinaryFn)>,
    analog_inputs: Vec<(u16, AnalogFn)>,
    counters: Vec<(u16, CounterFn)>,
    frozen_counters: Vec<(u16, FrozenCounterFn)>,
}

pub struct SimulatorHandle {
    stop: watch::Sender<bool>,
    task: JoinHandle<()>,
}

impl SimulatorHandle {
    pub async fn stop(self) {
        let _ = self.stop.send(true);
        let _ = self.task.await;
    }
}

impl PointSimulator {
    pub fn new(handle: OutstationHandle, interval: Duration) -> Self {
        Self {
            handle,
            interval,
            binary_inputs: Vec::new(),
            analog_inputs: Vec::new(),
            counters: Vec::new(),
            frozen_counters: Vec::new(),
        }
    }

    pub fn add_binary_input(&mut self, index: u16, f: BinaryFn) {
        self.binary_inputs.push((index, f));
    }
    pub fn add_analog_input(&mut self, index: u16, f: AnalogFn) {
        self.analog_inputs.push((index, f));
    }
    pub fn add_counter(&mut self, index: u16, f: CounterFn) {
        self.counters.push((index, f));
    }
    pub fn add_frozen_counter(&mut self, index: u16, f: FrozenCounterFn) {
        self.frozen_counters.push((index, f));
    }

    pub fn start(mut self) -> SimulatorHandle {
        let (stop_tx, mut stop_rx) = watch::channel(false);

        let task = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(self.interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            ticker.tick().await;

            let mut cycle: u64 = 0;
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        self.run_cycle(cycle);
                        cycle += 1;
                    }
                    _ = stop_rx.changed() => break,
                }
            }
        });

        SimulatorHandle { stop: stop_tx, task }
    }

    /// One transaction per cycle: all points are updated together.
    fn run_cycle(&mut self, cycle: u64) {
        let opts = UpdateOptions::detect_event();
        let binary = &mut self.binary_inputs;
        let analog = &mut self.analog_inputs;
        let counters = &mut self.counters;
        let frozen = &mut self.frozen_counters;

        self.handle.transaction(|db| {
            for (index, f) in binary.iter_mut() {
                db.update(*index, &f(cycle), opts);
            }
            for (index, f) in analog.iter_mut() {
                db.update(*index, &f(cycle), opts);
            }
            for (index, f) in counters.iter_mut() {
                db.update(*index, &f(cycle), opts);
            }
            for (index, f) in frozen.iter_mut() {
                db.update(*index, &f(cycle), opts);
            }
        });
    }
}