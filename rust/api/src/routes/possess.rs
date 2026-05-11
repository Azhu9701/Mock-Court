use std::sync::Arc;

use axum::extract::{Multipart, Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use foundation::{LLMRequest, CallConfig, Prompt, PromptMessage, Provider, SoulListEntry, SoulProfile};
use possession::PossessionInput;
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt as _;

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
    
    // 议题搜索：在召唤魂之前，通过 SearXNG 实时搜索 Web 获取背景信息
    let search_results = if body.search_topic {
        tracing::info!("Running SearXNG topic search for: {}", &body.task[..body.task.len().min(80)]);
        match state.collector.search_topic_searxng(&body.task, 3).await {
            Ok(md) => {
                tracing::info!("SearXNG topic search returned {} bytes", md.len());
                Some(md)
            }
            Err(e) => {
                tracing::warn!("SearXNG topic search failed: {}", e);
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
    let (tx, rx) = tokio::sync::mpsc::channel::<possession::WsEvent>(256);
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
struct AnalyzeRequest { task: String, #[serde(default)] judgment: Option<String>, #[serde(default)] worry: Option<String>, #[serde(default)] unknown: Option<String>, #[serde(default)] reviewer: Option<String> }

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
        messages.push(PromptMessage { role: "system".into(), content: sys.into(), reasoning_content: None, tool_call_id: None, tool_calls: None });
    }
    messages.push(PromptMessage { role: "user".into(), content: user_prompt.into(), reasoning_content: None, tool_call_id: None, tool_calls: None });

    let req = LLMRequest { provider, prompt: Prompt { messages }, config: CallConfig { temperature: temp, max_tokens, stream: false, model: model.map(String::from), reasoning_effort: None, structured_output: None, thinking_enabled: None, tools: None, tool_choice: None } };
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

fn spawn_banner_lord_review(
    gateway: Arc<ai_gateway::GatewayRegistry>,
    banner_lord: SoulProfile,
    task: String,
    candidate_profiles: Vec<SoulProfile>,
    judgment: String,
    worry: String,
    unknown: String,
) -> tokio::task::JoinHandle<Result<serde_json::Value, String>> {
    tokio::spawn(async move {
        let provider = gateway
            .list_providers()
            .into_iter()
            .find(|i| i.available)
            .map(|i| i.provider)
            .ok_or_else(|| "No LLM provider available".to_string())?;

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
            task,
            &judgment,
            &worry,
            &unknown,
            candidates_info
        );

        let messages = vec![
            PromptMessage {
                role: "system".into(),
                content: review_system,
                reasoning_content: None,
                tool_call_id: None,
                tool_calls: None,
            },
            PromptMessage {
                role: "user".into(),
                content: review_user,
                reasoning_content: None,
                tool_call_id: None,
                tool_calls: None,
            },
        ];

        let req = LLMRequest {
            provider,
            prompt: Prompt { messages },
            config: CallConfig {
                temperature: 0.3,
                max_tokens: 4096,
                stream: false,
                model: None,
                reasoning_effort: None,
                structured_output: None,
                thinking_enabled: None,
                tools: None,
                tool_choice: None,
            },
        };

        let mut rx = gateway
            .call(&req)
            .map_err(|e| format!("LLM call failed: {}", e))?;
        let mut resp = String::new();
        while let Some(result) = rx.recv().await {
            if let Ok(chunk) = result {
                resp.push_str(&chunk.content);
            }
        }

        let json_str = extract_json(&resp).unwrap_or(&resp);
        Ok(serde_json::from_str(json_str).unwrap_or_default())
    })
}

async fn analyze_task(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AnalyzeRequest>,
) -> Sse<UnboundedReceiverStream<Result<Event, std::convert::Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<Event, std::convert::Infallible>>();

    tokio::spawn(async move {
        let send = |data: String| {
            let _ = tx.send(Ok(Event::default().data(data)));
        };

        let trunc_len = body.task.len().min(50);
        let safe_idx = (0..=trunc_len).rev().find(|&i| body.task.is_char_boundary(i)).unwrap_or(0);
        tracing::info!("Starting task analysis for: {}", &body.task[..safe_idx]);

        // Entry classification
        let has_specific = body.task.contains("我") && (body.task.contains("做了") || body.task.contains("正在") || body.task.contains("经历过") || body.task.contains("我公司") || body.task.contains("我的项目") || body.task.contains("我的工厂"));
        let has_first_person = body.task.starts_with("我");
        let has_concrete = !body.task.contains("通常") && !body.task.contains("一般") && (body.task.contains("上次") || body.task.contains("最近") || body.task.contains("今天") || body.task.contains("昨天") || body.task.contains("这周"));
        let score = [has_specific, has_first_person, has_concrete].iter().filter(|&&x| x).count();

        if score >= 2 {
            send(serde_json::json!({
                "phase": "practice_opening"
            }).to_string());
            send(serde_json::json!({
                "phase": "done",
                "response": {
                    "entry_type": "practice_opening",
                    "matched_souls": serde_json::json!([]),
                    "recommended_mode": "practice_opening",
                    "review": { "verdict": "practice_opening", "checks": [], "notes": "", "reviewer": "" },
                    "task_cards": {}
                }
            }).to_string());
            return;
        }

        let entry_type = if score == 1 { "hybrid" } else { "conventional" };

        let provider = match pick_provider(&state) {
            Ok(p) => p,
            Err(_) => { return; }
        };

        let all_souls = match state.registry.list_souls(&foundation::IsmismFilter::default()) {
            Ok(s) => s,
            Err(_) => { return; }
        };
        let task_lower = body.task.to_lowercase();

        // ── Phase: classifying ──
        send(serde_json::json!({ "phase": "classifying", "entry_type": entry_type }).to_string());

        // ── Step 1: Algorithmic matching ──
        let ft_results = state.registry.search_souls(&body.task).unwrap_or_default();
        let ft_scores: std::collections::HashMap<String, f64> = ft_results.iter()
            .map(|m| (m.entry.name.clone(), m.relevance))
            .collect();

        let mut scored: Vec<(&SoulListEntry, f64, usize)> = all_souls.iter().map(|s| {
            let kw_hits = s.trigger_keywords.iter()
                .filter(|kw| task_lower.contains(&kw.to_lowercase()))
                .count() as usize;
            let ft_score = ft_scores.get(&s.name).copied().unwrap_or(0.0);
            let composite = ft_score * 10.0 + (kw_hits as f64);
            (s, composite, kw_hits)
        }).collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.0.summon_count.cmp(&a.0.summon_count)));

        let top_n = 8usize.min(scored.len()).max(2);
        let souls: Vec<SoulMatch> = scored.iter().take(top_n).map(|(s, composite, kw)| {
            let rationale = if *kw > 0 {
                format!("命中 {} 个关键词, 全文相关性 {:.1}", kw, composite)
            } else {
                format!("全文相关性 {:.1}", composite)
            };
            SoulMatch {
                name: s.name.clone(),
                field: s.field.clone(),
                ismism_code: s.ismism_code.clone(),
                rationale,
            }
        }).collect();

        let mode = if souls.len() <= 1 {
            "single".to_string()
        } else {
            let unique_fields: std::collections::HashSet<&str> = souls.iter()
                .filter_map(|s| if s.field.is_empty() { None } else { Some(s.field.as_str()) })
                .collect();
            if unique_fields.len() >= 3 { "debate".to_string() } else { "conference".to_string() }
        };

        // ── Phase: matched ──
        send(serde_json::json!({
            "phase": "matched",
            "souls": souls.iter().map(|s| serde_json::json!({
                "name": s.name, "field": s.field, "ismism_code": s.ismism_code, "rationale": s.rationale,
            })).collect::<Vec<_>>(),
            "mode": mode,
        }).to_string());

        // ── Step 2: Banner lord review ──
        let reviewer_name = body.reviewer.clone()
            .unwrap_or_else(|| std::env::var("AIONUI_REVIEWER_SOUL").unwrap_or_else(|_| "未明子".to_string()));
        let mut task_cards: std::collections::HashMap<String, String> = Default::default();

        // ── Phase: reviewing (stream before waiting for LLM) ──
        send(serde_json::json!({
            "phase": "reviewing",
            "reviewer": reviewer_name,
        }).to_string());

        let (verdict, checks, notes) = if let Ok(banner_lord) = state.registry.get_soul(&reviewer_name) {
            let candidate_profiles: Vec<SoulProfile> = souls.iter()
                .filter_map(|s| state.registry.get_soul(&s.name).ok())
                .collect();

            let review_handle = spawn_banner_lord_review(
                state.engine.gateway().clone(),
                banner_lord,
                body.task.clone(),
                candidate_profiles,
                body.judgment.clone().unwrap_or_default(),
                body.worry.clone().unwrap_or_default(),
                body.unknown.clone().unwrap_or_default(),
            );

            let result = match review_handle.await {
                Ok(Ok(json)) => json,
                Ok(Err(e)) => {
                    tracing::error!("Banner lord review sub-agent failed: {}", e);
                    serde_json::Value::default()
                }
                Err(e) => {
                    tracing::error!("Banner lord review sub-agent panicked: {}", e);
                    serde_json::Value::default()
                }
            };

            if let Some(cards) = result["task_cards"].as_object() {
                for (k, v) in cards {
                    if let Some(val) = v.as_str() {
                        task_cards.insert(k.clone(), val.to_string());
                    }
                }
            }
            let missing_perspectives: Vec<String> = result["missing_perspectives"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let boundary_risks: Vec<String> = result["boundary_risks"].as_array()
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
            ("pass".to_string(), vec!["审查官不可用，默认通过".into()], String::new())
        };

        // ── Phase: review_done ──
        send(serde_json::json!({
            "phase": "review_done",
            "review": {
                "verdict": verdict,
                "checks": checks,
                "notes": notes,
                "reviewer": reviewer_name,
            }
        }).to_string());

        // ── Step 3: Apply Review Verdict ──
        let final_souls = if verdict == "pass" {
            souls
        } else {
            // ── Phase: adjusting ──
            send(serde_json::json!({ "phase": "adjusting" }).to_string());

            let adjustment_prompt = format!(
                "## 任务\n{}\n\n## 当前魂组合\n{}\n\n## 幡主审查结果\n裁决：{}\n{}\n备注：{}\n\n## 指令\n根据审查结果调整魂组合。如果是条件通过——增删魂以满足约束。如果是拒绝——完全重新匹配。\n返回JSON：{{\"souls\":[{{\"name\":\"魂名\",\"rationale\":\"调整理由\"}}]}}",
                body.task,
                souls.iter().map(|s| format!("- {} [{}] {}", s.name, s.field, s.rationale)).collect::<Vec<_>>().join("\n"),
                verdict, checks.join("\n"), notes
            );
            let provider3 = provider.clone();
            match llm_call_json(&state, provider3, None, &adjustment_prompt, 0.3, 4096, None).await {
                Ok(adjusted) => {
                    let adjusted_souls = adjusted["souls"].as_array()
                        .map(|arr| arr.iter().filter_map(|s| build_soul_match(s, &all_souls)).collect())
                        .unwrap_or(souls.clone());
                    if adjusted_souls.iter().any(|s| !souls.iter().any(|orig| orig.name == s.name))
                        || souls.iter().any(|orig| !adjusted_souls.iter().any(|s| s.name == orig.name))
                    {
                        if !adjusted_souls.is_empty() {
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
                        }
                        adjusted_souls
                    } else {
                        souls
                    }
                }
                Err(_) => souls,
            }
        };

        // ── Phase: done ──
        send(serde_json::json!({
            "phase": "done",
            "response": {
                "entry_type": entry_type,
                "matched_souls": final_souls.iter().map(|s| serde_json::json!({
                    "name": s.name, "field": s.field, "ismism_code": s.ismism_code, "rationale": s.rationale,
                })).collect::<Vec<_>>(),
                "recommended_mode": mode,
                "review": { "verdict": verdict, "checks": checks, "notes": notes, "reviewer": reviewer_name },
                "task_cards": task_cards,
            }
        }).to_string());
    });

    Sse::new(UnboundedReceiverStream::new(rx)).keep_alive(KeepAlive::default())
}

fn spawn_follow_up_agent(
    gateway: Arc<ai_gateway::GatewayRegistry>,
    banner_lord: SoulProfile,
    question: String,
    history: Vec<String>,
    ws: possession::WsSessionManager,
    session_id: String,
    archive: Arc<archive::ArchiveSystem>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let provider = gateway
            .list_providers()
            .into_iter()
            .find(|i| i.available)
            .map(|i| i.provider)
            .unwrap_or(foundation::Provider::Claude);

        let system_prompt = banner_lord.summon_prompt;
        let user_prompt = if history.is_empty() {
            format!(
                "## 新问题\n{}\n\n根据这个新问题，以你的立场和视角回应。你是{}，ismism坐标{}。",
                question, banner_lord.name, banner_lord.ismism_code
            )
        } else {
            format!(
                "## 历史对话\n{}\n\n## 新问题\n{}\n\n以上是之前的多魂附体会话。作为{}（ismism={}），请以你的立场和视角回应这个追问。",
                history.join("\n\n"), question, banner_lord.name, banner_lord.ismism_code
            )
        };

        let q_msg = foundation::Message {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.clone(),
            role: foundation::MessageRole::User,
            soul_name: None,
            content: question.clone(),
            seq: 990,
            created_at: chrono::Utc::now(),
        };
        let _ = archive.append_message(&q_msg).await;

        let config = CallConfig {
            temperature: 0.7,
            max_tokens: 8192,
            stream: true,
            model: None,
            reasoning_effort: Some(foundation::ReasoningEffort::Think),
            structured_output: None,
            thinking_enabled: None,
            tools: None,
            tool_choice: None,
        };

        let req = LLMRequest {
            provider,
            prompt: Prompt {
                messages: vec![
                    PromptMessage {
                        role: "system".into(),
                        content: system_prompt,
                        reasoning_content: None,
                        tool_call_id: None,
                        tool_calls: None,
                    },
                    PromptMessage {
                        role: "user".into(),
                        content: user_prompt,
                        reasoning_content: None,
                        tool_call_id: None,
                        tool_calls: None,
                    },
                ],
            },
            config,
        };

        match gateway.call(&req) {
            Ok(rx) => {
                match possession::stream::stream_synthesis(rx, &session_id, &ws).await {
                    Ok((content, _)) => {
                        let msg = foundation::Message {
                            id: uuid::Uuid::new_v4().to_string(),
                            session_id,
                            role: foundation::MessageRole::Synthesis,
                            soul_name: Some(banner_lord.name),
                            content,
                            seq: 991,
                            created_at: chrono::Utc::now(),
                        };
                        let _ = archive.append_message(&msg).await;
                    }
                    Err(e) => {
                        tracing::error!("Follow-up agent stream error: {}", e);
                        let _ = ws.broadcast_system(
                            &session_id,
                            &possession::WsEvent {
                                event_type: possession::WsEventType::Error,
                                payload: format!("流式响应错误: {}", e),
                                reasoning_content: None,
                                soul_name: None,
                                seq: 0,
                            },
                        );
                    }
                }
            }
            Err(e) => {
                tracing::error!("Follow-up agent LLM call failed: {}", e);
                let _ = ws.broadcast_system(
                    &session_id,
                    &possession::WsEvent {
                        event_type: possession::WsEventType::Error,
                        payload: format!("启动 LLM 失败: {}", e),
                        reasoning_content: None,
                        soul_name: None,
                        seq: 0,
                    },
                );
            }
        }
    })
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

    let ws = state.engine.ws_manager().clone();
    let sid = session_id.clone();
    ws.create_session(&sid);

    let history: Vec<String> = match state.archive.get_session_detail(&session_id).await {
        Ok(session) => {
            session.messages.iter().map(|m| format!("[{:?}] {}: {}", m.role, m.soul_name.as_deref().unwrap_or("系统"), m.content)).collect()
        }
        Err(e) => {
            tracing::warn!("Session not found: {}, proceeding with basic prompt", e);
            vec![]
        }
    };

    let reviewer_name = std::env::var("AIONUI_REVIEWER_SOUL").unwrap_or_else(|_| "未明子".to_string());
    let banner_lord = state.registry.get_soul(&reviewer_name).map_err(|e| {
        tracing::error!("Reviewer soul not found: {}", e);
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: format!("审查官 {} 不可用", reviewer_name) }))
    })?;

    let question = body.question.clone();
    let gateway = state.engine.gateway().clone();
    let archive = state.archive.clone();

    let _ = spawn_follow_up_agent(
        gateway,
        banner_lord,
        question,
        history,
        ws,
        sid,
        archive,
    );

    tracing::info!("Follow-up sub-agent spawned, returning immediately");
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
