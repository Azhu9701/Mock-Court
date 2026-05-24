/// 当前登录用户，从请求 extension 中提取。
/// 由 middleware.rs 中的 auth_middleware 注入。
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CurrentUser {
    pub username: String,
}

/// 从请求 Extensions 中提取 CurrentUser
#[allow(dead_code)]
pub fn user_from_request<B>(req: &axum::http::Request<B>) -> Option<CurrentUser> {
    req.extensions().get::<CurrentUser>().cloned()
}
