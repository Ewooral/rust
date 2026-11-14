//! Authorisation: what is this caller allowed to do? Pure rules, no I/O.
//!
//! Authentication (the `AuthUser` extractor) answered "who are you?";
//! this answers "may you?". Keeping the rules in one small function makes
//! them easy to read, to test exhaustively, and to change.

use crate::auth::AuthUser;
use crate::domain::Role;
use crate::error::AppError;

/// What someone wants to do to an existing review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Change it.
    Edit,
    /// Remove it.
    Delete,
}

/// The rules: admins may edit and delete anything; members only their own.
/// (Reading is public and creating needs only a login, so neither is here.)
pub fn may(actor: &AuthUser, _action: Action, author_id: i64) -> bool {
    match actor.role {
        Role::Admin => true,
        Role::Member => actor.id == author_id,
    }
}

/// [`may`] as a `Result`, for `?` in the service layer: 403 when refused.
pub fn authorize(actor: &AuthUser, action: Action, author_id: i64) -> Result<(), AppError> {
    if may(actor, action, author_id) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_action_and_owner_combination() {
        let member = AuthUser {
            id: 1,
            role: Role::Member,
        };
        let admin = AuthUser {
            id: 2,
            role: Role::Admin,
        };
        for action in [Action::Edit, Action::Delete] {
            assert!(may(&member, action, 1), "members may {action:?} their own");
            assert!(!may(&member, action, 99), "but not other people's");
            assert!(may(&admin, action, 1), "admins may {action:?} anyone's");
            assert!(may(&admin, action, 2));
        }
        assert!(matches!(
            authorize(&member, Action::Delete, 99),
            Err(AppError::Forbidden)
        ));
    }
}
