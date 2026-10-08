use crate::domain::dnp3_packet::Dnp3Frame;
use crate::error::CaptureError;

pub trait PacketAdapter {
    fn adapt(&self, raw: &[u8]) -> Result<Dnp3Frame, CaptureError>;
}