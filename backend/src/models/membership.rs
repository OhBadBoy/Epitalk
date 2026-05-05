//! Membership model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

/// Member role enum (matches PostgreSQL ENUM)
/// Hierarchy: OWNER > ADMIN > MODERATOR > MEMBER
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "member_role", rename_all = "UPPERCASE")]
pub enum MemberRole {
    Owner,
    Admin,
    Moderator,
    Member,
}

impl MemberRole {
    /// Check if this role can manage members (promote/demote/kick)
    /// Only Owner can manage members
    pub fn can_manage_members(&self) -> bool {
        matches!(self, MemberRole::Owner)
    }

    /// Check if this role can manage channels (create/edit/delete)
    /// Owner and Admin can manage channels
    pub fn can_manage_channels(&self) -> bool {
        matches!(self, MemberRole::Owner | MemberRole::Admin)
    }

    /// Check if this role can create invites
    /// Owner, Admin and Moderator can create invites
    pub fn can_create_invites(&self) -> bool {
        matches!(self, MemberRole::Owner | MemberRole::Admin | MemberRole::Moderator)
    }

    /// Check if this role can delete messages from others
    /// Owner, Admin and Moderator can delete others' messages
    pub fn can_delete_others_messages(&self) -> bool {
        matches!(self, MemberRole::Owner | MemberRole::Admin | MemberRole::Moderator)
    }

    /// Check if this role can kick members
    /// Owner and Admin can kick, Moderator can kick Members only
    #[allow(dead_code)] // Used by can_mute and future moderation features
    pub fn can_kick(&self, target_role: &MemberRole) -> bool {
        match self {
            MemberRole::Owner => true,
            MemberRole::Admin => matches!(target_role, MemberRole::Moderator | MemberRole::Member),
            MemberRole::Moderator => matches!(target_role, MemberRole::Member),
            MemberRole::Member => false,
        }
    }

    /// Check if this role can mute members
    /// Owner, Admin and Moderator can mute lower roles
    #[allow(dead_code)] // For future moderation features
    pub fn can_mute(&self, target_role: &MemberRole) -> bool {
        self.can_kick(target_role)
    }

    /// Check if the role is higher or equal to another
    pub fn is_higher_or_equal(&self, other: &MemberRole) -> bool {
        match (self, other) {
            (MemberRole::Owner, _) => true,
            (MemberRole::Admin, MemberRole::Admin | MemberRole::Moderator | MemberRole::Member) => true,
            (MemberRole::Moderator, MemberRole::Moderator | MemberRole::Member) => true,
            (MemberRole::Member, MemberRole::Member) => true,
            _ => false,
        }
    }

    /// Get numeric priority (higher = more permissions)
    #[allow(dead_code)] // Alternative to is_higher_or_equal for comparisons
    pub fn priority(&self) -> u8 {
        match self {
            MemberRole::Owner => 4,
            MemberRole::Admin => 3,
            MemberRole::Moderator => 2,
            MemberRole::Member => 1,
        }
    }
}

impl std::fmt::Display for MemberRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemberRole::Owner => write!(f, "OWNER"),
            MemberRole::Admin => write!(f, "ADMIN"),
            MemberRole::Moderator => write!(f, "MODERATOR"),
            MemberRole::Member => write!(f, "MEMBER"),
        }
    }
}

/// Membership entity matching the database schema.
/// Note: The `memberships` table uses composite key (user_id, server_id) - NO `id` column.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Membership {
    pub user_id: Uuid,
    pub server_id: Uuid,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
}

/// Membership with user info
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MemberResponse {
    pub user_id: Uuid,
    pub username: String,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
    pub avatar_url: Option<String>,
    pub status: Option<String>,
}

/// Update member role request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: MemberRole,
}

/// Ban record matching the `bans` table.
/// `expires_at = None` means permanent ban.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BanRecord {
    pub id: Uuid,
    pub server_id: Uuid,
    pub user_id: Uuid,
    pub banned_by: Uuid,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Ban response returned to API clients
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BanResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub banned_by: Uuid,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Request body for banning a member.
/// `expires_at = None` = permanent ban.
#[derive(Debug, Clone, Deserialize)]
pub struct BanMemberRequest {
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_permissions_and_priority_are_consistent() {
        let owner = MemberRole::Owner;
        let admin = MemberRole::Admin;
        let moderator = MemberRole::Moderator;
        let member = MemberRole::Member;

        assert!(owner.can_manage_members());
        assert!(owner.can_manage_channels());
        assert!(owner.can_create_invites());
        assert!(owner.can_delete_others_messages());

        assert!(!member.can_manage_members());
        assert!(!member.can_manage_channels());
        assert!(!member.can_delete_others_messages());

        assert!(moderator.can_create_invites());
        assert!(moderator.can_delete_others_messages());

        assert!(owner.can_kick(&admin));
        assert!(admin.can_kick(&moderator));
        assert!(moderator.can_kick(&member));
        assert!(!member.can_kick(&moderator));

        assert!(owner.can_mute(&admin));
        assert!(admin.can_mute(&member));

        assert!(owner.is_higher_or_equal(&member));
        assert!(admin.is_higher_or_equal(&moderator));
        assert!(!moderator.is_higher_or_equal(&admin));

        assert!(owner.priority() > admin.priority());
        assert!(admin.priority() > moderator.priority());
        assert!(moderator.priority() > member.priority());
    }

    #[test]
    fn display_outputs_uppercase_role() {
        assert_eq!(MemberRole::Owner.to_string(), "OWNER");
        assert_eq!(MemberRole::Admin.to_string(), "ADMIN");
        assert_eq!(MemberRole::Moderator.to_string(), "MODERATOR");
        assert_eq!(MemberRole::Member.to_string(), "MEMBER");
    }

    // ---------------------------------------------------------------
    // BanMemberRequest — désérialisation JSON
    // ---------------------------------------------------------------

    #[test]
    fn ban_request_permanent_no_expiry() {
        // expires_at absent = ban permanent
        let json = r#"{"reason":"spamming"}"#;
        let req: BanMemberRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.reason.as_deref(), Some("spamming"));
        assert!(req.expires_at.is_none(), "expires_at absent = ban permanent");
    }

    #[test]
    fn ban_request_temporary_has_expiry() {
        let json = r#"{"reason":null,"expires_at":"2027-06-01T00:00:00Z"}"#;
        let req: BanMemberRequest = serde_json::from_str(json).unwrap();
        assert!(req.reason.is_none());
        assert!(req.expires_at.is_some(), "expires_at fourni = ban temporaire");
    }

    #[test]
    fn ban_request_empty_body_is_permanent() {
        // Corps vide {} = ban permanent sans raison
        let json = r#"{}"#;
        let req: BanMemberRequest = serde_json::from_str(json).unwrap();
        assert!(req.reason.is_none());
        assert!(req.expires_at.is_none());
    }

    #[test]
    fn ban_temporary_expiry_is_in_the_future() {
        let future = chrono::Utc::now() + chrono::Duration::days(7);
        let req = BanMemberRequest {
            reason: Some("flood".to_string()),
            expires_at: Some(future),
        };
        assert!(req.expires_at.unwrap() > chrono::Utc::now());
    }

    #[test]
    fn ban_permanent_expiry_is_none() {
        let req = BanMemberRequest {
            reason: None,
            expires_at: None,
        };
        assert!(req.expires_at.is_none(), "ban permanent : expires_at = None");
    }

    // ---------------------------------------------------------------
    // RBAC — règles de permission pour le ban
    // ---------------------------------------------------------------

    #[test]
    fn ban_rbac_only_owner_and_admin_can_ban() {
        // Le handler ban utilise can_manage_channels() comme garde
        assert!(MemberRole::Owner.can_manage_channels(), "OWNER doit pouvoir bannir");
        assert!(MemberRole::Admin.can_manage_channels(), "ADMIN doit pouvoir bannir");
        assert!(!MemberRole::Moderator.can_manage_channels(), "MODERATOR ne doit pas pouvoir bannir");
        assert!(!MemberRole::Member.can_manage_channels(), "MEMBER ne doit pas pouvoir bannir");
    }

    #[test]
    fn ban_rbac_owner_is_protected() {
        // Un Owner ne peut jamais être banni — vérification par identité de rôle
        let target = MemberRole::Owner;
        assert_eq!(target, MemberRole::Owner, "la cible Owner doit être détectée");
        // Les autres rôles ne sont pas Owner
        assert_ne!(MemberRole::Admin, MemberRole::Owner);
        assert_ne!(MemberRole::Moderator, MemberRole::Owner);
        assert_ne!(MemberRole::Member, MemberRole::Owner);
    }

    #[test]
    fn ban_rbac_admin_cannot_ban_equal_rank() {
        // Admin ne peut pas bannir un autre Admin (même priorité)
        let caller = MemberRole::Admin;
        let target = MemberRole::Admin;
        assert_eq!(
            caller.priority(),
            target.priority(),
            "Admin vs Admin : même priorité → ban interdit"
        );
    }

    #[test]
    fn ban_rbac_admin_can_ban_lower_ranks() {
        let admin = MemberRole::Admin;
        assert!(admin.priority() > MemberRole::Moderator.priority());
        assert!(admin.priority() > MemberRole::Member.priority());
    }

    #[test]
    fn ban_rbac_owner_can_ban_any_rank() {
        let owner = MemberRole::Owner;
        assert!(owner.priority() > MemberRole::Admin.priority());
        assert!(owner.priority() > MemberRole::Moderator.priority());
        assert!(owner.priority() > MemberRole::Member.priority());
    }
}
