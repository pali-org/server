// Core modules for the Pali todo server
mod models;     // Data structures and API types
mod db;         // Database operations and D1 integration
mod auth;       // API key authentication middleware
mod handlers;   // HTTP endpoint handlers

use axum::{
    routing::{get, post, put, delete, patch},
    Router,
};
use tower_service::Service;
use worker::*;
use models::hash_api_key;
use db::Database;

// TODO: Consider extracting route definitions to separate module when adding more endpoints
fn router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        // TODO: Add authentication middleware when we figure out how to access Env in handlers
        .route("/todos", post(handlers::create_todo))
        .route("/todos", get(handlers::list_todos))
        .route("/todos/search", get(handlers::search_todos))
        .route("/todos/:id", get(handlers::get_todo))
        .route("/todos/:id", put(handlers::update_todo))
        .route("/todos/:id", delete(handlers::delete_todo))
        .route("/todos/:id/toggle", patch(handlers::toggle_todo))
        // Admin routes
        .route("/admin/keys/rotate", post(handlers::rotate_admin_key))
        .route("/admin/keys/generate", post(handlers::create_api_key))
        .route("/admin/keys", get(handlers::list_api_keys))
        .route("/admin/keys/:id", delete(handlers::revoke_api_key))
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    
    // TODO: Move admin key initialization to a proper migration system
    // Initialize admin key on first run if provided
    if let Ok(initial_key) = env.var("INITIAL_ADMIN_KEY") {
        if let Ok(d1) = env.d1("DB") {
            let db = Database::new(d1);
            let key_hash = hash_api_key(&initial_key.to_string());
            let _ = db.init_admin_key(key_hash).await;
        }
    }
    
    // Store env in request extensions for handlers to access
    // TODO: Find the proper way to pass Env to handlers in Workers Axum
    
    Ok(router().call(req).await?)
}

// TODO: Add version info from Cargo.toml and git commit hash
pub async fn root() -> &'static str {
    "Pali Server API v1.0 - Self-hosted todo management"
}

// TODO: Add proper health check with database connectivity test
pub async fn health_check() -> &'static str {
    "OK"
}