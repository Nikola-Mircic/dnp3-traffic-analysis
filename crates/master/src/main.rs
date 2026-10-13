mod master;
mod soe_handler;

use std::time::Duration;

use channel::client::ClientChannel;
use channel::dnp3::link::EndpointAddress;
use channel::dnp3::master::{Classes, EventClasses, MasterChannelConfig, ReadRequest};

use master::{association_config, NoopAssociationHandler, NoopAssociationInformation};
use soe_handler::SoeHandler;

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

    // class scan of all event classes every 2 seconds
    let _poll = association
        .add_poll(ReadRequest::ClassScan(Classes::new(false, EventClasses::all())), Duration::from_secs(2))
        .await?;

    client.enable().await?;
    println!("Initialization complete. Polling data.");


    let input =

    println!("Exiting...");
    Ok(()) // dropping `client` shuts the channel down
}