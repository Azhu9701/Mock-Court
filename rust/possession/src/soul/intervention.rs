use std::collections::HashSet;
use std::sync::Arc;

use ai_gateway::GatewayRegistry;
use foundation::{Prompt, PromptMessage, Provider, LLMRequest};

/// 干预决策结果：三级门控级联的最终判定
#[derive(Debug, Clone)]
pub enum InterventionDecision {
    /// 无需干预
    NoAction,
    /// 注入矛盾追问
    InjectQuestion { question: String },
    /// 重定向至新方向
    Redirect { target: String },
    /// 请求深化特定维度
    DeepenRequest { aspect: String, reason: String },
}

/// 信念冲突关键词表
fn default_belief_conflict_keywords() -> Vec<String> {
    vec![
        // 中文哲学/辩证关键词
        "矛盾".to_string(),
        "对立".to_string(),
        "冲突".to_string(),
        "相反".to_string(),
        "不一致".to_string(),
        "悖论".to_string(),
        "对立统一".to_string(),
        "辩证".to_string(),
        "否定".to_string(),
        "批判".to_string(),
        "不可调和".to_string(),
        "张力".to_string(),
        "背反".to_string(),
        "二律背反".to_string(),
        "扬弃".to_string(),
        "反思".to_string(),
        // 英文关键词
        "contradiction".to_string(),
        "conflict".to_string(),
        "paradox".to_string(),
        "opposite".to_string(),
        "tension".to_string(),
        "dichotomy".to_string(),
        "antinomy".to_string(),
        "dialectic".to_string(),
        "negation".to_string(),
        "sublation".to_string(),
        // 交叉语境关键词
        "分歧".to_string(),
        "颠覆".to_string(),
        "挑战".to_string(),
        "质疑".to_string(),
        "破缺".to_string(),
        "盲点".to_string(),
    ]
}

/// 三级门控结构：L1 关键词规则 → L2 embedding 余弦相似度 → L3 Flash LLM
///
/// 魂上下文隔离，计算分层：能用规则绝不用 LLM
#[derive(Clone)]
pub struct InterventionGate {
    /// L1: 信念冲突关键词表
    pub belief_conflict_keywords: Vec<String>,
    /// L2: embedding 余弦相似度阈值（低于此值视为冲突信号）
    pub similarity_threshold: f32,
    /// L3: Flash LLM 网关（可选，仅在前两级无法判定时使用）
    pub gateway: Option<Arc<GatewayRegistry>>,
}

impl std::fmt::Debug for InterventionGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InterventionGate")
            .field("belief_conflict_keywords", &self.belief_conflict_keywords)
            .field("similarity_threshold", &self.similarity_threshold)
            .field("gateway", &self.gateway.as_ref().map(|_| "GatewayRegistry"))
            .finish()
    }
}

impl InterventionGate {
    /// 创建新的门控实例
    pub fn new(gateway: Option<Arc<GatewayRegistry>>) -> Self {
        Self {
            belief_conflict_keywords: default_belief_conflict_keywords(),
            similarity_threshold: 0.3,
            gateway,
        }
    }

    /// 三级级联入口：L1 → L2 → L3 依次尝试，任一级命中即返回
    pub async fn gate(
        &self,
        soul_output: &str,
        peer_outputs: &[String],
    ) -> InterventionDecision {
        // L1: 关键词规则匹配（微秒级）
        if let Some(decision) = self.try_l1_keyword(soul_output, peer_outputs) {
            tracing::debug!("InterventionGate L1 hit");
            return decision;
        }

        // L2: embedding 余弦相似度（毫秒级）
        if let Some(decision) = self.try_l2_similarity(soul_output, peer_outputs) {
            tracing::debug!("InterventionGate L2 hit");
            return decision;
        }

        // L3: Flash LLM 判定（秒级）
        if let Some(decision) = self.try_l3_flash_llm(soul_output, peer_outputs).await {
            tracing::debug!("InterventionGate L3 hit");
            return decision;
        }

        InterventionDecision::NoAction
    }

    /// L1: 关键词规则匹配
    ///
    /// 如果当前魂输出和任一同伴输出同时命中同一关键词，
    /// 且关键词属于信念冲突表，则触发追问。
    fn try_l1_keyword(&self, soul_output: &str, peer_outputs: &[String]) -> Option<InterventionDecision> {
        let soul_lower = soul_output.to_lowercase();
        // 没有同行魂时不触发干预
        if peer_outputs.is_empty() {
            return None;
        }
        for keyword in &self.belief_conflict_keywords {
            let kw_lower = keyword.to_lowercase();
            if soul_lower.contains(&kw_lower) {
                for peer in peer_outputs {
                    if peer.to_lowercase().contains(&kw_lower) {
                        return Some(InterventionDecision::InjectQuestion {
                            question: format!(
                                "同行魂也讨论了「{}」，你的立场与它有本质差异吗？请指出分歧的核心。",
                                keyword
                            ),
                        });
                    }
                }
                // 当前魂命中关键词但同伴未命中，可能是盲点
                return Some(InterventionDecision::DeepenRequest {
                    aspect: keyword.clone(),
                    reason: format!(
                        "你提到了「{}」，但同行魂尚未覆盖此维度。请展开论述，提供更深的洞察。",
                        keyword
                    ),
                });
            }
        }
        None
    }

    /// L2: 基于 trigram 的 Jaccard 相似度（embedding 的轻量近似）
    ///
    /// 在无 ONNX 运行时用 trigram 集合模拟语义重叠检测。
    /// 高重叠 → 冗余信号（重定向）；中等重叠 → 互补信号（深化请求）。
    fn try_l2_similarity(&self, soul_output: &str, peer_outputs: &[String]) -> Option<InterventionDecision> {
        if peer_outputs.is_empty() {
            return None;
        }

        let soul_tokens = tokenize_trigrams(&soul_output.to_lowercase());
        if soul_tokens.is_empty() {
            return None;
        }

        for (i, peer) in peer_outputs.iter().enumerate() {
            let peer_tokens = tokenize_trigrams(&peer.to_lowercase());
            if peer_tokens.is_empty() {
                continue;
            }
            let overlap = jaccard_similarity(&soul_tokens, &peer_tokens);

            if overlap > 0.55 {
                // 高重叠度：两魂输出高度相似，需要重定向以避免冗余
                return Some(InterventionDecision::Redirect {
                    target: format!(
                        "你与第{}位同行魂的内容高度重叠（相似度 {:.0}%）。请从不同的角度或前提重新切入。",
                        i + 1,
                        (overlap * 100.0).round()
                    ),
                });
            }
            if overlap > 0.25 {
                // 中等重叠度：存在互补空间，请求深化
                return Some(InterventionDecision::DeepenRequest {
                    aspect: "互补视角".to_string(),
                    reason: format!(
                        "你与第{}位同行魂有 {:.0}% 的内容重叠，存在互补空间。请深化你独有的分析维度。",
                        i + 1,
                        (overlap * 100.0).round()
                    ),
                });
            }
        }
        None
    }

    /// L3: Flash LLM 判定（使用廉价模型做最终裁决）
    ///
    /// 仅在前两级均未命中时调用，用微小成本换取准确判定。
    async fn try_l3_flash_llm(
        &self,
        soul_output: &str,
        peer_outputs: &[String],
    ) -> Option<InterventionDecision> {
        let gateway = self.gateway.as_ref()?;
        if peer_outputs.is_empty() || soul_output.len() < 50 {
            return None;
        }

        let peer_summary: Vec<String> = peer_outputs
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let truncated: String = p.chars().take(200).collect();
                format!("同行魂{}: {}", i + 1, truncated)
            })
            .collect();

        let system_prompt = "你是一个干预判定器。根据当前魂和同伴的输出，判断是否需要干预。\
            只回复 JSON: {\"action\":\"no_action\"|\"inject_question\"|\"redirect\"|\"deepen\",\
            \"reason\":\"简短理由\"}。";

        let user_prompt = format!(
            "当前魂输出（前200字）: {}\n\n同伴输出:\n{}\n\n判定是否需要干预。",
            soul_output.chars().take(200).collect::<String>(),
            peer_summary.join("\n")
        );

        let prompt = Prompt {
            messages: vec![
                PromptMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                    reasoning_content: None,
                    ..Default::default()
                },
                PromptMessage {
                    role: "user".to_string(),
                    content: user_prompt,
                    reasoning_content: None,
                    ..Default::default()
                },
            ],
        };

        let config = foundation::CallConfig {
            temperature: 0.0,
            max_tokens: 128,
            stream: false,
            model: None,
            tools: None,
            tool_choice: None,
            reasoning_effort: None,
            structured_output: None,
            thinking_enabled: None,
        };

        // 自动选择可用的 provider，不再硬编码 Claude
        let provider = gateway.pick_provider().unwrap_or(Provider::Claude);

        let req = LLMRequest {
            provider,
            prompt,
            config,
        };

        match gateway.call(&req) {
            Ok(mut rx) => {
                let mut content = String::new();
                while let Some(chunk_result) = rx.recv().await {
                    match chunk_result {
                        Ok(chunk) => {
                            content.push_str(&chunk.content);
                        }
                        Err(e) => {
                            tracing::warn!("L3 Flash LLM error: {}", e);
                            return None;
                        }
                    }
                }
                parse_l3_response(&content)
            }
            Err(e) => {
                tracing::warn!("L3 Flash LLM call failed: {}", e);
                None
            }
        }
    }
}

/// 解析 L3 Flash LLM 的 JSON 回应
fn parse_l3_response(response: &str) -> Option<InterventionDecision> {
    let json_str = response.trim();

    // 尝试直接解析
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
        let action = v.get("action").and_then(|a| a.as_str()).unwrap_or("no_action");
        let reason = v
            .get("reason")
            .and_then(|r| r.as_str())
            .unwrap_or("")
            .to_string();

        match action {
            "inject_question" => {
                return Some(InterventionDecision::InjectQuestion {
                    question: reason,
                });
            }
            "redirect" => {
                return Some(InterventionDecision::Redirect { target: reason });
            }
            "no_action" => {
                return Some(InterventionDecision::NoAction);
            }
            "deepen" => {
                return Some(InterventionDecision::DeepenRequest {
                    aspect: "LLM判定".to_string(),
                    reason,
                });
            }
            _ => return None,
        }
    }

    // 容错：尝试从可能包含 markdown 代码块的文本中提取 JSON
    if let Some(start) = json_str.find('{') {
        if let Some(end) = json_str.rfind('}') {
            let inner = &json_str[start..=end];
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(inner) {
                let action = v.get("action").and_then(|a| a.as_str()).unwrap_or("no_action");
                let reason = v
                    .get("reason")
                    .and_then(|r| r.as_str())
                    .unwrap_or("")
                    .to_string();
                match action {
                    "inject_question" => {
                        return Some(InterventionDecision::InjectQuestion { question: reason });
                    }
                    "redirect" => {
                        return Some(InterventionDecision::Redirect { target: reason });
                    }
                    "deepen" => {
                        return Some(InterventionDecision::DeepenRequest {
                            aspect: "LLM判定".to_string(),
                            reason,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    None
}

/// 将文本拆分为字符 trigram 集合
fn tokenize_trigrams(text: &str) -> HashSet<[char; 3]> {
    text.chars()
        .collect::<Vec<_>>()
        .windows(3)
        .map(|w| [w[0], w[1], w[2]])
        .collect()
}

/// Jaccard 相似度 = |A ∩ B| / |A ∪ B|
fn jaccard_similarity(a: &HashSet<[char; 3]>, b: &HashSet<[char; 3]>) -> f64 {
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_l1_keyword_hit_same_keyword() {
        let gate = InterventionGate::new(None);
        let soul = "这里存在一个深刻的矛盾，需要进一步辩证分析。";
        let peer = "我也认为这里存在矛盾，但是另一种形式的对立。";

        let decision = gate.gate(soul, &[peer.to_string()]).await;
        match decision {
            InterventionDecision::InjectQuestion { .. } => {} // expected
            other => panic!("Expected InjectQuestion, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_l1_keyword_soul_only() {
        let gate = InterventionGate::new(None);
        let soul = "这里存在一个深刻的悖论需要反思。";
        let peer = "一切都很和谐，没有冲突。";

        let decision = gate.gate(soul, &[peer.to_string()]).await;
        match decision {
            InterventionDecision::DeepenRequest { .. } => {} // expected
            other => panic!("Expected DeepenRequest, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_l2_high_similarity() {
        let gate = InterventionGate::new(None);
        let text = "魂长驻进程通过 tokio select 实现推理与干预的竞态";
        // Same text, slightly reworded → high trigram overlap
        let similar = "通过 tokio select 实现魂长驻进程推理与干预的竞态";

        let decision = gate.try_l2_similarity(text, &[similar.to_string()]);
        match decision {
            Some(InterventionDecision::Redirect { .. }) => {} // expected
            other => panic!("Expected Redirect, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_l2_low_similarity() {
        let gate = InterventionGate::new(None);
        let text = "量子力学的不确定性原理";
        let different = "深度学习图像分类的 CNN 架构";

        let decision = gate.try_l2_similarity(text, &[different.to_string()]);
        assert!(decision.is_none(), "Expected no decision for different topics");
    }

    #[tokio::test]
    async fn test_no_peers_no_trigger() {
        let gate = InterventionGate::new(None);
        let soul = "这里存在矛盾";

        let decision = gate.gate(soul, &[]).await;
        match decision {
            InterventionDecision::NoAction => {} // expected
            other => panic!("Expected NoAction with no peers, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_l3_no_action() {
        let resp = r#"{"action": "no_action", "reason": ""}"#;
        let decision = parse_l3_response(resp);
        match decision {
            Some(InterventionDecision::NoAction) => {}
            _ => panic!("Expected NoAction"),
        }
    }

    #[test]
    fn test_parse_l3_inject() {
        let resp = r#"{"action": "inject_question", "reason": "发现矛盾"}"#;
        let decision = parse_l3_response(resp);
        match decision {
            Some(InterventionDecision::InjectQuestion { question }) => {
                assert_eq!(question, "发现矛盾");
            }
            _ => panic!("Expected InjectQuestion"),
        }
    }

    #[test]
    fn test_jaccard_identical() {
        let a = tokenize_trigrams("abcabc");
        let b = tokenize_trigrams("abcabc");
        assert!((jaccard_similarity(&a, &b) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_jaccard_disjoint() {
        let a = tokenize_trigrams("abc");
        let b = tokenize_trigrams("xyz");
        assert!((jaccard_similarity(&a, &b) - 0.0).abs() < 0.001);
    }
}
