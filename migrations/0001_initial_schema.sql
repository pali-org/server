-- Migration: Initial schema for Pali todo server
-- Created: 2025-08-27

-- Create api_keys table
CREATE TABLE api_keys (
    id TEXT PRIMARY KEY,
    key_hash TEXT NOT NULL UNIQUE,
    client_name TEXT NOT NULL,
    key_type TEXT NOT NULL CHECK(key_type IN ('admin', 'client')),
    last_used INTEGER,
    created_at INTEGER NOT NULL,
    active INTEGER NOT NULL DEFAULT 1
);

-- Create todos table
CREATE TABLE todos (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    completed INTEGER NOT NULL DEFAULT 0,
    priority INTEGER DEFAULT 2 CHECK(priority BETWEEN 1 AND 5),
    due_date INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Create indexes for better query performance
CREATE INDEX idx_todos_completed ON todos(completed);
CREATE INDEX idx_todos_priority ON todos(priority);
CREATE INDEX idx_todos_due_date ON todos(due_date);
CREATE INDEX idx_api_keys_active ON api_keys(active);
CREATE INDEX idx_api_keys_key_hash ON api_keys(key_hash);