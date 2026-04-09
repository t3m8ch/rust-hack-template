use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

#[derive(Debug)]
pub struct UserAuthRow {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

pub async fn insert_user(
    state: &AppState,
    email: &str,
    password_hash: &str,
) -> Result<UserRow, AppError> {
    sqlx::query_as!(
        UserRow,
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        RETURNING id, email, created_at
        "#,
        email,
        password_hash,
    )
    .fetch_one(&state.pgpool)
    .await
    .map_err(map_register_error)
}

pub async fn find_user_auth_by_email(
    state: &AppState,
    email: &str,
) -> Result<Option<UserAuthRow>, AppError> {
    sqlx::query_as!(
        UserAuthRow,
        "SELECT id, email, password_hash, created_at FROM users WHERE email = $1",
        email,
    )
    .fetch_optional(&state.pgpool)
    .await
    .map_err(Into::into)
}

fn map_register_error(err: sqlx::Error) -> AppError {
    const UNIQUE_VIOLATION: &str = "23505";

    if let sqlx::Error::Database(db_err) = &err
        && db_err.code().as_deref() == Some(UNIQUE_VIOLATION)
    {
        return AppError::EmailAlreadyExists;
    }

    err.into()
}
