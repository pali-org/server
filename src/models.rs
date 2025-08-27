use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export shared types
pub use pali_types::*;

// Server-specific API key model with key_hash field
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiKey {
    pub id: String,
    pub key_hash: String,
    pub client_name: String,
    pub key_type: KeyType,
    pub last_used: Option<i64>,
    pub created_at: i64,
    pub active: bool,
}

// TODO: Consider using a more secure key generation method
// TODO: Add configurable key prefixes (e.g., 'pali_admin_', 'pali_client_')
pub fn generate_api_key() -> String {
    let uuid = Uuid::new_v4();
    format!("pali_{}", uuid.to_string().replace("-", ""))
}

// TODO: Consider adding salt to hash function for additional security
// TODO: Implement key stretching (PBKDF2, bcrypt, or argon2) for production use
pub fn hash_api_key(key: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}