// Core modules for the Pali todo server
mod models;     // Data structures and API types
mod db;         // Database operations and D1 integration
mod auth;       // API key authentication middleware
mod handlers;   // HTTP endpoint handlers

use worker::*;
use models::hash_api_key;
use db::Database;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
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
    
    // Workers router - much cleaner for edge computing
    Router::new()
        .get("/", handlers::root)
        .get("/health", handlers::health_check)
        // Todo routes
        .post_async("/todos", handlers::create_todo)
        .get_async("/todos", handlers::list_todos) 
        .get_async("/todos/search", handlers::search_todos)
        .get_async("/todos/:id", handlers::get_todo)
        .put_async("/todos/:id", handlers::update_todo)
        .delete_async("/todos/:id", handlers::delete_todo)
        .patch_async("/todos/:id/toggle", handlers::toggle_todo)
        // Admin routes  
        .post_async("/admin/keys/rotate", handlers::rotate_admin_key)
        .post_async("/admin/keys/generate", handlers::create_api_key)
        .get_async("/admin/keys", handlers::list_api_keys)
        .delete_async("/admin/keys/:id", handlers::revoke_api_key)
        .run(req, env)
        .await
}