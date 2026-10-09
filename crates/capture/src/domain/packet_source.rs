use crate::error::CaptureError;

pub trait PacketSource {
    fn next_packet(&mut self) -> Result<&[u8], CaptureError>;
}