use serde::{Deserialize, Serialize};
use rand::{thread_rng, Rng};
use pbkdf2::pbkdf2_hmac_array;
use sha2::Sha256;

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

// Generate cryptographically secure API key with 256 bits of entropy
pub fn generate_api_key() -> String {
    let mut rng = thread_rng();
    let key_bytes: [u8; 32] = rng.gen(); // 256 bits of entropy
    format!("pali_{}", hex::encode(key_bytes))
}

// Hash API key using PBKDF2 with fixed salt for security
// Uses server-wide salt for consistency without per-key storage
pub fn hash_api_key(key: &str) -> String {
    // Use fixed server-wide salt (better than key-derived salt)
    // In production, consider storing random salts per key in database
    const SERVER_SALT: &[u8] = b"pali_server_salt_2024_secure_todo_api_v1";
    
    // Use PBKDF2 with 100,000 iterations for key stretching
    let hash = pbkdf2_hmac_array::<Sha256, 32>(key.as_bytes(), SERVER_SALT, 100_000);
    hex::encode(hash)
}

// Response for ID prefix resolution
#[derive(Debug, Serialize, Deserialize)]
pub struct IdResolutionResponse {
    pub full_id: String,
}