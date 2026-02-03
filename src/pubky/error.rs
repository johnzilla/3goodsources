#[derive(Debug, thiserror::Error)]
pub enum PubkyError {
    #[error("Invalid secret key: {0}")]
    InvalidSecretKey(&'static str),

    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),
}
