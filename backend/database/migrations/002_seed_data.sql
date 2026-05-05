-- =============================================================================
-- Migration 002: Seed Data (Development Only)
-- =============================================================================
-- Creates test data for development environment
-- DO NOT RUN IN PRODUCTION
-- =============================================================================

BEGIN;

-- Vérifier que la migration précédente a été appliquée
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM _migrations WHERE name = '001_initial_schema') THEN
        RAISE EXCEPTION 'Migration 001_initial_schema must be applied first';
    END IF;
END $$;

-- =============================================================================
-- TEST USERS
-- =============================================================================
-- Password for all test users: "password123"
-- Hash generated with: argon2 (ou bcrypt selon l'implémentation backend)

INSERT INTO users (id, email, username, password_hash) VALUES
    ('11111111-1111-1111-1111-111111111111', 'alice@example.com', 'Alice', '$argon2id$v=19$m=65536,t=3,p=4$c29tZXNhbHQ$hash1'),
    ('22222222-2222-2222-2222-222222222222', 'bob@example.com', 'Bob', '$argon2id$v=19$m=65536,t=3,p=4$c29tZXNhbHQ$hash2'),
    ('33333333-3333-3333-3333-333333333333', 'charlie@example.com', 'Charlie', '$argon2id$v=19$m=65536,t=3,p=4$c29tZXNhbHQ$hash3'),
    ('44444444-4444-4444-4444-444444444444', 'diana@example.com', 'Diana', '$argon2id$v=19$m=65536,t=3,p=4$c29tZXNhbHQ$hash4');

-- =============================================================================
-- TEST SERVERS
-- =============================================================================

INSERT INTO servers (id, name, description, owner_id) VALUES
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'General Chat', 'A general discussion server', '11111111-1111-1111-1111-111111111111'),
    ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Gaming Hub', 'For gamers only!', '22222222-2222-2222-2222-222222222222'),
    ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'Dev Team', 'Development discussions', '11111111-1111-1111-1111-111111111111');

-- =============================================================================
-- TEST MEMBERSHIPS
-- =============================================================================

-- Server: General Chat (owner: Alice)
INSERT INTO memberships (server_id, user_id, role) VALUES
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '11111111-1111-1111-1111-111111111111', 'OWNER'),
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '22222222-2222-2222-2222-222222222222', 'ADMIN'),
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '33333333-3333-3333-3333-333333333333', 'MEMBER'),
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '44444444-4444-4444-4444-444444444444', 'MEMBER');

-- Server: Gaming Hub (owner: Bob)
INSERT INTO memberships (server_id, user_id, role) VALUES
    ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '22222222-2222-2222-2222-222222222222', 'OWNER'),
    ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '11111111-1111-1111-1111-111111111111', 'MEMBER'),
    ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '33333333-3333-3333-3333-333333333333', 'ADMIN');

-- Server: Dev Team (owner: Alice)
INSERT INTO memberships (server_id, user_id, role) VALUES
    ('cccccccc-cccc-cccc-cccc-cccccccccccc', '11111111-1111-1111-1111-111111111111', 'OWNER'),
    ('cccccccc-cccc-cccc-cccc-cccccccccccc', '22222222-2222-2222-2222-222222222222', 'ADMIN');

-- =============================================================================
-- TEST CHANNELS
-- =============================================================================

-- Channels for General Chat
INSERT INTO channels (id, server_id, name, description, position) VALUES
    ('11111111-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'general', 'General discussion', 0),
    ('22222222-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'random', 'Random stuff', 1),
    ('33333333-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'announcements', 'Important announcements', 2);

-- Channels for Gaming Hub
INSERT INTO channels (id, server_id, name, description, position) VALUES
    ('11111111-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'general', 'General gaming chat', 0),
    ('22222222-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'fps-games', 'FPS discussion', 1),
    ('33333333-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'mmorpg', 'MMORPG discussion', 2);

-- Channels for Dev Team
INSERT INTO channels (id, server_id, name, description, position) VALUES
    ('11111111-cccc-cccc-cccc-cccccccccccc', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'general', 'General dev chat', 0),
    ('22222222-cccc-cccc-cccc-cccccccccccc', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'backend', 'Backend discussions', 1),
    ('33333333-cccc-cccc-cccc-cccccccccccc', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'frontend', 'Frontend discussions', 2),
    ('44444444-cccc-cccc-cccc-cccccccccccc', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'devops', 'DevOps and CI/CD', 3);

-- =============================================================================
-- TEST INVITES
-- =============================================================================

INSERT INTO invites (code, server_id, created_by, expires_at, max_uses) VALUES
    ('ABC123', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '11111111-1111-1111-1111-111111111111', NOW() + INTERVAL '7 days', 10),
    ('GAMING', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '22222222-2222-2222-2222-222222222222', NULL, NULL),
    ('DEVJOIN', 'cccccccc-cccc-cccc-cccc-cccccccccccc', '11111111-1111-1111-1111-111111111111', NOW() + INTERVAL '1 day', 5);

-- =============================================================================
-- MIGRATION RECORD
-- =============================================================================

INSERT INTO _migrations (name) VALUES ('002_seed_data');

COMMIT;
