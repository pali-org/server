// API key authentication logic for Pali server
// Authentication is fully integrated into all handlers
// TODO: Add rate limiting per API key
// TODO: Implement key usage analytics/metrics

use worker::*;
use crate::models::{KeyType, hash_api_key};
use crate::db::Database;

#[derive(Clone)]
pub struct AuthContext {
    pub key_type: KeyType,
    pub client_name: String,
}

// Helper function to validate API key from request headers
// Integrated into all protected handlers
pub async fn validate_api_key_from_request(req: &Request, env: &Env) -> Option<AuthContext> {
    let api_key = req.headers().get("X-API-Key").ok()??;
    let key_hash = hash_api_key(&api_key);
    
    let d1 = env.d1("DB").ok()?;
    let db = Database::new(d1);
    
    let key_info = db.validate_api_key(&key_hash).await.ok()??;
    
    Some(AuthContext {
        key_type: key_info.key_type,
        client_name: key_info.client_name,
    })
}

// Helper to check if auth context has admin privileges
pub fn is_admin(auth: &AuthContext) -> bool {
    auth.key_type == KeyType::Admin
}