use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// 信誉变更事件类型
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ReputationEvent {
    /// 修正提案被批准
    Approval {
        soul_name: String,
        delta: f32,
        timestamp: DateTime<Utc>,
    },
    /// 输出被判定为矛盾/错误
    Contradiction {
        soul_name: String,
        description: String,
        delta: f32,
        timestamp: DateTime<Utc>,
    },
    /// 实践验证通过
    PracticeValidation {
        soul_name: String,
        observation: String,
        delta: f32,
        timestamp: DateTime<Utc>,
    },
    /// 记录了新颖洞察
    Insight {
        soul_name: String,
        summary: String,
        delta: f32,
        timestamp: DateTime<Utc>,
    },
}

/// 单个魂的信誉档案
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SoulReputation {
    pub soul_name: String,
    /// 修正提案被批准比例 (0.0 ~ 1.0)
    pub approval_rate: f32,
    /// 被判定为矛盾/错误的比例 (0.0 ~ 1.0)
    pub contradiction_rate: f32,
    /// 实践验证通过率 (0.0 ~ 1.0)
    pub practice_validation_rate: f32,
    /// 独特观点比例 (0.0 ~ 1.0)
    pub insight_novelty: f32,
    /// 加权总分 (0.0 ~ 1.0)
    pub reputation_score: f32,
    /// 总贡献次数（approval + practice + insight + contradiction）
    pub total_contributions: u64,
    /// 最近一次评估时间
    pub last_evaluated: DateTime<Utc>,
}

impl SoulReputation {
    pub fn new(soul_name: String) -> Self {
        SoulReputation {
            soul_name,
            approval_rate: 0.5,
            contradiction_rate: 0.0,
            practice_validation_rate: 0.5,
            insight_novelty: 0.5,
            reputation_score: 0.475,
            total_contributions: 0,
            last_evaluated: Utc::now(),
        }
    }
}

/// 魂信誉管理器
///
/// 所有计算均为确定性逻辑，不依赖 LLM 调用。
/// 加权公式：0.3 * approval + 0.25 * (1 - contradiction) + 0.25 * practice + 0.2 * novelty
#[derive(Debug, Clone)]
pub struct ReputationManager {
    entries: HashMap<String, SoulReputation>,
    /// 审查阈值：信誉分低于此值的魂需要优先审查
    review_threshold: f32,
    /// 休眠阈值：信誉分低于此值的魂自动休眠
    dormant_threshold: f32,
}

impl ReputationManager {
    pub fn new() -> Self {
        ReputationManager {
            entries: HashMap::new(),
            review_threshold: 0.4,
            dormant_threshold: 0.2,
        }
    }

    pub fn with_thresholds(review: f32, dormant: f32) -> Self {
        ReputationManager {
            entries: HashMap::new(),
            review_threshold: review,
            dormant_threshold: dormant,
        }
    }

    /// 获取或创建魂的信誉记录
    pub fn get_or_create(&mut self, soul_name: &str) -> &mut SoulReputation {
        self.entries
            .entry(soul_name.to_string())
            .or_insert_with(|| SoulReputation::new(soul_name.to_string()))
    }

    /// 获取魂的信誉记录（只读）
    pub fn get(&self, soul_name: &str) -> Option<&SoulReputation> {
        self.entries.get(soul_name)
    }

    /// 记录一次批准事件
    pub fn record_approval(&mut self, soul_name: &str) -> ReputationEvent {
        let entry = self.get_or_create(soul_name);
        let delta = 0.05_f32.min(1.0 - entry.approval_rate);
        entry.approval_rate = (entry.approval_rate + delta).min(1.0);
        entry.total_contributions += 1;
        self.recalculate(soul_name);

        ReputationEvent::Approval {
            soul_name: soul_name.to_string(),
            delta,
            timestamp: Utc::now(),
        }
    }

    /// 记录一次矛盾/错误
    pub fn record_contradiction(
        &mut self,
        soul_name: &str,
        description: &str,
    ) -> ReputationEvent {
        let entry = self.get_or_create(soul_name);
        let delta = 0.05_f32.min(entry.contradiction_rate + 0.05);
        entry.contradiction_rate = (entry.contradiction_rate + 0.05).min(1.0);
        entry.total_contributions += 1;
        self.recalculate(soul_name);

        ReputationEvent::Contradiction {
            soul_name: soul_name.to_string(),
            description: description.to_string(),
            delta,
            timestamp: Utc::now(),
        }
    }

    /// 记录一次实践验证通过
    pub fn record_practice_validation(
        &mut self,
        soul_name: &str,
        observation: &str,
    ) -> ReputationEvent {
        let entry = self.get_or_create(soul_name);
        let delta = 0.05_f32.min(1.0 - entry.practice_validation_rate);
        entry.practice_validation_rate = (entry.practice_validation_rate + delta).min(1.0);
        entry.total_contributions += 1;
        self.recalculate(soul_name);

        ReputationEvent::PracticeValidation {
            soul_name: soul_name.to_string(),
            observation: observation.to_string(),
            delta,
            timestamp: Utc::now(),
        }
    }

    /// 记录一次新颖洞察
    pub fn record_insight(&mut self, soul_name: &str, summary: &str) -> ReputationEvent {
        let entry = self.get_or_create(soul_name);
        let delta = 0.05_f32.min(1.0 - entry.insight_novelty);
        entry.insight_novelty = (entry.insight_novelty + delta).min(1.0);
        entry.total_contributions += 1;
        self.recalculate(soul_name);

        ReputationEvent::Insight {
            soul_name: soul_name.to_string(),
            summary: summary.to_string(),
            delta,
            timestamp: Utc::now(),
        }
    }

    /// 重新计算魂的信誉分数
    ///
    /// 加权公式：
    /// score = 0.3 * approval_rate + 0.25 * (1 - contradiction_rate) + 0.25 * practice_validation_rate + 0.2 * insight_novelty
    pub fn recalculate(&mut self, soul_name: &str) -> f32 {
        if let Some(entry) = self.entries.get_mut(soul_name) {
            let score = 0.3 * entry.approval_rate
                + 0.25 * (1.0 - entry.contradiction_rate)
                + 0.25 * entry.practice_validation_rate
                + 0.2 * entry.insight_novelty;
            entry.reputation_score = score.clamp(0.0, 1.0);
            entry.last_evaluated = Utc::now();
            entry.reputation_score
        } else {
            0.0
        }
    }

    /// 获取信誉最高的前 n 个魂
    pub fn get_top_souls(&self, n: usize) -> Vec<SoulReputation> {
        let mut entries: Vec<_> = self.entries.values().cloned().collect();
        entries.sort_by(|a, b| {
            b.reputation_score
                .partial_cmp(&a.reputation_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.total_contributions.cmp(&a.total_contributions))
        });
        entries.truncate(n);
        entries
    }

    /// 获取信誉最低的前 n 个魂（用于审查优先级）
    pub fn get_bottom_souls(&self, n: usize) -> Vec<SoulReputation> {
        let mut entries: Vec<_> = self.entries.values().cloned().collect();
        entries.sort_by(|a, b| {
            a.reputation_score
                .partial_cmp(&b.reputation_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.total_contributions.cmp(&b.total_contributions))
        });
        entries.truncate(n);
        entries
    }

    /// 判断魂是否需要优先审查
    pub fn should_review(&self, soul_name: &str) -> bool {
        self.entries
            .get(soul_name)
            .map(|e| e.reputation_score < self.review_threshold && e.total_contributions >= 3)
            .unwrap_or(false)
    }

    /// 获取魂在辩论中的权重（基于信誉分数）
    pub fn get_debate_weight(&self, soul_name: &str) -> f32 {
        self.entries
            .get(soul_name)
            .map(|e| {
                // 权重映射：信誉分 0.0→0.5, 0.5→1.0, 1.0→1.5
                0.5 + e.reputation_score
            })
            .unwrap_or(1.0)
    }

    /// 获取应自动休眠的魂列表（信誉分低于休眠阈值）
    pub fn get_dormant_souls(&self) -> Vec<SoulReputation> {
        self.entries
            .values()
            .filter(|e| e.reputation_score < self.dormant_threshold && e.total_contributions >= 5)
            .cloned()
            .collect()
    }

    /// 获取所有魂的信誉快照
    pub fn all_entries(&self) -> Vec<SoulReputation> {
        self.entries.values().cloned().collect()
    }

    /// 获取当前条目数量
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for ReputationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded_manager() -> ReputationManager {
        let mut mgr = ReputationManager::new();

        // Soul A: 高信誉 — 高 approval, 低 contradiction, 高 practice, 高 novelty
        mgr.record_approval("SoulA");
        mgr.record_approval("SoulA");
        mgr.record_approval("SoulA");
        mgr.record_practice_validation("SoulA", "实践A正确");
        mgr.record_insight("SoulA", "新颖观点A");

        // Soul B: 中等 — 中 approval, 中 contradiction, 中 practice
        mgr.record_approval("SoulB");
        mgr.record_contradiction("SoulB", "逻辑矛盾");

        // Soul C: 低信誉 — 高 contradiction
        mgr.record_contradiction("SoulC", "事实错误");
        mgr.record_contradiction("SoulC", "推理缺陷");
        mgr.record_contradiction("SoulC", "预设不成立");

        mgr
    }

    #[test]
    fn test_new_soul_defaults() {
        let rep = SoulReputation::new("TestSoul".into());
        assert_eq!(rep.soul_name, "TestSoul");
        assert!((rep.reputation_score - 0.475).abs() < 0.001);
        assert_eq!(rep.total_contributions, 0);
    }

    #[test]
    fn test_weighted_formula_components() {
        // 验证公式权重加和为 1.0
        // 0.3 + 0.25 + 0.25 + 0.2 = 1.0
        let mut mgr = ReputationManager::new();

        // Perfect soul
        let perfect = mgr.get_or_create("Perfect");
        perfect.approval_rate = 1.0;
        perfect.contradiction_rate = 0.0;
        perfect.practice_validation_rate = 1.0;
        perfect.insight_novelty = 1.0;

        let score = mgr.recalculate("Perfect");
        // 0.3*1.0 + 0.25*1.0 + 0.25*1.0 + 0.2*1.0 = 1.0
        assert!((score - 1.0).abs() < 0.001, "Perfect soul should score 1.0, got {}", score);

        // Worst soul
        let worst = mgr.get_or_create("Worst");
        worst.approval_rate = 0.0;
        worst.contradiction_rate = 1.0;
        worst.practice_validation_rate = 0.0;
        worst.insight_novelty = 0.0;

        let score = mgr.recalculate("Worst");
        // 0.3*0.0 + 0.25*0.0 + 0.25*0.0 + 0.2*0.0 = 0.0
        assert!((score - 0.0).abs() < 0.001, "Worst soul should score 0.0, got {}", score);
    }

    #[test]
    fn test_recalculate_mid_range() {
        let mut mgr = ReputationManager::new();
        let mid = mgr.get_or_create("Mid");
        mid.approval_rate = 0.5;
        mid.contradiction_rate = 0.5;
        mid.practice_validation_rate = 0.5;
        mid.insight_novelty = 0.5;

        let score = mgr.recalculate("Mid");
        // 0.3*0.5 + 0.25*0.5 + 0.25*0.5 + 0.2*0.5 = 0.5
        assert!((score - 0.5).abs() < 0.001, "Mid soul should score 0.5, got {}", score);
    }

    #[test]
    fn test_record_approval_increases_score() {
        let mut mgr = ReputationManager::new();
        let before = mgr.get_or_create("Soul").reputation_score;
        mgr.record_approval("Soul");
        let after = mgr.get("Soul").unwrap().reputation_score;
        assert!(after > before, "Approval should increase score");
    }

    #[test]
    fn test_record_contradiction_decreases_score() {
        let mut mgr = ReputationManager::new();
        // Start from a good baseline so the drop is measurable
        mgr.record_approval("Soul");
        mgr.record_approval("Soul");
        let before = mgr.get("Soul").unwrap().reputation_score;
        mgr.record_contradiction("Soul", "测试矛盾");
        let after = mgr.get("Soul").unwrap().reputation_score;
        assert!(after < before, "Contradiction should decrease score");
    }

    #[test]
    fn test_top_bottom_sorting() {
        let mgr = seeded_manager();

        let top = mgr.get_top_souls(3);
        assert_eq!(top.len(), 3);
        // SoulA should be first (highest score)
        assert_eq!(top[0].soul_name, "SoulA");

        let bottom = mgr.get_bottom_souls(3);
        assert_eq!(bottom.len(), 3);
        // SoulC should be first (lowest score)
        assert_eq!(bottom[0].soul_name, "SoulC");
    }

    #[test]
    fn test_should_review() {
        let mut mgr = ReputationManager::with_thresholds(0.6, 0.2);

        // Fresh soul with few contributions — no review needed
        mgr.record_contradiction("NewSoul", "err");
        assert!(!mgr.should_review("NewSoul"), "New soul with <3 contributions should not trigger review");

        // Soul with many contradictions and enough contributions
        for _ in 0..5 {
            mgr.record_contradiction("BadSoul", "repeated errors");
        }
        assert!(mgr.should_review("BadSoul"), "Low-reputation soul with >=3 contributions should trigger review");
    }

    #[test]
    fn test_debate_weight() {
        let mut mgr = ReputationManager::new();

        // Unknown soul gets default weight 1.0
        assert!((mgr.get_debate_weight("Unknown") - 1.0).abs() < 0.001);

        // High reputation soul
        for _ in 0..4 {
            mgr.record_approval("HighRep");
            mgr.record_practice_validation("HighRep", "validated");
            mgr.record_insight("HighRep", "insight");
        }
        let weight = mgr.get_debate_weight("HighRep");
        assert!(weight > 1.0, "High reputation soul should have weight > 1.0, got {}", weight);
    }

    #[test]
    fn test_dormant_souls() {
        let mut mgr = ReputationManager::with_thresholds(0.6, 0.6);

        // Soul with many contradictions and enough contributions
        for _ in 0..6 {
            mgr.record_contradiction("DormantSoul", "error");
        }
        // After 6 contradictions, contradiction_rate should be high, score drops below 0.6
        let entry = mgr.get("DormantSoul").unwrap();
        assert!(entry.total_contributions >= 5);

        let dormant = mgr.get_dormant_souls();
        assert!(!dormant.is_empty(), "Should have dormant souls");
    }

    #[test]
    fn test_all_entries() {
        let mgr = seeded_manager();
        let all = mgr.all_entries();
        assert_eq!(all.len(), 3);
    }
}
