-- Migration 004: Ajout du système de bannissement
-- Supports: ban permanent + ban temporaire (expires_at NULL = permanent)

CREATE TABLE bans (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    server_id   UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    banned_by   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reason      TEXT,
    expires_at  TIMESTAMP WITH TIME ZONE,  -- NULL = ban permanent
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_ban_per_server_user UNIQUE (server_id, user_id)
);

CREATE INDEX idx_bans_server_id ON bans(server_id);
CREATE INDEX idx_bans_user_id   ON bans(user_id);
-- Pour filtrer les bans expirés efficacement
CREATE INDEX idx_bans_expires_at ON bans(expires_at) WHERE expires_at IS NOT NULL;