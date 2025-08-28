#!/bin/bash
# Test all server endpoints with provided admin key
# Usage: ./scripts/test-endpoints.sh [ADMIN_KEY] [BASE_URL]
# Example: ./scripts/test-endpoints.sh "your-admin-key-here" "http://localhost:8787"

ADMIN_KEY=${1}
BASE_URL=${2:-"http://localhost:8787"}

if [ -z "$ADMIN_KEY" ]; then
    echo "❌ Admin key required"
    echo "Usage: $0 [ADMIN_KEY] [BASE_URL]"
    echo "Example: $0 'your-admin-key-here' 'http://localhost:8787'"
    exit 1
fi

echo "🧪 Testing Pali Server Endpoints"
echo "Base URL: $BASE_URL"
echo "Admin Key: ${ADMIN_KEY:0:8}..."
echo ""

# Health check
echo "1. Testing health check..."
curl -s "$BASE_URL/health" | jq . || echo "Health check failed"
echo ""

# Create todo
echo "2. Creating test todo..."
TODO_RESPONSE=$(curl -s -X POST "$BASE_URL/todos" \
  -H "X-API-Key: $ADMIN_KEY" \
  -H "Content-Type: application/json" \
  -d '{"title": "Test Todo", "description": "Created by test script", "priority": 3}')

echo "$TODO_RESPONSE" | jq .
TODO_ID=$(echo "$TODO_RESPONSE" | jq -r '.id')
echo ""

# List todos
echo "3. Listing todos..."
curl -s -H "X-API-Key: $ADMIN_KEY" "$BASE_URL/todos" | jq .
echo ""

# Generate client key
echo "4. Generating client API key..."
CLIENT_RESPONSE=$(curl -s -X POST "$BASE_URL/admin/keys/generate" \
  -H "X-API-Key: $ADMIN_KEY" \
  -H "Content-Type: application/json" \
  -d '{"client_name": "Test Client", "key_type": "client"}')

echo "$CLIENT_RESPONSE" | jq .
CLIENT_KEY=$(echo "$CLIENT_RESPONSE" | jq -r '.api_key')
echo ""

# Test client permissions
echo "5. Testing client key permissions..."
curl -s -H "X-API-Key: $CLIENT_KEY" "$BASE_URL/todos" | jq .
echo ""

# Update todo
if [ "$TODO_ID" != "null" ] && [ -n "$TODO_ID" ]; then
    echo "6. Updating todo..."
    curl -s -X PUT "$BASE_URL/todos/$TODO_ID" \
      -H "X-API-Key: $ADMIN_KEY" \
      -H "Content-Type: application/json" \
      -d '{"title": "Updated Test Todo", "completed": true}' | jq .
    echo ""

    # Toggle todo
    echo "7. Toggling todo completion..."
    curl -s -X PATCH "$BASE_URL/todos/$TODO_ID/toggle" \
      -H "X-API-Key: $ADMIN_KEY" | jq .
    echo ""

    # Delete todo
    echo "8. Deleting todo..."
    curl -s -X DELETE "$BASE_URL/todos/$TODO_ID" \
      -H "X-API-Key: $ADMIN_KEY"
    echo ""
fi

echo "✅ Endpoint testing complete!"