mod master;
mod soe_handler;

use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use channel::client::ClientChannel;
use channel::dnp3::app::control::{CommandStatus, ControlCode, Group12Var1, OpType};
use channel::dnp3::link::EndpointAddress;
use channel::dnp3::master::{Classes, CommandBuilder, CommandMode, CommandSupport, EventClasses, MasterChannelConfig, ReadRequest};

use master::{association_config, NoopAssociationHandler, NoopAssociationInformation};
use soe_handler::SoeHandler;

/// Funkcija koja vraca CROB komandu na osnovu tipa operacije ( uvek koristim on i off time na
/// 5 sekundi zbog jednostavnosti )
fn get_crob_command(op_type: OpType) -> Group12Var1 {
    Group12Var1 {
        code: ControlCode::from_op_type(op_type),
        count: 1,
        on_time: 5000,
        off_time: 5000,
        status: CommandStatus::Success,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let host = std::env::var("OUTSTATION_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = std::env::var("OUTSTATION_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(20000);

    // master = 2, outstation = 1 (same as the Python version)
    let mut client = ClientChannel::new(
        &host,
        port,
        MasterChannelConfig::new(EndpointAddress::try_new(2)?),
    );

    let mut association = client
        .add_master(
            EndpointAddress::try_new(1)?,
            association_config(),
            SoeHandler::boxed(),
            Box::new(NoopAssociationHandler),
            Box::new(NoopAssociationInformation),
        )
        .await?;

    // class scan of all event classes every 10 seconds
    let _poll = association
        .add_poll(ReadRequest::ClassScan(Classes::new(false, EventClasses::all())), Duration::from_secs(10))
        .await?;

    client.enable().await?;
    println!("Initialization complete. Polling data.");

    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    while let Some(line) = lines.next_line().await? {
        match line.trim() {
            "exit" => break,
            "pulse on" => {
                println!("Executing pulse on command...");
                association.operate(
                    CommandMode::DirectOperate,
                    CommandBuilder::single_header_u16(
                        get_crob_command(OpType::PulseOn),
                        0
                    )).await?;
            },
            "pulse off" => {
                println!("Executing pulse off command...");
                association.operate(
                    CommandMode::DirectOperate,
                    CommandBuilder::single_header_u16(
                        get_crob_command(OpType::PulseOff),
                        0
                    )
                ).await?;
            }
            _ => {}
        }
    }

    println!("Exiting...");
    Ok(()) // dropping `client` shuts the channel down
}