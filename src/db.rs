// Database operations and D1 integration for Pali server
// TODO: Add connection pooling if/when D1 supports it
// TODO: Implement proper transaction support
// TODO: Add database migration system

use worker::*;
use crate::models::{Todo, ApiKey, KeyType, CreateTodoRequest, UpdateTodoRequest};
use uuid::Uuid;
use chrono::Utc;

pub struct Database {
    d1: D1Database,
}

impl Database {
    pub fn new(d1: D1Database) -> Self {
        Self { d1 }
    }

    // TODO: Replace with proper migration system
    // TODO: Add check for existing admin keys before creating
    pub async fn init_admin_key(&self, key_hash: String) -> Result<()> {
        let stmt = self.d1.prepare(
            "INSERT OR IGNORE INTO api_keys (id, key_hash, client_name, key_type, created_at, active) 
             VALUES (?1, ?2, ?3, ?4, ?5, 1)"
        );
        
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();
        
        stmt.bind(&[
            id.into(),
            key_hash.into(),
            "Initial Admin Key".into(),
            "admin".into(),
            now.into(),
        ])?
        .run()
        .await?;
        
        Ok(())
    }

    pub async fn validate_api_key(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let stmt = self.d1.prepare(
            "SELECT id, key_hash, client_name, key_type, last_used, created_at, active 
             FROM api_keys WHERE key_hash = ?1 AND active = 1"
        );
        
        let result = stmt.bind(&[key_hash.into()])?.first::<ApiKey>(None).await?;
        
        if result.is_some() {
            let update_stmt = self.d1.prepare(
                "UPDATE api_keys SET last_used = ?1 WHERE key_hash = ?2"
            );
            let now = Utc::now().timestamp();
            update_stmt.bind(&[now.into(), key_hash.into()])?.run().await?;
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
            now.into(),
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

    pub async fn list_api_keys(&self) -> Result<Vec<ApiKey>> {
        let stmt = self.d1.prepare(
            "SELECT id, key_hash, client_name, key_type, last_used, created_at, active 
             FROM api_keys ORDER BY created_at DESC"
        );
        
        let results = stmt.bind(&[])?.all().await?;
        
        let keys: Vec<ApiKey> = results.results::<ApiKey>()?;
        
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
        
        stmt.bind(&[
            id.clone().into(),
            req.title.clone().into(),
            req.description.clone().into(),
            priority.into(),
            req.due_date.into(),
            now.into(),
            now.into(),
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
                stmt.bind(&[(completed as i32).into()])?
            },
            None => {
                let stmt = self.d1.prepare(
                    "SELECT * FROM todos ORDER BY priority DESC, created_at DESC"
                );
                stmt.bind(&[])?
            }
        };
        
        let results = query.all().await?;
        let todos: Vec<Todo> = results.results::<Todo>()?;
        
        Ok(todos)
    }

    pub async fn get_todo(&self, id: &str) -> Result<Option<Todo>> {
        let stmt = self.d1.prepare("SELECT * FROM todos WHERE id = ?1");
        stmt.bind(&[id.into()])?.first::<Todo>(None).await
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
                todo.description.clone().into(),
                (todo.completed as i32).into(),
                todo.priority.into(),
                todo.due_date.into(),
                todo.updated_at.into(),
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
                (todo.completed as i32).into(),
                todo.updated_at.into(),
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
        
        let search_pattern = format!("%{}%", query);
        let results = stmt.bind(&[search_pattern.into()])?.all().await?;
        
        let todos: Vec<Todo> = results.results::<Todo>()?;
        
        Ok(todos)
    }
}