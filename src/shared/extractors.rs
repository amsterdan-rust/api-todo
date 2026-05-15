use std::{collections::HashMap, str::FromStr};

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
    T: FromStr + Send,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(params) =
            match Path::<HashMap<String, String>>::from_request_parts(parts, state).await {
                Ok(params) => params,
                Err(_rejection) => {
                    let error = AppError::Validation("Parâmetro de rota inválido".to_string());

                    return Err(error.into_response());
                }
            };

        let Some((param_name, param_value)) = params.into_iter().next() else {
            let error = AppError::Validation("Parâmetro de rota não encontrado".to_string());

            return Err(error.into_response());
        };

        match param_value.parse::<T>() {
            Ok(value) => Ok(Self(value)),
            Err(_) => {
                let error = AppError::Validation(format!(
                    "O parâmetro '{param_name}' deve ser um UUID válido"
                ));

                Err(error.into_response())
            }
        }
    }
}
