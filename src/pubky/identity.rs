use pkarr::Keypair;
use crate::pubky::error::PubkyError;

/// Generate a new random PKARR keypair or load from a hex-encoded secret key.
///
/// If `secret_key_hex` is Some, decodes the 64-character hex string into 32 bytes
/// and creates a deterministic keypair. If None, generates a random keypair using
/// the OS CSPRNG and logs a warning about ephemeral identity.
pub fn generate_or_load_keypair(secret_key_hex: Option<&str>) -> Result<Keypair, PubkyError> {
    match secret_key_hex {
        Some(hex_str) => {
            if hex_str.len() != 64 {
                return Err(PubkyError::InvalidSecretKey(
                    "hex string must be 64 characters (32 bytes)"
                ));
            }
            let bytes = hex::decode(hex_str)?;
            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&bytes);
            Ok(Keypair::from_secret_key(&key_bytes))
        }
        None => {
            tracing::warn!(
                "PKARR_SECRET_KEY not set, generating ephemeral keypair. \
                 Identity will change on restart."
            );
            Ok(Keypair::random())
        }
    }
}
