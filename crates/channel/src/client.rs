use dnp3::app::ConnectStrategy;
use dnp3::link::{EndpointAddress, LinkErrorMode};
use dnp3::master::{
    AssociationConfig, AssociationHandle, AssociationHandler, AssociationInformation,
    MasterChannel, MasterChannelConfig, ReadHandler,
};
use dnp3::tcp::EndpointList;

use crate::{listener::LogListener, Error};

pub struct ClientChannel {
    channel: MasterChannel,
}

impl ClientChannel {
    /// Must be called from inside a tokio runtime (it spawns a task).
    pub fn new(host: &str, port: u16, config: MasterChannelConfig) -> Self {
        let channel = dnp3::tcp::spawn_master_tcp_client(
            LinkErrorMode::Close,
            config,
            EndpointList::single(format!("{host}:{port}")),
            ConnectStrategy::default(),
            LogListener::create("Master Client"),
        );
        Self { channel }
    }

    /// `read_handler` is the equivalent of pydnp3's `soe_handler`.
    pub async fn add_master(
        &mut self,
        outstation_address: EndpointAddress,
        config: AssociationConfig,
        read_handler: Box<dyn ReadHandler>,
        assoc_handler: Box<dyn AssociationHandler>,
        assoc_info: Box<dyn AssociationInformation>,
    ) -> Result<AssociationHandle, Error> {
        self.channel
            .add_association(outstation_address, config, read_handler, assoc_handler, assoc_info)
            .await
            .map_err(|e| Error::Dnp3(e.to_string()))
    }

    /// The channel starts disabled; this starts connecting.
    pub async fn enable(&mut self) -> Result<(), Error> {
        self.channel
            .enable()
            .await
            .map_err(|e| Error::Dnp3(e.to_string()))
    }

    /// Gives access to the raw channel for polls, commands, etc.
    pub fn inner(&mut self) -> &mut MasterChannel {
        &mut self.channel
    }

    pub fn shutdown(self) {} // dropping the channel shuts it down
}