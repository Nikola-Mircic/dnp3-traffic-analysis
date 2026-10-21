mod application;
mod config_builder;
mod control;
mod point_simulator;
mod util;

use std::time::Duration;

use channel::dnp3::app::measurement::*;
use channel::dnp3::outstation::database::EventClass;
use channel::server::ServerChannel;

use application::{SimulatorApplication, SimulatorInformation};
use config_builder::OutstationConfigBuilder;
use control::SimulatorControlHandler;
use point_simulator::{csv_analog, csv_counter, BinaryFn, FrozenCounterFn, PointSimulator};
use util::now;

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
    let handle = server.add_outstation(
        setup.config.clone(),
        Box::new(SimulatorApplication),
        Box::new(SimulatorInformation),
        Box::new(SimulatorControlHandler::new(2, 1)), // 2 binary outputs, 1 analog output
    )?;
    setup.apply(&handle);

    // ---- simulation ----
    let mut sim = PointSimulator::new(handle, Duration::from_secs(3));
    sim.add_analog_input(0, csv_analog(&csv_path, "Voltage L1 / U (V)")?);
    sim.add_analog_input(1, csv_analog(&csv_path, "Voltage L2 / V (V)")?);
    sim.add_analog_input(2, csv_analog(&csv_path, "Voltage L3 / W (V)")?);
    sim.add_counter(0, csv_counter(&csv_path, "Energy Export counter (kWh)")?);
    let sim = sim.start();

    server.start().await?;
    println!("Running. Press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await?;

    println!("Exiting...");
    sim.stop().await;
    server.shutdown();
    Ok(())
}