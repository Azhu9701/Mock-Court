use foundation::{ModelTier, Provider, ProviderInfo, ReasoningEffort, CallConfig};

pub struct ModelRouter;

#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub provider: Provider,
    pub model: String,
    pub tier: ModelTier,
    pub use_cache_hint: bool,
    pub chosen_for: String,
    pub reasoning_effort: Option<ReasoningEffort>,
}

#[derive(Debug, Clone)]
pub enum RoutingRole {
    Soul,
    Synthesizer,
    Reviewer,
    KnowledgeCard,
}

/// DeepSeek 模型变体
#[derive(Debug, Clone)]
pub enum DeepSeekVariant {
    Flash,       // 高性价比，低延迟 → 知识搜索
    ProThink,    // 标准推理
    ProThinkMax, // 1M 窗口 + 深度推理 → 综合官、自我审计
}

impl ModelRouter {
    pub fn route(
        providers: &[ProviderInfo],
        role: RoutingRole,
    ) -> Option<RoutingDecision> {
        let available: Vec<&ProviderInfo> = providers.iter().filter(|p| p.available).collect();
        if available.is_empty() {
            return None;
        }

        // 优先选择 DeepSeek（因为有缓存优化）
        if let Some(deepseek) = available.iter().find(|p| p.provider == Provider::DeepSeek) {
            return Self::route_deepseek_decision(deepseek, role);
        }

        // 否则退回到通用路由
        match role {
            RoutingRole::Synthesizer => {
                Self::pick_tier(&available, &[ModelTier::Max, ModelTier::Pro, ModelTier::Economy], "辩证综合官", true, Some(ReasoningEffort::ThinkMax))
            }
            RoutingRole::Reviewer => {
                Self::pick_tier(&available, &[ModelTier::Pro, ModelTier::Max, ModelTier::Economy], "幡主审查官", false, Some(ReasoningEffort::ThinkHigh))
            }
            RoutingRole::Soul => {
                Self::pick_tier(&available, &[ModelTier::Pro, ModelTier::Max, ModelTier::Economy], "魂分析", true, Some(ReasoningEffort::Think))
            }
            RoutingRole::KnowledgeCard => {
                Self::pick_tier(&available, &[ModelTier::Economy, ModelTier::Pro], "知识卡片提取", false, Some(ReasoningEffort::NonThink))
            }
        }
    }

    /// DeepSeek 特化路由，返回完整决策
    fn route_deepseek_decision(
        deepseek: &&ProviderInfo,
        role: RoutingRole,
    ) -> Option<RoutingDecision> {
        let (variant, effort) = match role {
            RoutingRole::Synthesizer => (DeepSeekVariant::ProThinkMax, ReasoningEffort::ThinkMax),
            RoutingRole::Reviewer => (DeepSeekVariant::ProThink, ReasoningEffort::ThinkHigh),
            RoutingRole::Soul => (DeepSeekVariant::ProThink, ReasoningEffort::Think),
            RoutingRole::KnowledgeCard => (DeepSeekVariant::Flash, ReasoningEffort::NonThink),
        };

        let cache_hint = match &variant {
            DeepSeekVariant::Flash => false,
            DeepSeekVariant::ProThink | DeepSeekVariant::ProThinkMax => true,
        };

        Some(RoutingDecision {
            provider: Provider::DeepSeek,
            model: deepseek.model.clone(),
            tier: deepseek.tier.clone(),
            use_cache_hint: cache_hint,
            chosen_for: match role {
                RoutingRole::Synthesizer => "辩证综合官".to_string(),
                RoutingRole::Reviewer => "幡主审查官".to_string(),
                RoutingRole::Soul => "魂分析".to_string(),
                RoutingRole::KnowledgeCard => "知识卡片提取".to_string(),
            },
            reasoning_effort: Some(effort),
        })
    }

    /// DeepSeek 特化路由：根据任务复杂度选择 Flash / ProThink / ProThinkMax
    pub fn route_deepseek(
        providers: &[ProviderInfo],
        role: RoutingRole,
    ) -> Option<(Provider, String, DeepSeekVariant, bool)> {
        let deepseek = providers.iter().find(|p| p.provider == Provider::DeepSeek && p.available)?;

        let variant = match role {
            RoutingRole::Synthesizer => DeepSeekVariant::ProThinkMax,
            RoutingRole::Reviewer => DeepSeekVariant::ProThink,
            RoutingRole::Soul => DeepSeekVariant::ProThink,
            RoutingRole::KnowledgeCard => DeepSeekVariant::Flash,
        };

        let (model, cache_hint) = match &variant {
            DeepSeekVariant::Flash => (deepseek.model.clone(), false),
            DeepSeekVariant::ProThink => (deepseek.model.clone(), true),
            DeepSeekVariant::ProThinkMax => (deepseek.model.clone(), true),
        };

        Some((Provider::DeepSeek, model, variant, cache_hint))
    }

    /// 根据路由决策创建 CallConfig
    pub fn create_call_config(decision: &RoutingDecision) -> CallConfig {
        let mut config = CallConfig::default();
        if let Some(effort) = &decision.reasoning_effort {
            config = config.with_reasoning_effort(*effort);
        }
        config.model = Some(decision.model.clone());
        config
    }

    fn pick_tier(
        available: &[&ProviderInfo],
        tier_prefs: &[ModelTier],
        chosen_for: &str,
        use_cache_hint: bool,
        reasoning_effort: Option<ReasoningEffort>,
    ) -> Option<RoutingDecision> {
        for pref_tier in tier_prefs {
            for info in available {
                if info.tier == *pref_tier {
                    return Some(RoutingDecision {
                        provider: info.provider.clone(),
                        model: info.model.clone(),
                        tier: info.tier.clone(),
                        use_cache_hint,
                        chosen_for: chosen_for.to_string(),
                        reasoning_effort,
                    });
                }
            }
        }
        available.first().map(|info| RoutingDecision {
            provider: info.provider.clone(),
            model: info.model.clone(),
            tier: info.tier.clone(),
            use_cache_hint: false,
            chosen_for: chosen_for.to_string(),
            reasoning_effort: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{ModelTier, Provider, ProviderInfo, ReasoningEffort};

    fn create_test_providers() -> Vec<ProviderInfo> {
        vec![
            ProviderInfo {
                provider: Provider::DeepSeek,
                model: "deepseek-chat".to_string(),
                available: true,
                tier: ModelTier::Pro,
            },
            ProviderInfo {
                provider: Provider::Claude,
                model: "claude-3-sonnet".to_string(),
                available: true,
                tier: ModelTier::Pro,
            },
            ProviderInfo {
                provider: Provider::OpenAI,
                model: "gpt-4o".to_string(),
                available: true,
                tier: ModelTier::Pro,
            },
        ]
    }

    fn create_provider_with_tier(provider: Provider, model: &str, tier: ModelTier, available: bool) -> ProviderInfo {
        ProviderInfo {
            provider,
            model: model.to_string(),
            available,
            tier,
        }
    }

    #[test]
    fn test_empty_providers_returns_none() {
        let providers: Vec<ProviderInfo> = vec![];
        let result = ModelRouter::route(&providers, RoutingRole::Soul);
        assert!(result.is_none());
    }

    #[test]
    fn test_no_available_providers_returns_none() {
        let providers = vec![
            create_provider_with_tier(Provider::DeepSeek, "deepseek-chat", ModelTier::Pro, false),
        ];
        let result = ModelRouter::route(&providers, RoutingRole::Soul);
        assert!(result.is_none());
    }

    #[test]
    fn test_deepseek_priority_selection() {
        let providers = create_test_providers();
        let result = ModelRouter::route(&providers, RoutingRole::Soul);
        
        assert!(result.is_some());
        let decision = result.unwrap();
        assert_eq!(decision.provider, Provider::DeepSeek);
        assert_eq!(decision.model, "deepseek-chat");
    }

    #[test]
    fn test_soul_uses_prothink() {
        let providers = create_test_providers();
        let result = ModelRouter::route(&providers, RoutingRole::Soul);
        
        assert!(result.is_some());
        let decision = result.unwrap();
        assert_eq!(decision.provider, Provider::DeepSeek);
        assert_eq!(decision.use_cache_hint, true);
        assert_eq!(decision.reasoning_effort, Some(ReasoningEffort::Think));
    }

    #[test]
    fn test_synthesizer_uses_prothinkmax() {
        let providers = create_test_providers();
        let result = ModelRouter::route(&providers, RoutingRole::Synthesizer);
        
        assert!(result.is_some());
        let decision = result.unwrap();
        assert_eq!(decision.provider, Provider::DeepSeek);
        assert_eq!(decision.use_cache_hint, true);
        assert_eq!(decision.reasoning_effort, Some(ReasoningEffort::ThinkMax));
        assert_eq!(decision.chosen_for, "辩证综合官");
    }

    #[test]
    fn test_reviewer_uses_prothink() {
        let providers = create_test_providers();
        let result = ModelRouter::route(&providers, RoutingRole::Reviewer);
        
        assert!(result.is_some());
        let decision = result.unwrap();
        assert_eq!(decision.provider, Provider::DeepSeek);
        assert_eq!(decision.use_cache_hint, true);
        assert_eq!(decision.reasoning_effort, Some(ReasoningEffort::ThinkHigh));
        assert_eq!(decision.chosen_for, "幡主审查官");
    }

    #[test]
    fn test_knowledge_card_uses_flash() {
        let providers = create_test_providers();
        let result = ModelRouter::route(&providers, RoutingRole::KnowledgeCard);
        
        assert!(result.is_some());
        let decision = result.unwrap();
        assert_eq!(decision.provider, Provider::DeepSeek);
        assert_eq!(decision.use_cache_hint, false);
        assert_eq!(decision.reasoning_effort, Some(ReasoningEffort::NonThink));
        assert_eq!(decision.chosen_for, "知识卡片提取");
    }

    #[test]
    fn test_fallback_when_no_deepseek() {
        let providers = vec![
            create_provider_with_tier(Provider::Claude, "claude-3-opus", ModelTier::Max, true),
            create_provider_with_tier(Provider::OpenAI, "gpt-4o", ModelTier::Pro, true),
        ];
        
        let result = ModelRouter::route(&providers, RoutingRole::Synthesizer);
        assert!(result.is_some());
        let decision = result.unwrap();
        assert_eq!(decision.provider, Provider::Claude);
        assert_eq!(decision.tier, ModelTier::Max);
    }

    #[test]
    fn test_tier_pick_order_for_synthesizer() {
        let providers = vec![
            create_provider_with_tier(Provider::OpenAI, "gpt-4o-mini", ModelTier::Economy, true),
            create_provider_with_tier(Provider::Claude, "claude-3-sonnet", ModelTier::Pro, true),
        ];
        
        let result = ModelRouter::route(&providers, RoutingRole::Synthesizer);
        assert!(result.is_some());
        let decision = result.unwrap();
        assert_eq!(decision.tier, ModelTier::Pro);
    }

    #[test]
    fn test_tier_pick_order_for_soul() {
        let providers = vec![
            create_provider_with_tier(Provider::Claude, "claude-3-haiku", ModelTier::Economy, true),
            create_provider_with_tier(Provider::OpenAI, "gpt-4o", ModelTier::Pro, true),
        ];
        
        let result = ModelRouter::route(&providers, RoutingRole::Soul);
        assert!(result.is_some());
        let decision = result.unwrap();
        assert_eq!(decision.tier, ModelTier::Pro);
    }

    #[test]
    fn test_route_deepseek_function() {
        let providers = create_test_providers();
        let result = ModelRouter::route_deepseek(&providers, RoutingRole::Soul);
        
        assert!(result.is_some());
        let (provider, model, _, cache_hint) = result.unwrap();
        assert_eq!(provider, Provider::DeepSeek);
        assert_eq!(model, "deepseek-chat");
        assert!(cache_hint);
    }

    #[test]
    fn test_route_deepseek_returns_none_when_deepseek_unavailable() {
        let providers = vec![
            create_provider_with_tier(Provider::Claude, "claude-3-sonnet", ModelTier::Pro, true),
        ];
        let result = ModelRouter::route_deepseek(&providers, RoutingRole::Soul);
        assert!(result.is_none());
    }

    #[test]
    fn test_create_call_config() {
        let decision = RoutingDecision {
            provider: Provider::DeepSeek,
            model: "deepseek-chat".to_string(),
            tier: ModelTier::Pro,
            use_cache_hint: true,
            chosen_for: "测试".to_string(),
            reasoning_effort: Some(ReasoningEffort::Think),
        };
        
        let config = ModelRouter::create_call_config(&decision);
        assert_eq!(config.model, Some("deepseek-chat".to_string()));
        assert_eq!(config.reasoning_effort, Some(ReasoningEffort::Think));
    }

    #[test]
    fn test_create_call_config_without_reasoning() {
        let decision = RoutingDecision {
            provider: Provider::DeepSeek,
            model: "deepseek-chat".to_string(),
            tier: ModelTier::Pro,
            use_cache_hint: true,
            chosen_for: "测试".to_string(),
            reasoning_effort: None,
        };
        
        let config = ModelRouter::create_call_config(&decision);
        assert_eq!(config.model, Some("deepseek-chat".to_string()));
        assert!(config.reasoning_effort.is_none());
    }

    #[test]
    fn test_deepseek_variant_for_synthesizer() {
        let providers = create_test_providers();
        let result = ModelRouter::route_deepseek(&providers, RoutingRole::Synthesizer);
        assert!(result.is_some());
        let (_, _, _, cache_hint) = result.unwrap();
        assert!(cache_hint);
    }

    #[test]
    fn test_deepseek_variant_for_knowledge_card() {
        let providers = create_test_providers();
        let result = ModelRouter::route_deepseek(&providers, RoutingRole::KnowledgeCard);
        assert!(result.is_some());
        let (_, _, _, cache_hint) = result.unwrap();
        assert!(!cache_hint);
    }

    #[test]
    fn test_mixed_availability() {
        let providers = vec![
            create_provider_with_tier(Provider::DeepSeek, "deepseek-chat", ModelTier::Pro, false),
            create_provider_with_tier(Provider::Claude, "claude-3-sonnet", ModelTier::Pro, true),
        ];
        
        let result = ModelRouter::route(&providers, RoutingRole::Soul);
        assert!(result.is_some());
        let decision = result.unwrap();
        assert_eq!(decision.provider, Provider::Claude);
    }
}
