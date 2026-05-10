use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, put};
use axum::{Json, Router};
use axum::http::{header, StatusCode};
use archive::SessionDetail;
use foundation::{MessageRole, Session, SessionFilter, SessionStatus, SessionSummary};
use serde::{Deserialize, Serialize};

use crate::error::{map_api_error, ApiError};
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_sessions))
        .route("/:id", get(get_session_detail).delete(delete_session))
        .route("/:id/rename", put(rename_session))
        .route("/:id/fork", put(fork_session))
        .route("/:id/export/markdown", get(export_session_markdown))
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
