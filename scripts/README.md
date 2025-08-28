# Pali Server Utility Scripts

This directory contains utility scripts for managing the Pali server during development and deployment.

## Prerequisites

- `wrangler` CLI installed and configured
- `jq` installed (for JSON parsing in test scripts)
- `curl` installed (for API testing)

## Scripts Overview

### Database Management

- **`reset-dev-db.sh`** - Completely reset local development database
  - ⚠️ **WARNING**: Deletes ALL data
  - Usage: `./scripts/reset-dev-db.sh`

### Server Initialization

- **`initialize.sh`** - Initialize server (local or remote)
  - Usage: `./scripts/initialize.sh [local|remote] [WORKER_URL]`
  - Examples:
    - `./scripts/initialize.sh local`
    - `./scripts/initialize.sh remote https://pali-server.your-subdomain.workers.dev`

### API Key Management

- **`list-api-keys.sh`** - List all API keys in database
  - Shows key hash previews only (for security)
  - Usage: `./scripts/list-api-keys.sh [local|remote]`

### Testing

- **`test-endpoints.sh`** - Comprehensive endpoint testing
  - Tests all CRUD operations and authentication
  - Usage: `./scripts/test-endpoints.sh [ADMIN_KEY] [BASE_URL]`
  - Example: `./scripts/test-endpoints.sh "your-admin-key" "http://localhost:8787"`

## Typical Development Workflow

### First Time Setup
```bash
# 1. Reset and setup local database
./scripts/reset-dev-db.sh

# 2. Start development server
wrangler dev --local

# 3. Initialize server (in another terminal)
./scripts/initialize.sh local
```

### Daily Development
```bash
# Start development server
wrangler dev --local

# Test endpoints with your admin key
./scripts/test-endpoints.sh "your-admin-key-here"

# Check API keys status
./scripts/list-api-keys.sh local
```

### Production Deployment
```bash
# Deploy to Cloudflare Workers
wrangler deploy

# Initialize production server
./scripts/initialize.sh remote https://your-worker.workers.dev

# Test production endpoints
./scripts/test-endpoints.sh "your-prod-admin-key" "https://your-worker.workers.dev"
```

## Security Notes

- Admin keys are only displayed once during initialization
- Database stores only hashed keys for security
- Scripts show only key hash previews for identification
- Always save admin keys securely (password manager, etc.)
- Use different admin keys for development and production