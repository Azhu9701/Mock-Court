use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/status", get(apikey_status))
        .route("/set", post(set_apikey))
}

#[derive(Debug, Serialize)]
struct ApikeyStatus {
    keys: HashMap<String, bool>,
}

async fn apikey_status(
    State(_state): State<Arc<AppState>>,
) -> Json<ApikeyStatus> {
    let path = PathBuf::from("data/apikeys.json");
    let keys = read_keys(&path);
    Json(ApikeyStatus {
        keys: ["anthropic", "openai", "deepseek"]
            .iter()
            .map(|&k| {
                let env_key = match k {
                    "anthropic" => std::env::var("ANTHROPIC_API_KEY"),
                    "openai" => std::env::var("OPENAI_API_KEY"),
                    "deepseek" => std::env::var("DEEPSEEK_API_KEY"),
                    _ => Err(std::env::VarError::NotPresent),
                };
                (k.to_string(), env_key.is_ok() || keys.get(k).map_or(false, |v| !v.is_empty()))
            })
            .collect(),
    })
}

#[derive(Debug, Deserialize)]
struct SetApikeyRequest {
    provider: String,
    key: String,
}

async fn set_apikey(
    Json(body): Json<SetApikeyRequest>,
) -> Json<serde_json::Value> {
    let path = PathBuf::from("data/apikeys.json");
    let mut keys = read_keys(&path);
    keys.insert(body.provider, body.key);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = serde_json::to_string_pretty(&keys).unwrap_or_default();
    let _ = std::fs::write(&path, &content);
    Json(serde_json::json!({ "status": "ok" }))
}

fn read_keys(path: &std::path::Path) -> HashMap<String, String> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
