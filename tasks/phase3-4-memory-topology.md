# Phase 3+4: 魂记忆图谱 + 自适应合议拓扑

请根据 `/Users/huyi/Desktop/rust banner/rust-optimization-plan.md` 中 Phase 3 和 Phase 4 的方案实现。

## 背景
当前 SoulState 是线性历史，无法表达前提→结论的推导关系和跨魂知识融合。合议模式固定全部并行启动，简单任务浪费 LLM 成本。

## 工作内容

### 1. 创建 rust/possession/src/soul/memory_graph.rs（Phase 3）
- MemoryNode 枚举：Conclusion / Premise { status: Stable|Shaken|Overturned } / Observation { source: SelfOutput|PracticeFeedback|OtherSoul } / BlindSpot
- MemoryEdge 枚举：Supports { weight } / Contradicts { severity } / Refines / Questions / Integrates
- SoulMemoryGraph 结构体：基于 petgraph::stable_graph::StableGraph，node_index: HashMap<String, NodeIndex>
- 方法：add_conclusion / add_premise / add_observation / add_blind_spot → NodeIndex
- add_edge(from, to, edge) → Result
- find_contradictions(node_id) → Vec<(NodeIndex, f32)>：BFS 图遍历 O(n)
- propagate_premise_shaken(premise_id) → Vec<NodeIndex>：DFS 沿 Supports 传播
- find_cross_soul_integrations() → Vec<(NodeIndex, NodeIndex)>
- get_blind_spots() → Vec<&BlindSpot>
- merge_graphs(a, b) → SoulMemoryGraph

### 2. 创建 rust/possession/src/modes/topology.rs（Phase 4）
- ConferenceTopology 枚举：FullMesh / ClusteredParallel / SequentialLadder / Oppositional / Minimal
- TopologyPlanner：plan(complexity, diversity, budget_constrained, souls) → ConferenceTopology
- 决策树：复杂度<0.3+预算有限→Minimal / 多样性>0.7+复杂度>0.6→FullMesh / 存在天然对立→Oppositional / 默认→ClusteredParallel
- TopologyMonitor：check_and_adjust(elapsed, collision_count, semantic_overlap) → Option<ConferenceTopology>
- diversity_score(profiles) → f32：基于 ismism_code 坐标距离
- complexity_score(task, souls_count) → f32

### 3. 更新模块声明
- soul/mod.rs 添加 pub mod memory_graph;
- modes/mod.rs 添加 pub mod topology;
- Cargo.toml 添加 petgraph 依赖

## 约束
- 引擎与界面分离
- 拓扑决策考虑预算约束
- 匹配现有代码风格

## 验证
1. cargo check -p possession 通过
2. memory_graph.rs 包含 contradiction_detection 和 premise_propagation 单元测试
3. topology.rs 包含 topology_planner 决策逻辑和 diversity_score 单元测试

请先阅读所有相关源文件，理解现有架构，然后实现代码。完成后自检 cargo check。
