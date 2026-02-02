pub mod error;
pub mod types;

// Re-export types for convenient access
pub use error::RegistryError;
pub use types::{Category, Curator, Endorsement, Registry, Source, SourceType};
