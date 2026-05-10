use axum::http::StatusCode;
use axum::Json;
use foundation::FoundationError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}

pub fn map_api_error(e: FoundationError) -> (StatusCode, Json<ApiError>) {
    let status = match &e {
        FoundationError::SoulNotFound(_) | FoundationError::SessionNotFound(_) => StatusCode::NOT_FOUND,
        FoundationError::Validation(_) => StatusCode::BAD_REQUEST,
        _ => {
            tracing::error!("Unhandled error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    };
    (status, Json(ApiError { error: e.to_string() }))
}
