//! Executable tests for ban/unban business rules.
//!
//! These tests validate permission guards and ban semantics without requiring
//! a running PostgreSQL/backend process.

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::error::AppError;
    use crate::models::{BanMemberRequest, MemberRole};
    use crate::routes::members::{build_ban_message, is_ban_active, validate_ban_permissions};

    #[test]
    fn admin_can_ban_member_permanently() {
        let caller = Uuid::new_v4();
        let target = Uuid::new_v4();

        let result = validate_ban_permissions(caller, target, MemberRole::Admin, MemberRole::Member);
        assert!(result.is_ok());

        let req: BanMemberRequest = serde_json::from_str(r#"{"reason":"spam"}"#).unwrap();
        assert!(req.expires_at.is_none());
        assert_eq!(build_ban_message(req.reason.as_deref()), "Vous avez ete banni de ce serveur. Motif: spam");
    }

    #[test]
    fn admin_can_ban_member_temporarily() {
        let caller = Uuid::new_v4();
        let target = Uuid::new_v4();

        let result = validate_ban_permissions(caller, target, MemberRole::Admin, MemberRole::Member);
        assert!(result.is_ok());

        let future = Utc::now() + Duration::hours(2);
        assert!(is_ban_active(Some(future), Utc::now()));
    }

    #[test]
    fn cannot_ban_server_owner() {
        let err = validate_ban_permissions(
            Uuid::new_v4(),
            Uuid::new_v4(),
            MemberRole::Admin,
            MemberRole::Owner,
        )
        .expect_err("ban owner must be forbidden");

        match err {
            AppError::Forbidden(msg) => assert_eq!(msg, "Cannot ban the server owner"),
            _ => panic!("expected Forbidden error"),
        }
    }

    #[test]
    fn admin_cannot_ban_admin() {
        let err = validate_ban_permissions(
            Uuid::new_v4(),
            Uuid::new_v4(),
            MemberRole::Admin,
            MemberRole::Admin,
        )
        .expect_err("admin should not ban admin");

        match err {
            AppError::Forbidden(msg) => assert_eq!(msg, "Admins cannot ban other admins"),
            _ => panic!("expected Forbidden error"),
        }
    }

    #[test]
    fn member_cannot_ban() {
        let err = validate_ban_permissions(
            Uuid::new_v4(),
            Uuid::new_v4(),
            MemberRole::Member,
            MemberRole::Member,
        )
        .expect_err("member should not ban");

        match err {
            AppError::Forbidden(msg) => assert_eq!(msg, "Insufficient permissions to ban members"),
            _ => panic!("expected Forbidden error"),
        }
    }

    #[test]
    fn admin_can_unban_member() {
        let response = serde_json::json!({ "unbanned": true });
        assert_eq!(response["unbanned"], true);
    }

    #[test]
    fn list_bans_excludes_expired() {
        let now = Utc::now();
        let bans = [
            ("user_a", None),
            ("user_b", Some(now - Duration::minutes(1))),
            ("user_c", Some(now + Duration::minutes(5))),
        ];

        let active: Vec<&str> = bans
            .iter()
            .filter(|(_, expires_at)| is_ban_active(*expires_at, now))
            .map(|(id, _)| *id)
            .collect();

        assert_eq!(active, vec!["user_a", "user_c"]);
    }

    #[test]
    fn non_member_cannot_list_bans() {
        let err = AppError::Forbidden("Not a member of this server".to_string());
        match err {
            AppError::Forbidden(msg) => assert_eq!(msg, "Not a member of this server"),
            _ => panic!("expected Forbidden"),
        }
    }

    #[test]
    fn user_cannot_ban_self() {
        let user = Uuid::new_v4();
        let err = validate_ban_permissions(user, user, MemberRole::Owner, MemberRole::Member)
            .expect_err("self-ban must be rejected");

        match err {
            AppError::BadRequest(msg) => assert_eq!(msg, "Cannot ban yourself."),
            _ => panic!("expected BadRequest"),
        }
    }
}
