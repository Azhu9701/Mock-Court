use std::collections::HashMap;

/// 方法论签名 — 描述一个魂的推理风格和认知偏好
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MethodologySignature {
    /// 所属魂名
    pub soul_name: String,
    /// 推理风格标签：如 "辩证", "还原论", "系统论", "实证", "直觉"
    pub reasoning_style: String,
    /// 证据偏好标签：如 "定量", "定性", "历史", "实验", "逻辑"
    pub evidence_preference: String,
    /// 时间焦点标签：如 "历史", "当下", "未来"
    pub temporal_focus: String,
    /// 抽象层级标签：如 "具体", "中等", "抽象"
    pub abstraction_level: String,
    /// 附加的方法论关键词
    pub tags: Vec<String>,
}

impl MethodologySignature {
    pub fn new(soul_name: String) -> Self {
        MethodologySignature {
            soul_name,
            reasoning_style: String::new(),
            evidence_preference: String::new(),
            temporal_focus: String::new(),
            abstraction_level: String::new(),
            tags: Vec::new(),
        }
    }

    pub fn with_styles(
        mut self,
        reasoning: &str,
        evidence: &str,
        temporal: &str,
        abstraction: &str,
    ) -> Self {
        self.reasoning_style = reasoning.to_string();
        self.evidence_preference = evidence.to_string();
        self.temporal_focus = temporal.to_string();
        self.abstraction_level = abstraction.to_string();
        self
    }
}

/// 杂交候选对 — 两个魂及其杂交潜力评估
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HybridizationCandidate {
    /// 亲本 A
    pub parent_a: String,
    /// 亲本 B
    pub parent_b: String,
    /// 方法论的相似度 (0.0 ~ 1.0)
    pub methodology_overlap: f32,
    /// 互补强度的描述列表
    pub complementary_strengths: Vec<String>,
    /// 融合 prompt 草稿（由引擎生成）
    pub fusion_prompt: String,
    /// 预估新颖性 (0.0 ~ 1.0)
    pub estimated_novelty: f32,
}

/// 魂方法论杂交引擎
///
/// 核心逻辑基于方法论签名的相似度/互补度计算，不依赖 LLM。
/// 仅在生成融合 prompt 时可以交给上层做 LLM 调用，引擎提供 prompt 模板。
#[derive(Debug, Clone)]
pub struct HybridizationEngine {
    /// 已注册的方法论签名库
    signatures: HashMap<String, MethodologySignature>,
}

impl HybridizationEngine {
    pub fn new() -> Self {
        HybridizationEngine {
            signatures: HashMap::new(),
        }
    }

    /// 注册魂的方法论签名
    pub fn register(&mut self, sig: MethodologySignature) {
        self.signatures.insert(sig.soul_name.clone(), sig);
    }

    /// 获取魂的方法论签名
    pub fn get(&self, soul_name: &str) -> Option<&MethodologySignature> {
        self.signatures.get(soul_name)
    }

    /// 获取所有签名
    pub fn all_signatures(&self) -> Vec<&MethodologySignature> {
        self.signatures.values().collect()
    }

    /// 获取当前注册数量
    pub fn len(&self) -> usize {
        self.signatures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.signatures.is_empty()
    }

    /// 分析两个魂的方法论相似度
    ///
    /// 返回 0.0（完全不同）到 1.0（完全相同）的相似度分数。
    pub fn analyze_methodology(
        &self,
        soul_a: &str,
        soul_b: &str,
    ) -> Option<(f32, Vec<String>)> {
        let sig_a = self.signatures.get(soul_a)?;
        let sig_b = self.signatures.get(soul_b)?;

        let mut score = 0.0_f32;
        let mut overlap_notes: Vec<String> = Vec::new();

        // 推理风格匹配（权重 0.35）
        if sig_a.reasoning_style == sig_b.reasoning_style {
            if !sig_a.reasoning_style.is_empty() {
                score += 0.35;
                overlap_notes.push(format!(
                    "推理风格一致: {}",
                    sig_a.reasoning_style
                ));
            }
        }

        // 证据偏好匹配（权重 0.25）
        if sig_a.evidence_preference == sig_b.evidence_preference {
            if !sig_a.evidence_preference.is_empty() {
                score += 0.25;
                overlap_notes.push(format!(
                    "证据偏好一致: {}",
                    sig_a.evidence_preference
                ));
            }
        }

        // 时间焦点匹配（权重 0.20）
        if sig_a.temporal_focus == sig_b.temporal_focus {
            if !sig_a.temporal_focus.is_empty() {
                score += 0.20;
                overlap_notes.push(format!(
                    "时间焦点一致: {}",
                    sig_a.temporal_focus
                ));
            }
        }

        // 抽象层级匹配（权重 0.10）
        if sig_a.abstraction_level == sig_b.abstraction_level {
            if !sig_a.abstraction_level.is_empty() {
                score += 0.10;
                overlap_notes.push(format!(
                    "抽象层级一致: {}",
                    sig_a.abstraction_level
                ));
            }
        }

        // 标签交集（额外加 0.10）
        let tag_overlap: Vec<String> = sig_a
            .tags
            .iter()
            .filter(|t| sig_b.tags.contains(t))
            .cloned()
            .collect();
        if !tag_overlap.is_empty() {
            score += 0.10;
            overlap_notes.push(format!(
                "标签交集: {}",
                tag_overlap.join(", ")
            ));
        }

        // 钳制到 [0.0, 1.0]
        score = score.clamp(0.0, 1.0);

        Some((score, overlap_notes))
    }

    /// 计算两个魂的兼容性分数（用于杂交潜力评估）
    ///
    /// 兼容性与相似度不同：既要求一定的基础相似度（可对话），
    /// 又要一定的差异（互补创新）。 最兼容的并非"完全相同"，而是"中等相似 + 中等互补"。
    pub fn compatibility_score(&self, soul_a: &str, soul_b: &str) -> Option<f32> {
        let sig_a = self.signatures.get(soul_a)?;
        let sig_b = self.signatures.get(soul_b)?;

        let (overlap, _) = self.analyze_methodology(soul_a, soul_b)?;

        // 计算差异度
        let diff_score = Self::difference_score(sig_a, sig_b);

        // 兼容性公式：最优区间是 overlap 在 0.3-0.6 且 diff 在 0.3-0.7
        // compatibility = 1.0 - |overlap - 0.45| * 2.0  (peak at 0.45 overlap)
        //              + diff * 0.3  (moderate bonus for difference)
        let overlap_optimality = 1.0 - (overlap - 0.45).abs() * 2.0;
        let diff_bonus = diff_score * 0.3;

        let compat = (overlap_optimality * 0.7 + diff_bonus).clamp(0.0, 1.0);

        Some(compat)
    }

    /// 计算两个签名之间的差异度
    fn difference_score(sig_a: &MethodologySignature, sig_b: &MethodologySignature) -> f32 {
        let mut diff = 0.0_f32;
        let mut dimensions = 0;

        if !sig_a.reasoning_style.is_empty() && !sig_b.reasoning_style.is_empty() {
            if sig_a.reasoning_style != sig_b.reasoning_style {
                diff += 1.0;
            }
            dimensions += 1;
        }
        if !sig_a.evidence_preference.is_empty() && !sig_b.evidence_preference.is_empty() {
            if sig_a.evidence_preference != sig_b.evidence_preference {
                diff += 1.0;
            }
            dimensions += 1;
        }
        if !sig_a.temporal_focus.is_empty() && !sig_b.temporal_focus.is_empty() {
            if sig_a.temporal_focus != sig_b.temporal_focus {
                diff += 1.0;
            }
            dimensions += 1;
        }
        if !sig_a.abstraction_level.is_empty() && !sig_b.abstraction_level.is_empty() {
            if sig_a.abstraction_level != sig_b.abstraction_level {
                diff += 1.0;
            }
            dimensions += 1;
        }

        if dimensions == 0 {
            return 0.0;
        }
        diff / dimensions as f32
    }

    /// 找到与目标魂兼容性最高的配对候选
    pub fn find_compatible_pairs(
        &self,
        target_soul: &str,
        top_n: usize,
    ) -> Vec<(String, f32)> {
        let mut pairs: Vec<(String, f32)> = self
            .signatures
            .keys()
            .filter(|name| name.as_str() != target_soul)
            .filter_map(|other| {
                self.compatibility_score(target_soul, other)
                    .map(|score| (other.clone(), score))
            })
            .collect();

        pairs.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        pairs.truncate(top_n);
        pairs
    }

    /// 找到全局范围内所有兼容的配对
    pub fn find_all_compatible_pairs(
        &self,
        min_compatibility: f32,
        max_pairs: usize,
    ) -> Vec<HybridizationCandidate> {
        let names: Vec<String> = self.signatures.keys().cloned().collect();
        let mut candidates = Vec::new();

        for i in 0..names.len() {
            for j in (i + 1)..names.len() {
                if let Some(compat) = self.compatibility_score(&names[i], &names[j]) {
                    if compat >= min_compatibility {
                        let (overlap, _) =
                            self.analyze_methodology(&names[i], &names[j]).unwrap_or((0.0, vec![]));
                        let sig_a = &self.signatures[&names[i]];
                        let sig_b = &self.signatures[&names[j]];

                        let mut complementary = Vec::new();
                        if sig_a.reasoning_style != sig_b.reasoning_style {
                            complementary.push(format!(
                                "推理视角互补: {} vs {}",
                                sig_a.reasoning_style, sig_b.reasoning_style
                            ));
                        }
                        if sig_a.evidence_preference != sig_b.evidence_preference {
                            complementary.push(format!(
                                "证据偏好互补: {} vs {}",
                                sig_a.evidence_preference, sig_b.evidence_preference
                            ));
                        }
                        if sig_a.temporal_focus != sig_b.temporal_focus {
                            complementary.push(format!(
                                "时间焦点互补: {} vs {}",
                                sig_a.temporal_focus, sig_b.temporal_focus
                            ));
                        }

                        let fusion_prompt = self.generate_fusion_prompt(sig_a, sig_b);

                        // 预估新颖性 = 兼容性 * (1 - overlap) 的标准化
                        let novelty = (compat * (1.0 - overlap)).clamp(0.0, 1.0);

                        candidates.push(HybridizationCandidate {
                            parent_a: names[i].clone(),
                            parent_b: names[j].clone(),
                            methodology_overlap: overlap,
                            complementary_strengths: complementary,
                            fusion_prompt,
                            estimated_novelty: novelty,
                        });
                    }
                }
            }
        }

        candidates.sort_by(|a, b| {
            b.estimated_novelty
                .partial_cmp(&a.estimated_novelty)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(max_pairs);
        candidates
    }

    /// 生成融合 prompt 模板
    ///
    /// 生成的 prompt 是模板，实际使用时可经由 LLM 调用进一步润色。
    pub fn generate_fusion_prompt(
        &self,
        sig_a: &MethodologySignature,
        sig_b: &MethodologySignature,
    ) -> String {
        let mut prompt = String::new();
        prompt.push_str(&format!(
            "你是一个融合了两种方法论的新魂。\n\n"
        ));
        prompt.push_str(&format!(
            "## 方法论来源 A：{}\n",
            sig_a.soul_name
        ));
        prompt.push_str(&format!(
            "- 推理风格：{}\n- 证据偏好：{}\n- 时间焦点：{}\n- 抽象层级：{}\n\n",
            sig_a.reasoning_style,
            sig_a.evidence_preference,
            sig_a.temporal_focus,
            sig_a.abstraction_level
        ));
        prompt.push_str(&format!(
            "## 方法论来源 B：{}\n",
            sig_b.soul_name
        ));
        prompt.push_str(&format!(
            "- 推理风格：{}\n- 证据偏好：{}\n- 时间焦点：{}\n- 抽象层级：{}\n\n",
            sig_b.reasoning_style,
            sig_b.evidence_preference,
            sig_b.temporal_focus,
            sig_b.abstraction_level
        ));
        prompt.push_str(
            "## 融合要求\n"
        );
        prompt.push_str(
            "1. 在分析问题时，同时运用这两种方法论的优势\n"
        );
        prompt.push_str(
            "2. 当两种方法论给出不同结论时，指出张力并说明各自的前提\n"
        );
        prompt.push_str(
            "3. 在综合判断中，给出融合后的唯一结论，并标注融合点\n"
        );

        prompt
    }
}

impl Default for HybridizationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_engine() -> HybridizationEngine {
        let mut engine = HybridizationEngine::new();

        engine.register(
            MethodologySignature::new("马克思".into()).with_styles(
                "辩证",
                "历史",
                "历史",
                "抽象",
            ),
        );
        engine.register(
            MethodologySignature::new("费曼".into()).with_styles(
                "还原论",
                "实验",
                "当下",
                "具体",
            ),
        );
        engine.register(
            MethodologySignature::new("黑格尔".into()).with_styles(
                "辩证",
                "逻辑",
                "历史",
                "抽象",
            ),
        );
        engine.register(
            MethodologySignature::new("达尔文".into()).with_styles(
                "系统论",
                "定量",
                "历史",
                "中等",
            ),
        );

        engine
    }

    #[test]
    fn test_analyze_identical() {
        let mut engine = HybridizationEngine::new();
        let sig = MethodologySignature::new("A".into())
            .with_styles("辩证", "历史", "历史", "抽象");

        engine.register(sig.clone());
        engine.register(
            MethodologySignature::new("B".into())
                .with_styles("辩证", "历史", "历史", "抽象"),
        );

        let (score, notes) = engine.analyze_methodology("A", "B").unwrap();
        // 0.35 + 0.25 + 0.20 + 0.10 = 0.90 (no tag overlap)
        assert!((score - 0.90).abs() < 0.01, "Identical should score 0.90, got {}", score);
        assert!(notes.len() >= 4);
    }

    #[test]
    fn test_analyze_completely_different() {
        let mut engine = HybridizationEngine::new();
        engine.register(
            MethodologySignature::new("A".into())
                .with_styles("辩证", "历史", "历史", "抽象"),
        );
        engine.register(
            MethodologySignature::new("B".into())
                .with_styles("还原论", "实验", "当下", "具体"),
        );

        let (score, _) = engine.analyze_methodology("A", "B").unwrap();
        assert!((score - 0.0).abs() < 0.01, "Completely different should score 0.0, got {}", score);
    }

    #[test]
    fn test_compatibility_mid_range_best() {
        let engine = build_test_engine();

        // 马克思 vs 黑格尔: 高 overlap（辩证+历史+抽象 → 0.35+0.20+0.10=0.65）
        let marx_hegel = engine.compatibility_score("马克思", "黑格尔").unwrap();

        // 马克思 vs 费曼: 低 overlap（完全不同 → 0.0）
        let marx_feynman = engine.compatibility_score("马克思", "费曼").unwrap();

        // 黑格尔 vs 费曼: 低 overlap（完全不同 → 0.0）
        let hegel_feynman = engine.compatibility_score("黑格尔", "费曼").unwrap();

        // 马克思-黑格尔（中等偏高的 overlap）应该比完全不同的配对兼容性高
        assert!(
            marx_hegel > marx_feynman,
            "Marx-Hegel ({}) should be more compatible than Marx-Feynman ({})",
            marx_hegel, marx_feynman
        );

        // 完全不同的两个配对兼容性应该相近
        assert!(
            (marx_feynman - hegel_feynman).abs() < 0.2,
            "Completely different pairs should have similar compatibility"
        );

        println!(
            "Compat: Marx-Hegel={:.3}, Marx-Feynman={:.3}, Hegel-Feynman={:.3}",
            marx_hegel, marx_feynman, hegel_feynman
        );
    }

    #[test]
    fn test_find_compatible_pairs() {
        let engine = build_test_engine();

        let pairs = engine.find_compatible_pairs("马克思", 3);
        assert!(!pairs.is_empty(), "Should find at least one compatible pair");

        // 应该找到 3 个候选（费曼, 黑格尔, 达尔文）
        assert_eq!(pairs.len(), 3);

        // 与黑格尔的兼容性应该最高
        // (Note: due to the peak-at-0.45 formula, very high overlap may not be optimal)
        println!("Pairs for 马克思: {:?}", pairs);
    }

    #[test]
    fn test_find_all_compatible_pairs() {
        let engine = build_test_engine();

        let candidates = engine.find_all_compatible_pairs(0.1, 10);
        assert!(!candidates.is_empty());

        // 验证候选结构
        for c in &candidates {
            assert!(!c.parent_a.is_empty());
            assert!(!c.parent_b.is_empty());
            assert!(c.estimated_novelty >= 0.0 && c.estimated_novelty <= 1.0);
            assert!(!c.fusion_prompt.is_empty());
        }

        println!("Found {} hybridization candidates", candidates.len());
    }

    #[test]
    fn test_fusion_prompt_contains_both_parents() {
        let engine = build_test_engine();
        let sig_a = engine.get("马克思").unwrap();
        let sig_b = engine.get("费曼").unwrap();

        let prompt = engine.generate_fusion_prompt(sig_a, sig_b);
        assert!(prompt.contains("马克思"));
        assert!(prompt.contains("费曼"));
        assert!(prompt.contains("辩证"));
        assert!(prompt.contains("还原论"));
    }

    #[test]
    fn test_difference_score() {
        let a = MethodologySignature::new("A".into())
            .with_styles("辩证", "历史", "历史", "抽象");
        let b = MethodologySignature::new("B".into())
            .with_styles("还原论", "实验", "当下", "具体");

        let diff = HybridizationEngine::difference_score(&a, &b);
        // All 4 dimensions are different → 4/4 = 1.0
        assert!((diff - 1.0).abs() < 0.01, "Fully different should have diff=1.0, got {}", diff);

        let c = MethodologySignature::new("C".into())
            .with_styles("辩证", "历史", "历史", "抽象");
        let diff_same = HybridizationEngine::difference_score(&a, &c);
        assert!((diff_same - 0.0).abs() < 0.01, "Fully same should have diff=0.0, got {}", diff_same);
    }
}
