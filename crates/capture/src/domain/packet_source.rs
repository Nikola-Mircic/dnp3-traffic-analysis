use crate::error::CaptureError;

pub trait PacketSource: Iterator<Item = Vec<u8>> {
    fn next_packet(&mut self) -> Result<&[u8], CaptureError>;
}