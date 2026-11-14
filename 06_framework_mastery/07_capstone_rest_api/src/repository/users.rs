//! SQL for the `users` table.

use sqlx::SqlitePool;

use crate::domain::{Role, User};

/// A user as stored: the public [`User`] plus the password hash, which must
/// never leave the service layer.
#[derive(Debug, Clone)]
pub struct StoredUser {
    /// The public part.
    pub user: User,
    /// The Argon2 PHC string.
    pub password_hash: String,
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: i64,
    email: String,
    display_name: String,
    password_hash: String,
    role: String,
    created_at: String,
}

impl TryFrom<UserRow> for StoredUser {
    type Error = sqlx::Error;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let role = Role::from_db(&row.role)
            .ok_or_else(|| sqlx::Error::Decode(format!("unknown role {:?}", row.role).into()))?;
        Ok(StoredUser {
            user: User {
                id: row.id,
                email: row.email,
                display_name: row.display_name,
                role,
                created_at: row.created_at,
            },
            password_hash: row.password_hash,
        })
    }
}

/// Reads and writes users.
#[derive(Debug, Clone)]
pub struct UserRepository {
    pool: SqlitePool,
}

impl UserRepository {
    /// A repository on this pool.
    pub fn new(pool: SqlitePool) -> Self {
        UserRepository { pool }
    }

    /// Inserts a user. A duplicate e-mail fails with a `UNIQUE` violation,
    /// which the service turns into a 409.
    pub async fn insert(
        &self,
        email: &str,
        display_name: &str,
        password_hash: &str,
        role: Role,
    ) -> Result<User, sqlx::Error> {
        let row: UserRow = sqlx::query_as(
            "INSERT INTO users (email, display_name, password_hash, role) VALUES (?, ?, ?, ?) \
             RETURNING id, email, display_name, password_hash, role, created_at",
        )
        .bind(email)
        .bind(display_name)
        .bind(password_hash)
        .bind(role.as_str())
        .fetch_one(&self.pool)
        .await?;
        Ok(StoredUser::try_from(row)?.user)
    }

    /// The user with this (lower-case) e-mail, with their password hash.
    pub async fn find_by_email(&self, email: &str) -> Result<Option<StoredUser>, sqlx::Error> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT id, email, display_name, password_hash, role, created_at \
             FROM users WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        row.map(StoredUser::try_from).transpose()
    }

    /// The user with this id.
    pub async fn find_by_id(&self, id: i64) -> Result<Option<User>, sqlx::Error> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT id, email, display_name, password_hash, role, created_at \
             FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(StoredUser::try_from).transpose()?.map(|s| s.user))
    }
}
