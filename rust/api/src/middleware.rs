use std::time::Duration;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::error::ApiError;

pub fn apply_middleware(router: axum::Router) -> axum::Router {
    router.layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(StatusCode::GATEWAY_TIMEOUT, Duration::from_secs(120)))
        .layer(CatchPanicLayer::custom(panic_recovery))
}

fn panic_recovery(
    err: Box<dyn std::any::Any + Send + 'static>,
) -> Response {
    let msg = err
        .downcast_ref::<String>()
        .map(|s| s.clone())
        .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "Unknown panic".into());
    tracing::error!("Panic recovered: {}", msg);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            error: "Internal server error".into(),
        }),
    )
        .into_response()
}
