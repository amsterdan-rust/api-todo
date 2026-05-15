use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Path, Request},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;

use super::error::AppError;

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(Self(value)),
            Err(rejection) => {
                let message = rejection.body_text();

                let error = AppError::Validation(format!("JSON inválido: {message}"));

                Err(error.into_response())
            }
        }
    }
}

pub struct ValidatedPath<T>(pub T);

impl<S, T> FromRequestParts<S> for ValidatedPath<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Send,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Path::<T>::from_request_parts(parts, state).await {
            Ok(Path(value)) => Ok(Self(value)),
            Err(rejection) => {
                let message = rejection.body_text();

                let error = AppError::Validation(format!("parâmetro de rota inválido: {message}"));

                Err(error.into_response())
            }
        }
    }
}
