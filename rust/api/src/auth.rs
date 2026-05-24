use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde_json::json;

/// 从 Tinyauth/Caddy forward_auth 注入的 header
pub const FORWARDED_USER: &str = "X-Forwarded-User";

/// 当前登录用户，从请求 extension 中提取
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub username: String,
}

/// 从 Extensions 中提取 CurrentUser
pub fn user_from_request<B>(req: &axum::http::Request<B>) -> Option<CurrentUser> {
    req.extensions().get::<CurrentUser>().cloned()
}

/// 需要登录才能访问的 extractor —— 没有 CurrentUser 时返回 401
pub async fn require_user(
    user: Option<axum::extract::Extension<CurrentUser>>,
) -> Result<axum::extract::Extension<CurrentUser>, (StatusCode, Json<serde_json::Value>)> {
    user.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "authentication required" })),
        )
    })
}
