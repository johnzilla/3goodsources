#[derive(Debug, thiserror::Error)]
pub enum PubkyError {
    #[error("Pubky not implemented")]
    NotImplemented,
}
