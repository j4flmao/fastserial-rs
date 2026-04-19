use axum::{
    body::Bytes,
    extract::{FromRequest, Request},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use fastserial::{Decode, Encode, binary, json};

use crate::models::response::ApiResponse;

pub struct FastJson<T>(pub T);

impl<T> IntoResponse for FastJson<T>
where
    T: Encode,
{
    fn into_response(self) -> Response {
        match json::encode(&self.0) {
            Ok(bytes) => ([(header::CONTENT_TYPE, "application/json")], bytes).into_response(),
            Err(e) => {
                let error_response: ApiResponse<String> =
                    ApiResponse::error(format!("Serialization error: {:?}", e));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(header::CONTENT_TYPE, "application/json")],
                    json::encode(&error_response).unwrap_or_default(),
                )
                    .into_response()
            }
        }
    }
}

impl<S, T> FromRequest<S> for FastJson<T>
where
    T: for<'de> Decode<'de>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, FastJson<ApiResponse<String>>);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state).await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                FastJson(ApiResponse::error(format!("Failed to read body: {}", e))),
            )
        })?;

        let val = json::decode::<T>(&bytes).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                FastJson(ApiResponse::error(format!("JSON Decode error: {:?}", e))),
            )
        })?;

        Ok(FastJson(val))
    }
}

pub struct FastBinary<T>(pub T);

impl<T> FastBinary<T> {}

impl<T> IntoResponse for FastBinary<T>
where
    T: Encode,
{
    fn into_response(self) -> Response {
        match binary::encode_raw(&self.0) {
            Ok(bytes) => {
                ([(header::CONTENT_TYPE, "application/octet-stream")], bytes).into_response()
            }
            Err(e) => {
                let error_response: ApiResponse<String> =
                    ApiResponse::error(format!("Binary serialization error: {:?}", e));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(header::CONTENT_TYPE, "application/json")],
                    json::encode(&error_response).unwrap_or_default(),
                )
                    .into_response()
            }
        }
    }
}

impl<S, T> FromRequest<S> for FastBinary<T>
where
    T: for<'de> Decode<'de>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, FastJson<ApiResponse<String>>);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state).await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                FastJson(ApiResponse::error(format!("Failed to read body: {}", e))),
            )
        })?;

        let val = binary::decode_raw::<T>(&bytes).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                FastJson(ApiResponse::error(format!("Binary decode error: {:?}", e))),
            )
        })?;

        Ok(FastBinary(val))
    }
}
