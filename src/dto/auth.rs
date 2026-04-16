use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::auth::{UserAuthRow, UserRow},
    extractors::CurrentUser,
};

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl From<UserRow> for UserResponse {
    fn from(value: UserRow) -> Self {
        Self {
            id: value.id,
            email: value.email,
            created_at: value.created_at,
        }
    }
}

impl From<UserAuthRow> for UserResponse {
    fn from(value: UserAuthRow) -> Self {
        Self {
            id: value.id,
            email: value.email,
            created_at: value.created_at,
        }
    }
}

impl From<CurrentUser> for UserResponse {
    fn from(value: CurrentUser) -> Self {
        Self {
            id: value.id,
            email: value.email,
            created_at: value.created_at,
        }
    }
}
