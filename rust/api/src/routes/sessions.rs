use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use axum::http::{header, StatusCode};
use archive::SessionDetail;
use chrono::{DateTime, Utc};
use foundation::{MessageRole, Session, SessionFilter, SessionObservation, SessionStatus, SessionSummary};
use serde::{Deserialize, Serialize};

use crate::error::{map_api_error, ApiError};
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_sessions))
        .route("/batch-delete", post(batch_delete_sessions))
        .route("/:id", get(get_session_detail).delete(delete_session))
        .route("/:id/digest", get(get_session_digest))
        .route("/:id/distill", post(trigger_distill))
        .route("/:id/rename", put(rename_session))
        .route("/:id/fork", put(fork_session))
        .route("/:id/export/markdown", get(export_session_markdown))
        .route("/:id/review", post(save_review))
}

#[derive(Debug, Deserialize)]
struct SessionListQuery { mode: Option<String>, status: Option<String>, limit: Option<u32>, offset: Option<u32> }

impl From<SessionListQuery> for SessionFilter {
    fn from(q: SessionListQuery) -> Self {
        SessionFilter {
            mode: q.mode.and_then(|m| foundation::PossessionMode::from_str(&m)),
            status: q.status.and_then(|s| match s.to_lowercase().as_str() {
                "active" => Some(SessionStatus::Active), "completed" => Some(SessionStatus::Completed),
                "archived" => Some(SessionStatus::Archived), "inconsistent" => Some(SessionStatus::Inconsistent),
                _ => None,
            }),
            limit: q.limit.or(Some(50)), offset: q.offset,
        }
    }
}

async fn list_sessions(
    State(state): State<Arc<AppState>>, Query(query): Query<SessionListQuery>,
) -> Result<Json<Vec<SessionSummary>>, (axum::http::StatusCode, Json<ApiError>)> {
    state.archive.list_sessions(&query.into()).await.map(Json).map_err(map_api_error)
}

async fn get_session_detail(
    State(state): State<Arc<AppState>>, Path(id): Path<String>,
) -> Result<Json<SessionDetail>, (axum::http::StatusCode, Json<ApiError>)> {
    state.archive.get_session_detail(&id).await.map(Json).map_err(map_api_error)
}

#[derive(Debug, Deserialize)]
struct RenameRequest { title: String }

async fn rename_session(
    State(state): State<Arc<AppState>>, Path(id): Path<String>, Json(body): Json<RenameRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<ApiError>)> {
    let mut session = state.archive.get_session_detail(&id).await.map_err(|_| (
        axum::http::StatusCode::NOT_FOUND, Json(ApiError { error: "Not found".into() })
    ))?.session;
    session.title = body.title;
    state.archive.update_session(&session).await.map_err(map_api_error)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn delete_session(
    State(state): State<Arc<AppState>>, Path(id): Path<String>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<ApiError>)> {
    state.archive.delete_session(&id).await.map_err(map_api_error)?;
    Ok((axum::http::StatusCode::OK, Json(serde_json::json!({ "ok": true }))))
}

#[derive(Debug, Deserialize)]
struct BatchDeleteRequest { ids: Vec<String> }

async fn batch_delete_sessions(
    State(state): State<Arc<AppState>>, Json(body): Json<BatchDeleteRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<ApiError>)> {
    let mut deleted = 0u32;
    let mut errors: Vec<String> = Vec::new();
    for id in &body.ids {
        match state.archive.delete_session(id).await {
            Ok(()) => deleted += 1,
            Err(e) => errors.push(format!("{}: {}", id, e)),
        }
    }
    Ok(Json(serde_json::json!({ "deleted": deleted, "errors": errors })))
}

#[derive(Debug, Deserialize)]
struct ForkRequest { task: Option<String>, from_message_seq: Option<usize> }

#[derive(Debug, Serialize)]
struct ForkResponse { session_id: String }

async fn fork_session(
    State(state): State<Arc<AppState>>, Path(id): Path<String>, Json(body): Json<ForkRequest>,
) -> Result<Json<ForkResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    let detail = state.archive.get_session_detail(&id).await.map_err(|_| (
        axum::http::StatusCode::NOT_FOUND, Json(ApiError { error: "Not found".into() })
    ))?;
    let cutoff = body.from_message_seq.unwrap_or(0);
    let history: Vec<&foundation::Message> = detail.messages.iter().filter(|m| (m.seq as usize) >= cutoff).collect();
    let new_title = body.task.unwrap_or_else(|| format!("{} (分叉)", detail.session.title));

    let new_sid = uuid::Uuid::new_v4().to_string();
    let new_session = Session {
        id: new_sid.clone(), title: new_title, mode: detail.session.mode.clone(),
        status: foundation::SessionStatus::Active,
        created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        digest_summary: None, digest_at: None,
    };
    state.archive.create_session(&new_session).await.map_err(map_api_error)?;
    for msg in history {
        let nm = foundation::Message {
            id: uuid::Uuid::new_v4().to_string(), session_id: new_sid.clone(),
            role: msg.role.clone(), soul_name: msg.soul_name.clone(),
            content: msg.content.clone(), seq: msg.seq, created_at: chrono::Utc::now(),
        };
        let _ = state.archive.append_message(&nm).await;
    }

    Ok(Json(ForkResponse { session_id: new_sid }))
}

async fn export_session_markdown(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    let detail = state.archive.get_session_detail(&id).await.map_err(map_api_error)?;
    let markdown = format_session_as_markdown(&detail);
    let filename = sanitize_filename(&detail.session.title);

    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/markdown; charset=utf-8".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}.md\"", filename).parse().unwrap(),
    );

    Ok((StatusCode::OK, headers, markdown))
}

fn sanitize_filename(title: &str) -> String {
    title.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

fn format_session_as_markdown(detail: &SessionDetail) -> String {
    let mut md = String::new();

    md.push_str(&format!("# {}\n\n", detail.session.title));
    md.push_str(&format!("**模式**: `{}`  \n", detail.session.mode.as_str()));
    md.push_str(&format!("**创建时间**: {}  \n", detail.session.created_at.format("%Y-%m-%d %H:%M:%S")));
    md.push_str(&format!("**消息数**: {}  \n", detail.messages.len()));
    md.push_str("\n---\n\n");

    for msg in &detail.messages {
        match msg.role {
            MessageRole::User => {
                md.push_str("## 👤 用户\n\n");
                md.push_str(&msg.content);
                md.push_str("\n\n");
            }
            MessageRole::Soul => {
                let soul_name = msg.soul_name.as_deref().unwrap_or("未知魂");
                md.push_str(&format!("## 🎭 {}\n\n", soul_name));
                md.push_str(&msg.content);
                md.push_str("\n\n");
            }
            MessageRole::Synthesis => {
                md.push_str("## 🧠 辩证综合\n\n");
                md.push_str(&msg.content);
                md.push_str("\n\n");
            }
            MessageRole::System => {
                md.push_str(&format!("> {}\n\n", msg.content));
            }
        }
    }

    md
}

#[derive(Debug, Deserialize)]
struct ReviewRequest {
    most_unexpected: Option<String>,
    already_known: Option<String>,
    self_negation: Option<String>,
    empty_chair: Option<String>,
    effectiveness: Option<String>,
    effectiveness_note: Option<String>,
}

async fn save_review(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<ReviewRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let review_content = serde_json::json!({
        "type": "review",
        "most_unexpected": body.most_unexpected.unwrap_or_default(),
        "already_known": body.already_known.unwrap_or_default(),
        "self_negation": body.self_negation.unwrap_or_default(),
        "empty_chair": body.empty_chair.unwrap_or_default(),
        "effectiveness": body.effectiveness.unwrap_or_default(),
        "effectiveness_note": body.effectiveness_note.unwrap_or_default(),
    });

    let msg = foundation::Message {
        id: uuid::Uuid::new_v4().to_string(),
        session_id: id,
        role: MessageRole::System,
        soul_name: None,
        content: format!("[REVIEW]{}", review_content),
        seq: 999,
        created_at: chrono::Utc::now(),
    };

    state.archive.append_message(&msg).await.map_err(map_api_error)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ── Session Digest (claude-mem 3-layer access) ──

#[derive(Debug, Serialize)]
struct SessionDigest {
    session_id: String,
    title: String,
    mode: String,
    status: String,
    created_at: DateTime<Utc>,
    summary: Option<String>,
    digest_at: Option<DateTime<Utc>>,
    observations: Vec<SessionObservation>,
    total_read_tokens: u32,
    total_work_tokens: u32,
    savings_ratio: f32,
}

async fn get_session_digest(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SessionDigest>, (StatusCode, Json<ApiError>)> {
    let detail = state.archive.get_session_detail(&id).await
        .map_err(|_| (StatusCode::NOT_FOUND, Json(ApiError { error: "Not found".into() })))?;
    let session = detail.session;
    let obs = state.archive.get_session_observations(&id).await.map_err(map_api_error)?;
    let total_read: u32 = obs.iter().map(|o| o.read_tokens).sum();
    let total_work: u32 = obs.iter().map(|o| o.work_tokens).sum();
    let savings_ratio = if total_work > 0 {
        1.0 - (total_read as f32 / total_work as f32)
    } else {
        0.0
    };
    Ok(Json(SessionDigest {
        session_id: id,
        title: session.title,
        mode: session.mode.as_str().to_string(),
        status: session.status.as_str().to_string(),
        created_at: session.created_at,
        summary: session.digest_summary,
        digest_at: session.digest_at,
        observations: obs,
        total_read_tokens: total_read,
        total_work_tokens: total_work,
        savings_ratio: savings_ratio.max(0.0),
    }))
}

async fn trigger_distill(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let detail = state.archive.get_session_detail(&id).await
        .map_err(|_| (StatusCode::NOT_FOUND, Json(ApiError { error: "Not found".into() })))?;

    // Skip if already distilled
    if detail.session.digest_summary.is_some() {
        return Ok(Json(serde_json::json!({
            "ok": true,
            "message": "Already distilled",
            "summary": detail.session.digest_summary
        })));
    }

    let messages = state.archive.store().get_messages(&id).await.map_err(map_api_error)?;
    if messages.is_empty() {
        return Ok(Json(serde_json::json!({ "ok": false, "message": "No messages to distill" })));
    }

    // Trigger distill via possession engine's gateway
    possession::distiller::spawn_distill(
        state.archive.store(),
        state.engine.gateway().clone(),
        state.engine.ws_manager().clone(),
        id.clone(),
    );

    Ok(Json(serde_json::json!({ "ok": true, "message": "Distill started" })))
}
