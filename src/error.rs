use std::collections::HashMap;

use axum::{Json, extract::rejection::JsonRejection, http::StatusCode, response::IntoResponse};
use axum_valid::ValidationRejection;
use serde::Serialize;
use tracing::error;
use validator::{ValidationErrors, ValidationErrorsKind};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub error: ApiErrorData,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorData {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("request validation failed")]
    Validation {
        fields: HashMap<String, Vec<String>>,
    },
    #[error("bad request")]
    BadRequest { message: String },
    #[error("unauthorized")]
    Unauthorized,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("email already exists")]
    EmailAlreadyExists,
    #[error("internal server error")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Validation { .. } => StatusCode::BAD_REQUEST,
            Self::BadRequest { .. } => StatusCode::BAD_REQUEST,
            Self::Unauthorized | Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::EmailAlreadyExists => StatusCode::CONFLICT,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn body(&self) -> ApiErrorBody {
        match self {
            Self::Validation { fields } => ApiErrorBody {
                error: ApiErrorData {
                    code: "validation_error",
                    message: "Request validation failed".to_string(),
                    fields: Some(fields.clone()),
                },
            },
            Self::BadRequest { message } => ApiErrorBody {
                error: ApiErrorData {
                    code: "bad_request",
                    message: message.clone(),
                    fields: None,
                },
            },
            Self::Unauthorized => ApiErrorBody {
                error: ApiErrorData {
                    code: "unauthorized",
                    message: "Authentication required".to_string(),
                    fields: None,
                },
            },
            Self::InvalidCredentials => ApiErrorBody {
                error: ApiErrorData {
                    code: "invalid_credentials",
                    message: "Invalid email or password".to_string(),
                    fields: None,
                },
            },
            Self::EmailAlreadyExists => ApiErrorBody {
                error: ApiErrorData {
                    code: "email_already_exists",
                    message: "Email is already registered".to_string(),
                    fields: None,
                },
            },
            Self::Internal(_) => ApiErrorBody {
                error: ApiErrorData {
                    code: "internal_error",
                    message: "Internal server error".to_string(),
                    fields: None,
                },
            },
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        if let Self::Internal(err) = &self {
            error!(error = ?err, "request failed with internal error");
        }

        (self.status_code(), Json(self.body())).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self::Internal(value.into())
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::Internal(anyhow::anyhow!(value.to_string()))
    }
}

impl From<ValidationErrors> for AppError {
    fn from(value: ValidationErrors) -> Self {
        let fields = value
            .errors()
            .iter()
            .filter_map(|(field, kind)| match kind {
                ValidationErrorsKind::Field(errors) => {
                    let messages = errors
                        .iter()
                        .map(validation_error_message)
                        .collect::<Vec<_>>();

                    Some((field.to_string(), messages))
                }
                ValidationErrorsKind::Struct(_) | ValidationErrorsKind::List(_) => None,
            })
            .collect();

        Self::Validation { fields }
    }
}

impl<E> From<ValidationRejection<ValidationErrors, E>> for AppError
where
    E: Into<AppError>,
{
    fn from(value: ValidationRejection<ValidationErrors, E>) -> Self {
        match value {
            ValidationRejection::Valid(errors) => errors.into(),
            ValidationRejection::Inner(err) => err.into(),
        }
    }
}

impl From<JsonRejection> for AppError {
    fn from(value: JsonRejection) -> Self {
        Self::BadRequest {
            message: value.body_text(),
        }
    }
}

fn validation_error_message(error: &validator::ValidationError) -> String {
    error
        .message
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| "Invalid value".to_string())
}
