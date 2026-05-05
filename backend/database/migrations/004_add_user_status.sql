-- Migration 004: Add user presence status
BEGIN;

-- Add a simple text column for status with default 'offline'
ALTER TABLE users
  ADD COLUMN status TEXT NOT NULL DEFAULT 'offline';

-- Optional index for quick lookup by status
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);

INSERT INTO _migrations (name) VALUES ('004_add_user_status');
COMMIT;
