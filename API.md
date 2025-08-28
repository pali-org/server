# Pali Server API Documentation

## Authentication

All requests (except initialization) require the `X-API-Key` header:

```bash
curl -H "X-API-Key: your-api-key" https://your-worker.workers.dev/todos
```

## Response Format

All responses follow this structure:

```json
{
  "success": true,
  "data": { ... }        // Present on success
}

{
  "success": false,
  "error": "Error message"  // Present on failure
}
```

## Initialization Endpoints

### Initialize Server (One-Time Setup)
```
POST /initialize

# Creates the first admin API key
# Only available when no admin keys exist
# Returns: { "success": true, "data": { "admin_key": "..." } }
```

**Security**: This endpoint becomes unavailable after the first admin key is created, preventing unauthorized access.

### Emergency Reinitialize (Admin Key Reset)
```
POST /reinitialize

# Emergency endpoint to reset ALL admin keys and create a new one
# Deactivates ALL existing admin keys (emergency use only)
# Returns: { "success": true, "data": { "admin_key": "..." } }
```

**⚠️ Warning**: This endpoint deactivates ALL existing admin keys. Use only when locked out.

## Admin Endpoints

All admin endpoints require an admin-level API key.

### Rotate Admin Key
```
POST /admin/keys/rotate
X-API-Key: <admin-key>

# Creates a new admin key and deactivates the current one
# Returns: { "success": true, "data": { "admin_key": "..." } }
```

### Create API Key
```
POST /admin/keys
X-API-Key: <admin-key>
Content-Type: application/json

{
  "client_name": "My Todo App",
  "key_type": "Client"  // "Admin" or "Client"
}

# Returns: { "success": true, "data": { "id": "...", "key": "..." } }
```

### List API Keys
```
GET /admin/keys
X-API-Key: <admin-key>

# Returns: { "success": true, "data": [{ "id": "...", "client_name": "...", ... }] }
```

### Revoke API Key
```
DELETE /admin/keys/:id
X-API-Key: <admin-key>
```

## Todo Endpoints

### Create Todo
```
POST /todos
X-API-Key: <any-valid-key>
Content-Type: application/json

{
  "title": "Buy groceries",
  "description": "Milk, eggs, bread",
  "priority": 3,  // 1-5, optional, default 2
  "due_date": 1640995200  // Unix timestamp, optional
}
```

### List Todos
```
GET /todos
X-API-Key: <any-valid-key>

# Query parameters (all optional):
# ?priority=3           // Filter by priority (1-5)
# ?tag=work            // Filter by tag (not implemented)
```

### Get Single Todo
```
GET /todos/:id
X-API-Key: <any-valid-key>
```

### Update Todo
```
PUT /todos/:id
X-API-Key: <any-valid-key>
Content-Type: application/json

{
  "title": "Updated title",     // optional
  "description": "New desc",    // optional
  "completed": true,            // optional
  "priority": 4,                // optional
  "due_date": 1640995200        // optional
}
```

### Toggle Todo Completion
```
PATCH /todos/:id/toggle
X-API-Key: <any-valid-key>

# Toggles completed status: true ↔ false
```

### Delete Todo
```
DELETE /todos/:id
X-API-Key: <any-valid-key>
```

### Search Todos
```
GET /todos/search?q=groceries
X-API-Key: <any-valid-key>
```

### Resolve Todo ID Prefix
```
GET /todos/resolve/:prefix
X-API-Key: <any-valid-key>

# Returns: { "success": true, "data": { "full_id": "complete-uuid" } }
# Useful for CLI clients that want to use short IDs
```

## Client Integration

### Requirements

When building clients for Pali server:

1. **Authentication**: Include `X-API-Key` header in all requests (except `/initialize`)
2. **Content-Type**: Use `application/json` for POST/PUT requests  
3. **Error Handling**: Check `success` field in responses
4. **Base URL**: Replace `https://your-worker.workers.dev` with your actual Worker URL

### Security Features

- **API Key-based**: No passwords, just secure API keys
- **Role-based Access**: Admin keys can manage other keys, Client keys can only manage todos
- **Key Rotation**: Admin keys can be rotated without service interruption
- **Audit Trail**: Security events logged for monitoring and compliance