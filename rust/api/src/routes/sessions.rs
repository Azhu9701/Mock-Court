use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use axum::http::{header, StatusCode};
use archive::SessionDetail;
use chrono::{DateTime, Utc};
use foundation::{Annotation, MessageRole, Session, SessionFilter, SessionObservation, SessionReview, SessionStatus, SessionSummary};
use serde::{Deserialize, Serialize};

use crate::error::{map_api_error, ApiError};
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_sessions))
        .route("/batch-delete", post(batch_delete_sessions))
        .route("/:id", get(get_session_detail).delete(delete_session))
        .route("/:id/digest", get(get_session_digest))
        .route("/:id/annotations", get(get_session_annotations))
        .route("/:id/distill", post(trigger_distill))
        .route("/:id/rename", put(rename_session))
        .route("/:id/fork", put(fork_session))
        .route("/:id/export/markdown", get(export_session_markdown))
        .route("/reviews/profile", get(get_user_profile))
        .route("/:id/review", get(get_session_review).post(save_review))
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
struct ForkResponse { session_id: String, forked_message_count: usize }

async fn fork_session(
    State(state): State<Arc<AppState>>, Path(id): Path<String>, Json(body): Json<ForkRequest>,
) -> Result<Json<ForkResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    let detail = state.archive.get_session_detail(&id).await.map_err(|_| (
        axum::http::StatusCode::NOT_FOUND, Json(ApiError { error: "Not found".into() })
    ))?;
    let cutoff = body.from_message_seq.unwrap_or(0);
    let history: Vec<&foundation::Message> = detail.messages.iter()
        .filter(|m| (m.seq as usize) <= cutoff)
        .collect();
    let new_title = body.task.unwrap_or_else(|| format!("{} (分叉)", detail.session.title));

    let new_sid = uuid::Uuid::new_v4().to_string();
    let new_session = Session {
        id: new_sid.clone(), title: new_title, mode: detail.session.mode.clone(),
        status: foundation::SessionStatus::Active,
        created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        digest_summary: None, digest_at: None,
    };
    state.archive.create_session(&new_session).await.map_err(map_api_error)?;
    for (i, msg) in history.iter().enumerate() {
        let nm = foundation::Message {
            id: uuid::Uuid::new_v4().to_string(), session_id: new_sid.clone(),
            role: msg.role.clone(), soul_name: msg.soul_name.clone(),
            content: msg.content.clone(), seq: i as u32, created_at: chrono::Utc::now(),
        };
        let _ = state.archive.append_message(&nm).await;
    }

    Ok(Json(ForkResponse { session_id: new_sid, forked_message_count: history.len() }))
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
    practice_commitment: Option<String>,
    practice_horizon: Option<String>,
    #[serde(default)]
    interrogation_passed: Option<bool>,
    #[serde(default)]
    interrogation_reason: Option<String>,
}

async fn save_review(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<ReviewRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let mu = body.most_unexpected.clone().unwrap_or_default();
    let ak = body.already_known.clone().unwrap_or_default();
    let sn = body.self_negation.clone().unwrap_or_default();
    let ec = body.empty_chair.clone().unwrap_or_default();
    let ef = body.effectiveness.clone().unwrap_or_default();
    let en = body.effectiveness_note.clone().unwrap_or_default();
    let pc = body.practice_commitment.clone().unwrap_or_default();
    let ph = body.practice_horizon.clone().unwrap_or_default();

    let review_content = serde_json::json!({
        "type": "review",
        "most_unexpected": mu,
        "already_known": ak,
        "self_negation": sn,
        "empty_chair": ec,
        "effectiveness": ef,
        "effectiveness_note": en,
        "practice_commitment": pc,
        "practice_horizon": ph,
    });

    let review_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    // 写入 reviews 表（独立持久化，供匹配/蒸馏/画像消费）
    let review = foundation::SessionReview {
        id: review_id,
        session_id: id.clone(),
        most_unexpected: mu,
        already_known: ak,
        self_negation: sn,
        empty_chair: ec,
        effectiveness: ef,
        effectiveness_note: en,
        practice_commitment: pc,
        practice_horizon: ph,
        interrogation_passed: body.interrogation_passed,
        interrogation_reason: body.interrogation_reason.clone(),
        created_at: now,
    };
    if let Err(e) = state.archive.insert_session_review(&review).await {
        tracing::error!("Failed to write session review: {}", e);
    }

    // 保留 [REVIEW] message 作为兼容（历史数据也可见）
    let msg = foundation::Message {
        id: uuid::Uuid::new_v4().to_string(),
        session_id: id,
        role: MessageRole::System,
        soul_name: None,
        content: format!("[REVIEW]{}", review_content),
        seq: 999,
        created_at: now,
    };
    state.archive.append_message(&msg).await.map_err(map_api_error)?;
    Ok(Json(serde_json::json!({ "ok": true, "review_id": review.id })))
}

/// GET /sessions/reviews/profile
/// 使用者画像 — 聚合所有 review 数据，产出模式摘要
async fn get_user_profile(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let reviews = state.archive.get_recent_reviews(50).await.map_err(map_api_error)?;
    if reviews.is_empty() {
        return Ok(Json(serde_json::json!({"reviews": 0})));
    }

    let total = reviews.len();
    let effective = reviews.iter().filter(|r| r.effectiveness == "effective").count();
    let ineffective = reviews.iter().filter(|r| r.effectiveness == "invalid").count();

    let top_negations = top_phrases(reviews.iter().map(|r| r.self_negation.as_str()), 5);
    let top_chairs = top_phrases(reviews.iter().map(|r| r.empty_chair.as_str()), 5);

    Ok(Json(serde_json::json!({
        "total_reviews": total,
        "effective_rate": if total > 0 { effective as f64 / total as f64 } else { 0.0 },
        "ineffective_count": ineffective,
        "top_shaken_presets": top_negations.iter().map(|(w, c)| serde_json::json!({"phrase": w, "count": c})).collect::<Vec<_>>(),
        "top_missing_voices": top_chairs.iter().map(|(w, c)| serde_json::json!({"phrase": w, "count": c})).collect::<Vec<_>>(),
    })))
}

async fn get_session_review(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Option<SessionReview>>, (StatusCode, Json<ApiError>)> {
    let review = state.archive.get_session_review(&id).await.map_err(map_api_error)?;
    Ok(Json(review))
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

async fn get_session_annotations(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Annotation>>, (StatusCode, Json<ApiError>)> {
    let annotations = state.archive.get_annotations(&id).await.map_err(map_api_error)?;
    Ok(Json(annotations))
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

fn top_phrases<'a>(texts: impl Iterator<Item = &'a str>, limit: usize) -> Vec<(&'a str, u32)> {
    let mut counts: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
    for text in texts {
        if text.is_empty() { continue; }
        for word in text.split(|c: char| c == '，' || c == '。' || c == '、' || c == '；') {
            let w = word.trim();
            if w.len() >= 2 && w.len() <= 30 {
                *counts.entry(w).or_default() += 1;
            }
        }
    }
    let mut top: Vec<_> = counts.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1));
    top.truncate(limit);
    top
}
