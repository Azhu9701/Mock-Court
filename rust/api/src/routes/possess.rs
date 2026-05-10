use std::sync::Arc;

use axum::extract::{Multipart, Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use foundation::{LLMRequest, CallConfig, Prompt, PromptMessage, Provider, SoulListEntry, SoulProfile};
use possession::PossessionInput;
use serde::{Deserialize, Serialize};

use crate::error::{map_api_error, ApiError};
use crate::ocr;
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(start_possession))
        .route("/analyze", post(analyze_task))
        .route("/ocr", post(ocr_upload))
        .route("/:session_id/status", get(possession_status))
        .route("/:session_id/follow-up", post(follow_up))
}

#[derive(Debug, Deserialize)]
struct StartPossessionRequest {
    #[serde(default)] mode: Option<String>,
    task: String,
    #[serde(default)] souls: Vec<String>,
    #[serde(default)] topic: Option<String>,
    #[serde(default)] judgment: Option<String>,
    #[serde(default)] worry: Option<String>,
    #[serde(default)] unknown: Option<String>,
    #[serde(default)] task_cards: std::collections::HashMap<String, String>,
    #[serde(default)] search_topic: bool,
}

#[derive(Debug, Serialize)]
struct StartPossessionResponse { session_id: String, mode: String, ws_url: String }

async fn start_possession(
    State(state): State<Arc<AppState>>,
    Json(body): Json<StartPossessionRequest>,
) -> Result<Json<StartPossessionResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    if body.task.trim().is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, Json(ApiError { error: "task is required".into() })));
    }
    tracing::info!("Starting possession with {} souls, search_topic={}", body.souls.len(), body.search_topic);
    
    // 议题搜索：在召唤魂之前，用任务关键词实时搜索 Web 获取背景信息
    let search_results = if body.search_topic {
        tracing::info!("Running topic search for: {}", &body.task[..body.task.len().min(80)]);
        match state.collector.search_topic(&body.task, None, 3).await {
            Ok(md) => {
                tracing::info!("Topic search returned {} bytes", md.len());
                Some(md)
            }
            Err(e) => {
                tracing::warn!("Topic search failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    let input = PossessionInput {
        mode: body.mode.and_then(|m| foundation::PossessionMode::from_str(&m)),
        task: body.task, souls: body.souls, topic: body.topic,
        judgment: body.judgment, worry: body.worry, unknown: body.unknown,
        task_cards: body.task_cards,
        search_topic: body.search_topic,
        search_results,
    };
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let session_id = state.engine.start_possession(input, tx).await.map_err(map_api_error)?;
    tracing::info!("Possession session created: {}", session_id);

    // Relay: forward events from dispatch_mode to WebSocket subscribers
    let ws = state.engine.ws_manager().clone();
    let sid = session_id.clone();
    tokio::spawn(async move {
        let mut rx = rx;
        while let Some(event) = rx.recv().await {
            ws.broadcast_system(&sid, &event);
        }
        // Don't remove session - keep it for follow-up questions
    });

    Ok(Json(StartPossessionResponse {
        ws_url: format!("/ws/possess/{}/main", session_id), session_id, mode: "unknown".into(),
    }))
}

// ── AI Analyze: Matching + Review as separate sub-agent calls ──

#[derive(Debug, Deserialize)]
struct AnalyzeRequest { task: String, #[serde(default)] judgment: Option<String>, #[serde(default)] worry: Option<String>, #[serde(default)] unknown: Option<String> }

#[derive(Debug, Serialize)]
struct AnalyzeResponse { entry_type: String, matched_souls: Vec<SoulMatch>, recommended_mode: String, review: ReviewResult, #[serde(default)] task_cards: std::collections::HashMap<String, String> }

#[derive(Debug, Clone, Serialize)]
struct SoulMatch { name: String, field: String, ismism_code: String, rationale: String }

#[derive(Debug, Serialize)]
struct ReviewResult { verdict: String, checks: Vec<String>, notes: String, reviewer: String }

fn pick_provider(state: &AppState) -> Result<Provider, (axum::http::StatusCode, Json<ApiError>)> {
    state.engine.gateway().list_providers().into_iter().find(|i| i.available).map(|i| i.provider)
        .ok_or_else(|| {
            tracing::error!("No LLM provider available");
            (axum::http::StatusCode::SERVICE_UNAVAILABLE, Json(ApiError { error: "No LLM provider".into() }))
        })
}

/// Extract a JSON object from text that may contain reasoning prose before/after the JSON.
fn extract_json(text: &str) -> Option<&str> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end > start {
        Some(&text[start..=end])
    } else {
        None
    }
}

fn build_soul_match(js: &serde_json::Value, all_souls: &[SoulListEntry]) -> Option<SoulMatch> {
    let name = js["name"].as_str()?;
    all_souls.iter().find(|e| e.name == name).map(|entry| SoulMatch {
        name: name.into(),

        field: entry.field.clone(),
        ismism_code: entry.ismism_code.clone(),
        rationale: js["rationale"].as_str().unwrap_or("").into(),
    })
}

async fn llm_call_json(state: &AppState, provider: Provider, system_prompt: Option<&str>, user_prompt: &str, temp: f64, max_tokens: u32, model: Option<&str>) -> Result<serde_json::Value, (axum::http::StatusCode, Json<ApiError>)> {
    let mut messages = Vec::new();
    if let Some(sys) = system_prompt {
        messages.push(PromptMessage { role: "system".into(), content: sys.into(), reasoning_content: None });
    }
    messages.push(PromptMessage { role: "user".into(), content: user_prompt.into(), reasoning_content: None });

    let req = LLMRequest { provider, prompt: Prompt { messages }, config: CallConfig { temperature: temp, max_tokens, stream: false, model: model.map(String::from), reasoning_effort: None, structured_output: None, thinking_enabled: None } };
    let mut rx = state.engine.gateway().call(&req).map_err(|e| {
        tracing::error!("LLM call failed: {}", e);
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() }))
    })?;

    let mut resp = String::new();
    while let Some(result) = rx.recv().await {
        if let Ok(chunk) = result { resp.push_str(&chunk.content); }
    }
    // Extract JSON from response — DeepSeek reasoning models may prepend thinking text
    let json_str = extract_json(&resp).unwrap_or(&resp);
    tracing::debug!(len = resp.len(), json_len = json_str.len(), "llm_call_json response");
    Ok(serde_json::from_str(json_str).unwrap_or_else(|e| {
        // Safely truncate to avoid char boundary errors
        let trunc_len = resp.len().min(300);
        let safe_idx = (0..=trunc_len).rev().find(|&i| resp.is_char_boundary(i)).unwrap_or(0);
        tracing::warn!("llm_call_json parse failed: {} | raw[..{}]={}", e, safe_idx, &resp[..safe_idx]);
        serde_json::Value::default()
    }))
}

async fn analyze_task(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AnalyzeRequest>,
) -> Result<Json<AnalyzeResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    // Safely truncate to avoid char boundary errors
    let trunc_len = body.task.len().min(50);
    let safe_idx = (0..=trunc_len).rev().find(|&i| body.task.is_char_boundary(i)).unwrap_or(0);
    tracing::info!("Starting task analysis for: {}", &body.task[..safe_idx]);
    
    // Entry classification with improved detection
    let has_specific = body.task.contains("我") && (body.task.contains("做了") || body.task.contains("正在") || body.task.contains("经历过") || body.task.contains("我公司") || body.task.contains("我的项目") || body.task.contains("我的工厂"));
    let has_first_person = body.task.starts_with("我");
    let has_concrete = !body.task.contains("通常") && !body.task.contains("一般") && (body.task.contains("上次") || body.task.contains("最近") || body.task.contains("今天") || body.task.contains("昨天") || body.task.contains("这周"));
    let score = [has_specific, has_first_person, has_concrete].iter().filter(|&&x| x).count();

    tracing::debug!("Practice opening score: {}", score);

    if score >= 2 {
        tracing::info!("Practice opening detected");
        return Ok(Json(AnalyzeResponse { 
            entry_type: "practice_opening".into(), 
            matched_souls: vec![], 
            recommended_mode: "practice_opening".into(), 
            review: ReviewResult { 
                verdict: "practice_opening".into(), 
                checks: vec![], 
                notes: String::new(), 
                reviewer: String::new() 
            },
            task_cards: Default::default(),
        }));
    }

    let provider = pick_provider(&state)?;
    tracing::debug!("Using provider: {:?}", provider);
    
    let all_souls = state.registry.list_souls(&foundation::IsmismFilter::default()).map_err(map_api_error)?;
    let task_lower = body.task.to_lowercase();

    // Pre-filter: score by trigger keyword overlap, include self_declare boundary info
    let soul_list: Vec<String> = all_souls.iter().map(|s| {
        let kw_hits = s.trigger_keywords.iter().filter(|kw| task_lower.contains(&kw.to_lowercase())).count();
        let kws: Vec<&str> = s.trigger_keywords.iter().take(4).map(|x| x.as_str()).collect();
        let declare_short: String = s.self_declare.chars().take(60).collect();
        format!("- {} field={} code={} kw_hits={} kws=[{}] self=\"{}\"",
            s.name, s.field, s.ismism_code, kw_hits, kws.join(","), declare_short)
    }).collect();

    let provider2 = provider.clone();
    let provider3 = provider.clone();

    // ── Step 1: Match (matcher agent) ──
    tracing::info!("Step 1: Matching souls");
    let match_prompt = format!("## 任务\n{}\n\n## 魂列表（含触发关键词命中数和 self_declare 边界声明）\n{}\n\n## 指令\n根据任务和魂的触发关键词命中数、边界声明，选择最匹配的2-5个魂。优先选 kw_hits 高的。每个魂的 self_declare 声明了该魂的能力边界——如果任务超出边界则不应选择。返回JSON：{{\"souls\":[{{\"name\":\"魂名\",\"rationale\":\"理由\"}}],\"mode\":\"single|conference|debate\"}}", body.task, soul_list.join("\n"));
    let matched = llm_call_json(&state, provider.clone(), None, &match_prompt, 0.3, 4096, None).await?;

    let provider = provider2;

    let souls: Vec<SoulMatch> = matched["souls"].as_array()
        .map(|arr| arr.iter().filter_map(|s| build_soul_match(s, &all_souls)).collect())
        .unwrap_or_default();
    let mode = matched["mode"].as_str().unwrap_or("conference").to_string();
    
    tracing::info!("Matched {} souls, mode: {}", souls.len(), mode);

    // ── Step 2: 幡主审查 + 差异化任务分派 ──
    tracing::info!("Step 2: Banner lord review + task card assignment");
    let reviewer_name = std::env::var("AIONUI_REVIEWER_SOUL").unwrap_or_else(|_| "未明子".to_string());
    let mut task_cards: std::collections::HashMap<String, String> = Default::default();
    let mut missing_perspectives: Vec<String> = Vec::new();
    let mut boundary_risks: Vec<String> = Vec::new();

    let (verdict, checks, notes) = if let Ok(banner_lord) = state.registry.get_soul(&reviewer_name) {
        let candidate_profiles: Vec<SoulProfile> = souls.iter()
            .filter_map(|s| state.registry.get_soul(&s.name).ok())
            .collect();
        
        let review_system = format!(
            "{}你是{}，ismism坐标{}。你作为幡主审查官，需要完成两项任务：\n\
             1. 审查候选魂是否适合这个任务——不适合的要去掉或替换\n\
             2. 为每个确定使用的魂分派一个**差异化的子问题**——不是所有人分析同一个问题，\
             而是把你的总任务拆解成每个魂最擅长回答的那一个侧面\n\n\
             不读取文件——所有上下文已在 prompt 中。",
            banner_lord.summon_prompt, banner_lord.name, banner_lord.ismism_code
        );

        let mut candidates_info = String::new();
        for p in &candidate_profiles {
            let exclude_str = p.exclude_scenarios.join("、");
            candidates_info.push_str(&format!(
                "- **{}** [{}] ismism={} self_declare=\"{}\" exclude_scenarios=\"{}\"\n",
                p.name, p.field, p.ismism_code,
                if p.self_declare.is_empty() { "无" } else { &p.self_declare },
                if p.exclude_scenarios.is_empty() { "无" } else { &exclude_str }
            ));
        }

        let review_user = format!(
            "## 总任务\n{}\n\n## 使用者预设\n判断：{}\n担忧：{}\n未知：{}\n\n## 候选魂\n{}\n\n\
             ## 你的两阶段任务\n\n\
             ### 第一阶段：审查魂组合\n\
             逐魂检查：领域覆盖、场域定位、魂间互补、视角缺失。裁决：pass / conditional / reject\n\n\
             ### 第二阶段：差异化任务分派\n\
             为每个确认使用的魂分配一个**只有他能回答好的子问题**。原则：\n\
             - 利用场域差异——场域1做地基（\"这是什么\"），场域2做边界（\"这看不到什么\"），\
             场域3做自反（\"这个问法本身有什么问题\"），场域4做实践（\"怎么落地\"）\n\
             - 每个子问题要具体（\"请回答：X在Y条件下的Z\"），不要\"请分析\"这种空指令\n\n\
             返回JSON：\n\
             {{\"verdict\":\"pass|conditional|reject\",\
             \"verified_souls\":[\"魂名\"],\
             \"task_cards\":{{\"魂名\":\"专属子问题\"}},\
             \"checks\":[\"审查结果\"],\
             \"notes\":\"审查备注\",\
             \"missing_perspectives\":[\"缺失视角\"],\
             \"boundary_risks\":[\"边界风险\"]}}",
            body.task,
            body.judgment.as_deref().unwrap_or("无"),
            body.worry.as_deref().unwrap_or("无"),
            body.unknown.as_deref().unwrap_or("无"),
            candidates_info
        );
        
        let result = llm_call_json(&state, provider, Some(&review_system), &review_user, 0.3, 4096, None).await?;

        // Parse task cards
        if let Some(cards) = result["task_cards"].as_object() {
            for (k, v) in cards {
                if let Some(val) = v.as_str() {
                    task_cards.insert(k.clone(), val.to_string());
                }
            }
        }
        missing_perspectives = result["missing_perspectives"].as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        boundary_risks = result["boundary_risks"].as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let v = result["verdict"].as_str().unwrap_or("pass").to_string();
        let c: Vec<String> = result["checks"].as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let n = if !missing_perspectives.is_empty() {
            format!("{} | 缺失视角：{} | 边界风险：{}", 
                result["notes"].as_str().unwrap_or(""),
                missing_perspectives.join("、"),
                boundary_risks.join("、"))
        } else {
            result["notes"].as_str().unwrap_or("").to_string()
        };
        (v, c, n)
    } else {
        tracing::warn!("Reviewer soul not found, defaulting to pass");
        ("pass".to_string(), vec!["审查官不可用，默认通过".into()], String::new())
    };

    tracing::info!("Review verdict: {}, task cards for {} souls", verdict, task_cards.len());

    // ── Step 3: Apply Review Verdict ──
    let final_souls = if verdict == "pass" {
        souls
    } else {
        tracing::info!("Step 3: Adjusting soul combination based on review");
        let adjustment_prompt = format!(
            "## 任务\n{}\n\n## 当前魂组合\n{}\n\n## 幡主审查结果\n裁决：{}\n{}\n备注：{}\n\n## 指令\n根据审查结果调整魂组合。如果是条件通过——增删魂以满足约束。如果是拒绝——完全重新匹配。\n返回JSON：{{\"souls\":[{{\"name\":\"魂名\",\"rationale\":\"调整理由\"}}]}}",
            body.task,
            souls.iter().map(|s| format!("- {} [{}] {}", s.name, s.field, s.rationale)).collect::<Vec<_>>().join("\n"),
            verdict, checks.join("\n"), notes
        );
        match llm_call_json(&state, provider3, None, &adjustment_prompt, 0.3, 4096, None).await {
            Ok(adjusted) => {
                let adjusted_souls = adjusted["souls"].as_array()
                    .map(|arr| arr.iter().filter_map(|s| build_soul_match(s, &all_souls)).collect())
                    .unwrap_or(souls.clone());
                // Re-assign task cards for adjusted set
                let souls_changed = adjusted_souls.iter().any(|s| !souls.iter().any(|orig| orig.name == s.name))
                    || souls.iter().any(|orig| !adjusted_souls.iter().any(|s| s.name == orig.name));
                if souls_changed && !adjusted_souls.is_empty() {
                    let adjusted_names: Vec<String> = adjusted_souls.iter().map(|s| s.name.clone()).collect();
                    let adjusted_profiles: Vec<SoulProfile> = adjusted_souls.iter()
                        .filter_map(|s| state.registry.get_soul(&s.name).ok())
                        .collect();
                    if let Ok(banner_lord) = state.registry.get_soul(&reviewer_name) {
                        let rs = format!(
                            "{}你是{}，ismism坐标{}。你作为幡主审查官，为调整后的魂组合分配差异化子问题。",
                            banner_lord.summon_prompt, banner_lord.name, banner_lord.ismism_code
                        );
                        let p_info: Vec<String> = adjusted_profiles.iter().map(|p| {
                            format!("- {} [{}] ismism={}", p.name, p.field, p.ismism_code)
                        }).collect();
                        let ru = format!(
                            "## 总任务\n{}\n\n## 调整后的魂组合\n{}\n\n为每个魂分配一个只有他能回答好的子问题。\n返回JSON：{{\"task_cards\":{{\"魂名\":\"专属子问题\"}}}}",
                            body.task, p_info.join("\n")
                        );
                        if let Ok(tc) = llm_call_json(&state, provider.clone(), Some(&rs), &ru, 0.3, 4096, None).await {
                            if let Some(cards) = tc["task_cards"].as_object() {
                                task_cards.clear();
                                for (k, v) in cards {
                                    if let Some(val) = v.as_str() {
                                        task_cards.insert(k.clone(), val.to_string());
                                    }
                                }
                            }
                        }
                    }
                    tracing::info!("Re-assigned task cards for {} adjusted souls", adjusted_names.len());
                }
                tracing::info!("Adjusted to {} souls", adjusted_souls.len());
                adjusted_souls
            }
            Err(e) => {
                tracing::warn!("Adjustment failed, falling back to original: {:?}", e);
                souls
            }
        }
    };

    tracing::info!("Analysis complete, returning {} souls", final_souls.len());

    Ok(Json(AnalyzeResponse {
        entry_type: if score == 1 { "hybrid".into() } else { "conventional".into() },
        matched_souls: final_souls,
        recommended_mode: mode,
        review: ReviewResult { verdict, checks, notes, reviewer: reviewer_name.into() },
        task_cards,
    }))
}

// ── Follow-up ──

#[derive(Debug, Deserialize)]
struct FollowUpRequest { question: String }

async fn follow_up(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(body): Json<FollowUpRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<ApiError>)> {
    tracing::info!("Received follow-up request for session: {}", session_id);
    
    // 尝试获取会话，如果不存在，不会直接返回错误，而是使用空历史记录
    let history = match state.archive.get_session_detail(&session_id).await {
        Ok(session) => {
            tracing::info!("Session found, have {} messages", session.messages.len());
            session.messages.iter().map(|m| format!("[{:?}] {}: {}", m.role, m.soul_name.as_deref().unwrap_or("系统"), m.content)).collect::<Vec<_>>()
        }
        Err(e) => {
            tracing::warn!("Session not found or error: {}, will proceed with basic prompt", e);
            // 即使没有会话历史，也可以进行简单的 follow-up
            vec![]
        }
    };
    
    // 构建 prompt - 即使没有历史记录也可以工作
    let prompt_str = if history.is_empty() {
        format!("## 新问题\n{}\n\n根据这个问题，以你的立场和视角回应。", body.question)
    } else {
        format!("## 历史对话\n{}\n\n## 新问题\n{}\n\n根据历史对话上下文，以你的立场和视角回应。", history.join("\n\n"), body.question)
    };

    let question = body.question.clone();
    let gateway = state.engine.gateway().clone();
    let provider = pick_provider(&state)?;
    let ws = state.engine.ws_manager().clone();
    let archive = state.archive.clone();
    let sid = session_id.clone();

    // Ensure session exists in WS manager (for follow-up after main session has completed)
    tracing::info!("Creating/ensuring WS session exists");
    ws.create_session(&sid);

    // 如果会话存在，持久化用户的问题
    if state.archive.get_session_detail(&session_id).await.is_ok() {
        let q_msg = foundation::Message {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: sid.clone(),
            role: foundation::MessageRole::User,
            soul_name: None,
            content: question.clone(),
            seq: 990,
            created_at: chrono::Utc::now(),
        };
        if let Err(e) = archive.append_message(&q_msg).await {
            tracing::error!("Failed to store user question: {}", e);
        }
    }

    // Start LLM call immediately, don't wait for WS connection - messages will buffer
    tokio::spawn(async move {
        tracing::info!("Starting LLM call for follow-up immediately");
        
        let config = CallConfig::default().with_reasoning_effort(foundation::ReasoningEffort::ThinkMax);
        let req = LLMRequest { provider, prompt: Prompt { messages: vec![PromptMessage { role: "user".into(), content: prompt_str, reasoning_content: None }] }, config };
        
        match gateway.call(&req) {
            Ok(mut rx) => {
                tracing::info!("LLM call started successfully, streaming synthesis");
                match possession::stream::stream_synthesis(rx, &sid, &ws).await {
                    Ok((content, _)) => {
                        tracing::info!("Follow-up synthesis complete, content length: {}", content.len());
                        // 只有当会话存在时才持久化
                        if state.archive.get_session_detail(&session_id).await.is_ok() {
                            let msg = foundation::Message {
                                id: uuid::Uuid::new_v4().to_string(),
                                session_id: sid,
                                role: foundation::MessageRole::Synthesis,
                                soul_name: None,
                                content,
                                seq: 991,
                                created_at: chrono::Utc::now(),
                            };
                            if let Err(e) = archive.append_message(&msg).await {
                                tracing::error!("Failed to store follow-up: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Follow-up stream error: {}", e);
                        ws.broadcast_system(&sid, &possession::WsEvent {
                            event_type: possession::WsEventType::Error,
                            payload: format!("流式响应错误: {}", e),
                            reasoning_content: None,
                            soul_name: None,
                            seq: 0,
                        });
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to start LLM call: {}", e);
                ws.broadcast_system(&sid, &possession::WsEvent {
                    event_type: possession::WsEventType::Error,
                    payload: format!("启动 LLM 失败: {}", e),
                    reasoning_content: None,
                    soul_name: None,
                    seq: 0,
                });
            }
        }
    });
    
    tracing::info!("Follow-up request accepted, returning response");
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ── Status ──

#[derive(Debug, Serialize)]
struct PossessionStatusResponse { session_id: String, connected: bool }

async fn possession_status(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Json<PossessionStatusResponse> {
    Json(PossessionStatusResponse { session_id: session_id.clone(), connected: state.engine.ws_manager().has_session(&session_id) })
}

// ── OCR ──

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
const MAX_FILES: usize = 5;
const ALLOWED_MIMES: &[&str] = &["image/png", "image/jpeg", "image/webp", "image/gif"];

#[derive(Debug, Serialize)]
struct OcrResultItem {
    filename: String,
    text: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct OcrUploadResponse {
    results: Vec<OcrResultItem>,
}

async fn ocr_upload(
    State(_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<OcrUploadResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    let mut file_data: Vec<(String, Vec<u8>)> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let Some(filename) = field.file_name().map(String::from) else { continue; };
        let Some(content_type) = field.content_type().map(String::from) else { continue; };

        if !ALLOWED_MIMES.contains(&content_type.as_str()) {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                Json(ApiError { error: format!("不支持的文件类型: {}", content_type) }),
            ));
        }

        let Ok(data) = field.bytes().await else { continue; };
        if data.len() > MAX_FILE_SIZE {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                Json(ApiError { error: format!("文件 {} 超过 10MB 限制", filename) }),
            ));
        }

        file_data.push((filename, data.to_vec()));
        if file_data.len() >= MAX_FILES { break; }
    }

    if file_data.is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(ApiError { error: "未收到有效文件".into() }),
        ));
    }

    let mut handles = Vec::new();
    for (filename, data) in file_data {
        let handle = tokio::task::spawn_blocking(move || {
            let text = ocr::ocr_image(&data, "chi_sim+eng");
            (filename, text)
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok((filename, Ok(text))) => {
                let filtered = text.trim();
                results.push(OcrResultItem {
                    filename,
                    text: if filtered.is_empty() { None } else { Some(filtered.to_string()) },
                    error: None,
                });
            }
            Ok((filename, Err(e))) => {
                results.push(OcrResultItem { filename, text: None, error: Some(e) });
            }
            Err(e) => {
                results.push(OcrResultItem {
                    filename: "unknown".into(),
                    text: None,
                    error: Some(format!("spawn_blocking 错误: {}", e)),
                });
            }
        }
    }

    Ok(Json(OcrUploadResponse { results }))
}
