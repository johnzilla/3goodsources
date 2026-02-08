pub mod error;
pub mod loader;
pub mod types;

// Re-export types for convenient access
pub use error::RegistryError;
pub use loader::load;
pub use types::Registry;
