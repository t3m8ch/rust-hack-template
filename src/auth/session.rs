use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{DateTime, Duration, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{config::Config, error::AppResult, state::AppState};

#[derive(Debug, Clone)]
pub struct SessionToken {
    pub token: String,
    pub token_hash: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SessionUser {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

pub fn generate_session_token() -> SessionToken {
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let token_hash = hash_session_token(&token);

    SessionToken { token, token_hash }
}

pub fn hash_session_token(token: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hasher.finalize().to_vec()
}

pub async fn create_session(
    state: &AppState,
    user_id: Uuid,
    token_hash: &[u8],
) -> AppResult<DateTime<Utc>> {
    let expires_at = Utc::now() + Duration::days(state.config.session_ttl_days);

    sqlx::query!(
        "INSERT INTO sessions (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        user_id,
        token_hash,
        expires_at,
    )
    .execute(&state.pgpool)
    .await?;

    Ok(expires_at)
}

pub async fn find_user_by_session_token(
    state: &AppState,
    token: &str,
) -> AppResult<Option<SessionUser>> {
    let token_hash = hash_session_token(token);

    let user = sqlx::query_as!(
        SessionUser,
        r#"
        SELECT u.id, u.email, u.created_at
        FROM sessions s
        JOIN users u ON u.id = s.user_id
        WHERE s.token_hash = $1
          AND s.expires_at > NOW()
        "#,
        token_hash.as_slice(),
    )
    .fetch_optional(&state.pgpool)
    .await?;

    Ok(user)
}

pub async fn delete_session_by_token(state: &AppState, token: &str) -> AppResult<()> {
    let token_hash = hash_session_token(token);

    sqlx::query!(
        "DELETE FROM sessions WHERE token_hash = $1",
        token_hash.as_slice()
    )
    .execute(&state.pgpool)
    .await?;

    Ok(())
}

pub fn build_session_cookie(config: &Config, token: &str) -> Cookie<'static> {
    Cookie::build((config.session_cookie_name.clone(), token.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(config.session_secure_cookie)
        .max_age(time::Duration::days(config.session_ttl_days))
        .build()
}

pub fn build_remove_session_cookie(config: &Config) -> Cookie<'static> {
    Cookie::build((config.session_cookie_name.clone(), String::new()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(config.session_secure_cookie)
        .max_age(time::Duration::seconds(0))
        .build()
}
