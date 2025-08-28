#!/bin/bash
# Reset local development database completely
# Usage: ./scripts/reset-dev-db.sh
# WARNING: This will delete ALL data in your local development database!

echo "🚨 WARNING: This will completely reset your local development database!"
echo "All todos and API keys will be permanently deleted."
echo ""
read -p "Are you sure you want to continue? (y/N): " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Reset cancelled."
    exit 0
fi

echo ""
echo "Resetting local development database..."

# Drop existing tables and migration history
echo "1. Dropping existing tables and migration state..."
wrangler d1 execute pali-database --local --command="
DROP TABLE IF EXISTS todos; 
DROP TABLE IF EXISTS api_keys;
DROP TABLE IF EXISTS d1_migrations;
"

if [ $? -ne 0 ]; then
    echo "❌ Failed to drop tables"
    exit 1
fi

# Reapply migrations (will now work since migration history is reset)
echo "2. Applying migrations..."
wrangler d1 migrations apply pali-database --local

if [ $? -ne 0 ]; then
    echo "❌ Failed to apply migrations"
    exit 1
fi

echo ""
echo "✅ Database reset complete!"
echo ""
echo "Next steps:"
echo "1. Start development server: wrangler dev --local"
echo "2. Initialize server: ./scripts/initialize.sh local"