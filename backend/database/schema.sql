-- =============================================================================
-- EpiTalk (Real Time Chat) - PostgreSQL Schema
-- =============================================================================
-- Tables: users, servers, memberships, channels, invites
-- Roles: OWNER, ADMIN, MODERATOR, MEMBER
-- =============================================================================

-- Extension pour UUID et CITEXT
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "citext";

-- =============================================================================
-- ENUM TYPES
-- =============================================================================

-- Rôles des membres (OWNER > ADMIN > MODERATOR > MEMBER)
CREATE TYPE member_role AS ENUM ('OWNER', 'ADMIN', 'MODERATOR', 'MEMBER');

-- Types de channels (extensible pour VOICE plus tard)
CREATE TYPE channel_kind AS ENUM ('TEXT');

-- Statut utilisateur
CREATE TYPE user_status AS ENUM ('ONLINE', 'IDLE', 'DND', 'OFFLINE');

-- =============================================================================
-- TABLE: users
-- =============================================================================
-- Stocke les informations d'authentification et de profil des utilisateurs

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email CITEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    username TEXT NOT NULL,
    avatar_url TEXT,
    bio TEXT DEFAULT '',
    banner_color_1 TEXT DEFAULT '#023BFC',
    banner_color_2 TEXT DEFAULT '#3D6AFF',
    status user_status NOT NULL DEFAULT 'ONLINE',
    theme TEXT NOT NULL DEFAULT 'light',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Index pour recherche par email (login)
CREATE INDEX idx_users_email ON users(email);

-- =============================================================================
-- TABLE: servers
-- =============================================================================
-- Représente un serveur/communauté de chat

CREATE TABLE servers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    description TEXT,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Contrainte: nom non vide
    CONSTRAINT chk_server_name_not_empty CHECK (LENGTH(TRIM(name)) > 0)
);

-- Index pour lister les serveurs d'un owner
CREATE INDEX idx_servers_owner_id ON servers(owner_id);

-- =============================================================================
-- TABLE: memberships
-- =============================================================================
-- Table de jonction users <-> servers avec rôle

CREATE TABLE memberships (
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role member_role NOT NULL DEFAULT 'MEMBER',
    joined_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Clé primaire composite
    PRIMARY KEY (server_id, user_id)
);

-- Index pour lister les serveurs d'un utilisateur
CREATE INDEX idx_memberships_user_id ON memberships(user_id);

-- Index pour lister les membres d'un serveur
CREATE INDEX idx_memberships_server_id ON memberships(server_id);

-- Index pour filtrer par rôle (ex: trouver tous les admins)
CREATE INDEX idx_memberships_role ON memberships(server_id, role);

-- =============================================================================
-- TABLE: channels
-- =============================================================================
-- Channels de discussion au sein d'un serveur

CREATE TABLE channels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    kind channel_kind NOT NULL DEFAULT 'TEXT',
    position INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Contrainte: nom non vide
    CONSTRAINT chk_channel_name_not_empty CHECK (LENGTH(TRIM(name)) > 0),
    
    -- Contrainte: nom unique par serveur
    CONSTRAINT uq_channel_name_per_server UNIQUE (server_id, name)
);

-- Index pour lister les channels d'un serveur (triés par position)
CREATE INDEX idx_channels_server_id ON channels(server_id, position);

-- =============================================================================
-- TABLE: invites
-- =============================================================================
-- Codes d'invitation pour rejoindre un serveur

CREATE TABLE invites (
    code TEXT PRIMARY KEY,
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMP WITH TIME ZONE,
    max_uses INTEGER,
    uses INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Contrainte: code non vide (min 6 caractères)
    CONSTRAINT chk_invite_code_length CHECK (LENGTH(code) >= 6),
    
    -- Contrainte: max_uses positif si défini
    CONSTRAINT chk_invite_max_uses_positive CHECK (max_uses IS NULL OR max_uses > 0),
    
    -- Contrainte: uses ne dépasse pas max_uses
    CONSTRAINT chk_invite_uses_limit CHECK (max_uses IS NULL OR uses <= max_uses)
);

-- Index pour rechercher les invites d'un serveur
CREATE INDEX idx_invites_server_id ON invites(server_id);

-- Index pour rechercher les invites créées par un utilisateur
CREATE INDEX idx_invites_created_by ON invites(created_by);

-- Index pour filtrer les invites non expirées
CREATE INDEX idx_invites_expires_at ON invites(expires_at) WHERE expires_at IS NOT NULL;

-- =============================================================================
-- FUNCTIONS
-- =============================================================================

-- Fonction pour mettre à jour updated_at automatiquement
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- TRIGGERS
-- =============================================================================

-- Auto-update updated_at pour users
CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Auto-update updated_at pour servers
CREATE TRIGGER trg_servers_updated_at
    BEFORE UPDATE ON servers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Auto-update updated_at pour channels
CREATE TRIGGER trg_channels_updated_at
    BEFORE UPDATE ON channels
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- =============================================================================
-- COMMENTS (Documentation)
-- =============================================================================

COMMENT ON TABLE users IS 'Utilisateurs de l''application EpiTalk';
COMMENT ON TABLE servers IS 'Serveurs/communautés de chat';
COMMENT ON TABLE memberships IS 'Appartenance des utilisateurs aux serveurs avec rôle';
COMMENT ON TABLE channels IS 'Channels de discussion au sein des serveurs';
COMMENT ON TABLE invites IS 'Codes d''invitation pour rejoindre un serveur';

COMMENT ON COLUMN users.email IS 'Email unique (case-insensitive via CITEXT)';
COMMENT ON COLUMN users.password_hash IS 'Hash Argon2/bcrypt du mot de passe';
COMMENT ON COLUMN memberships.role IS 'OWNER: propriétaire unique, ADMIN: modérateur, MEMBER: membre standard';
COMMENT ON COLUMN channels.kind IS 'Type de channel (TEXT pour l''instant, VOICE prévu)';
COMMENT ON COLUMN channels.position IS 'Position pour le tri dans la sidebar';
COMMENT ON COLUMN invites.code IS 'Code d''invitation unique (min 6 caractères)';
COMMENT ON COLUMN invites.max_uses IS 'Nombre max d''utilisations (NULL = illimité)';
