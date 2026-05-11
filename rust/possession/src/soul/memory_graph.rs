use chrono::{DateTime, Utc};
use petgraph::stable_graph::StableGraph;
use petgraph::visit::{Bfs, Dfs, EdgeRef};
use petgraph::Direction;
use std::collections::HashMap;

use foundation::SoulProfile;

// ── Node types ──

/// 前提状态的阶段性标记
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PremiseStatus {
    Stable,
    Shaken,
    Overturned,
}

/// 观测来源
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservationSource {
    /// 魂自身推理输出
    SelfOutput,
    /// 实践开口的反馈
    PracticeFeedback,
    /// 其他魂的输出
    OtherSoul { soul_name: String },
}

/// 记忆节点：魂推理图谱中的原子单元
#[derive(Debug, Clone)]
pub enum MemoryNode {
    Conclusion {
        id: String,
        content: String,
        confidence: f32,
        premises: Vec<String>,   // 关联前提 id
        timestamp: DateTime<Utc>,
        soul_name: String,
    },
    Premise {
        id: String,
        content: String,
        status: PremiseStatus,
        soul_name: String,
    },
    Observation {
        id: String,
        content: String,
        source: ObservationSource,
        timestamp: DateTime<Utc>,
    },
    BlindSpot {
        id: String,
        description: String,
        discovered_by: String,
        dimension: String,
    },
}

impl MemoryNode {
    pub fn id(&self) -> &str {
        match self {
            MemoryNode::Conclusion { id, .. }
            | MemoryNode::Premise { id, .. }
            | MemoryNode::Observation { id, .. }
            | MemoryNode::BlindSpot { id, .. } => id,
        }
    }

    pub fn soul_name(&self) -> Option<&str> {
        match self {
            MemoryNode::Conclusion { soul_name, .. } | MemoryNode::Premise { soul_name, .. } => {
                Some(soul_name)
            }
            MemoryNode::Observation { source, .. } => match source {
                ObservationSource::OtherSoul { soul_name } => Some(soul_name),
                _ => None,
            },
            MemoryNode::BlindSpot { .. } => None,
        }
    }
}

// ── Edge types ──

/// 记忆边：节点间的关系
#[derive(Debug, Clone)]
pub enum MemoryEdge {
    Supports { weight: f32 },
    Contradicts { severity: f32 },
    Refines,
    Questions,
    /// 跨魂融合边
    Integrates,
}

impl MemoryEdge {
    pub fn is_cross_soul(&self) -> bool {
        matches!(self, MemoryEdge::Integrates)
    }
}

// ── Graph ──

/// 魂记忆图谱
///
/// 基于 `petgraph::stable_graph::StableGraph` 的有向图，
/// 存储推理节点（结论、前提、观测、盲区）及其关系边。
///
/// # 核心能力
/// - BFS 矛盾检测（零 LLM 成本）
/// - DFS 前提动摇传播（自动标记受影响结论）
/// - 跨魂融合发现
/// - 图谱合并
pub struct SoulMemoryGraph {
    graph: StableGraph<MemoryNode, MemoryEdge>,
    node_index: HashMap<String, petgraph::stable_graph::NodeIndex>,
    /// 图谱归属的魂名（None 表示已融合的复合图谱）
    soul_name: Option<String>,
}

impl SoulMemoryGraph {
    pub fn new(soul_name: Option<String>) -> Self {
        SoulMemoryGraph {
            graph: StableGraph::new(),
            node_index: HashMap::new(),
            soul_name,
        }
    }

    pub fn for_soul(soul_name: &str) -> Self {
        Self::new(Some(soul_name.to_string()))
    }

    // ── Node insertion ──

    fn insert_node(&mut self, node: MemoryNode) -> petgraph::stable_graph::NodeIndex {
        let id = node.id().to_string();
        let idx = self.graph.add_node(node);
        self.node_index.insert(id, idx);
        idx
    }

    pub fn add_conclusion(
        &mut self,
        id: &str,
        content: &str,
        confidence: f32,
        premises: Vec<String>,
        soul_name: &str,
    ) -> petgraph::stable_graph::NodeIndex {
        self.insert_node(MemoryNode::Conclusion {
            id: id.to_string(),
            content: content.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            premises,
            timestamp: Utc::now(),
            soul_name: soul_name.to_string(),
        })
    }

    pub fn add_premise(
        &mut self,
        id: &str,
        content: &str,
        status: PremiseStatus,
        soul_name: &str,
    ) -> petgraph::stable_graph::NodeIndex {
        self.insert_node(MemoryNode::Premise {
            id: id.to_string(),
            content: content.to_string(),
            status,
            soul_name: soul_name.to_string(),
        })
    }

    pub fn add_observation(
        &mut self,
        id: &str,
        content: &str,
        source: ObservationSource,
    ) -> petgraph::stable_graph::NodeIndex {
        self.insert_node(MemoryNode::Observation {
            id: id.to_string(),
            content: content.to_string(),
            source,
            timestamp: Utc::now(),
        })
    }

    pub fn add_blind_spot(
        &mut self,
        id: &str,
        description: &str,
        discovered_by: &str,
        dimension: &str,
    ) -> petgraph::stable_graph::NodeIndex {
        self.insert_node(MemoryNode::BlindSpot {
            id: id.to_string(),
            description: description.to_string(),
            discovered_by: discovered_by.to_string(),
            dimension: dimension.to_string(),
        })
    }

    // ── Edge insertion ──

    pub fn add_edge(
        &mut self,
        from_id: &str,
        to_id: &str,
        edge: MemoryEdge,
    ) -> Result<(), GraphError> {
        let from = self
            .node_index
            .get(from_id)
            .copied()
            .ok_or_else(|| GraphError::NodeNotFound(from_id.to_string()))?;
        let to = self
            .node_index
            .get(to_id)
            .copied()
            .ok_or_else(|| GraphError::NodeNotFound(to_id.to_string()))?;
        self.graph.add_edge(from, to, edge);
        Ok(())
    }

    // ── Queries ──

    pub fn get_node(&self, id: &str) -> Option<&MemoryNode> {
        self.node_index.get(id).and_then(|idx| self.graph.node_weight(*idx))
    }

    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut MemoryNode> {
        self.node_index.get(id).and_then(|idx| self.graph.node_weight_mut(*idx))
    }

    /// BFS 矛盾检测：从给定节点出发，沿所有出边做 BFS，
    /// 收集所有 Contradicts 边及其权重
    pub fn find_contradictions(&self, node_id: &str) -> Vec<(String, f32)> {
        let Some(start) = self.node_index.get(node_id).copied() else {
            return Vec::new();
        };

        let mut bfs = Bfs::new(&self.graph, start);
        let mut results = Vec::new();

        while let Some(current) = bfs.next(&self.graph) {
            for edge_idx in self.graph.edges_directed(current, Direction::Outgoing) {
                if let MemoryEdge::Contradicts { severity } = edge_idx.weight() {
                    let target = edge_idx.target();
                    if let Some(node) = self.graph.node_weight(target) {
                        results.push((node.id().to_string(), *severity));
                    }
                }
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// 定位前提节点并获取其当前状态
    pub fn get_premise_status(&self, premise_id: &str) -> Option<PremiseStatus> {
        self.node_index.get(premise_id).and_then(|idx| {
            self.graph.node_weight(*idx).and_then(|n| match n {
                MemoryNode::Premise { status, .. } => Some(status.clone()),
                _ => None,
            })
        })
    }

    /// DFS 前提动摇传播：从被摇动的前提出发，沿 Supports 边 DFS，
    /// 返回所有受影响节点 id（被支撑的结论等）
    pub fn propagate_premise_shaken(
        &self,
        premise_id: &str,
    ) -> Vec<String> {
        let Some(start) = self.node_index.get(premise_id).copied() else {
            return Vec::new();
        };

        // 确认起始节点确实是前提且已动摇
        let is_shaken = self
            .graph
            .node_weight(start)
            .map(|n| matches!(n, MemoryNode::Premise { status, .. } if *status != PremiseStatus::Stable))
            .unwrap_or(false);

        if !is_shaken {
            return Vec::new();
        }

        let mut dfs = Dfs::new(&self.graph, start);
        let mut affected = Vec::new();

        while let Some(current) = dfs.next(&self.graph) {
            if current == start {
                continue; // 跳过前提自身
            }

            // 检查是否有 Supports 或 Refines 边指向当前节点
            let has_supporting_edge = self
                .graph
                .edges_directed(current, Direction::Incoming)
                .any(|e| matches!(e.weight(), MemoryEdge::Supports { .. } | MemoryEdge::Refines));

            if has_supporting_edge {
                if let Some(node) = self.graph.node_weight(current) {
                    affected.push(node.id().to_string());
                }
            }
        }

        affected
    }

    /// 更新前提状态并返回受传播影响的所有节点
    pub fn shake_premise(&mut self, premise_id: &str) -> Vec<String> {
        let affected = self.propagate_premise_shaken(premise_id);
        if let Some(node) = self.get_node_mut(premise_id) {
            if let MemoryNode::Premise { status, .. } = node {
                *status = PremiseStatus::Shaken;
            }
        }
        affected
    }

    /// 发现跨魂融合边：返回所有 Integrates 边的 (from_id, to_id)
    pub fn find_cross_soul_integrations(&self) -> Vec<(String, String)> {
        let mut results = Vec::new();
        for edge_idx in self.graph.edge_indices() {
            if let MemoryEdge::Integrates = self.graph.edge_weight(edge_idx).unwrap() {
                let (from, to) = self.graph.edge_endpoints(edge_idx).unwrap();
                if let (Some(from_n), Some(to_n)) = (
                    self.graph.node_weight(from),
                    self.graph.node_weight(to),
                ) {
                    // 仅当两端来自不同魂时才视为跨魂融合
                    let from_soul = from_n.soul_name();
                    let to_soul = to_n.soul_name();
                    if from_soul != to_soul
                        || (from_soul.is_some() && to_soul.is_some() && from_soul != to_soul)
                    {
                        results.push((from_n.id().to_string(), to_n.id().to_string()));
                    }
                }
            }
        }
        results
    }

    /// 获取所有 BlindSpot 节点
    pub fn get_blind_spots(&self) -> Vec<&MemoryNode> {
        self.graph
            .node_weights()
            .filter(|n| matches!(n, MemoryNode::BlindSpot { .. }))
            .collect()
    }

    /// 获取所有结论节点
    pub fn get_conclusions(&self) -> Vec<&MemoryNode> {
        self.graph
            .node_weights()
            .filter(|n| matches!(n, MemoryNode::Conclusion { .. }))
            .collect()
    }

    /// 获取所有前提节点
    pub fn get_premises(&self) -> Vec<&MemoryNode> {
        self.graph
            .node_weights()
            .filter(|n| matches!(n, MemoryNode::Premise { .. }))
            .collect()
    }

    /// 从魂配置中创建初始记忆图谱（包含前置知识节点）
    pub fn from_profile(profile: &SoulProfile) -> Self {
        let mut graph = Self::for_soul(&profile.name);

        // 将魂的 iusmism_code 拆分为四字段作为基础前提
        let parts: Vec<&str> = profile.ismism_code.split('-').collect();
        let field_names = ["本体论", "认识论", "目的论", "方法论"];
        for (i, part) in parts.iter().enumerate().take(4) {
            if let Some(field) = field_names.get(i) {
                let premise_id = format!("{}-{}-base", profile.name, field);
                let parsed: i32 = part.parse().unwrap_or(0);
                let status = if parsed == 0 {
                    PremiseStatus::Overturned
                } else {
                    PremiseStatus::Stable
                };
                graph.add_premise(&premise_id, &format!("{}: 坐标={}", field, part), status, &profile.name);
            }
        }

        // 将 self_declare 作为结论节点
        if !profile.self_declare.is_empty() {
            let decl_id = format!("{}-self-declare", profile.name);
            graph.add_conclusion(
                &decl_id,
                &profile.self_declare,
                0.85,
                field_names
                    .iter()
                    .enumerate()
                    .map(|(_i, f)| format!("{}-{}-base", profile.name, f))
                    .collect(),
                &profile.name,
            );
        }

        // 登记实践观测
        for (i, obs) in profile.practice_observations.iter().enumerate() {
            let obs_id = format!("{}-practice-{}", profile.name, i);
            graph.add_observation(
                &obs_id,
                &obs.observation,
                ObservationSource::PracticeFeedback,
            );
        }

        graph
    }

    /// 合并两个记忆图谱（图谱 A 的节点 + 图谱 B 的节点 + 新边）
    pub fn merge_graphs(a: &SoulMemoryGraph, b: &SoulMemoryGraph) -> SoulMemoryGraph {
        let mut merged = SoulMemoryGraph::new(None); // 融合图谱无单一归属

        // id 映射：从旧 NodeIndex 映射到新图中的 NodeIndex
        let mut id_to_new_idx: HashMap<String, petgraph::stable_graph::NodeIndex> =
            HashMap::new();

        // 插入图谱 A 的所有节点
        for node_weight in a.graph.node_weights() {
            let id = node_weight.id().to_string();
            let new_idx = merged.graph.add_node(node_weight.clone());
            id_to_new_idx.insert(id.clone(), new_idx);
            merged.node_index.insert(id, new_idx);
        }

        // 插入图谱 B 的所有节点（跳过重复 id）
        for node_weight in b.graph.node_weights() {
            let id = node_weight.id().to_string();
            if merged.node_index.contains_key(&id) {
                continue;
            }
            let new_idx = merged.graph.add_node(node_weight.clone());
            id_to_new_idx.insert(id.clone(), new_idx);
            merged.node_index.insert(id, new_idx);
        }

        // 复制图谱 A 的边
        for edge_idx in a.graph.edge_indices() {
            if let Some((from, to)) = a.graph.edge_endpoints(edge_idx) {
                let from_id = a.graph.node_weight(from).map(|n| n.id().to_string());
                let to_id = a.graph.node_weight(to).map(|n| n.id().to_string());
                if let (Some(fid), Some(tid)) = (from_id, to_id) {
                    if let (Some(&new_from), Some(&new_to)) =
                        (id_to_new_idx.get(&fid), id_to_new_idx.get(&tid))
                    {
                        merged
                            .graph
                            .add_edge(new_from, new_to, a.graph.edge_weight(edge_idx).unwrap().clone());
                    }
                }
            }
        }

        // 复制图谱 B 的边
        for edge_idx in b.graph.edge_indices() {
            if let Some((from, to)) = b.graph.edge_endpoints(edge_idx) {
                let from_id = b.graph.node_weight(from).map(|n| n.id().to_string());
                let to_id = b.graph.node_weight(to).map(|n| n.id().to_string());
                if let (Some(fid), Some(tid)) = (from_id, to_id) {
                    if let (Some(&new_from), Some(&new_to)) =
                        (id_to_new_idx.get(&fid), id_to_new_idx.get(&tid))
                    {
                        // 检查此边是否已存在
                        let edge_exists = merged
                            .graph
                            .find_edge(new_from, new_to)
                            .is_some();
                        if !edge_exists {
                            merged
                                .graph
                                .add_edge(new_from, new_to, b.graph.edge_weight(edge_idx).unwrap().clone());
                        }
                    }
                }
            }
        }

        merged
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}

// ── Error ──

#[derive(Debug, Clone)]
pub enum GraphError {
    NodeNotFound(String),
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::NodeNotFound(id) => write!(f, "Node not found: {}", id),
        }
    }
}

impl std::error::Error for GraphError {}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_graph() -> SoulMemoryGraph {
        let mut g = SoulMemoryGraph::for_soul("TestSoul");

        // 添加前提
        g.add_premise("prem-a", "历史唯物主义框架有效", PremiseStatus::Stable, "TestSoul");
        g.add_premise("prem-b", "经济学模型可量化", PremiseStatus::Stable, "TestSoul");

        // 添加结论
        g.add_conclusion(
            "conc-1",
            "资本积累必然导致危机",
            0.9,
            vec!["prem-a".to_string(), "prem-b".to_string()],
            "TestSoul",
        );
        g.add_conclusion(
            "conc-2",
            "市场自我调节可避免危机",
            0.7,
            vec!["prem-b".to_string()],
            "TestSoul",
        );

        // 添加关系边
        let _ = g.add_edge("prem-a", "conc-1", MemoryEdge::Supports { weight: 0.8 });
        let _ = g.add_edge("prem-b", "conc-1", MemoryEdge::Supports { weight: 0.6 });
        let _ = g.add_edge("prem-b", "conc-2", MemoryEdge::Supports { weight: 0.7 });
        let _ = g.add_edge(
            "conc-1",
            "conc-2",
            MemoryEdge::Contradicts { severity: 0.9 },
        );

        g
    }

    #[test]
    fn test_add_conclusion_and_premise() {
        let g = build_test_graph();
        assert_eq!(g.node_count(), 4); // 2 premises + 2 conclusions
        assert_eq!(g.edge_count(), 4);
        assert!(g.get_node("prem-a").is_some());
        assert!(g.get_node("conc-1").is_some());
        assert!(g.get_node("nonexistent").is_none());
    }

    #[test]
    fn test_find_contradictions() {
        let g = build_test_graph();
        let contradictions = g.find_contradictions("conc-1");
        assert!(!contradictions.is_empty());

        // "conc-2" 应出现在矛盾列表中
        let has_conc2 = contradictions.iter().any(|(id, _)| id == "conc-2");
        assert!(has_conc2, "Expected conc-2 to be found as contradiction");

        // 严重度应 > 0
        let sev = contradictions.iter().find(|(id, _)| id == "conc-2").map(|(_, s)| *s);
        assert!(sev.unwrap() > 0.0);
    }

    #[test]
    fn test_no_contradictions_from_clean_node() {
        let mut g = SoulMemoryGraph::for_soul("CleanSoul");
        g.add_premise("p1", "天是蓝的", PremiseStatus::Stable, "CleanSoul");
        g.add_conclusion("c1", "天很漂亮", 0.9, vec!["p1".to_string()], "CleanSoul");
        let _ = g.add_edge("p1", "c1", MemoryEdge::Supports { weight: 0.5 });

        let contradictions = g.find_contradictions("c1");
        assert!(contradictions.is_empty());
    }

    #[test]
    fn test_propagate_premise_shaken() {
        let mut g = build_test_graph();

        // 先动摇前提
        g.shake_premise("prem-a");

        let affected = g.propagate_premise_shaken("prem-a");
        // conc-1 被 prem-a 支撑，应该受影响
        let has_conc1 = affected.iter().any(|id| id == "conc-1");
        assert!(has_conc1, "conc-1 should be affected when prem-a is shaken");
    }

    #[test]
    fn test_propagate_stable_premise_returns_empty() {
        let g = build_test_graph();
        // preb-b 是 Stable 的，不应产生传播
        let affected = g.propagate_premise_shaken("prem-b");
        assert!(affected.is_empty());
    }

    #[test]
    fn test_get_premise_status() {
        let g = build_test_graph();
        assert_eq!(g.get_premise_status("prem-a"), Some(PremiseStatus::Stable));
        assert_eq!(g.get_premise_status("conc-1"), None); // 结论不是前提
        assert_eq!(g.get_premise_status("nonexistent"), None);
    }

    #[test]
    fn test_blind_spot() {
        let mut g = SoulMemoryGraph::for_soul("Test");
        g.add_blind_spot("bs-1", "对量子力学方法论无覆盖", "TestSoul", "方法论");
        g.add_blind_spot("bs-2", "缺乏历史维度的分析", "TestSoul", "本体论");

        let spots = g.get_blind_spots();
        assert_eq!(spots.len(), 2);
    }

    #[test]
    fn test_merge_graphs() {
        let mut a = SoulMemoryGraph::for_soul("A");
        a.add_premise("a-prem", "A的前提", PremiseStatus::Stable, "A");
        a.add_conclusion("a-conc", "A的结论", 0.8, vec!["a-prem".to_string()], "A");
        let _ = a.add_edge("a-prem", "a-conc", MemoryEdge::Supports { weight: 0.9 });

        let mut b = SoulMemoryGraph::for_soul("B");
        b.add_premise("b-prem", "B的前提", PremiseStatus::Stable, "B");
        b.add_conclusion("b-conc", "B的结论", 0.75, vec!["b-prem".to_string()], "B");
        let _ = b.add_edge("b-prem", "b-conc", MemoryEdge::Supports { weight: 0.8 });

        let merged = SoulMemoryGraph::merge_graphs(&a, &b);
        assert_eq!(merged.node_count(), 4);
        assert_eq!(merged.edge_count(), 2);
        assert!(merged.get_node("a-prem").is_some());
        assert!(merged.get_node("b-conc").is_some());
        // 融合图谱应无单一归属
        assert!(merged.soul_name.is_none());
    }

    #[test]
    fn test_cross_soul_integrations() {
        let mut g = SoulMemoryGraph::new(None);
        g.add_conclusion("c-a", "A的观点", 0.8, vec![], "A");
        g.add_conclusion("c-b", "B的观点", 0.7, vec![], "B");
        g.add_conclusion("c-c", "C的观点", 0.9, vec![], "A");
        let _ = g.add_edge("c-a", "c-b", MemoryEdge::Integrates);
        let _ = g.add_edge("c-a", "c-c", MemoryEdge::Supports { weight: 0.5 }); // 同魂，不算跨魂

        let integrations = g.find_cross_soul_integrations();
        assert_eq!(integrations.len(), 1);
        assert_eq!(integrations[0].0, "c-a");
        assert_eq!(integrations[0].1, "c-b");
    }

    #[test]
    fn test_observation_sources() {
        let mut g = SoulMemoryGraph::for_soul("Observer");
        g.add_observation("o1", "实践反馈：方法A在场景X失效", ObservationSource::PracticeFeedback);
        g.add_observation("o2", "自我反思：逻辑链缺少中间步骤", ObservationSource::SelfOutput);
        g.add_observation(
            "o3",
            "魂B指出框架Y的假设不成立",
            ObservationSource::OtherSoul {
                soul_name: "B".to_string(),
            },
        );

        assert!(g.get_node("o1").is_some());
        assert!(g.get_node("o2").is_some());
        assert!(g.get_node("o3").is_some());
    }

    #[test]
    fn test_from_profile() {
        let profile = SoulProfile {
            name: "TestProfile".to_string(),
            ismism_code: "1-2-3-0".to_string(),
            field: "哲学".to_string(),
            ontology: "".to_string(),
            epistemology: "".to_string(),
            teleology: "".to_string(),
            domains: vec![],
            exclude_scenarios: vec![],
            summon_count: 0,
            effectiveness: foundation::EffectivenessStats::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec![],
            summon_prompt: "".to_string(),
            practice_observations: vec![],
            title: "".to_string(),
            description: "".to_string(),
            voice: "".to_string(),
            mind: "".to_string(),
            self_declare: "我分析哲学问题".to_string(),
            skills_expertise: vec![],
            model: "".to_string(),
            tools: "".to_string(),
            trigger_keywords: vec![],
            compat: vec![],
            incompat: vec![],
        };

        let g = SoulMemoryGraph::from_profile(&profile);

        // 4 个基础前提 + 1 个 self-declare 结论
        assert!(g.node_count() >= 5);

        // "本体论" 坐标为 1 应该 Stable
        let onto_status = g.get_premise_status("TestProfile-本体论-base");
        assert_eq!(onto_status, Some(PremiseStatus::Stable));

        // "方法论" 坐标为 0 应该 Overturned
        let method_status = g.get_premise_status("TestProfile-方法论-base");
        assert_eq!(method_status, Some(PremiseStatus::Overturned));
    }

    #[test]
    fn test_add_edge_error() {
        let mut g = SoulMemoryGraph::for_soul("Test");
        g.add_premise("p1", "前提", PremiseStatus::Stable, "Test");

        let result = g.add_edge("p1", "nonexistent", MemoryEdge::Supports { weight: 0.5 });
        assert!(result.is_err());
    }
}
