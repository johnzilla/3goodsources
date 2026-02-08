pub mod config;
pub mod error;
pub mod normalize;
pub mod scorer;

pub use config::MatchConfig;
pub use error::MatchError;
pub use scorer::match_query;
