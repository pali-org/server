use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Core data models for the Pali todo server
// TODO: Consider adding validation traits (e.g., title length limits, priority bounds)

// Main todo item structure
// TODO: Add optional tags/categories field
// TODO: Consider adding recurrence patterns for repeating todos
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub priority: i32,
    pub due_date: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub due_date: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
    pub priority: Option<i32>,
    pub due_date: Option<i64>,
}

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum KeyType {
    Admin,
    Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub client_name: String,
    pub key_type: KeyType,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: String,
    pub client_name: String,
    pub key_type: KeyType,
    pub api_key: String,
    pub created_at: i64,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyInfo {
    pub id: String,
    pub client_name: String,
    pub key_type: KeyType,
    pub last_used: Option<i64>,
    pub created_at: i64,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
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