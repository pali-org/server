# Pali Server

A self-hosted todo application backend built with Rust, Cloudflare Workers, and D1 database.

**Pali** (from Toki Pona: "to do, work, make") - Simple, secure, self-hosted task management.

## Features

- API key-based authentication (no user accounts needed)
- Admin and client API keys with different permission levels
- Full CRUD operations for todos
- Todo search functionality
- API key rotation for security
- Priority levels and due dates for todos

## Setup

### 1. Create D1 Database

```bash
# Create the database
wrangler d1 create pali-database

# Copy the database_id from the output and update it in wrangler.toml under [[d1_databases]]
```

### 2. Run Database Migrations

```bash
# Apply migrations to local database (for development)
wrangler d1 migrations apply pali-database --local

# Apply migrations to remote database (for production)
wrangler d1 migrations apply pali-database
```

### 3. Configure Initial Admin Key

Edit `wrangler.toml` and set a secure initial admin key:

```toml
[vars]
INITIAL_ADMIN_KEY = "your-secure-admin-key-here"
```

### 4. Deploy

```bash
# Deploy to Cloudflare Workers
wrangler deploy
```

## API Documentation

### Authentication

All API endpoints (except `/` and `/health`) require authentication via the `X-API-Key` header.

### Admin Endpoints

#### Rotate Admin Key
```
POST /admin/keys/rotate
X-API-Key: <current-admin-key>

Response:
{
  "success": true,
  "data": {
    "id": "...",
    "client_name": "Rotated Admin Key",
    "key_type": "admin",
    "api_key": "pali_...",  // Save this! Only shown once
    "created_at": 1234567890
  }
}
```

#### Create API Key
```
POST /admin/keys/generate
X-API-Key: <admin-key>
Content-Type: application/json

{
  "client_name": "Mobile App",
  "key_type": "client"  // or "admin"
}

Response:
{
  "success": true,
  "data": {
    "id": "...",
    "client_name": "Mobile App",
    "key_type": "client",
    "api_key": "pali_...",  // Save this! Only shown once
    "created_at": 1234567890
  }
}
```

#### List API Keys
```
GET /admin/keys
X-API-Key: <admin-key>

Response:
{
  "success": true,
  "data": [
    {
      "id": "...",
      "client_name": "Mobile App",
      "key_type": "client",
      "last_used": 1234567890,
      "created_at": 1234567890,
      "active": true
    }
  ]
}
```

#### Revoke API Key
```
DELETE /admin/keys/:id
X-API-Key: <admin-key>
```

### Todo Endpoints

#### Create Todo
```
POST /todos
X-API-Key: <any-valid-key>
Content-Type: application/json

{
  "title": "Buy groceries",
  "description": "Milk, eggs, bread",
  "priority": 3,  // 1-5, optional, default 2
  "due_date": 1234567890  // Unix timestamp, optional
}
```

#### List Todos
```
GET /todos?completed=false
X-API-Key: <any-valid-key>

Query params:
- completed: true/false (optional, shows all if not specified)
```

#### Get Single Todo
```
GET /todos/:id
X-API-Key: <any-valid-key>
```

#### Update Todo
```
PUT /todos/:id
X-API-Key: <any-valid-key>
Content-Type: application/json

{
  "title": "Updated title",
  "description": "Updated description",
  "completed": true,
  "priority": 4,
  "due_date": 1234567890
}
```

#### Toggle Todo Completion
```
PATCH /todos/:id/toggle
X-API-Key: <any-valid-key>
```

#### Delete Todo
```
DELETE /todos/:id
X-API-Key: <any-valid-key>
```

#### Search Todos
```
GET /todos/search?q=groceries
X-API-Key: <any-valid-key>
```

## Client Integration

Each client application should:

1. Store its API key securely (not hard-coded)
2. Include the API key in the `X-API-Key` header for all requests
3. Handle API responses checking the `success` field
4. Implement proper error handling for network failures

## Security Notes

- The initial admin key should be removed from `wrangler.toml` after first use
- Use the key rotation API regularly for admin keys
- Each client should have its own API key
- Revoke unused or compromised keys immediately
- All API keys are hashed before storage (SHA-256)

## Development

```bash
# Local development
wrangler dev --local

# Local development with remote D1 database
wrangler dev

# View logs
wrangler tail
```