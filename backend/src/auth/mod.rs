//! Authentication and authorization module
//!
//! Provides JWT-based authentication, password hashing, and RBAC middleware.

pub mod jwt;
mod middleware;
mod password;

pub use jwt::{Claims, JwtService};
pub use middleware::{RequireAuth, RequireAuthAllowExpired};
pub use password::PasswordService;

#[cfg(test)]
pub fn test_require_auth(user_id: uuid::Uuid, email: &str, username: &str) -> RequireAuth {
	let claims = Claims::new(user_id, email.to_string(), username.to_string(), 24);
	let user = middleware::AuthUser {
		user_id,
		email: email.to_string(),
		username: username.to_string(),
		claims,
	};
	RequireAuth(user)
}
