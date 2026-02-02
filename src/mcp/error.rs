#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("MCP not implemented")]
    NotImplemented,
}
