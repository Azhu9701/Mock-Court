use std::sync::Arc;
use std::time::Duration;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde_json::json;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::error::ApiError;
use crate::rate_limiter::RateLimiter;

pub fn apply_middleware(router: axum::Router, rate_limiter: Arc<RateLimiter>) -> axum::Router {
    router
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .layer(Extension(rate_limiter.clone()))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(StatusCode::GATEWAY_TIMEOUT, Duration::from_secs(120)))
        .layer(CatchPanicLayer::custom(panic_recovery))
}

async fn rate_limit_middleware(
    Extension(limiter): Extension<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Response {
    let ip = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("X-Real-Ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    let (allowed, retry_after) = limiter.check(&ip);
    if !allowed {
        let retry = retry_after.max(1);
        let mut resp = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "error": "rate limited",
                "retry_after": retry,
            })),
        )
            .into_response();
        resp.headers_mut().insert(
            axum::http::header::RETRY_AFTER,
            axum::http::HeaderValue::from_str(&retry.to_string()).unwrap(),
        );
        return resp;
    }

    next.run(request).await
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
