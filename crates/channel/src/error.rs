#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("DNP3 Error: {0}")]
    Dnp3(String),
}