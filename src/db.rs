// Database operations and D1 integration for Pali server
// TODO: Add connection pooling if/when D1 supports it
// TODO: Implement proper transaction support
// TODO: Add database migration system

use worker::*;
use wasm_bindgen::JsValue;
use crate::models::{Todo, ApiKey, KeyType, CreateTodoRequest, UpdateTodoRequest};
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};

// WORKAROUND: D1 serialization issue with booleans
// Issue: https://github.com/cloudflare/workers-rs/issues/387
// D1 databases serialize booleans as floating point numbers (0.0/1.0) 
// which causes "invalid type: floating point 0.0, expected a boolean" errors
// when deserializing directly into structs with bool fields.
// Solution: Use intermediate row struct with i32, convert to bool manually.
#[derive(Debug, Serialize, Deserialize)]
struct TodoRow {
    id: String,
    title: String,
    description: Option<String>,
    completed: i32,  // 0 = false, 1 = true (D1 limitation)
    priority: i32,
    due_date: Option<i64>,
    created_at: i64,
    updated_at: i64,
}

impl From<TodoRow> for Todo {
    fn from(row: TodoRow) -> Self {
        Todo {
            id: row.id,
            title: row.title,
            description: row.description,
            completed: row.completed != 0,  // Convert i32 to bool (D1 workaround)
            priority: row.priority,
            due_date: row.due_date,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

// WORKAROUND: ApiKey serialization pattern (same D1 boolean issue)
// Apply same workaround pattern as TodoRow for consistent handling
#[derive(Debug, Serialize, Deserialize)]
struct ApiKeyRow {
    id: String,
    key_hash: String,
    client_name: String,
    key_type: String,  // Store as string, convert to enum
    last_used: Option<i64>,
    created_at: i64,
    active: i32,  // 0 = false, 1 = true (D1 limitation)
}

impl From<ApiKeyRow> for ApiKey {
    fn from(row: ApiKeyRow) -> Self {
        ApiKey {
            id: row.id,
            key_hash: row.key_hash,
            client_name: row.client_name,
            key_type: match row.key_type.to_lowercase().as_str() {
                "admin" => KeyType::Admin,
                _ => KeyType::Client,  // Default to Client for any non-admin type
            },
            last_used: row.last_used,
            created_at: row.created_at,
            active: row.active != 0,  // Convert i32 to bool
        }
    }
}

pub struct Database {
    d1: D1Database,
}

impl Database {
    pub fn new(d1: D1Database) -> Self {
        Self { d1 }
    }

    fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }

    fn current_timestamp() -> i64 {
        Utc::now().timestamp()
    }

    #[allow(clippy::cast_precision_loss)] // D1 requires f64 for timestamps
    fn timestamp_to_f64(timestamp: i64) -> JsValue {
        (timestamp as f64).into()
    }

    // Check if the database has been initialized (has any admin keys)
    pub async fn is_initialized(&self) -> Result<bool> {
        let stmt = self.d1.prepare(
            "SELECT COUNT(*) as count FROM api_keys WHERE key_type = 'admin'"
        );
        
        let result = stmt.bind(&[])?
            .first::<serde_json::Value>(None)
            .await?;
            
        if let Some(value) = result {
            if let Some(count) = value.get("count") {
                if let Some(count_num) = count.as_f64() {
                    return Ok(count_num > 0.0);
                }
            }
        }
        
        Ok(false)
    }

    // Initialize the database with the first admin key (one-time operation)
    pub async fn initialize_with_admin_key(&self, key_hash: String) -> Result<String> {
        // Double-check that we're not already initialized
        if self.is_initialized().await? {
            return Err(worker::Error::RustError("Database already initialized".to_string()));
        }
        
        let stmt = self.d1.prepare(
            "INSERT INTO api_keys (id, key_hash, client_name, key_type, created_at, active) 
             VALUES (?1, ?2, ?3, ?4, ?5, 1)"
        );
        
        let id = Self::generate_id();
        let now = Self::current_timestamp();
        
        stmt.bind(&[
            id.clone().into(),
            key_hash.into(),
            "Initial Admin Key".into(),
            "admin".into(),
            Self::timestamp_to_f64(now),
        ])?
        .run()
        .await?;
        
        Ok(id)
    }

    pub async fn validate_api_key(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let stmt = self.d1.prepare(
            "SELECT id, key_hash, client_name, key_type, last_used, created_at, active 
             FROM api_keys WHERE key_hash = ?1 AND active = 1"
        );
        
        let result = stmt.bind(&[key_hash.into()])?.first::<ApiKeyRow>(None).await?;
        let result = result.map(Into::into);
        
        if result.is_some() {
            let update_stmt = self.d1.prepare(
                "UPDATE api_keys SET last_used = ?1 WHERE key_hash = ?2"
            );
            let now = Utc::now().timestamp();
            update_stmt.bind(&[(now as f64).into(), key_hash.into()])?.run().await?;
        }
        
        Ok(result)
    }

    pub async fn create_api_key(&self, key_hash: String, client_name: String, key_type: KeyType) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();
        let key_type_str = match key_type {
            KeyType::Admin => "admin",
            KeyType::Client => "client",
        };
        
        let stmt = self.d1.prepare(
            "INSERT INTO api_keys (id, key_hash, client_name, key_type, created_at, active) 
             VALUES (?1, ?2, ?3, ?4, ?5, 1)"
        );
        
        stmt.bind(&[
            id.clone().into(),
            key_hash.into(),
            client_name.into(),
            key_type_str.into(),
            (now as f64).into(),
        ])?
        .run()
        .await?;
        
        Ok(id)
    }

    pub async fn rotate_admin_key(&self, old_key_hash: &str, new_key_hash: String) -> Result<String> {
        let tx = self.d1.prepare(
            "UPDATE api_keys SET active = 0 WHERE key_hash = ?1 AND key_type = 'admin'"
        );
        tx.bind(&[old_key_hash.into()])?.run().await?;
        
        let id = self.create_api_key(new_key_hash, "Rotated Admin Key".to_string(), KeyType::Admin).await?;
        Ok(id)
    }

    // Reinitialize: deactivate ALL admin keys and create a new one (emergency rotation)
    pub async fn reinitialize_admin_keys(&self, new_key_hash: String) -> Result<String> {
        // Deactivate all existing admin keys
        let deactivate_stmt = self.d1.prepare(
            "UPDATE api_keys SET active = 0 WHERE key_type = 'admin'"
        );
        deactivate_stmt.bind(&[])?.run().await?;
        
        // Create new admin key
        let id = self.create_api_key(new_key_hash, "Reinitialized Admin Key".to_string(), KeyType::Admin).await?;
        Ok(id)
    }

    pub async fn list_api_keys(&self) -> Result<Vec<ApiKey>> {
        let stmt = self.d1.prepare(
            "SELECT id, key_hash, client_name, key_type, last_used, created_at, active 
             FROM api_keys ORDER BY created_at DESC"
        );
        
        let results = stmt.bind(&[])?.all().await?;
        
        let rows: Vec<ApiKeyRow> = results.results::<ApiKeyRow>()?;
        let keys: Vec<ApiKey> = rows.into_iter().map(Into::into).collect();
        
        Ok(keys)
    }

    pub async fn revoke_api_key(&self, id: &str) -> Result<()> {
        let stmt = self.d1.prepare(
            "UPDATE api_keys SET active = 0 WHERE id = ?1"
        );
        
        stmt.bind(&[id.into()])?.run().await?;
        Ok(())
    }

    pub async fn create_todo(&self, req: CreateTodoRequest) -> Result<Todo> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();
        let priority = req.priority.unwrap_or(2);
        
        let stmt = self.d1.prepare(
            "INSERT INTO todos (id, title, description, completed, priority, due_date, created_at, updated_at) 
             VALUES (?1, ?2, ?3, 0, ?4, ?5, ?6, ?7)"
        );
        
        // WORKAROUND: D1 NULL handling issue  
        // Issue: https://github.com/cloudflare/workers-rs/issues/678
        // D1 binding doesn't support Option<T> directly - None becomes "undefined"
        // which causes "Type 'undefined' not supported" errors.
        // Solution: Manually convert None to JsValue::NULL for database compatibility.
        stmt.bind(&[
            id.clone().into(),
            req.title.clone().into(),
            req.description.clone().unwrap_or_default().into(),
            priority.into(),
            match req.due_date {
                Some(date) => (date as f64).into(),
                None => JsValue::NULL,  // Required for D1 NULL handling
            },
            (now as f64).into(),
            (now as f64).into(),
        ])?
        .run()
        .await?;
        
        Ok(Todo {
            id,
            title: req.title,
            description: req.description,
            completed: false,
            priority,
            due_date: req.due_date,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn list_todos(&self, completed_filter: Option<bool>) -> Result<Vec<Todo>> {
        let query = match completed_filter {
            Some(completed) => {
                let stmt = self.d1.prepare(
                    "SELECT * FROM todos WHERE completed = ?1 ORDER BY priority DESC, created_at DESC"
                );
                stmt.bind(&[i32::from(completed).into()])?
            },
            None => {
                let stmt = self.d1.prepare(
                    "SELECT * FROM todos ORDER BY priority DESC, created_at DESC"
                );
                stmt.bind(&[])?
            }
        };
        
        let results = query.all().await?;
        let rows: Vec<TodoRow> = results.results::<TodoRow>()?;
        let todos: Vec<Todo> = rows.into_iter().map(Into::into).collect();
        
        Ok(todos)
    }

    pub async fn get_todo(&self, id: &str) -> Result<Option<Todo>> {
        let stmt = self.d1.prepare("SELECT * FROM todos WHERE id = ?1");
        let result = stmt.bind(&[id.into()])?.first::<TodoRow>(None).await?;
        Ok(result.map(Into::into))
    }

    pub async fn update_todo(&self, id: &str, req: UpdateTodoRequest) -> Result<Option<Todo>> {
        let existing = self.get_todo(id).await?;
        
        if let Some(mut todo) = existing {
            if let Some(title) = req.title {
                todo.title = title;
            }
            if let Some(desc) = req.description {
                todo.description = Some(desc);
            }
            if let Some(completed) = req.completed {
                todo.completed = completed;
            }
            if let Some(priority) = req.priority {
                todo.priority = priority;
            }
            if let Some(due_date) = req.due_date {
                todo.due_date = Some(due_date);
            }
            
            todo.updated_at = Utc::now().timestamp();
            
            let stmt = self.d1.prepare(
                "UPDATE todos SET title = ?1, description = ?2, completed = ?3, 
                 priority = ?4, due_date = ?5, updated_at = ?6 WHERE id = ?7"
            );
            
            stmt.bind(&[
                todo.title.clone().into(),
                todo.description.clone().unwrap_or_default().into(),
                i32::from(todo.completed).into(),
                todo.priority.into(),
                match todo.due_date {
                    Some(date) => (date as f64).into(),
                    None => JsValue::NULL,  // D1 NULL handling workaround
                },
                (todo.updated_at as f64).into(),
                id.into(),
            ])?
            .run()
            .await?;
            
            Ok(Some(todo))
        } else {
            Ok(None)
        }
    }

    pub async fn toggle_todo(&self, id: &str) -> Result<Option<Todo>> {
        if let Some(mut todo) = self.get_todo(id).await? {
            todo.completed = !todo.completed;
            todo.updated_at = Utc::now().timestamp();
            
            let stmt = self.d1.prepare(
                "UPDATE todos SET completed = ?1, updated_at = ?2 WHERE id = ?3"
            );
            
            stmt.bind(&[
                i32::from(todo.completed).into(),
                (todo.updated_at as f64).into(),
                id.into(),
            ])?
            .run()
            .await?;
            
            Ok(Some(todo))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_todo(&self, id: &str) -> Result<bool> {
        let stmt = self.d1.prepare("DELETE FROM todos WHERE id = ?1");
        let result = stmt.bind(&[id.into()])?.run().await?;
        Ok(result.meta()?.is_some())
    }

    // TODO: Implement proper full-text search indexing
    // TODO: Add search relevance scoring
    // TODO: Consider using D1's full-text search features when available
    pub async fn search_todos(&self, query: &str) -> Result<Vec<Todo>> {
        let stmt = self.d1.prepare(
            "SELECT * FROM todos WHERE title LIKE ?1 OR description LIKE ?1 
             ORDER BY priority DESC, created_at DESC"
        );
        
        let search_pattern = format!("%{query}%");
        let results = stmt.bind(&[search_pattern.into()])?.all().await?;
        
        let rows: Vec<TodoRow> = results.results::<TodoRow>()?;
        let todos: Vec<Todo> = rows.into_iter().map(Into::into).collect();
        
        Ok(todos)
    }
}