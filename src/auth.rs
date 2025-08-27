// API key authentication middleware for Pali server
// TODO: Add rate limiting per API key
// TODO: Implement key usage analytics/metrics
// TODO: Add request logging for security auditing

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use worker::*;
use crate::models::{ApiResponse, KeyType, hash_api_key};
use crate::db::Database;

#[derive(Clone)]
pub struct AuthContext {
    pub key_type: KeyType,
    pub client_name: String,
}

// TODO: Add support for multiple auth methods (Bearer tokens, etc.)
// TODO: Implement key expiration checking
pub async fn auth_middleware(
    State(env): State<Env>,
    mut req: Request,
    next: Next,
) -> Response {
    let d1 = match env.d1("DB") {
        Ok(db) => db,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(axum::body::Body::from(serde_json::to_string(&ApiResponse::<()>::error("Database not configured".to_string())).unwrap()))
                .unwrap();
        }
    };

    let db = Database::new(d1);

    let api_key = match req.headers().get("X-API-Key").and_then(|v| v.to_str().ok()) {
        Some(key) => key,
        None => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(axum::body::Body::from(serde_json::to_string(&ApiResponse::<()>::error("Missing API key".to_string())).unwrap()))
                .unwrap();
        }
    };

    let key_hash = hash_api_key(api_key);
    
    let key_info = match db.validate_api_key(&key_hash).await {
        Ok(Some(info)) => info,
        Ok(None) => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(axum::body::Body::from(serde_json::to_string(&ApiResponse::<()>::error("Invalid API key".to_string())).unwrap()))
                .unwrap();
        },
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(axum::body::Body::from(serde_json::to_string(&ApiResponse::<()>::error(format!("Database error: {}", e))).unwrap()))
                .unwrap();
        }
    };

    let auth_context = AuthContext {
        key_type: key_info.key_type,
        client_name: key_info.client_name,
    };

    req.extensions_mut().insert(auth_context);
    
    next.run(req).await
}

pub async fn admin_only_middleware(
    req: Request,
    next: Next,
) -> Response {
    let auth_context = match req.extensions().get::<AuthContext>() {
        Some(ctx) => ctx,
        None => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(axum::body::Body::from(serde_json::to_string(&ApiResponse::<()>::error("Authentication required".to_string())).unwrap()))
                .unwrap();
        }
    };

    if auth_context.key_type != KeyType::Admin {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(axum::body::Body::from(serde_json::to_string(&ApiResponse::<()>::error("Admin access required".to_string())).unwrap()))
            .unwrap();
    }

    next.run(req).await
}