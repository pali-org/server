#!/bin/bash
# List all API keys from D1 database (local or remote)
# Usage: ./scripts/list-api-keys.sh [local|remote]

MODE=${1:-"local"}

if [ "$MODE" = "local" ]; then
    echo "Listing all API keys from local D1 database..."
    WRANGLER_FLAGS="--local"
elif [ "$MODE" = "remote" ]; then
    echo "Listing all API keys from remote D1 database..."
    WRANGLER_FLAGS=""
else
    echo "❌ Invalid mode. Use 'local' or 'remote'"
    echo "Usage: $0 [local|remote]"
    exit 1
fi

wrangler d1 execute pali-database $WRANGLER_FLAGS --command="
SELECT 
  client_name,
  key_type,
  SUBSTR(key_hash, 1, 8) || '...' as key_preview,
  datetime(created_at, 'unixepoch') as created_at,
  datetime(last_used, 'unixepoch') as last_used,
  CASE WHEN active = 1 THEN 'ACTIVE' ELSE 'INACTIVE' END as status
FROM api_keys 
ORDER BY key_type DESC, created_at DESC;
"