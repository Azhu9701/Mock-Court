use std::sync::Arc;

use axum::extract::Request;
use axum::routing::get;
use axum::{Json, Router};

use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/me", get(me))
}

async fn me(req: Request) -> Json<serde_json::Value> {
    let user = crate::auth::user_from_request(&req);
    match user {
        Some(u) => Json(serde_json::json!({
            "authenticated": true,
            "username": u.username,
        })),
        None => Json(serde_json::json!({
            "authenticated": false,
            "username": null,
        })),
    }
}
