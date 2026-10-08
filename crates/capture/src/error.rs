#[derive(Debug, thiserror::Error)]
pub enum CaptureError{
    #[error("Source error: {0}")]
    PcapError(pcap::Error),
    #[error("IO error: {0}")]
    IO(std::io::Error),
    #[error("Packet format error: {0}")]
    FormatError(std::io::Error),
}