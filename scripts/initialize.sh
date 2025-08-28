#!/bin/bash
# Initialize server and capture admin key (local or remote)
# Usage: ./scripts/initialize.sh [local|remote] [WORKER_URL]
# Examples: 
#   ./scripts/initialize.sh local
#   ./scripts/initialize.sh remote https://pali-server.your-subdomain.workers.dev

MODE=${1:-"local"}
WORKER_URL=${2}

if [ "$MODE" = "local" ]; then
    echo "Initializing local server..."
    echo "Make sure your local server is running with: wrangler dev --local"
    BASE_URL="http://localhost:8787"
elif [ "$MODE" = "remote" ]; then
    if [ -z "$WORKER_URL" ]; then
        echo "❌ Worker URL required for remote initialization"
        echo "Usage: $0 remote [WORKER_URL]"
        echo "Example: $0 remote https://pali-server.your-subdomain.workers.dev"
        exit 1
    fi
    echo "Initializing remote server at: $WORKER_URL"
    BASE_URL="$WORKER_URL"
else
    echo "❌ Invalid mode. Use 'local' or 'remote'"
    echo "Usage: $0 [local|remote] [WORKER_URL]"
    exit 1
fi

echo ""

response=$(curl -s -X POST "$BASE_URL/initialize" 2>/dev/null)

if [ $? -eq 0 ] && [ -n "$response" ]; then
    echo "✅ Server initialized successfully!"
    echo ""
    echo "Response:"
    echo "$response"
    echo ""
    echo "🔐 IMPORTANT: Save your admin key securely!"
    echo "This key will not be displayed again."
else
    echo "❌ Failed to initialize server."
    if [ "$MODE" = "local" ]; then
        echo "Make sure:"
        echo "1. Local server is running: wrangler dev --local"
        echo "2. Server is not already initialized"
        echo "3. Database migrations are applied: wrangler d1 migrations apply pali-database --local"
    else
        echo "Make sure:"
        echo "1. Server is deployed: wrangler deploy"
        echo "2. Server is not already initialized"
        echo "3. Database migrations are applied: wrangler d1 migrations apply pali-database"
        echo "4. Correct worker URL provided"
    fi
fi