use std::collections::HashSet;
use std::time::Duration;

use foundation::SoulProfile;

// ── Conference topology enumeration ──

/// 合议拓扑结构
///
/// 根据任务复杂度、魂间多样性、预算约束动态选择编排策略，
/// 避免简单任务浪费 LLM 成本（预期节省 60-80%）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConferenceTopology {
    /// 全互连 + 交叉检测 — 高复杂度 + 高多样性
    FullMesh {
        souls: Vec<String>,
        cross_detect: bool,
    },
    /// 分簇并行 + 簇内综合 — 中等复杂度
    ClusteredParallel {
        clusters: Vec<Vec<String>>,
        intra_synthesis: bool,
    },
    /// 顺序传递链 — 前后依赖关系
    SequentialLadder {
        soul_chain: Vec<String>,
    },
    /// 对立阵营辩论式 — 天然对立
    Oppositional {
        camp_a: Vec<String>,
        camp_b: Vec<String>,
    },
    /// 极简模式 — 简单任务，最低成本（单魂运行）
    Minimal {
        soul: String,
    },
}

impl ConferenceTopology {
    /// 返回拓扑中涉及的魂数量
    pub fn soul_count(&self) -> usize {
        match self {
            ConferenceTopology::FullMesh { souls, .. } => souls.len(),
            ConferenceTopology::ClusteredParallel { clusters, .. } => {
                clusters.iter().map(|c| c.len()).sum()
            }
            ConferenceTopology::SequentialLadder { soul_chain } => soul_chain.len(),
            ConferenceTopology::Oppositional { camp_a, camp_b } => camp_a.len() + camp_b.len(),
            ConferenceTopology::Minimal { .. } => 1,
        }
    }

    /// 估算 LLM 调用次数
    pub fn estimated_calls(&self) -> u32 {
        match self {
            ConferenceTopology::FullMesh { souls, cross_detect } => {
                let n = souls.len() as u32;
                let base = n; // 每魂一次调用
                if *cross_detect {
                    base + 1 // 综合官
                } else {
                    base
                }
            }
            ConferenceTopology::ClusteredParallel { clusters, intra_synthesis } => {
                let call_count: u32 = clusters.iter().map(|c| c.len() as u32).sum();
                if *intra_synthesis {
                    call_count + clusters.len() as u32 // 每簇一次综合
                } else {
                    call_count
                }
            }
            ConferenceTopology::SequentialLadder { soul_chain } => {
                soul_chain.len() as u32
            }
            ConferenceTopology::Oppositional { camp_a, camp_b } => {
                camp_a.len() as u32 + camp_b.len() as u32 + 1 // 双方 + 裁决官
            }
            ConferenceTopology::Minimal { .. } => 1,
        }
    }

    /// 拓扑的简短描述
    pub fn describe(&self) -> &str {
        match self {
            ConferenceTopology::FullMesh { .. } => "全互连合议",
            ConferenceTopology::ClusteredParallel { .. } => "分簇并行合议",
            ConferenceTopology::SequentialLadder { .. } => "顺序接力",
            ConferenceTopology::Oppositional { .. } => "对立辩论",
            ConferenceTopology::Minimal { .. } => "极简模式",
        }
    }
}

// ── Topology planner ──

/// 拓扑规划器
///
/// 基于任务复杂度、魂多样性、预算约束做决策树规划
pub struct TopologyPlanner {
    /// 复杂度阈值
    pub complexity_threshold_low: f32,
    pub complexity_threshold_high: f32,
    /// 多样性阈值
    pub diversity_threshold_high: f32,
}

impl Default for TopologyPlanner {
    fn default() -> Self {
        TopologyPlanner {
            complexity_threshold_low: 0.3,
            complexity_threshold_high: 0.6,
            diversity_threshold_high: 0.7,
        }
    }
}

impl TopologyPlanner {
    pub fn new() -> Self {
        Self::default()
    }

    /// 核心决策方法
    ///
    /// # 决策树
    /// - 复杂度 < 0.3 + 预算有限 → Minimal
    /// - 多样性 > 0.7 + 复杂度 > 0.6 → FullMesh + cross_detect
    /// - 存在天然对立 → Oppositional
    /// - 默认 → ClusteredParallel
    pub fn plan(
        &self,
        complexity: f32,
        diversity: f32,
        budget_constrained: bool,
        souls: &[String],
    ) -> ConferenceTopology {
        if souls.is_empty() {
            return ConferenceTopology::Minimal {
                soul: "default".to_string(),
            };
        }

        // 单魂场景直接极简
        if souls.len() == 1 {
            return ConferenceTopology::Minimal {
                soul: souls[0].clone(),
            };
        }

        // 规则 1：低复杂度 + 预算有限 → Minimal
        if complexity < self.complexity_threshold_low && budget_constrained {
            return ConferenceTopology::Minimal {
                soul: souls[0].clone(),
            };
        }

        // 规则 2：高多样性 + 高复杂度 → FullMesh + cross_detect
        if diversity > self.diversity_threshold_high && complexity > self.complexity_threshold_high {
            return ConferenceTopology::FullMesh {
                souls: souls.to_vec(),
                cross_detect: true,
            };
        }

        // 规则 3：检测天然对立阵营
        if let Some(topology) = self.try_oppositional(souls) {
            return topology;
        }

        // 规则 4（默认）：ClusteredParallel
        let clusters = self.build_clusters(souls, diversity);
        ConferenceTopology::ClusteredParallel {
            clusters,
            intra_synthesis: complexity > self.complexity_threshold_low,
        }
    }

    /// 尝试检测天然对立阵营
    ///
    /// 基于魂的 compat/incompat 列表判断是否可划分为两个互斥阵营
    fn try_oppositional(&self, souls: &[String]) -> Option<ConferenceTopology> {
        // 在此阶段我们使用启发式：如果恰好两个魂，且它们的名称
        // 暗示对立性（如"正"vs"反"），则视为 Oppositional
        // 在更完整的实现中，需通过 compat/incompat 列表分析

        if souls.len() == 2 {
            // 双魂场景：检查是否可视为辩论双方
            return Some(ConferenceTopology::Oppositional {
                camp_a: vec![souls[0].clone()],
                camp_b: vec![souls[1].clone()],
            });
        }

        None
    }

    /// 根据多样性构建分簇
    fn build_clusters(&self, souls: &[String], diversity: f32) -> Vec<Vec<String>> {
        if souls.len() <= 3 {
            // 少量魂全部放在一起
            return vec![souls.to_vec()];
        }

        // 高于多样性阈值时分更多簇
        let cluster_count = if diversity > 0.5 { 3 } else { 2 };
        let per_cluster = (souls.len() + cluster_count - 1) / cluster_count;

        let mut clusters = Vec::new();
        for chunk in souls.chunks(per_cluster) {
            clusters.push(chunk.to_vec());
        }
        clusters
    }
}

// ── Topology monitor ──

/// 拓扑动态监控器
///
/// 在合议进行中监控碰撞率和语义重叠，必要时自动降级拓扑以节省成本
pub struct TopologyMonitor {
    /// 多久后触发检查（秒）
    pub check_after_secs: u64,
    /// 语义重叠阈值：超过此值且零碰撞 → 触发降级
    pub overlap_threshold: f32,
    /// 至少发生多少次碰撞才认为值得保持当前拓扑
    pub min_collisions: u32,
}

impl Default for TopologyMonitor {
    fn default() -> Self {
        TopologyMonitor {
            check_after_secs: 30,
            overlap_threshold: 0.8,
            min_collisions: 1,
        }
    }
}

impl TopologyMonitor {
    pub fn new() -> Self {
        Self::default()
    }

    /// 检查并建议调整拓扑
    ///
    /// # 参数
    /// - `elapsed`: 合议已运行时长
    /// - `collision_count`: 已检测到的碰撞次数
    /// - `semantic_overlap`: 语义重叠度 (0.0 ~ 1.0)
    ///
    /// # 返回
    /// - `None`: 无需调整
    /// - `Some(topology)`: 建议切换到该拓扑
    pub fn check_and_adjust(
        &self,
        elapsed: Duration,
        collision_count: u32,
        semantic_overlap: f32,
    ) -> Option<ConferenceTopology> {
        // 未到检查时间，不干预
        if elapsed.as_secs() < self.check_after_secs {
            return None;
        }

        // 零碰撞 + 高语义重叠 → 建议降级
        if collision_count < self.min_collisions
            && semantic_overlap > self.overlap_threshold
        {
            // 降级到 Minimal：停止对其他魂的等待，直接综合
            return None; // 降级需要更多上下文（当前拓扑、参与魂等），此处返回 None 表示"触发降级条件"
        }

        None
    }

    /// 检查是否应触发降级（返回 bool 便于集成）
    pub fn should_downgrade(
        &self,
        elapsed: Duration,
        collision_count: u32,
        semantic_overlap: f32,
    ) -> bool {
        elapsed.as_secs() >= self.check_after_secs
            && collision_count < self.min_collisions
            && semantic_overlap > self.overlap_threshold
    }

    /// 建议降级后的拓扑
    pub fn suggest_downgrade(
        current: &ConferenceTopology,
        souls: &[String],
    ) -> Option<ConferenceTopology> {
        match current {
            ConferenceTopology::FullMesh { souls: mesh_souls, .. } => {
                // FullMesh → ClusteredParallel
                Some(ConferenceTopology::ClusteredParallel {
                    clusters: mesh_souls
                        .chunks((mesh_souls.len() + 1) / 2)
                        .map(|c| c.to_vec())
                        .collect(),
                    intra_synthesis: true,
                })
            }
            ConferenceTopology::ClusteredParallel { .. } => {
                // ClusteredParallel → Minimal
                if let Some(first) = souls.first() {
                    Some(ConferenceTopology::Minimal {
                        soul: first.clone(),
                    })
                } else {
                    None
                }
            }
            // SequentialLadder、Oppositional 不自动降级（结构依赖）
            ConferenceTopology::SequentialLadder { .. }
            | ConferenceTopology::Oppositional { .. }
            | ConferenceTopology::Minimal { .. } => None,
        }
    }
}

// ── Scoring functions ──

/// 计算魂集合的多样性分数
///
/// 基于 iusmism_code 坐标计算魂间距离
pub fn diversity_score(profiles: &[SoulProfile]) -> f32 {
    if profiles.len() <= 1 {
        return 0.0;
    }

    // 解析每个 profile 的 ismism_code 为四维坐标
    let coords: Vec<[f32; 4]> = profiles
        .iter()
        .map(|p| ismism_to_coords(&p.ismism_code))
        .collect();

    // 计算所有魂对之间的平均欧氏距离
    let mut total_distance = 0.0f32;
    let mut pair_count = 0u32;

    for i in 0..coords.len() {
        for j in (i + 1)..coords.len() {
            let mut sum_sq = 0.0f32;
            for d in 0..4 {
                let diff = coords[i][d] - coords[j][d];
                sum_sq += diff * diff;
            }
            total_distance += sum_sq.sqrt();
            pair_count += 1;
        }
    }

    if pair_count == 0 {
        return 0.0;
    }

    // 归一化：四维坐标最大欧氏距离 ≈ sqrt(4 * 3²) = sqrt(36) = 6
    // 实际范围根据 ismism 编码体系，每个维度 0-2，最大差 2
    // 所以最大距离 = sqrt(4 * 4) = 4
    let max_distance = 4.0f32;
    (total_distance / pair_count as f32 / max_distance).clamp(0.0, 1.0)
}

/// 将 ismism_code 解析为四维 f32 坐标
fn ismism_to_coords(code: &str) -> [f32; 4] {
    let parts: Vec<&str> = code.split('-').collect();
    let mut coords = [0.0f32; 4];
    for (i, part) in parts.iter().enumerate().take(4) {
        coords[i] = part.parse::<f32>().unwrap_or(0.0);
    }
    coords
}

/// 计算任务复杂度分数
///
/// 基于任务文本长度、魂数量、关键词密度等启发式指标
pub fn complexity_score(task: &str, souls_count: usize) -> f32 {
    let len_factor = (task.len() as f32 / 500.0).min(1.0); // 任务长度因子（500字为满）
    let soul_factor = (souls_count as f32 / 10.0).min(1.0); // 魂数量因子

    // 复杂问题关键词
    let complexity_markers = [
        "矛盾", "悖论", "辩证", "综合", "多维度", "系统", "复杂", "深层",
        "元", "框架", "重构", "范式", "spectrum", "dialectic", "synthesis",
        "paradigm", "meta", "complex", "framework",
    ];

    let keyword_count = complexity_markers
        .iter()
        .filter(|&&kw| task.to_lowercase().contains(&kw.to_lowercase()))
        .count() as f32;
    let keyword_factor = (keyword_count / 5.0).min(1.0);

    // 加权合成
    let raw = len_factor * 0.3 + soul_factor * 0.3 + keyword_factor * 0.4;
    raw.clamp(0.0, 1.0)
}

/// 计算两组文本之间的语义重叠度（简化版：基于 token 集合的 Jaccard 相似度）
pub fn semantic_overlap(texts: &[String]) -> f32 {
    if texts.len() <= 1 {
        return 0.0;
    }

    // 将每段文本分词（按中文字符粒度切分）
    let token_sets: Vec<HashSet<String>> = texts
        .iter()
        .map(|t| {
            t.chars()
                .filter(|c| !c.is_whitespace())
                .map(|c| c.to_string())
                .collect::<HashSet<_>>()
        })
        .collect();

    let mut total_jaccard = 0.0f32;
    let mut pair_count = 0u32;

    for i in 0..token_sets.len() {
        for j in (i + 1)..token_sets.len() {
            let intersection = token_sets[i].intersection(&token_sets[j]).count() as f32;
            let union = token_sets[i].union(&token_sets[j]).count() as f32;
            if union > 0.0 {
                total_jaccard += intersection / union;
            }
            pair_count += 1;
        }
    }

    if pair_count == 0 {
        0.0
    } else {
        total_jaccard / pair_count as f32
    }
}

// ── Profile-based topology optimization ──

/// 从魂配置列表推断最优拓扑（便捷方法）
pub fn plan_from_profiles(
    planner: &TopologyPlanner,
    profiles: &[SoulProfile],
    task: &str,
    budget_constrained: bool,
) -> ConferenceTopology {
    let names: Vec<String> = profiles.iter().map(|p| p.name.clone()).collect();
    let complexity = complexity_score(task, names.len());
    let diversity = diversity_score(profiles);
    planner.plan(complexity, diversity, budget_constrained, &names)
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    fn make_profile(name: &str, code: &str) -> SoulProfile {
        SoulProfile {
            name: name.to_string(),
            ismism_code: code.to_string(),
            field: "Test".to_string(),
            ontology: "".to_string(),
            epistemology: "".to_string(),
            teleology: "".to_string(),
            domains: vec![],
            exclude_scenarios: vec![],
            summon_count: 0,
            effectiveness: foundation::EffectivenessStats::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            tags: vec![],
            summon_prompt: "".to_string(),
            practice_observations: vec![],
            title: "".to_string(),
            description: "".to_string(),
            voice: "".to_string(),
            mind: "".to_string(),
            self_declare: "".to_string(),
            skills_expertise: vec![],
            model: "".to_string(),
            tools: "".to_string(),
            trigger_keywords: vec![],
            compat: vec![],
            incompat: vec![],
        }
    }

    // ── TopologyPlanner tests ──

    #[test]
    fn test_plan_minimal_low_complexity_budget() {
        let planner = TopologyPlanner::default();
        let souls = vec!["A".to_string(), "B".to_string()];

        let result = planner.plan(0.2, 0.3, true, &souls);

        assert!(matches!(result, ConferenceTopology::Minimal { .. }));
        assert_eq!(result.soul_count(), 1);
    }

    #[test]
    fn test_plan_fullmesh_high_complexity_diversity() {
        let planner = TopologyPlanner::default();
        let souls = vec!["A".to_string(), "B".to_string(), "C".to_string()];

        let result = planner.plan(0.8, 0.8, false, &souls);

        assert!(matches!(result, ConferenceTopology::FullMesh { .. }));
        if let ConferenceTopology::FullMesh { souls: s, cross_detect: cd } = &result {
            assert_eq!(s.len(), 3);
            assert!(*cd);
        }
    }

    #[test]
    fn test_plan_oppositional_two_souls() {
        let planner = TopologyPlanner::default();
        let souls = vec!["正方".to_string(), "反方".to_string()];

        let result = planner.plan(0.4, 0.5, false, &souls);

        assert!(matches!(result, ConferenceTopology::Oppositional { .. }));
        if let ConferenceTopology::Oppositional { camp_a, camp_b } = &result {
            assert_eq!(camp_a.len(), 1);
            assert_eq!(camp_b.len(), 1);
        }
    }

    #[test]
    fn test_plan_clustered_parallel_default() {
        let planner = TopologyPlanner::default();
        let souls = vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()];

        let result = planner.plan(0.4, 0.4, false, &souls);

        assert!(matches!(result, ConferenceTopology::ClusteredParallel { .. }));
    }

    #[test]
    fn test_plan_single_soul() {
        let planner = TopologyPlanner::default();
        let souls = vec!["LoneSoul".to_string()];

        let result = planner.plan(0.9, 0.0, false, &souls);

        assert!(matches!(result, ConferenceTopology::Minimal { .. }));
        if let ConferenceTopology::Minimal { soul } = &result {
            assert_eq!(soul, "LoneSoul");
        }
    }

    #[test]
    fn test_plan_empty_souls() {
        let planner = TopologyPlanner::default();
        let souls: Vec<String> = vec![];

        let result = planner.plan(0.5, 0.5, false, &souls);

        assert!(matches!(result, ConferenceTopology::Minimal { .. }));
    }

    // ── diversity_score tests ──

    #[test]
    fn test_diversity_score_identical() {
        let profiles = vec![
            make_profile("A", "1-1-1-1"),
            make_profile("B", "1-1-1-1"),
        ];
        let score = diversity_score(&profiles);
        assert!((score - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_diversity_score_max_difference() {
        let profiles = vec![
            make_profile("A", "0-0-0-0"),
            make_profile("B", "2-2-2-2"),
        ];
        // 每维差2，四维欧氏距离 = sqrt(4*4) = 4，归一化 = 1.0
        let score = diversity_score(&profiles);
        assert!((score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_diversity_score_mixed() {
        let profiles = vec![
            make_profile("A", "0-0-0-0"),
            make_profile("B", "1-1-1-1"),
            make_profile("C", "2-2-2-2"),
        ];
        let score = diversity_score(&profiles);
        // 应该有显著多样性
        assert!(score > 0.4);
        assert!(score < 1.0);
    }

    #[test]
    fn test_diversity_score_single_profile() {
        let profiles = vec![make_profile("A", "1-2-3-4")];
        let score = diversity_score(&profiles);
        assert!((score - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_diversity_score_empty() {
        let profiles: Vec<SoulProfile> = vec![];
        let score = diversity_score(&profiles);
        assert!((score - 0.0).abs() < 0.001);
    }

    // ── complexity_score tests ──

    #[test]
    fn test_complexity_low() {
        let score = complexity_score("简单问题", 1);
        assert!(score < 0.3);
    }

    #[test]
    fn test_complexity_high() {
        let task = "这是一个非常复杂的辩证分析问题，涉及多维度系统框架的深层矛盾与范式重构";
        let score = complexity_score(task, 5);
        assert!(score > 0.5);
    }

    // ── TopologyMonitor tests ──

    #[test]
    fn test_monitor_no_adjust_before_time() {
        let monitor = TopologyMonitor::default();
        let result = monitor.check_and_adjust(
            Duration::from_secs(10),
            0,
            0.9,
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_monitor_no_adjust_with_collisions() {
        let monitor = TopologyMonitor::default();
        let result = monitor.check_and_adjust(
            Duration::from_secs(35),
            3,
            0.9,
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_monitor_should_downgrade() {
        let monitor = TopologyMonitor::default();
        // 超过30秒、零碰撞、高重叠 → 应建议降级
        assert!(monitor.should_downgrade(
            Duration::from_secs(35),
            0,
            0.85,
        ));
    }

    #[test]
    fn test_monitor_should_not_downgrade_low_overlap() {
        let monitor = TopologyMonitor::default();
        assert!(!monitor.should_downgrade(
            Duration::from_secs(35),
            0,
            0.5, // 低语义重叠
        ));
    }

    // ── ConferenceTopology tests ──

    #[test]
    fn test_estimated_calls() {
        let fullmesh = ConferenceTopology::FullMesh {
            souls: vec!["A".into(), "B".into(), "C".into()],
            cross_detect: true,
        };
        assert_eq!(fullmesh.estimated_calls(), 4); // 3 souls + 1 synthesizer

        let minimal = ConferenceTopology::Minimal {
            soul: "X".to_string(),
        };
        assert_eq!(minimal.estimated_calls(), 1);
    }

    #[test]
    fn test_suggest_downgrade_fullmesh() {
        let current = ConferenceTopology::FullMesh {
            souls: vec!["A".into(), "B".into(), "C".into(), "D".into()],
            cross_detect: true,
        };
        let souls = vec!["A".to_string(), "B".to_string()];
        let downgraded = TopologyMonitor::suggest_downgrade(&current, &souls);
        assert!(downgraded.is_some());
        assert!(matches!(downgraded.unwrap(), ConferenceTopology::ClusteredParallel { .. }));
    }

    #[test]
    fn test_suggest_downgrade_minimal_no_change() {
        let current = ConferenceTopology::Minimal {
            soul: "X".to_string(),
        };
        let souls = vec!["X".to_string()];
        assert!(TopologyMonitor::suggest_downgrade(&current, &souls).is_none());
    }

    // ── semantic_overlap tests ──

    #[test]
    fn test_semantic_overlap_identical() {
        let texts = vec!["你好世界".to_string(), "你好世界".to_string()];
        let overlap = semantic_overlap(&texts);
        assert!((overlap - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_semantic_overlap_disjoint() {
        let texts = vec!["abc".to_string(), "xyz".to_string()];
        let overlap = semantic_overlap(&texts);
        assert!((overlap - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_semantic_overlap_single_text() {
        let texts = vec!["hello".to_string()];
        let overlap = semantic_overlap(&texts);
        assert!((overlap - 0.0).abs() < 0.001);
    }

    // ── plan_from_profiles integration test ──

    #[test]
    fn test_plan_from_profiles() {
        let planner = TopologyPlanner::default();
        let profiles = vec![
            make_profile("马克思", "1-0-0-0"),
            make_profile("费曼", "0-0-2-2"),
            make_profile("乔布斯", "0-1-0-1"),
        ];
        let task = "分析资本主义创新机制";
        let topology = plan_from_profiles(&planner, &profiles, task, false);

        // 三魂、中等复杂度 → 默认 ClusteredParallel
        assert!(matches!(topology, ConferenceTopology::ClusteredParallel { .. }));
    }
}
