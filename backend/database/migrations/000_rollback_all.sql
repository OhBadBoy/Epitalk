-- =============================================================================
-- Rollback: Undo all migrations (DESTRUCTIVE)
-- =============================================================================
-- WARNING: This will delete ALL data!
-- =============================================================================

BEGIN;

-- Drop tables in reverse order of dependencies
DROP TABLE IF EXISTS invites CASCADE;
DROP TABLE IF EXISTS channels CASCADE;
DROP TABLE IF EXISTS memberships CASCADE;
DROP TABLE IF EXISTS servers CASCADE;
DROP TABLE IF EXISTS users CASCADE;
DROP TABLE IF EXISTS _migrations CASCADE;

-- Drop types
DROP TYPE IF EXISTS channel_kind CASCADE;
DROP TYPE IF EXISTS member_role CASCADE;

-- Drop functions
DROP FUNCTION IF EXISTS update_updated_at_column CASCADE;

-- Drop extensions (optional - keep if other DBs use them)
-- DROP EXTENSION IF EXISTS "citext";
-- DROP EXTENSION IF EXISTS "uuid-ossp";

COMMIT;
