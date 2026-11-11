mod application;
mod cli;
mod config_builder;
mod control;
mod point_simulator;
mod util;

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use channel::dnp3::outstation::database::EventClass;
use channel::dnp3::outstation::ApplicationIin;
use channel::server::ServerChannel;
use tokio::io::{AsyncBufReadExt, BufReader};

use application::{SimulatorApplication, SimulatorInformation};
use config_builder::OutstationConfigBuilder;
use control::SimulatorControlHandler;
use point_simulator::{csv_analog, csv_counter, BinaryFn, FrozenCounterFn, PointSimulator};
use util::now;
use crate::cli::ctx::Ctx;
use crate::cli::logger::Logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let csv_path = std::env::var("TURBINE_DATA").unwrap_or_else(|_| "turbines/turbine_1.csv".into());

    let mut server = ServerChannel::new("0.0.0.0:20000".parse()?);

    // ---- config ----
    let mut b = OutstationConfigBuilder::new();
    b.set_event_buffer_size(10)
        .set_local_addr(1)
        .set_remote_addr(2)
        .set_name_and_model("Outstation Simulator")
        .set_manufacturer("Nikola Mircic")
        .set_serial("1652003NM")
        .add_analog_inputs(2, Some(EventClass::Class1))
        .add_analog_inputs(1, Some(EventClass::Class2))
        .add_binary_outputs(1, Some(EventClass::Class1))
        .add_counters(1, Some(EventClass::Class1));
    let setup = b.build()?;

    // ---- outstation ----
    let iin = Arc::new(Mutex::new(ApplicationIin::default())); // shared with the CLI
    let fail_next = Arc::new(Mutex::new(None)); // shared with the CLI
    let handle = server.add_outstation(
        setup.config.clone(),
        Box::new(SimulatorApplication::new(iin.clone())),
        Box::new(SimulatorInformation),
        Box::new(SimulatorControlHandler::new(2, 1, fail_next.clone())), // 2 binary outputs, 1 analog output
    )?;
    setup.apply(&handle);
    let db = handle.get_database_handle(); // the simulator below takes ownership of `handle`

    // ---- simulation ----
    let mut sim = PointSimulator::new(handle, Duration::from_secs(3));
    sim.add_analog_input(0, csv_analog(&csv_path, "Voltage L1 / U (V)")?);
    sim.add_analog_input(1, csv_analog(&csv_path, "Voltage L2 / V (V)")?);
    sim.add_analog_input(2, csv_analog(&csv_path, "Voltage L3 / W (V)")?);
    sim.add_counter(0, csv_counter(&csv_path, "Energy Export counter (kWh)")?);
    let sim = sim.start();

    server.start().await?;

    // ---- cli ----
    let log_path = std::env::var("OUTSTATION_LOG").unwrap_or_else(|_| "outstation_commands.log".into());
    let mut ctx = Ctx::new(db, iin, fail_next, Logger::open(&log_path)?);
    println!("Running. Type a command, `exit` or press Ctrl+C to quit.");
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    loop {
        print!("outstation> ");
        std::io::stdout().flush()?;
        tokio::select! {
            line = lines.next_line() => match line? {
                Some(l) => if !ctx.exec_line(&l) { break },
                None => break, // EOF
            },
            _ = tokio::signal::ctrl_c() => break,
        }
    }

    println!("Exiting...");
    sim.stop().await;
    server.shutdown();
    // tokio's stdin reader is a blocking thread; don't wait for it
    std::process::exit(0);
}