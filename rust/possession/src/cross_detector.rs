use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

/// 碰撞事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CollisionType {
    /// 直接矛盾 - 明确相反的观点
    Contradiction,
    /// 视角差异 - 不同的分析角度
    PerspectiveDifference,
    /// 前提分歧 - 对基本假设的不同看法
    PremiseDisagreement,
    /// 补充挑战 - 一个魂对另一个魂的观点提出补充或质疑
    SupplementaryChallenge,
}

/// 碰撞事件数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionEvent {
    /// 碰撞类型
    pub collision_type: CollisionType,
    /// 发起碰撞的魂
    pub from_soul: String,
    /// 被碰撞的魂
    pub to_soul: String,
    /// 碰撞的具体内容描述
    pub content: String,
    /// 触发碰撞的关键词或短语
    pub trigger_keywords: Vec<String>,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 是否已经注入到对话中
    pub injected: bool,
}

/// 单个魂的 token 流缓冲区
#[derive(Debug, Clone)]
pub struct SoulTokenBuffer {
    /// 魂的名称
    pub soul_name: String,
    /// 当前累积的文本
    pub current_text: String,
    /// 历史文本片段（用于检测上下文）
    pub history_fragments: Vec<String>,
    /// 最大缓冲区大小（字符数）
    pub max_buffer_size: usize,
    /// 最大历史片段数量
    pub max_history_fragments: usize,
}

impl SoulTokenBuffer {
    /// 创建新的缓冲区
    pub fn new(soul_name: String) -> Self {
        SoulTokenBuffer {
            soul_name,
            current_text: String::new(),
            history_fragments: Vec::new(),
            max_buffer_size: 2000,
            max_history_fragments: 10,
        }
    }

    /// 添加 token 到缓冲区
    pub fn add_token(&mut self, token: &str) {
        self.current_text.push_str(token);
        
        // 如果缓冲区达到或超过最大大小，保存到历史
        if self.current_text.len() >= self.max_buffer_size {
            self.rotate_buffer();
        }
    }

    /// 旋转缓冲区，保存当前文本到历史
    fn rotate_buffer(&mut self) {
        self.history_fragments.push(self.current_text.clone());
        
        // 保持历史片段数量在限制内
        if self.history_fragments.len() > self.max_history_fragments {
            self.history_fragments.remove(0);
        }
        
        self.current_text.clear();
    }

    /// 获取完整的上下文文本（当前 + 历史）
    pub fn get_context(&self) -> String {
        let mut context = String::new();
        for fragment in &self.history_fragments {
            context.push_str(fragment);
        }
        context.push_str(&self.current_text);
        context
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.current_text.clear();
        self.history_fragments.clear();
    }
}

/// 关键词匹配规则
#[derive(Debug, Clone)]
pub struct KeywordRule {
    /// 规则名称
    pub name: String,
    /// 触发关键词组（任一关键词出现即触发）
    pub trigger_keywords: Vec<String>,
    /// 对应碰撞类型
    pub collision_type: CollisionType,
    /// 规则描述
    pub description: String,
}

impl KeywordRule {
    pub fn new(name: &str, keywords: Vec<&str>, collision_type: CollisionType, description: &str) -> Self {
        KeywordRule {
            name: name.to_string(),
            trigger_keywords: keywords.iter().map(|s| s.to_string()).collect(),
            collision_type,
            description: description.to_string(),
        }
    }

    /// 检查文本是否匹配此规则
    pub fn matches(&self, text: &str) -> Option<Vec<String>> {
        let lower_text = text.to_lowercase();
        let mut matched_keywords = Vec::new();
        
        for keyword in &self.trigger_keywords {
            if lower_text.contains(&keyword.to_lowercase()) {
                matched_keywords.push(keyword.clone());
            }
        }
        
        if matched_keywords.is_empty() {
            None
        } else {
            Some(matched_keywords)
        }
    }
}

/// 流式交叉检测器
#[derive(Clone)]
pub struct CrossDetector {
    /// 所有魂的缓冲区
    buffers: Arc<Mutex<HashMap<String, SoulTokenBuffer>>>,
    /// 检测规则
    rules: Vec<KeywordRule>,
    /// 已检测到的碰撞事件
    collisions: Arc<Mutex<Vec<CollisionEvent>>>,
    /// 已处理的魂对（防止重复检测）
    processed_pairs: Arc<Mutex<HashSet<(String, String)>>>,
}

impl CrossDetector {
    /// 创建新的交叉检测器
    pub fn new() -> Self {
        let mut detector = CrossDetector {
            buffers: Arc::new(Mutex::new(HashMap::new())),
            rules: Vec::new(),
            collisions: Arc::new(Mutex::new(Vec::new())),
            processed_pairs: Arc::new(Mutex::new(HashSet::new())),
        };
        detector.add_default_rules();
        detector
    }

    /// 添加默认检测规则
    fn add_default_rules(&mut self) {
        // 直接矛盾检测
        self.rules.push(KeywordRule::new(
            "contradiction_but",
            vec!["但是", "然而", "可是", "不过", "but", "however", "yet"],
            CollisionType::Contradiction,
            "检测转折词，可能表示相反观点",
        ));
        
        self.rules.push(KeywordRule::new(
            "contradiction_no",
            vec!["不对", "不是", "错误", "不同意", "反对", "no", "wrong", "disagree"],
            CollisionType::Contradiction,
            "检测否定词，可能表示直接反对",
        ));
        
        // 视角差异检测
        self.rules.push(KeywordRule::new(
            "perspective_different",
            vec!["从另一个角度", "换个视角", "另一方面", "另一方面来看", "different perspective", "another angle"],
            CollisionType::PerspectiveDifference,
            "检测视角转换词",
        ));
        
        // 前提分歧检测
        self.rules.push(KeywordRule::new(
            "premise_assumption",
            vec!["假设", "前提", "如果", "假定", "assumption", "premise", "if"],
            CollisionType::PremiseDisagreement,
            "检测前提假设相关词汇",
        ));
        
        // 补充挑战检测
        self.rules.push(KeywordRule::new(
            "supplementary_challenge",
            vec!["补充", "需要注意", "还有一点", "考虑", "consider", "note", "add"],
            CollisionType::SupplementaryChallenge,
            "检测补充和挑战相关词汇",
        ));
    }

    /// 注册一个魂的缓冲区
    pub fn register_soul(&self, soul_name: String) {
        let Ok(mut buffers) = self.buffers.lock() else { return; };
        buffers.insert(soul_name.clone(), SoulTokenBuffer::new(soul_name));
    }

    /// 为特定魂添加 token
    pub fn add_token(&self, soul_name: &str, token: &str) {
        let Ok(mut buffers) = self.buffers.lock() else { return; };
        if let Some(buffer) = buffers.get_mut(soul_name) {
            buffer.add_token(token);
        }
    }

    /// 添加自定义检测规则
    pub fn add_rule(&mut self, rule: KeywordRule) {
        self.rules.push(rule);
    }

    /// 执行交叉检测
    pub fn detect_collisions(&self) -> Vec<CollisionEvent> {
        let Ok(buffers) = self.buffers.lock() else { return Vec::new(); };
        let mut new_collisions = Vec::new();
        let Ok(mut processed) = self.processed_pairs.lock() else { return Vec::new(); };
        
        let soul_names: Vec<String> = buffers.keys().cloned().collect();
        
        // 检查每一对魂之间的潜在碰撞
        for i in 0..soul_names.len() {
            for j in (i + 1)..soul_names.len() {
                let soul_a = &soul_names[i];
                let soul_b = &soul_names[j];
                
                // 检查是否已处理过这对魂
                let pair = (soul_a.clone(), soul_b.clone());
                if processed.contains(&pair) {
                    continue;
                }
                
                // 获取两个魂的上下文
                if let (Some(buffer_a), Some(buffer_b)) = (buffers.get(soul_a), buffers.get(soul_b)) {
                    let context_a = buffer_a.get_context();
                    let context_b = buffer_b.get_context();
                    
                    // 双向检测
                    if let Some(collision) = self.detect_between(soul_a, soul_b, &context_a, &context_b) {
                        new_collisions.push(collision.clone());
                        if let Ok(mut c) = self.collisions.lock() { c.push(collision); }
                        processed.insert(pair.clone());
                    }
                    
                    if let Some(collision) = self.detect_between(soul_b, soul_a, &context_b, &context_a) {
                        new_collisions.push(collision.clone());
                        if let Ok(mut c) = self.collisions.lock() { c.push(collision); }
                        processed.insert((soul_b.clone(), soul_a.clone()));
                    }
                }
            }
        }
        
        new_collisions
    }

    /// 检测两个魂之间的碰撞
    fn detect_between(
        &self,
        from_soul: &str,
        to_soul: &str,
        from_context: &str,
        to_context: &str,
    ) -> Option<CollisionEvent> {
        // 检查 from_context 中是否有针对 to_context 的碰撞
        for rule in &self.rules {
            if let Some(matched_keywords) = rule.matches(from_context) {
                // 如果目标魂也有相关内容，产生碰撞
                if !to_context.is_empty() {
                    return Some(CollisionEvent {
                        collision_type: rule.collision_type.clone(),
                        from_soul: from_soul.to_string(),
                        to_soul: to_soul.to_string(),
                        content: format!("{} 对 {} 的观点提出了{}", from_soul, to_soul, rule.description),
                        trigger_keywords: matched_keywords,
                        timestamp: chrono::Utc::now(),
                        injected: false,
                    });
                }
            }
        }
        
        None
    }

    /// 获取所有碰撞事件
    pub fn get_collisions(&self) -> Vec<CollisionEvent> {
        self.collisions.lock().map(|c| c.clone()).unwrap_or_default()
    }

    /// 标记碰撞为已注入
    pub fn mark_injected(&self, index: usize) {
        let Ok(mut collisions) = self.collisions.lock() else { return; };
        if let Some(collision) = collisions.get_mut(index) {
            collision.injected = true;
        }
    }

    /// 获取特定魂的缓冲区上下文
    pub fn get_soul_context(&self, soul_name: &str) -> Option<String> {
        let Ok(buffers) = self.buffers.lock() else { return None; };
        buffers.get(soul_name).map(|b| b.get_context())
    }

    /// 清空所有缓冲区和碰撞记录
    pub fn clear(&self) {
        let Ok(mut buffers) = self.buffers.lock() else { return; };
        for buffer in buffers.values_mut() {
            buffer.clear();
        }
        if let Ok(mut c) = self.collisions.lock() { c.clear(); }
        if let Ok(mut p) = self.processed_pairs.lock() { p.clear(); }
    }
}

impl Default for CrossDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_add_token() {
        let mut buffer = SoulTokenBuffer::new("test".to_string());
        buffer.add_token("hello");
        buffer.add_token(" world");
        assert_eq!(buffer.get_context(), "hello world");
    }

    #[test]
    fn test_buffer_rotation() {
        let mut buffer = SoulTokenBuffer::new("test".to_string());
        buffer.max_buffer_size = 10;
        
        // 添加刚好等于缓冲区大小的文本 - 触发旋转
        buffer.add_token("1234567890"); // 10 chars
        
        assert_eq!(buffer.history_fragments.len(), 1);
        assert_eq!(buffer.current_text, "");
        
        // 再添加一些文本
        buffer.add_token("abcdef");
        assert_eq!(buffer.current_text, "abcdef");
    }

    #[test]
    fn test_keyword_rule_matches() {
        let rule = KeywordRule::new(
            "test",
            vec!["但是", "however"],
            CollisionType::Contradiction,
            "test rule",
        );
        
        assert!(rule.matches("但是这个观点不对").is_some());
        assert!(rule.matches("However, I disagree").is_some());
        assert!(rule.matches("正常文本").is_none());
    }

    #[test]
    fn test_cross_detector_registration() {
        let detector = CrossDetector::new();
        detector.register_soul("马克思".to_string());
        detector.register_soul("费曼".to_string());
        
        assert!(detector.get_soul_context("马克思").is_some());
        assert!(detector.get_soul_context("费曼").is_some());
    }

    #[test]
    fn test_collision_detection() {
        let detector = CrossDetector::new();
        detector.register_soul("A".to_string());
        detector.register_soul("B".to_string());
        
        // 添加一些可能触发碰撞的文本
        detector.add_token("A", "这个观点是对的");
        detector.add_token("B", "但是我不同意这个看法");
        
        let collisions = detector.detect_collisions();
        assert!(!collisions.is_empty());
    }

    #[test]
    fn test_get_collisions() {
        let detector = CrossDetector::new();
        detector.register_soul("A".to_string());
        detector.register_soul("B".to_string());
        
        detector.add_token("A", "test");
        detector.add_token("B", "但是 test");
        
        let _ = detector.detect_collisions();
        let collisions = detector.get_collisions();
        
        assert!(!collisions.is_empty());
    }

    #[test]
    fn test_mark_injected() {
        let detector = CrossDetector::new();
        detector.register_soul("A".to_string());
        detector.register_soul("B".to_string());
        
        detector.add_token("A", "test");
        detector.add_token("B", "但是 test");
        
        let _ = detector.detect_collisions();
        detector.mark_injected(0);
        
        let collisions = detector.get_collisions();
        assert!(collisions[0].injected);
    }

    #[test]
    fn test_clear() {
        let detector = CrossDetector::new();
        detector.register_soul("A".to_string());
        detector.add_token("A", "some text");
        
        detector.clear();
        
        let context = detector.get_soul_context("A").unwrap();
        assert!(context.is_empty());
    }

    #[test]
    fn test_add_custom_rule() {
        let mut detector = CrossDetector::new();
        
        let custom_rule = KeywordRule::new(
            "custom",
            vec!["自定义关键词"],
            CollisionType::PerspectiveDifference,
            "自定义规则",
        );
        
        detector.add_rule(custom_rule);
        
        // 验证规则已添加
        detector.register_soul("X".to_string());
        detector.register_soul("Y".to_string());
        
        detector.add_token("X", "自定义关键词触发");
        detector.add_token("Y", "回应内容");
        
        let _collisions = detector.detect_collisions();
        // 自定义规则应该能工作
    }

    #[test]
    fn test_collision_event_serialization() {
        let event = CollisionEvent {
            collision_type: CollisionType::Contradiction,
            from_soul: "A".to_string(),
            to_soul: "B".to_string(),
            content: "Test collision".to_string(),
            trigger_keywords: vec!["test".to_string()],
            timestamp: chrono::Utc::now(),
            injected: false,
        };
        
        // 测试序列化
        let serialized = serde_json::to_string(&event).unwrap();
        assert!(!serialized.is_empty());
        
        // 测试反序列化
        let deserialized: CollisionEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.from_soul, "A");
        assert_eq!(deserialized.to_soul, "B");
    }

    #[test]
    fn test_empty_buffer_no_collision() {
        let detector = CrossDetector::new();
        detector.register_soul("A".to_string());
        detector.register_soul("B".to_string());

        let collisions = detector.detect_collisions();
        assert!(collisions.is_empty());

        let stored = detector.get_collisions();
        assert!(stored.is_empty());
    }

    #[test]
    fn test_same_soul_no_self_collision() {
        let detector = CrossDetector::new();
        detector.register_soul("A".to_string());
        detector.add_token("A", "这个方案很好");
        detector.add_token("A", "但是也有不足");

        let collisions = detector.detect_collisions();
        assert!(collisions.is_empty());
    }

    #[test]
    fn test_multiple_collision_types() {
        let detector = CrossDetector::new();
        detector.register_soul("马克思".to_string());
        detector.register_soul("费曼".to_string());

        detector.add_token("马克思", "剩余价值理论是科学的，但是剩余价值理论有缺陷");
        detector.add_token("费曼", "这个假设有问题，需要重新审视");

        let collisions = detector.detect_collisions();
        assert!(!collisions.is_empty());

        let has_contradiction = collisions.iter().any(|c| c.collision_type == CollisionType::Contradiction);
        let has_premise = collisions.iter().any(|c| c.collision_type == CollisionType::PremiseDisagreement);

        assert!(has_contradiction, "Expected at least one Contradiction collision");
        assert!(has_premise, "Expected at least one PremiseDisagreement collision");
    }
}