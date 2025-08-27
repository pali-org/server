// HTTP endpoint handlers for the Pali todo server API
// Authentication is fully integrated with security logging
// TODO: Add request validation middleware
// TODO: Implement rate limiting per API key

use worker::*;
#[allow(clippy::wildcard_imports)]
use crate::models::*;
use crate::db::Database;
use crate::auth::{validate_api_key_from_request, is_admin};

// Security logging helper
fn log_auth_attempt(method: &str, path: &str, client_name: Option<&str>, success: bool) {
    let status = if success { "SUCCESS" } else { "FAILED" };
    let client = client_name.unwrap_or("unknown");
    console_log!("AUTH {}: {} {} - client: {}", status, method, path, client);
}


// Simple sync handlers for basic routes
pub fn root(_: Request, _: RouteContext<()>) -> Result<Response> {
    Response::ok("Pali Server API v1.0 - Self-hosted todo management")
}

pub fn health_check(_: Request, _: RouteContext<()>) -> Result<Response> {
    Response::ok("OK")
}

// Async handlers for database operations
pub async fn create_todo(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Validate API key
    let _auth = match validate_api_key_from_request(&req, &ctx.env).await {
        Some(auth) => {
            log_auth_attempt(req.method().to_string().as_str(), req.url()?.path(), Some(&auth.client_name), true);
            auth
        },
        None => {
            log_auth_attempt(req.method().to_string().as_str(), req.url()?.path(), None, false);
            return Ok(Response::from_json(&ApiResponse::<()>::error("Invalid or missing API key".to_string()))?
                .with_status(401));
        }
    };
    
    let body: CreateTodoRequest = match req.json().await {
        Ok(body) => body,
        Err(_) => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Invalid JSON body".to_string()))
                ?.with_status(400));
        }
    };
    
    let d1 = match ctx.env.d1("DB") {
        Ok(db) => db,
        Err(_) => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Database not configured".to_string()))?
                .with_status(500));
        }
    };

    let db = Database::new(d1);
    
    match db.create_todo(body).await {
        Ok(todo) => Ok(Response::from_json(&ApiResponse::success(todo))?),
        Err(e) => Ok(Response::from_json(&ApiResponse::<()>::error(format!("Failed to create todo: {}", e)))?
            .with_status(500)),
    }
}

pub async fn list_todos(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Validate API key
    let _auth = match validate_api_key_from_request(&req, &ctx.env).await {
        Some(auth) => auth,
        None => {
            return Ok(Response::from_json(&ApiResponse::<Vec<Todo>>::error("Invalid or missing API key".to_string()))?
                .with_status(401));
        }
    };
    
    // Parse query parameters manually
    let url = req.url()?;
    let completed_filter = url.query_pairs()
        .find(|(key, _)| key == "completed")
        .and_then(|(_, value)| value.parse::<bool>().ok());
    
    let d1 = match ctx.env.d1("DB") {
        Ok(db) => db,
        Err(_) => {
            return Ok(Response::from_json(&ApiResponse::<Vec<Todo>>::error("Database not configured".to_string()))?
                .with_status(500));
        }
    };

    let db = Database::new(d1);
    
    match db.list_todos(completed_filter).await {
        Ok(todos) => Ok(Response::from_json(&ApiResponse::success(todos))?),
        Err(e) => Ok(Response::from_json(&ApiResponse::<()>::error(format!("Failed to list todos: {}", e)))?
            .with_status(500)),
    }
}

pub async fn search_todos(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Validate API key
    let _auth = match validate_api_key_from_request(&req, &ctx.env).await {
        Some(auth) => auth,
        None => {
            return Ok(Response::from_json(&ApiResponse::<Vec<Todo>>::error("Invalid or missing API key".to_string()))?
                .with_status(401));
        }
    };
    
    let url = req.url()?;
    let query = match url.query_pairs().find(|(key, _)| key == "q") {
        Some((_, value)) => value.to_string(),
        None => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Missing 'q' query parameter".to_string()))?
                .with_status(400));
        }
    };
    
    let d1 = match ctx.env.d1("DB") {
        Ok(db) => db,
        Err(_) => {
            return Ok(Response::from_json(&ApiResponse::<Vec<Todo>>::error("Database not configured".to_string()))?
                .with_status(500));
        }
    };

    let db = Database::new(d1);
    
    match db.search_todos(&query).await {
        Ok(todos) => Ok(Response::from_json(&ApiResponse::success(todos))?),
        Err(e) => Ok(Response::from_json(&ApiResponse::<()>::error(format!("Failed to search todos: {}", e)))?
            .with_status(500)),
    }
}

pub async fn get_todo(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Validate API key
    let _auth = match validate_api_key_from_request(&req, &ctx.env).await {
        Some(auth) => auth,
        None => {
            return Ok(Response::from_json(&ApiResponse::<Todo>::error("Invalid or missing API key".to_string()))?
                .with_status(401));
        }
    };
    
    let id = match ctx.param("id") {
        Some(id) => id,
        None => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Missing todo ID".to_string()))?
                .with_status(400));
        }
    };
    
    let d1 = match ctx.env.d1("DB") {
        Ok(db) => db,
        Err(_) => {
            return Ok(Response::from_json(&ApiResponse::<Todo>::error("Database not configured".to_string()))?
                .with_status(500));
        }
    };

    let db = Database::new(d1);
    
    match db.get_todo(id).await {
        Ok(Some(todo)) => Ok(Response::from_json(&ApiResponse::success(todo))?),
        Ok(None) => Ok(Response::from_json(&ApiResponse::<()>::error("Todo not found".to_string()))?
            .with_status(404)),
        Err(e) => Ok(Response::from_json(&ApiResponse::<()>::error(format!("Failed to get todo: {}", e)))?
            .with_status(500)),
    }
}

pub async fn update_todo(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Validate API key
    let _auth = match validate_api_key_from_request(&req, &ctx.env).await {
        Some(auth) => auth,
        None => {
            return Ok(Response::from_json(&ApiResponse::<Todo>::error("Invalid or missing API key".to_string()))?
                .with_status(401));
        }
    };
    
    let id = match ctx.param("id") {
        Some(id) => id,
        None => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Missing todo ID".to_string()))?
                .with_status(400));
        }
    };
    
    let body: UpdateTodoRequest = match req.json().await {
        Ok(body) => body,
        Err(_) => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Invalid JSON body".to_string()))?
                .with_status(400));
        }
    };
    
    let d1 = match ctx.env.d1("DB") {
        Ok(db) => db,
        Err(_) => {
            return Ok(Response::from_json(&ApiResponse::<Todo>::error("Database not configured".to_string()))?
                .with_status(500));
        }
    };

    let db = Database::new(d1);
    
    match db.update_todo(id, body).await {
        Ok(Some(todo)) => Ok(Response::from_json(&ApiResponse::success(todo))?),
        Ok(None) => Ok(Response::from_json(&ApiResponse::<()>::error("Todo not found".to_string()))?
            .with_status(404)),
        Err(e) => Ok(Response::from_json(&ApiResponse::<()>::error(format!("Failed to update todo: {}", e)))?
            .with_status(500)),
    }
}

pub async fn delete_todo(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Validate API key
    let _auth = match validate_api_key_from_request(&req, &ctx.env).await {
        Some(auth) => auth,
        None => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Invalid or missing API key".to_string()))?
                .with_status(401));
        }
    };
    
    let id = match ctx.param("id") {
        Some(id) => id,
        None => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Missing todo ID".to_string()))?
                .with_status(400));
        }
    };
    
    let d1 = match ctx.env.d1("DB") {
        Ok(db) => db,
        Err(_) => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Database not configured".to_string()))?
                .with_status(500));
        }
    };

    let db = Database::new(d1);
    
    match db.delete_todo(id).await {
        Ok(true) => Ok(Response::from_json(&ApiResponse::success(()))?),
        Ok(false) => Ok(Response::from_json(&ApiResponse::<()>::error("Todo not found".to_string()))?
            .with_status(404)),
        Err(e) => Ok(Response::from_json(&ApiResponse::<()>::error(format!("Failed to delete todo: {}", e)))?
            .with_status(500)),
    }
}

pub async fn toggle_todo(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Validate API key
    let _auth = match validate_api_key_from_request(&req, &ctx.env).await {
        Some(auth) => auth,
        None => {
            return Ok(Response::from_json(&ApiResponse::<Todo>::error("Invalid or missing API key".to_string()))?
                .with_status(401));
        }
    };
    
    let id = match ctx.param("id") {
        Some(id) => id,
        None => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Missing todo ID".to_string()))?
                .with_status(400));
        }
    };
    
    let d1 = match ctx.env.d1("DB") {
        Ok(db) => db,
        Err(_) => {
            return Ok(Response::from_json(&ApiResponse::<Todo>::error("Database not configured".to_string()))?
                .with_status(500));
        }
    };

    let db = Database::new(d1);
    
    match db.toggle_todo(id).await {
        Ok(Some(todo)) => Ok(Response::from_json(&ApiResponse::success(todo))?),
        Ok(None) => Ok(Response::from_json(&ApiResponse::<()>::error("Todo not found".to_string()))?
            .with_status(404)),
        Err(e) => Ok(Response::from_json(&ApiResponse::<()>::error(format!("Failed to toggle todo: {}", e)))?
            .with_status(500)),
    }
}

// Admin handlers
pub async fn rotate_admin_key(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    // Admin authentication check deprecated - endpoint replaced with /reinitialize
    
    Ok(Response::from_json(&ApiResponse::<()>::error("Use POST /reinitialize for admin key rotation".to_string()))?
        .with_status(410)) // Gone - use new endpoint
}

pub async fn create_api_key(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Validate admin API key
    let auth = match validate_api_key_from_request(&req, &ctx.env).await {
        Some(auth) => auth,
        None => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Invalid or missing API key".to_string()))?
                .with_status(401));
        }
    };
    
    if !is_admin(&auth) {
        return Ok(Response::from_json(&ApiResponse::<()>::error("Admin privileges required".to_string()))?
            .with_status(403));
    }
    
    let body: CreateApiKeyRequest = match req.json().await {
        Ok(body) => body,
        Err(_) => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Invalid JSON body".to_string()))?
                .with_status(400));
        }
    };
    
    let d1 = match ctx.env.d1("DB") {
        Ok(db) => db,
        Err(_) => {
            return Ok(Response::from_json(&ApiResponse::<ApiKeyResponse>::error("Database not configured".to_string()))?
                .with_status(500));
        }
    };

    let db = Database::new(d1);
    let api_key = generate_api_key();
    let key_hash = hash_api_key(&api_key);
    
    match db.create_api_key(key_hash, body.client_name.clone(), body.key_type.clone()).await {
        Ok(id) => {
            let response = ApiKeyResponse {
                id,
                client_name: body.client_name,
                key_type: body.key_type,
                api_key,
                created_at: chrono::Utc::now().timestamp(),
            };
            Ok(Response::from_json(&ApiResponse::success(response))?)
        },
        Err(e) => Ok(Response::from_json(&ApiResponse::<()>::error(format!("Failed to create API key: {}", e)))?
            .with_status(500)),
    }
}

pub async fn list_api_keys(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Validate admin API key
    let auth = match validate_api_key_from_request(&req, &ctx.env).await {
        Some(auth) => auth,
        None => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Invalid or missing API key".to_string()))?
                .with_status(401));
        }
    };
    
    if !is_admin(&auth) {
        return Ok(Response::from_json(&ApiResponse::<()>::error("Admin privileges required".to_string()))?
            .with_status(403));
    }
    
    let d1 = match ctx.env.d1("DB") {
        Ok(db) => db,
        Err(_) => {
            return Ok(Response::from_json(&ApiResponse::<Vec<ApiKeyInfo>>::error("Database not configured".to_string()))?
                .with_status(500));
        }
    };

    let db = Database::new(d1);
    
    match db.list_api_keys().await {
        Ok(keys) => {
            let key_infos: Vec<ApiKeyInfo> = keys.into_iter().map(|k| ApiKeyInfo {
                id: k.id,
                client_name: k.client_name,
                key_type: k.key_type,
                last_used: k.last_used,
                created_at: k.created_at,
                active: k.active,
            }).collect();
            Ok(Response::from_json(&ApiResponse::success(key_infos))?)
        },
        Err(e) => Ok(Response::from_json(&ApiResponse::<()>::error(format!("Failed to list API keys: {}", e)))?
            .with_status(500)),
    }
}

pub async fn revoke_api_key(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Validate admin API key
    let auth = match validate_api_key_from_request(&req, &ctx.env).await {
        Some(auth) => auth,
        None => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Invalid or missing API key".to_string()))?
                .with_status(401));
        }
    };
    
    if !is_admin(&auth) {
        return Ok(Response::from_json(&ApiResponse::<()>::error("Admin privileges required".to_string()))?
            .with_status(403));
    }
    
    let id = match ctx.param("id") {
        Some(id) => id,
        None => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Missing key ID".to_string()))?
                .with_status(400));
        }
    };
    
    let d1 = match ctx.env.d1("DB") {
        Ok(db) => db,
        Err(_) => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Database not configured".to_string()))?
                .with_status(500));
        }
    };

    let db = Database::new(d1);
    
    match db.revoke_api_key(id).await {
        Ok(_) => Ok(Response::from_json(&ApiResponse::success(()))?),
        Err(e) => Ok(Response::from_json(&ApiResponse::<()>::error(format!("Failed to revoke API key: {}", e)))?
            .with_status(500)),
    }
}

// One-time initialization endpoint - creates the first admin key
pub async fn initialize_server(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = match ctx.env.d1("DB") {
        Ok(db) => db,
        Err(_) => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Database not configured".to_string()))?
                .with_status(500));
        }
    };

    let db = Database::new(d1);
    
    // Check if already initialized
    match db.is_initialized().await {
        Ok(true) => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Server already initialized".to_string()))?
                .with_status(409)); // Conflict
        },
        Ok(false) => {
            // Proceed with initialization
        },
        Err(e) => {
            return Ok(Response::from_json(&ApiResponse::<()>::error(format!("Failed to check initialization status: {}", e)))?
                .with_status(500));
        }
    }
    
    // Generate the first admin key
    let api_key = generate_api_key();
    let key_hash = hash_api_key(&api_key);
    
    match db.initialize_with_admin_key(key_hash).await {
        Ok(id) => {
            let response = ApiKeyResponse {
                id,
                client_name: "Initial Admin Key".to_string(),
                key_type: KeyType::Admin,
                api_key,
                created_at: chrono::Utc::now().timestamp(),
            };
            Ok(Response::from_json(&ApiResponse::success(response))?)
        },
        Err(e) => Ok(Response::from_json(&ApiResponse::<()>::error(format!("Failed to initialize server: {}", e)))?
            .with_status(500)),
    }
}

// Emergency reinitialize endpoint - deactivates ALL admin keys and creates new one
pub async fn reinitialize_server(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = match ctx.env.d1("DB") {
        Ok(db) => db,
        Err(_) => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Database not configured".to_string()))?
                .with_status(500));
        }
    };

    let db = Database::new(d1);
    
    // Check if database is initialized (has any admin keys)
    match db.is_initialized().await {
        Ok(false) => {
            return Ok(Response::from_json(&ApiResponse::<()>::error("Server not initialized. Use POST /initialize first".to_string()))?
                .with_status(400));
        },
        Ok(true) => {
            // Proceed with reinitialization
        },
        Err(e) => {
            return Ok(Response::from_json(&ApiResponse::<()>::error(format!("Failed to check initialization status: {}", e)))?
                .with_status(500));
        }
    }
    
    // Generate new admin key
    let api_key = generate_api_key();
    let key_hash = hash_api_key(&api_key);
    
    match db.reinitialize_admin_keys(key_hash).await {
        Ok(id) => {
            let response = ApiKeyResponse {
                id,
                client_name: "Reinitialized Admin Key".to_string(),
                key_type: KeyType::Admin,
                api_key,
                created_at: chrono::Utc::now().timestamp(),
            };
            Ok(Response::from_json(&ApiResponse::success(response))?)
        },
        Err(e) => Ok(Response::from_json(&ApiResponse::<()>::error(format!("Failed to reinitialize server: {}", e)))?
            .with_status(500)),
    }
}