-- =============================================================================
-- Migration: Add MODERATOR role to member_role ENUM
-- =============================================================================
-- This migration adds the MODERATOR role between ADMIN and MEMBER
-- Hierarchy: OWNER > ADMIN > MODERATOR > MEMBER
-- =============================================================================

-- Add MODERATOR value to the existing enum
-- PostgreSQL allows adding values to enums with ALTER TYPE
ALTER TYPE member_role ADD VALUE IF NOT EXISTS 'MODERATOR' AFTER 'ADMIN';

-- Note: The IF NOT EXISTS clause prevents errors if the value already exists
-- The AFTER clause ensures proper ordering in the enum

-- =============================================================================
-- Permissions for MODERATOR role:
-- - Can create invites (like Admin)
-- - Can delete others' messages (like Admin)
-- - Cannot manage channels (Admin+ only)
-- - Cannot manage members (Owner only)
-- =============================================================================
