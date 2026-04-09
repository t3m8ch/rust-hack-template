use std::ops::{Deref, DerefMut};

use axum::{
    Json,
    extract::{FromRequest, Request},
};
use axum_valid::Valid;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct ValidatedJson<T>(pub T);

impl<T> ValidatedJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for ValidatedJson<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ValidatedJson<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
    Json<T>: FromRequest<S>,
    AppError: From<<Valid<Json<T>> as FromRequest<S>>::Rejection>,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let validated = Valid::<Json<T>>::from_request(req, state)
            .await
            .map_err(AppError::from)?;

        Ok(Self(validated.into_inner().0))
    }
}
