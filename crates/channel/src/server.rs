use std::net::SocketAddr;

use dnp3::link::LinkErrorMode;
use dnp3::outstation::{
    ControlHandler, OutstationApplication, OutstationConfig, OutstationHandle,
    OutstationInformation,
};
use dnp3::tcp::{AddressFilter, ServerHandle, Server};

use crate::{listener::LogListener, Error};

pub struct ServerChannel {
    server: Option<Server>,   // consumed by start()
    handle: Option<ServerHandle>, // keeps the server alive
}

impl ServerChannel {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            server: Some(Server::new_tcp_server(LinkErrorMode::Close, addr)),
            handle: None,
        }
    }

    /// Must be called before `start()`.
    pub fn add_outstation(
        &mut self,
        config: OutstationConfig,
        application: Box<dyn OutstationApplication>,
        information: Box<dyn OutstationInformation>,
        control_handler: Box<dyn ControlHandler>,
    ) -> Result<OutstationHandle, Error> {
        let server = self
            .server
            .as_mut()
            .ok_or_else(|| Error::Dnp3("server already started".into()))?;

        server
            .add_outstation(
                config,
                application,
                information,
                control_handler,
                LogListener::create("Outstation Server"),
                AddressFilter::Any,
            )
            .map_err(|e| Error::Dnp3(e.to_string()))
    }

    /// Binds the TCP listener and starts accepting connections.
    pub async fn start(&mut self) -> Result<(), Error> {
        let server = self
            .server
            .take()
            .ok_or_else(|| Error::Dnp3("server already started".into()))?;
        self.handle = Some(server.bind().await?);
        Ok(())
    }

    /// Dropping the server handle shuts everything down.
    pub fn shutdown(&mut self) {
        self.handle = None;
    }
}