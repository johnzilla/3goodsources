use serde::{Deserialize, Serialize};

/// A platform identity claim linking a public key to a social account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformClaim {
    pub platform: Platform,
    pub handle: String,
    pub proof_url: String,
}

/// Supported identity platforms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    X,
    Nostr,
    Github,
}

/// Whether an identity represents a human or a bot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdentityType {
    Human,
    Bot,
}

/// A registered identity with platform claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Identity {
    pub name: String,

    #[serde(rename = "type")]
    pub identity_type: IdentityType,

    pub platforms: Vec<PlatformClaim>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_pubkey: Option<String>,
}

impl Default for Identity {
    fn default() -> Self {
        Self {
            name: String::new(),
            identity_type: IdentityType::Human,
            platforms: Vec::new(),
            operator_pubkey: None,
        }
    }
}
