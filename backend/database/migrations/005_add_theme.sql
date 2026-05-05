-- Migration 005: Add theme column to users table
-- Stores the user's preferred theme (light, ash, dim, dark)

ALTER TABLE users ADD COLUMN theme TEXT NOT NULL DEFAULT 'light';
