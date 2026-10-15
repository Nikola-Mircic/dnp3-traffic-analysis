use std::path::Path;
use pcap::{Capture, Offline};
use crate::domain::packet_source::PacketSource;
use crate::error::CaptureError;

pub struct FilePacketSource{
    capture: Capture<Offline>,
}

fn create_capture(path: &str) -> Capture<Offline> {
    let mut capture = Capture::from_file(Path::new(path)).unwrap();

    capture.filter("tcp port 20000", true).unwrap();

    capture
}

impl FilePacketSource {
    pub fn new(path: &str) -> FilePacketSource {
        let capture = create_capture(path);

        FilePacketSource {
            capture
        }
    }
}

impl Iterator for FilePacketSource {
    type Item=Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let packet = self.next_packet();

        match packet {
            Ok(packet) => {Some(packet.to_vec())},
            Err(CaptureError::PcapError(pcap::Error::NoMorePackets)) => None,
            Err(err) => {
                println!("{:?}", err);
                None
            }
        }
    }
}

impl PacketSource for FilePacketSource {
    fn next_packet(&mut self) -> Result<&[u8], CaptureError> {
        let packet = self.capture.next_packet();

        match packet {
            Ok(packet) => Ok(packet.data),
            Err(e) => Err(CaptureError::PcapError(e))
        }
    }
}