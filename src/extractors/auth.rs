use std::future::Future;

use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{auth::session::find_user_by_session_token, error::AppError, state::AppState};

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let state = state.clone();

        async move {
            let jar = CookieJar::from_request_parts(parts, &state)
                .await
                .map_err(|_| AppError::Unauthorized)?;

            let token = jar
                .get(&state.config.session_cookie_name)
                .map(|cookie| cookie.value().to_string())
                .ok_or(AppError::Unauthorized)?;

            let user = find_user_by_session_token(&state, &token)
                .await?
                .ok_or(AppError::Unauthorized)?;

            Ok(Self {
                id: user.id,
                email: user.email,
                created_at: user.created_at,
            })
        }
    }
}
