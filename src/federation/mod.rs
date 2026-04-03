pub mod error;
pub mod types;

pub use error::FederationError;
pub use types::{
    CachedPeer, FederatedMatch, PeerCurator, PeerEndorsement, PeerRegistry, PeerStatus, TrustLevel,
};
