#!/bin/bash
# Debug server latency issues
# Usage: ./scripts/debug-latency.sh [ADMIN_KEY] [BASE_URL]

ADMIN_KEY=${1}
BASE_URL=${2:-"http://localhost:8787"}

if [ -z "$ADMIN_KEY" ]; then
    echo "❌ Admin key required"
    echo "Usage: $0 [ADMIN_KEY] [BASE_URL]"
    exit 1
fi

echo "🔍 Debugging Pali Server Latency"
echo "Base URL: $BASE_URL"
echo ""

# Test simple endpoint (no DB)
echo "1. Testing /health (no database)..."
time curl -s "$BASE_URL/health" > /dev/null
echo ""

# Test database read
echo "2. Testing /todos (database read)..."
time curl -s -H "X-API-Key: $ADMIN_KEY" "$BASE_URL/todos" > /dev/null
echo ""

# Test database write
echo "3. Testing todo creation (database write)..."
time curl -s -X POST "$BASE_URL/todos" \
  -H "X-API-Key: $ADMIN_KEY" \
  -H "Content-Type: application/json" \
  -d '{"title": "Latency Test", "description": "Testing response times"}' > /dev/null
echo ""

# Multiple rapid requests to test cold starts
echo "4. Testing cold start behavior (5 rapid requests)..."
for i in {1..5}; do
    echo "Request $i:"
    time curl -s "$BASE_URL/health" > /dev/null
    sleep 1
done
echo ""

# Test with detailed timing
echo "5. Detailed timing breakdown:"
curl -w "Total: %{time_total}s | DNS: %{time_namelookup}s | Connect: %{time_connect}s | SSL: %{time_appconnect}s | Transfer: %{time_starttransfer}s\n" \
  -s -o /dev/null \
  -H "X-API-Key: $ADMIN_KEY" \
  "$BASE_URL/todos"