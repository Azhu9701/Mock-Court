use std::sync::Arc;

use ai_gateway::GatewayRegistry;
use chrono::Utc;
use foundation::{
    CallConfig, LLMRequest, ObservationType, Prompt, PromptMessage, Provider, SessionObservation,
    Storage,
};
use serde::{Deserialize, Serialize};
use crate::{WsEvent, WsEventType, WsSessionManager};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DigestSummary {
    pub summary: String,
    pub observation_count: usize,
}

#[derive(Debug, serde::Deserialize)]
struct ParsedDigest {
    summary: String,
    observations: Vec<ParsedObservation>,
}

#[derive(Debug, serde::Deserialize)]
struct ParsedObservation {
    #[serde(rename = "type")]
    obs_type: String,
    title: String,
    content: String,
    soul: Option<String>,
    seq: Option<i64>,
    confidence: Option<f32>,
}

const SYSTEM_PROMPT: &str = r#"你是万民幡的记忆压缩器。你的任务是从一次"魂合议"对话中提取 5-10 条原子级知识点（observation），并给出一句整体总结。

## observation 类型定义
每条 observation 必须归入以下 8 类之一：
- session: 🎯 整体回顾/元信息
- discovery: 🔵 新发现/新认知
- decision: ⚖️ 做出的决策或立场
- bugfix: 🔴 发现的问题或修正
- feature: 🟣 新能力或新方案
- refactor: 🔄 重组/重新框架化
- change: ✅ 一般性变更/事实
- security: 🚨 安全或边界问题

## 输出格式（严格 JSON，不要 markdown 代码块）
{
  "summary": "一句话总结这次合议的核心结论",
  "observations": [
    {
      "type": "discovery",
      "title": "简短标题（≤30字）",
      "content": "详细描述（≤200字）",
      "soul": "产出此内容的魂名称（可空）",
      "seq": 5,
      "confidence": 0.9
    }
  ]
}

要求：
1. 每条 observation 必须有独立的认知价值，不重复
2. title 简洁、content 具体
3. type 必须是上述 8 类之一
4. soul 和 seq 对应对话中产出该内容的魂和消息序号
5. confidence 表示此 observation 的可靠程度 (0.0-1.0)
6. summary 是给用户看的总览，≤100字"#;

/// Compress a completed session into observations + digest summary.
pub async fn distill_session(
    store: Arc<dyn Storage>,
    gateway: Arc<GatewayRegistry>,
    session_id: &str,
) -> foundation::error::Result<DigestSummary> {
    let session = store.get_session(session_id).await?;
    let messages = store.get_messages(session_id).await?;

    if messages.is_empty() {
        return Ok(DigestSummary {
            summary: "空会话".to_string(),
            observation_count: 0,
        });
    }

    // Build conversation text for the prompt (truncate to ~8000 chars to fit context)
    let conversation = build_conversation_text(&messages, &session.title);
    let truncated = if conversation.len() > 8000 {
        format!("{}...(共 {} 字符，已截断)", &conversation[..8000], conversation.len())
    } else {
        conversation
    };

    let user_msg = format!("会话标题: {}\n模式: {}\n\n对话内容:\n{}", session.title, session.mode.as_str(), truncated);

    let prompt = Prompt {
        messages: vec![
            PromptMessage {
                role: "system".into(),
                content: SYSTEM_PROMPT.to_string(),
                reasoning_content: None,
                ..Default::default()
            },
            PromptMessage {
                role: "user".into(),
                content: user_msg,
                reasoning_content: None,
                ..Default::default()
            },
        ],
    };

    let config = CallConfig {
        temperature: 0.3,
        max_tokens: 2048,
        stream: false,
        model: Some("deepseek-chat".to_string()),
        ..Default::default()
    };

    let req = LLMRequest {
        provider: Provider::DeepSeek,
        prompt,
        config,
    };

    let mut rx = gateway.call(&req).map_err(|e| {
        foundation::error::FoundationError::InvalidState(format!("distill call failed: {}", e))
    })?;

    let mut raw = String::new();
    while let Some(r) = rx.recv().await {
        match r {
            Ok(c) => raw.push_str(&c.content),
            Err(e) => {
                tracing::warn!("distill chunk error: {}", e);
                break;
            }
        }
    }

    if raw.is_empty() {
        return Ok(DigestSummary {
            summary: "压缩失败：LLM 无输出".to_string(),
            observation_count: 0,
        });
    }

    // Strip markdown code fences if present
    let json_str = strip_code_fence(&raw);

    let parsed: ParsedDigest = match serde_json::from_str(json_str) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("distill parse error: {} | raw: {}", e, &raw[..raw.len().min(200)]);
            return Ok(DigestSummary {
                summary: format!("压缩解析失败: {}", e),
                observation_count: 0,
            });
        }
    };

    // Estimate tokens: rough heuristic ~1.5 tokens per CJK char, ~0.75 per ASCII char
    let estimate_tokens = |text: &str| -> u32 {
        let cjk = text.chars().filter(|c| ('\u{4e00}'..='\u{9fff}').contains(c)).count() as u32;
        let other = text.len() as u32 / 2;
        (cjk * 2 + other) / 3
    };

    let work_tokens: u32 = messages.iter().map(|m| estimate_tokens(&m.content)).sum();
    let now = Utc::now();

    let rows: Vec<SessionObservation> = parsed
        .observations
        .iter()
        .map(|o| SessionObservation {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            soul_name: o.soul.clone(),
            obs_type: ObservationType::from_str(&o.obs_type).unwrap_or(ObservationType::Discovery),
            title: o.title.clone(),
            content: o.content.clone(),
            source_seq: o.seq,
            read_tokens: estimate_tokens(&o.content),
            work_tokens: (work_tokens as f32 / parsed.observations.len().max(1) as f32) as u32,
            confidence: o.confidence.unwrap_or(0.7).clamp(0.0, 1.0),
            created_at: now,
        })
        .collect();

    let count = rows.len();
    store.insert_session_observations(&rows).await?;
    store.update_session_digest(session_id, &parsed.summary).await?;

    Ok(DigestSummary {
        summary: parsed.summary,
        observation_count: count,
    })
}

fn build_conversation_text(messages: &[foundation::Message], _title: &str) -> String {
    use foundation::MessageRole;
    let mut out = String::new();
    for msg in messages {
        let role_label = match msg.role {
            MessageRole::User => "👤用户",
            MessageRole::Soul => {
                let name = msg.soul_name.as_deref().unwrap_or("魂");
                // Use a generic label to avoid emoji in code
                &*format!("🎭{}", name)
            }
            MessageRole::Synthesis => "🧠综合",
            MessageRole::System => "⚙️系统",
        };
        // Truncate individual messages to ~1500 chars (respect UTF-8 boundaries)
        let content = if msg.content.len() > 1500 {
            let boundary = msg.content.char_indices()
                .take_while(|(i, _)| *i < 1500)
                .last()
                .map(|(i, c)| i + c.len_utf8())
                .unwrap_or(1500);
            format!("{}...(截断)", &msg.content[..boundary])
        } else {
            msg.content.clone()
        };
        out.push_str(&format!("[seq={}] {}: {}\n\n", msg.seq, role_label, content));
    }
    out
}

fn strip_code_fence(s: &str) -> &str {
    let trimmed = s.trim();
    if (trimmed.starts_with("```json") || trimmed.starts_with("```")) && trimmed.ends_with("```") {
        let start = trimmed.find('\n').map(|i| i + 1).unwrap_or(7);
        let end = trimmed.len() - 3;
        if start < end {
            return &trimmed[start..end];
        }
    }
    s
}

/// Run distill in background and notify via WebSocket on completion.
pub fn spawn_distill(
    store: Arc<dyn Storage>,
    gateway: Arc<GatewayRegistry>,
    ws: WsSessionManager,
    session_id: String,
) {
    tokio::spawn(async move {
        tracing::info!("distill started for session {}", session_id);
        match distill_session(store, gateway, &session_id).await {
            Ok(digest) => {
                tracing::info!(
                    "distill completed for session {}: {} observations",
                    session_id,
                    digest.observation_count
                );
                let _ = ws.broadcast_system(
                    &session_id,
                    &WsEvent {
                        event_type: WsEventType::ObservationsReady,
                        payload: serde_json::to_string(&digest).unwrap_or_default(),
                        reasoning_content: None,
                        soul_name: None,
                        seq: 0,
                    },
                );
            }
            Err(e) => {
                tracing::warn!("distill failed for session {}: {}", session_id, e);
            }
        }
    });
}
