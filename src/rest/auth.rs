use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use axum_extra::extract::CookieJar;

use crate::{
    auth::{
        password::{hash_password, verify_password},
        session::{
            build_remove_session_cookie, build_session_cookie, create_session,
            delete_session_by_token, generate_session_token,
        },
    },
    db::auth::{find_user_auth_by_email, insert_user},
    dto::auth::{LoginRequest, RegisterRequest, UserResponse},
    error::{AppError, AppResult},
    extractors::{CurrentUser, ValidatedJson},
    state::AppState,
};

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

async fn register(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    let email = normalize_email(&payload.email);
    let password_hash = hash_password(&payload.password)?;

    let user = insert_user(&state, &email, &password_hash).await?;

    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> AppResult<(CookieJar, Json<UserResponse>)> {
    let email = normalize_email(&payload.email);
    let user = find_user_auth_by_email(&state, &email)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    if !verify_password(&user.password_hash, &payload.password)? {
        return Err(AppError::InvalidCredentials);
    }

    let session_token = generate_session_token();
    create_session(&state, user.id, &session_token.token_hash).await?;

    let jar = jar.add(build_session_cookie(&state.config, &session_token.token));

    Ok((jar, Json(UserResponse::from(user))))
}

async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<(CookieJar, StatusCode)> {
    let mut jar = jar;

    if let Some(cookie) = jar.get(&state.config.session_cookie_name) {
        delete_session_by_token(&state, cookie.value()).await?;
    }

    jar = jar.remove(build_remove_session_cookie(&state.config));

    Ok((jar, StatusCode::NO_CONTENT))
}

async fn me(current_user: CurrentUser) -> Json<UserResponse> {
    Json(UserResponse::from(current_user))
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}
