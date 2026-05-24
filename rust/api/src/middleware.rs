use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde_json::json;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::error::ApiError;
use crate::rate_limiter::RateLimiter;

pub fn apply_middleware(
    router: axum::Router,
    rate_limiter: Arc<RateLimiter>,
    api_token: Option<String>,
    cors_origins: Vec<String>,
) -> axum::Router {
    let cors = if cors_origins.is_empty()
        || cors_origins.iter().any(|o| o == "*")
    {
        CorsLayer::permissive()
    } else {
        let origins: Vec<_> = cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any)
    };

    router
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(Extension(api_token.map(Arc::new)))
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .layer(Extension(rate_limiter.clone()))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(StatusCode::GATEWAY_TIMEOUT, Duration::from_secs(120)))
        .layer(CatchPanicLayer::custom(panic_recovery))
}

async fn auth_middleware(
    Extension(token): Extension<Option<Arc<String>>>,
    request: Request,
    next: Next,
) -> Response {
    // 模式 1：配置了 API_TOKEN → 检查 Bearer token
    if let Some(expected) = token {
        let authorized = request
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|t| t == expected.as_str())
            .unwrap_or(false);

        if !authorized {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "unauthorized" })),
            )
                .into_response();
        }
        return next.run(request).await;
    }

    // 模式 2：无 API_TOKEN → 检查 X-Forwarded-User（tinyauth forward-auth）
    let forwarded_user = request
        .headers()
        .get("X-Forwarded-User")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    if let Some(user) = forwarded_user {
        use crate::auth::CurrentUser;
        let mut req = request;
        req.extensions_mut().insert(CurrentUser { username: user });
        return next.run(req).await;
    }

    // 模式 3：无 API_TOKEN 也无 X-Forwarded-User → 放行（本地开发模式）
    next.run(request).await
}

async fn rate_limit_middleware(
    Extension(limiter): Extension<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Response {
    // 取 X-Forwarded-For 最右值（离本机最近的代理），左端值最易被伪造
    // 没有代理头时取直连 IP，不再 fallback 到 "unknown" 共享 bucket
    let ip = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next_back())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("X-Real-Ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            format!("anon-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
        });

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
