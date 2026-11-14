//! Exercise 6: an authorisation policy, written as one pure function.
//!
//! A forum with three roles. Implement `may(actor, action, post)`:
//!
//!   | Action        | Admin | Moderator               | Member         | Suspended |
//!   |---------------|-------|-------------------------|----------------|-----------|
//!   | Read          | yes   | yes                     | yes            | yes       |
//!   | Create        | yes   | yes                     | yes            | no        |
//!   | Edit          | yes   | only their own          | only their own | no        |
//!   | Delete        | yes   | yes, unless it's locked | only their own | no        |
//!   | Lock          | yes   | yes                     | no             | no        |
//!
//! ("their own": `post.author_id == actor.id`. "locked": `post.locked`.)
//! Then `authorize` turns `false` into `Err(Denied)` so services can use `?`.
//!
//! Tip: `match (actor.role, action)` with one arm per row reads like the table.
//!
//! Test: cargo test -p capstone_rest_api --features exercises --bin ex06_authorization_policy

#![allow(unused_variables, dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Admin,
    Moderator,
    Member,
    Suspended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Read,
    Create,
    Edit,
    Delete,
    Lock,
}

#[derive(Debug, Clone, Copy)]
struct Actor {
    id: i64,
    role: Role,
}

#[derive(Debug, Clone, Copy)]
struct Post {
    author_id: i64,
    locked: bool,
}

#[derive(Debug, PartialEq, Eq)]
struct Denied;

fn may(actor: &Actor, action: Action, post: &Post) -> bool {
    todo!()
}

fn authorize(actor: &Actor, action: Action, post: &Post) -> Result<(), Denied> {
    todo!()
}

fn main() {
    let ama = Actor {
        id: 1,
        role: Role::Member,
    };
    let post = Post {
        author_id: 2,
        locked: false,
    };
    println!("may Ama edit post by 2? {}", may(&ama, Action::Edit, &post));
    println!(
        "authorize Ama to read it: {:?}",
        authorize(&ama, Action::Read, &post)
    );

    // The whole table, for someone else's unlocked post.
    let roles = [Role::Admin, Role::Moderator, Role::Member, Role::Suspended];
    let actions = [
        Action::Read,
        Action::Create,
        Action::Edit,
        Action::Delete,
        Action::Lock,
    ];
    for role in roles {
        let actor = Actor { id: 1, role };
        let row: Vec<String> = actions
            .iter()
            .map(|&action| format!("{action:?}={}", may(&actor, action, &post)))
            .collect();
        println!("{role:<9?} {}", row.join(" "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ME: i64 = 1;
    const SOMEONE_ELSE: i64 = 2;

    fn actor(role: Role) -> Actor {
        Actor { id: ME, role }
    }

    fn post(author_id: i64, locked: bool) -> Post {
        Post { author_id, locked }
    }

    #[test]
    fn everyone_may_read_and_admins_may_do_anything() {
        for role in [Role::Admin, Role::Moderator, Role::Member, Role::Suspended] {
            assert!(
                may(&actor(role), Action::Read, &post(SOMEONE_ELSE, true)),
                "{role:?}"
            );
        }
        for action in [Action::Create, Action::Edit, Action::Delete, Action::Lock] {
            assert!(
                may(&actor(Role::Admin), action, &post(SOMEONE_ELSE, true)),
                "{action:?}"
            );
        }
    }

    #[test]
    fn moderators() {
        let m = actor(Role::Moderator);
        assert!(may(&m, Action::Create, &post(ME, false)));
        assert!(may(&m, Action::Edit, &post(ME, false)));
        assert!(!may(&m, Action::Edit, &post(SOMEONE_ELSE, false)));
        assert!(may(&m, Action::Delete, &post(SOMEONE_ELSE, false)));
        assert!(!may(&m, Action::Delete, &post(SOMEONE_ELSE, true)));
        assert!(may(&m, Action::Lock, &post(SOMEONE_ELSE, false)));
    }

    #[test]
    fn members_and_suspended_users() {
        let m = actor(Role::Member);
        assert!(may(&m, Action::Create, &post(ME, false)));
        assert!(may(&m, Action::Edit, &post(ME, true)));
        assert!(may(&m, Action::Delete, &post(ME, false)));
        assert!(!may(&m, Action::Delete, &post(SOMEONE_ELSE, false)));
        assert!(!may(&m, Action::Lock, &post(ME, false)));
        let s = actor(Role::Suspended);
        for action in [Action::Create, Action::Edit, Action::Delete, Action::Lock] {
            assert!(!may(&s, action, &post(ME, false)), "{action:?}");
        }
    }

    #[test]
    fn authorize_is_may_as_a_result() {
        assert_eq!(
            authorize(
                &actor(Role::Member),
                Action::Read,
                &post(SOMEONE_ELSE, false)
            ),
            Ok(())
        );
        assert_eq!(
            authorize(&actor(Role::Member), Action::Lock, &post(ME, false)),
            Err(Denied)
        );
    }
}
