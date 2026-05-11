use std::collections::{HashMap, VecDeque};

use crate::cross_detector::{CollisionPath, CollisionType};

// ── EmbeddingProvider trait ────────────────────────────────────────────────

/// 嵌入向量提供者 trait
///
/// 设计为 trait 接口，暂不实际集成 ONNX。具体实现（如 ONNX Runtime）
/// 可由调用方注入，保持模块对运行时依赖的解耦。
pub trait EmbeddingProvider: Send + Sync {
    /// 对文本生成嵌入向量
    fn embed(&self, text: &str) -> Result<Vec<f32>, String>;

    /// 嵌入向量维度
    fn dimension(&self) -> usize;
}

// ── SemanticFragment ───────────────────────────────────────────────────────

/// 语义片段——表示魂输出中一个滑动窗口内的文本及其语义编码
#[derive(Debug, Clone)]
pub struct SemanticFragment {
    /// 片段唯一标识
    pub id: String,
    /// 所属魂名称
    pub soul_name: String,
    /// 片段文本
    pub text: String,
    /// 嵌入向量（可选，由 EmbeddingProvider 填充）
    pub embedding: Option<Vec<f32>>,
    /// 主义主义四维坐标 [场域, 秩序, 价值, 本体]
    pub ismism_coords: [f32; 4],
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl SemanticFragment {
    pub fn new(id: String, soul_name: String, text: String) -> Self {
        SemanticFragment {
            id,
            soul_name,
            text,
            embedding: None,
            ismism_coords: [0.0; 4],
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    pub fn with_ismism_coords(mut self, coords: [f32; 4]) -> Self {
        self.ismism_coords = coords;
        self
    }
}

// ── SlidingWindow ──────────────────────────────────────────────────────────

/// 滑动窗口——管理一个魂的语义片段环形缓冲区
#[derive(Debug, Clone)]
pub struct SlidingWindow {
    /// 最大窗口大小（保留片段数）
    pub max_window_size: usize,
    fragments: VecDeque<SemanticFragment>,
}

impl SlidingWindow {
    pub fn new(max_window_size: usize) -> Self {
        SlidingWindow {
            max_window_size,
            fragments: VecDeque::with_capacity(max_window_size),
        }
    }

    /// 推入新片段，超出窗口大小时自动弹出最旧的
    pub fn push(&mut self, fragment: SemanticFragment) {
        if self.fragments.len() >= self.max_window_size {
            self.fragments.pop_front();
        }
        self.fragments.push_back(fragment);
    }

    /// 获取所有片段引用
    pub fn fragments(&self) -> &VecDeque<SemanticFragment> {
        &self.fragments
    }

    /// 获取片段数量
    pub fn len(&self) -> usize {
        self.fragments.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.fragments.is_empty()
    }

    /// 清空窗口
    pub fn clear(&mut self) {
        self.fragments.clear();
    }
}

// ── 数学工具 ──────────────────────────────────────────────────────────────

/// 余弦相似度：cos(theta) = A·B / (||A|| × ||B||)
///
/// 返回值范围 [-1.0, 1.0]；向量维度不一致或空向量时返回 0.0。
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let (dot, norm_a_sq, norm_b_sq) = a
        .iter()
        .zip(b.iter())
        .fold((0.0f32, 0.0f32, 0.0f32), |(d, na, nb), (&x, &y)| {
            (d + x * y, na + x * x, nb + y * y)
        });

    let norm_a = norm_a_sq.sqrt();
    let norm_b = norm_b_sq.sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

/// 主义主义四维坐标欧几里得距离
///
/// [场域, 秩序, 价值, 本体] 四维空间中的 L2 距离。
/// 距离越小表示两段文本在主义主义框架下的立场越接近。
pub fn ismism_distance(a: [f32; 4], b: [f32; 4]) -> f32 {
    let sum_sq: f32 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum();
    sum_sq.sqrt()
}

// ── CollisionDetected ──────────────────────────────────────────────────────

/// 语义碰撞引擎检测到的碰撞结果
#[derive(Debug, Clone)]
pub struct CollisionDetected {
    /// 碰撞类型
    pub collision_type: CollisionType,
    /// 发起碰撞的魂
    pub from_soul: String,
    /// 被碰撞的魂
    pub to_soul: String,
    /// 置信度 [0.0, 1.0]
    pub confidence: f32,
    /// 检测路径
    pub path: CollisionPath,
    /// 碰撞描述
    pub description: String,
    /// 涉及的片段 ID
    pub fragment_ids: Vec<String>,
}

// ── SemanticCollisionEngine ────────────────────────────────────────────────

/// 语义碰撞检测引擎
///
/// 三路径并行检测：
/// 1. 嵌入路径（EmbeddingPath）—— 余弦相似度 > embedding_threshold 触发
/// 2. 结构路径（StructurePath）—— 主义主义坐标距离 < ismism_threshold 触发盲点互补；
///    距离 > 2.0 触发视角差异
/// 3. 冗余路径 —— 同一魂内高相似度片段标记为 Redundancy
pub struct SemanticCollisionEngine<P: EmbeddingProvider> {
    provider: P,
    windows: HashMap<String, SlidingWindow>,
    embedding_threshold: f32,
    ismism_threshold: f32,
    window_size: usize,
}

impl<P: EmbeddingProvider> SemanticCollisionEngine<P> {
    pub fn new(provider: P) -> Self {
        SemanticCollisionEngine {
            provider,
            windows: HashMap::new(),
            embedding_threshold: 0.85,
            ismism_threshold: 0.3,
            window_size: 64,
        }
    }

    /// 设置嵌入路径的相似度阈值（超过此值触发碰撞）
    pub fn with_embedding_threshold(mut self, threshold: f32) -> Self {
        self.embedding_threshold = threshold;
        self
    }

    /// 设置结构路径的距离阈值（低于此值触发盲点互补标记）
    pub fn with_ismism_threshold(mut self, threshold: f32) -> Self {
        self.ismism_threshold = threshold;
        self
    }

    /// 设置每个魂的滑动窗口大小
    pub fn with_window_size(mut self, window_size: usize) -> Self {
        self.window_size = window_size;
        self
    }

    /// 注册一个语义片段到对应魂的滑动窗口中
    pub fn register_fragment(&mut self, fragment: SemanticFragment) {
        let window = self
            .windows
            .entry(fragment.soul_name.clone())
            .or_insert_with(|| SlidingWindow::new(self.window_size));
        window.push(fragment);
    }

    /// 运行三路径并行碰撞检测，返回所有检测结果
    pub fn detect_all(&mut self) -> Vec<CollisionDetected> {
        let mut collisions = Vec::new();

        // ── 路径 1: 嵌入检测 ──
        let embedding_collisions = self.detect_embedding_path();
        collisions.extend(embedding_collisions);

        // ── 路径 2: 结构检测 ──
        let structure_collisions = self.detect_structure_path();
        collisions.extend(structure_collisions);

        // ── 路径 3: 冗余检测 ──
        let redundancy_collisions = self.detect_redundancy();
        collisions.extend(redundancy_collisions);

        // ── 路径 4: 盲点互补检测 ──
        let blindspot_collisions = self.detect_blind_spot();
        collisions.extend(blindspot_collisions);

        collisions
    }

    /// 嵌入路径：通过余弦相似度检测不同魂的语义重叠
    fn detect_embedding_path(&mut self) -> Vec<CollisionDetected> {
        let mut collisions = Vec::new();
        let soul_names: Vec<String> = self.windows.keys().cloned().collect();

        for i in 0..soul_names.len() {
            for j in (i + 1)..soul_names.len() {
                let (soul_a, soul_b) = (&soul_names[i], &soul_names[j]);

                let window_a = match self.windows.get(soul_a) {
                    Some(w) => w,
                    None => continue,
                };
                let window_b = match self.windows.get(soul_b) {
                    Some(w) => w,
                    None => continue,
                };

                for frag_a in window_a.fragments().iter().filter(|f| f.embedding.is_some()) {
                    for frag_b in window_b.fragments().iter().filter(|f| f.embedding.is_some()) {
                        if let (Some(ref emb_a), Some(ref emb_b)) =
                            (&frag_a.embedding, &frag_b.embedding)
                        {
                            let similarity = cosine_similarity(emb_a, emb_b);

                            // 高相似度 → 可能视角重叠或冗余
                            if similarity > self.embedding_threshold {
                                collisions.push(CollisionDetected {
                                    collision_type: CollisionType::PerspectiveDifference,
                                    from_soul: soul_a.clone(),
                                    to_soul: soul_b.clone(),
                                    confidence: similarity.clamp(0.0, 1.0),
                                    path: CollisionPath::EmbeddingPath,
                                    description: format!(
                                        "高语义相似度 ({:.2})：{} 与 {} 可能讨论同一子话题",
                                        similarity, soul_a, soul_b
                                    ),
                                    fragment_ids: vec![frag_a.id.clone(), frag_b.id.clone()],
                                });
                            }
                        }
                    }
                }
            }
        }

        collisions
    }

    /// 结构路径：通过主义主义四维坐标距离检测立场接近/远离
    fn detect_structure_path(&self) -> Vec<CollisionDetected> {
        let mut collisions = Vec::new();
        let soul_names: Vec<String> = self.windows.keys().cloned().collect();

        for i in 0..soul_names.len() {
            for j in (i + 1)..soul_names.len() {
                let (soul_a, soul_b) = (&soul_names[i], &soul_names[j]);

                let window_a = match self.windows.get(soul_a) {
                    Some(w) => w,
                    None => continue,
                };
                let window_b = match self.windows.get(soul_b) {
                    Some(w) => w,
                    None => continue,
                };

                for frag_a in window_a.fragments() {
                    for frag_b in window_b.fragments() {
                        let dist = ismism_distance(frag_a.ismism_coords, frag_b.ismism_coords);

                        // 距离极近 → 立场高度重叠
                        if dist < self.ismism_threshold {
                            let confidence = 1.0 - (dist / self.ismism_threshold).min(1.0);
                            collisions.push(CollisionDetected {
                                collision_type: CollisionType::BlindSpotComplement,
                                from_soul: soul_a.clone(),
                                to_soul: soul_b.clone(),
                                confidence,
                                path: CollisionPath::StructurePath,
                                description: format!(
                                    "结构接近 ({:.2})：{} 与 {} 在主义主义空间中立场接近",
                                    dist, soul_a, soul_b
                                ),
                                fragment_ids: vec![frag_a.id.clone(), frag_b.id.clone()],
                            });
                        }
                    }
                }
            }
        }

        collisions
    }

    /// 冗余检测：同一魂内高相似度片段标记为 Redundancy
    fn detect_redundancy(&self) -> Vec<CollisionDetected> {
        let mut collisions = Vec::new();
        let soul_names: Vec<String> = self.windows.keys().cloned().collect();

        for soul_name in &soul_names {
            if let Some(window) = self.windows.get(soul_name) {
                let frags: Vec<&SemanticFragment> = window.fragments().iter().collect();
                for i in 0..frags.len() {
                    for j in (i + 1)..frags.len() {
                        if let (Some(ref emb_a), Some(ref emb_b)) =
                            (&frags[i].embedding, &frags[j].embedding)
                        {
                            let similarity = cosine_similarity(emb_a, emb_b);
                            // 极高相似度（> 0.95）表示同一魂在重复表述相同观点
                            if similarity > 0.95 {
                                collisions.push(CollisionDetected {
                                    collision_type: CollisionType::Redundancy,
                                    from_soul: soul_name.clone(),
                                    to_soul: soul_name.clone(),
                                    confidence: similarity.clamp(0.0, 1.0),
                                    path: CollisionPath::EmbeddingPath,
                                    description: format!(
                                        "冗余输出：{} 的两个片段相似度达 {:.2}",
                                        soul_name, similarity
                                    ),
                                    fragment_ids: vec![frags[i].id.clone(), frags[j].id.clone()],
                                });
                            }
                        }
                    }
                }
            }
        }

        collisions
    }

    /// 盲点互补检测：两个魂在主义主义空间中相距很远 → 可能彼此补充盲区
    fn detect_blind_spot(&self) -> Vec<CollisionDetected> {
        let mut collisions = Vec::new();
        let soul_names: Vec<String> = self.windows.keys().cloned().collect();

        for i in 0..soul_names.len() {
            for j in (i + 1)..soul_names.len() {
                let (soul_a, soul_b) = (&soul_names[i], &soul_names[j]);

                let window_a = match self.windows.get(soul_a) {
                    Some(w) => w,
                    None => continue,
                };
                let window_b = match self.windows.get(soul_b) {
                    Some(w) => w,
                    None => continue,
                };

                for frag_a in window_a.fragments() {
                    for frag_b in window_b.fragments() {
                        let dist = ismism_distance(frag_a.ismism_coords, frag_b.ismism_coords);

                        // 大距离 → 立场差异显著，可能互补盲区
                        if dist > 2.0 {
                            let confidence = (dist / 3.0).min(1.0);
                            collisions.push(CollisionDetected {
                                collision_type: CollisionType::BlindSpotComplement,
                                from_soul: soul_a.clone(),
                                to_soul: soul_b.clone(),
                                confidence,
                                path: CollisionPath::StructurePath,
                                description: format!(
                                    "盲点互补：{} 与 {} 在主义主义空间中距离 {:.2}，覆盖互补维度",
                                    soul_a, soul_b, dist
                                ),
                                fragment_ids: vec![frag_a.id.clone(), frag_b.id.clone()],
                            });
                        }
                    }
                }
            }
        }

        collisions
    }

    /// 获取指定魂的滑动窗口
    pub fn get_window(&self, soul_name: &str) -> Option<&SlidingWindow> {
        self.windows.get(soul_name)
    }

    /// 清空所有窗口
    pub fn clear(&mut self) {
        self.windows.clear();
    }

    /// 已注册的魂数量
    pub fn soul_count(&self) -> usize {
        self.windows.len()
    }

    /// 尝试对窗口中所有无 embedding 的片段生成 embedding
    pub fn embed_pending(&mut self) -> Result<usize, String> {
        let mut count = 0;
        for window in self.windows.values_mut() {
            for fragment in window.fragments.make_contiguous().iter_mut() {
                if fragment.embedding.is_none() && !fragment.text.is_empty() {
                    let emb = self.provider.embed(&fragment.text)?;
                    if emb.len() == self.provider.dimension() {
                        fragment.embedding = Some(emb);
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Dummy EmbeddingProvider for testing ──

    struct DummyProvider {
        dimension: usize,
    }

    impl EmbeddingProvider for DummyProvider {
        fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
            // Simple hash-based embedding for testing
            let mut vec = vec![0.0f32; self.dimension];
            for (i, byte) in text.bytes().enumerate() {
                vec[i % self.dimension] = ((byte as f32) / 255.0) * 2.0 - 1.0;
            }
            // Normalize
            let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for v in &mut vec {
                    *v /= norm;
                }
            }
            Ok(vec)
        }

        fn dimension(&self) -> usize {
            self.dimension
        }
    }

    // ── SlidingWindow tests ──

    #[test]
    fn test_sliding_window_push_and_bounds() {
        let mut window = SlidingWindow::new(3);

        window.push(SemanticFragment::new("1".into(), "A".into(), "text1".into()));
        window.push(SemanticFragment::new("2".into(), "A".into(), "text2".into()));
        window.push(SemanticFragment::new("3".into(), "A".into(), "text3".into()));
        assert_eq!(window.len(), 3);

        // Fourth push pops the oldest
        window.push(SemanticFragment::new("4".into(), "A".into(), "text4".into()));
        assert_eq!(window.len(), 3);
        assert_eq!(window.fragments()[0].id, "2");
        assert_eq!(window.fragments()[2].id, "4");
    }

    #[test]
    fn test_sliding_window_empty() {
        let window = SlidingWindow::new(10);
        assert!(window.is_empty());
        assert_eq!(window.len(), 0);
    }

    #[test]
    fn test_sliding_window_clear() {
        let mut window = SlidingWindow::new(5);
        window.push(SemanticFragment::new("1".into(), "A".into(), "text".into()));
        window.push(SemanticFragment::new("2".into(), "A".into(), "text".into()));
        window.clear();
        assert!(window.is_empty());
    }

    // ── cosine_similarity tests ──

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 2.0, 3.0];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        assert!((cosine_similarity(&a, &b) - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_dimension_mismatch() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_cosine_similarity_zero_vector() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 2.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    // ── ismism_distance tests ──

    #[test]
    fn test_ismism_distance_zero() {
        let a = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(ismism_distance(a, a), 0.0);
    }

    #[test]
    fn test_ismism_distance_positive() {
        let a = [0.0, 0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0, 0.0];
        assert_eq!(ismism_distance(a, b), 1.0);
    }

    #[test]
    fn test_ismism_distance_symmetric() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [4.0, 3.0, 2.0, 1.0];
        assert_eq!(ismism_distance(a, b), ismism_distance(b, a));
    }

    // ── SemanticFragment tests ──

    #[test]
    fn test_semantic_fragment_builder() {
        let fragment = SemanticFragment::new("id1".into(), "鲁迅".into(), "文本内容".into())
            .with_embedding(vec![0.1, 0.2, 0.3])
            .with_ismism_coords([1.0, 2.0, 3.0, 4.0]);

        assert_eq!(fragment.id, "id1");
        assert_eq!(fragment.soul_name, "鲁迅");
        assert_eq!(fragment.text, "文本内容");
        assert!(fragment.embedding.is_some());
        assert_eq!(fragment.ismism_coords, [1.0, 2.0, 3.0, 4.0]);
    }

    // ── SemanticCollisionEngine tests ──

    #[test]
    fn test_engine_register_and_count() {
        let provider = DummyProvider { dimension: 8 };
        let mut engine = SemanticCollisionEngine::new(provider);

        engine.register_fragment(SemanticFragment::new(
            "f1".into(),
            "A".into(),
            "hello".into(),
        ));
        engine.register_fragment(SemanticFragment::new(
            "f2".into(),
            "B".into(),
            "world".into(),
        ));

        assert_eq!(engine.soul_count(), 2);
    }

    #[test]
    fn test_engine_detect_empty() {
        let provider = DummyProvider { dimension: 8 };
        let mut engine = SemanticCollisionEngine::new(provider);

        // No fragments registered → no collisions
        let collisions = engine.detect_all();
        assert!(collisions.is_empty());
    }

    #[test]
    fn test_engine_embedding_path_collision() {
        let provider = DummyProvider { dimension: 8 };
        let mut engine = SemanticCollisionEngine::new(provider)
            .with_embedding_threshold(0.5);

        // Two souls with identical text → high similarity
        engine.register_fragment(
            SemanticFragment::new("f1".into(), "A".into(), "完全相同的文本".into())
                .with_embedding(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        );
        engine.register_fragment(
            SemanticFragment::new("f2".into(), "B".into(), "完全相同的文本".into())
                .with_embedding(vec![0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        );

        let collisions = engine.detect_all();
        // High cosine similarity should trigger embedding path
        let embedding_collisions: Vec<_> = collisions
            .iter()
            .filter(|c| c.path == CollisionPath::EmbeddingPath)
            .collect();
        assert!(!embedding_collisions.is_empty());
    }

    #[test]
    fn test_engine_structure_path_collision() {
        let provider = DummyProvider { dimension: 8 };
        let mut engine = SemanticCollisionEngine::new(provider)
            .with_ismism_threshold(1.0);

        // Two souls with close ismism coordinates
        engine.register_fragment(
            SemanticFragment::new("f1".into(), "A".into(), "text".into())
                .with_ismism_coords([0.0, 0.0, 0.0, 0.0]),
        );
        engine.register_fragment(
            SemanticFragment::new("f2".into(), "B".into(), "text".into())
                .with_ismism_coords([0.1, 0.1, 0.0, 0.0]),
        );

        let collisions = engine.detect_all();
        let structure_collisions: Vec<_> = collisions
            .iter()
            .filter(|c| c.path == CollisionPath::StructurePath)
            .collect();
        // Distance ≈ 0.141, below threshold 1.0 → collision
        assert!(!structure_collisions.is_empty());
    }

    #[test]
    fn test_engine_redundancy_detection() {
        let provider = DummyProvider { dimension: 8 };
        let mut engine = SemanticCollisionEngine::new(provider);

        // Same soul, nearly identical embeddings → redundancy
        engine.register_fragment(
            SemanticFragment::new("f1".into(), "A".into(), "text".into())
                .with_embedding(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        );
        engine.register_fragment(
            SemanticFragment::new("f2".into(), "A".into(), "text".into())
                .with_embedding(vec![0.999, 0.001, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        );

        let collisions = engine.detect_all();
        let has_redundancy = collisions.iter().any(|c| c.collision_type == CollisionType::Redundancy);
        assert!(has_redundancy);
    }

    #[test]
    fn test_engine_blind_spot_detection() {
        let provider = DummyProvider { dimension: 8 };
        let mut engine = SemanticCollisionEngine::new(provider);

        // Two souls with far apart ismism coordinates
        engine.register_fragment(
            SemanticFragment::new("f1".into(), "A".into(), "text".into())
                .with_ismism_coords([0.0, 0.0, 0.0, 0.0]),
        );
        engine.register_fragment(
            SemanticFragment::new("f2".into(), "B".into(), "text".into())
                .with_ismism_coords([2.0, 2.0, 0.0, 0.0]),
        );

        let collisions = engine.detect_all();
        let has_blindspot = collisions
            .iter()
            .any(|c| c.collision_type == CollisionType::BlindSpotComplement);
        // Distance ≈ 2.828, > 2.0 → blind spot
        assert!(has_blindspot);
    }

    #[test]
    fn test_engine_clear() {
        let provider = DummyProvider { dimension: 8 };
        let mut engine = SemanticCollisionEngine::new(provider);

        engine.register_fragment(SemanticFragment::new(
            "f1".into(),
            "A".into(),
            "text".into(),
        ));
        assert_eq!(engine.soul_count(), 1);

        engine.clear();
        assert_eq!(engine.soul_count(), 0);
    }
}
