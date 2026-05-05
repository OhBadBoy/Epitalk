-- =============================================================================
-- Migration 001: Initial Schema
-- =============================================================================
-- Creates the base tables for EpiTalk application
-- Run: psql -d EpiTalk -f 001_initial_schema.sql
-- =============================================================================

BEGIN;

-- Extension pour UUID et CITEXT
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "citext";

-- =============================================================================
-- ENUM TYPES
-- =============================================================================

CREATE TYPE member_role AS ENUM ('OWNER', 'ADMIN', 'MEMBER');
CREATE TYPE channel_kind AS ENUM ('TEXT');

-- =============================================================================
-- TABLE: users
-- =============================================================================

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email CITEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    username TEXT NOT NULL,
    display_name VARCHAR(100),
    avatar_url TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);

-- =============================================================================
-- TABLE: servers
-- =============================================================================

CREATE TABLE servers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    description TEXT,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT chk_server_name_not_empty CHECK (LENGTH(TRIM(name)) > 0)
);

CREATE INDEX idx_servers_owner_id ON servers(owner_id);

-- =============================================================================
-- TABLE: memberships
-- =============================================================================

CREATE TABLE memberships (
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role member_role NOT NULL DEFAULT 'MEMBER',
    joined_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (server_id, user_id)
);

CREATE INDEX idx_memberships_user_id ON memberships(user_id);
CREATE INDEX idx_memberships_server_id ON memberships(server_id);
CREATE INDEX idx_memberships_role ON memberships(server_id, role);

-- =============================================================================
-- TABLE: channels
-- =============================================================================

CREATE TABLE channels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    kind channel_kind NOT NULL DEFAULT 'TEXT',
    position INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT chk_channel_name_not_empty CHECK (LENGTH(TRIM(name)) > 0),
    CONSTRAINT uq_channel_name_per_server UNIQUE (server_id, name)
);

CREATE INDEX idx_channels_server_id ON channels(server_id, position);

-- =============================================================================
-- TABLE: invites
-- =============================================================================

CREATE TABLE invites (
    code TEXT PRIMARY KEY,
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMP WITH TIME ZONE,
    max_uses INTEGER,
    uses INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT chk_invite_code_length CHECK (LENGTH(code) >= 6),
    CONSTRAINT chk_invite_max_uses_positive CHECK (max_uses IS NULL OR max_uses > 0),
    CONSTRAINT chk_invite_uses_limit CHECK (max_uses IS NULL OR uses <= max_uses)
);

CREATE INDEX idx_invites_server_id ON invites(server_id);
CREATE INDEX idx_invites_created_by ON invites(created_by);
CREATE INDEX idx_invites_expires_at ON invites(expires_at) WHERE expires_at IS NOT NULL;

-- =============================================================================
-- FUNCTIONS & TRIGGERS
-- =============================================================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_servers_updated_at
    BEFORE UPDATE ON servers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_channels_updated_at
    BEFORE UPDATE ON channels
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- =============================================================================
-- MIGRATION RECORD
-- =============================================================================

CREATE TABLE IF NOT EXISTS _migrations (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    applied_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

INSERT INTO _migrations (name) VALUES ('001_initial_schema');

COMMIT;
