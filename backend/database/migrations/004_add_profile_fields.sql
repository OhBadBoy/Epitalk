-- =============================================================================
-- Migration 004: Add profile customization fields to users
-- =============================================================================
-- Adds: avatar_url, bio, banner_color_1, banner_color_2, status

-- User status enum
CREATE TYPE user_status AS ENUM ('ONLINE', 'IDLE', 'DND', 'OFFLINE');

ALTER TABLE users
    ADD COLUMN avatar_url TEXT,
    ADD COLUMN bio TEXT DEFAULT '',
    ADD COLUMN banner_color_1 TEXT DEFAULT '#023BFC',
    ADD COLUMN banner_color_2 TEXT DEFAULT '#3D6AFF',
    ADD COLUMN status user_status NOT NULL DEFAULT 'ONLINE';
