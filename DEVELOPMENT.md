# Development Guide for Pali Server

## Local Development Setup

### Prerequisites
- Rust 1.70+
- `wrangler` CLI installed
- D1 database configured

### First Time Setup

1. **Apply Database Migrations**
   ```bash
   # For local development
   wrangler d1 migrations apply pali-database --local
   
   # For remote development
   wrangler d1 migrations apply pali-database
   ```

2. **Start Development Server**
   ```bash
   # Local development (uses local D1)
   wrangler dev --local
   
   # Remote development (uses remote D1)
   wrangler dev
   ```

3. **Initialize Server (Get Admin Key)**
   ```bash
   # After server is running, initialize to get admin key
   curl -X POST http://localhost:8787/initialize
   
   # Save the returned admin key for testing!
   ```

### Development Workflow

#### Resetting Local Database
If you need to reset your local development database:

```bash
# Stop wrangler dev (Ctrl+C)

# Reset local database (removes all data)
wrangler d1 execute pali-database --local --command="DROP TABLE IF EXISTS todos; DROP TABLE IF EXISTS api_keys;"

# Reapply migrations
wrangler d1 migrations apply pali-database --local

# Start development server
wrangler dev --local

# Reinitialize (get new admin key)
curl -X POST http://localhost:8787/initialize
```

#### Quick Reset Script
Create `reset-dev.sh`:
```bash
#!/bin/bash
echo "Resetting local development database..."
wrangler d1 execute pali-database --local --command="DROP TABLE IF EXISTS todos; DROP TABLE IF EXISTS api_keys;"
wrangler d1 migrations apply pali-database --local
echo "Database reset! Start 'wrangler dev --local' and call POST /initialize"
```

### Testing Endpoints

With your admin key from initialization:

```bash
# Set your admin key
ADMIN_KEY="your-admin-key-from-initialize"

# Test todo creation
curl -X POST http://localhost:8787/todos \
  -H "X-API-Key: $ADMIN_KEY" \
  -H "Content-Type: application/json" \
  -d '{"title": "Test todo", "description": "Testing local dev"}'

# List todos
curl -H "X-API-Key: $ADMIN_KEY" http://localhost:8787/todos

# Create client API key
curl -X POST http://localhost:8787/admin/keys/generate \
  -H "X-API-Key: $ADMIN_KEY" \
  -H "Content-Type: application/json" \
  -d '{"client_name": "Dev Client", "key_type": "client"}'
```

### Code Quality

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo test

# Clippy check
cargo clippy

# Clippy with pedantic mode
cargo clippy -- -W clippy::pedantic
```

## Production Deployment

### Deploy to Cloudflare Workers
```bash
# Deploy to production
wrangler deploy

# Initialize production server
curl -X POST https://your-worker.workers.dev/initialize

# Save the admin key securely!
```

## Current Implementation Status

✅ **Completed:**
- Complete authentication system with API key validation
- All CRUD endpoints for todos and API key management  
- Secure initialization system (no hardcoded keys)
- Emergency admin key reset functionality
- Database migrations and D1 integration
- Security audit logging
- Production-ready error handling

✅ **Code Quality:**
- Zero compilation errors
- Minimal Clippy warnings (only D1-related precision loss)
- Modern Rust patterns and idioms
- Comprehensive documentation

🚀 **Ready for:**
- Production deployment
- Integration testing with CLI client
- Public preview release