#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("JSON parsing failed: {0}")]
    ParseError(String),
    #[error("Response serialization failed: {0}")]
    SerializationError(String),
}
