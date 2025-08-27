# Pali Server

Cloudflare Workers backend for the Pali todo management ecosystem.

## Features

- **Cloudflare Workers + D1**: Serverless API with global edge deployment
- **Secure Authentication**: API key-based auth with admin/client roles
- **One-Time Initialization**: Secure setup without hardcoded credentials
- **Emergency Admin Reset**: Never get locked out of your server
- **Full Todo API**: CRUD operations, search, priority levels, due dates
- **Security Audit Logging**: Track authentication attempts and admin actions

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

### 3. Deploy

```bash
# Deploy to Cloudflare Workers
wrangler deploy
```

### 4. Initialize Server

After deployment, initialize the server to create your first admin key:

```bash
curl -X POST https://your-worker-url.workers.dev/initialize
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "...",
    "client_name": "Initial Admin Key",
    "key_type": "admin", 
    "api_key": "pali_abc123...",  // SAVE THIS! Only shown once
    "created_at": 1234567890
  }
}
```

⚠️ **Important**: Save the returned API key - it's only shown once!

Subsequent calls return `409 Conflict` (server already initialized).

## API Documentation

### Authentication

All API endpoints (except `/`, `/health`, and `/initialize`) require authentication via the `X-API-Key` header.

### Initialization Endpoints

#### Initialize Server (One-Time Setup)
```
POST /initialize

No authentication required (only works if server is uninitialized)

Response:
{
  "success": true,
  "data": {
    "id": "...",
    "client_name": "Initial Admin Key",
    "key_type": "admin",
    "api_key": "pali_...",  // Save this! Only shown once
    "created_at": 1234567890
  }
}
```

#### Emergency Reinitialize (Admin Key Reset)
```
POST /reinitialize

No authentication required (emergency reset - deactivates ALL admin keys)

Response:
{
  "success": true,
  "data": {
    "id": "...",
    "client_name": "Reinitialized Admin Key", 
    "key_type": "admin",
    "api_key": "pali_...",  // Save this! Only shown once
    "created_at": 1234567890
  }
}
```

⚠️ **Emergency Use Only**: `/reinitialize` deactivates ALL existing admin keys and creates a new one. Use only when admin access is lost.

### Admin Endpoints

#### Rotate Admin Key
```
POST /admin/keys/rotate
X-API-Key: <current-admin-key>

Status: 410 Gone (Deprecated)
Response:
{
  "success": false, 
  "error": "Use POST /reinitialize for admin key rotation"
}
```

**Note**: This endpoint is deprecated. Use `POST /reinitialize` for admin key rotation.

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

## Client Development

### API Integration Requirements

Client applications should:

1. **Authentication**: Include API key in `X-API-Key` header for all requests
2. **Response Handling**: Check `success` field in JSON responses  
3. **Error Handling**: Handle HTTP status codes (401, 403, 404, 500)
4. **Security**: Store API keys securely (never hard-coded)

### Response Format

All endpoints return JSON in this format:
```json
{
  "success": boolean,
  "data": T | null,
  "error": string | null
}
```

## Security Features

### Authentication System
- **Production-Ready**: All endpoints require valid API keys (except health checks)
- **Role-Based Access**: Admin keys can manage API keys, client keys can only access todos
- **Secure Storage**: API keys are SHA-256 hashed before database storage
- **Request Logging**: All authentication attempts logged for security auditing

### Key Management Best Practices
- **No Hardcoded Keys**: Server initializes securely without hardcoded credentials
- **One-Time Setup**: Initialize endpoint only works once, prevents replay attacks
- **Emergency Reset**: Reinitialize endpoint for complete admin access recovery
- **Individual Keys**: Each client/app should have its own unique API key
- **Immediate Revocation**: Compromised keys can be revoked instantly via admin API

### HTTP Security
- **Status Codes**: Proper 401 (Unauthorized) and 403 (Forbidden) responses
- **Error Messages**: Descriptive but secure error responses
- **Audit Trail**: Security events logged for monitoring and compliance

## Development

```bash
# Local development
wrangler dev --local

# Local development with remote D1 database
wrangler dev

# View logs
wrangler tail
```