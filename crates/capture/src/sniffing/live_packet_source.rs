use crate::domain::packet_source::PacketSource;
use crate::error::CaptureError;
use pcap::{Active, Capture, Device};

pub struct LivePacketSource {
    capture: Capture<Active>,
}

impl LivePacketSource {
    pub fn default() -> Self{
        let device = Device::lookup().unwrap().unwrap();
        let mut capture = Capture::from_device(device).unwrap()
                                            .promisc(true)
                                            .immediate_mode(true)
                                            .timeout(50)
                                            .open().unwrap();
        capture.filter("tcp port 20000", true).unwrap();
        LivePacketSource{
            capture
        }
    }

    pub fn from_dev(device: &Device) -> Self {
        let mut capture = Capture::from_device(device.clone()).unwrap()
                                                .promisc(true)
                                                .immediate_mode(true)
                                                .timeout(50)
                                                .open().unwrap();
        capture.filter("tcp port 20000", true).unwrap();
        LivePacketSource{
            capture
        }
    }
}

impl PacketSource for LivePacketSource {
    fn next_packet(&mut self) -> Result<&[u8], CaptureError> {
        let packet = self.capture.next_packet();

        match packet {
            Ok(packet) => Ok(packet.data),
            Err(e) => Err(CaptureError::PcapError(e))
        }
    }
}
