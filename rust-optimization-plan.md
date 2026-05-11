# 万民幡 Rust 版 创新优化方案

> 基于 `rust-port-guide.md` 的全面分析与现有代码库 `rust/` 的深度调研（2026-05-12）

---

## 代码现状总览

### 整体成熟度

六个 crate 全部实现到位，功能完成度超过 90%：

| Crate | 状态 | 核心功能 |
|-------|------|---------|
| `foundation` | ✅ 完整 | 24+ 数据模型、SQLite FTS5、向量索引、配置加载 |
| `registry` | ✅ 完整 | 全文搜索 + 主义主义坐标距离搜索 + 向量搜索 |
| `ai-gateway` | ✅ 完整 | DeepSeek/Claude/OpenAI 多 provider、prompt 构建、模型路由 |
| `archive` | ✅ 完整 | 归档、审计、成本追踪、调用记录 |
| `possession` | ✅ 完整 | 六种附体模式、入口分流、流式输出、WebSocket、碰撞检测 |
| `api` | ✅ 完整 | Axum HTTP + WebSocket，所有路由已实现 |

### 关键差距

| 差距 | 严重度 | 设计文档要求 | 实际状态 |
|------|--------|-------------|---------|
| 流式碰撞动态追问注入 | 🔴 高 | "实时交叉→矛盾→动态注入追问" | 碰撞仅汇总注入综合官，未运行时干预 |
| 魂长驻进程未启用 | 🔴 高 | "魂是长驻进程，有记忆连续性" | `soul/process.rs` 已定义但合议模式使用一次性 API 调用 |
| 多用户上下文隔离 | 🟡 中 | `multi_user.rs` | 未实现 |
| gRPC vs HTTP | 🟢 低 | gRPC API (tonic) | Axum HTTP + WebSocket（功能等价） |
| TUI 界面 | 🟢 低 | ratatui | 未实现 |
| Ollama provider | 🟢 低 | 本地模型 | 未实现 |

---

## 优化方案

### 一、架构跃迁：干预感知的长驻魂进程

**问题**：碰撞检测结果只能事后汇总给综合官，魂在推理过程中无法被实时干预。

**方案**：利用 `tokio::select!` 实现推理与干预的竞态：

```rust
enum Intervention {
    ContradictionQuestion  { from_soul, contradiction, question },
    BlindSpotRedirect      { covered_by, suggested_direction },
    DeepenRequest           { aspect, reason },
}

impl SoulProcess {
    async fn run(mut self) {
        while let Some(task) = self.rx.recv().await {
            let mut context = self.build_context(&task);
            tokio::select! {
                output = self.llm.chat_stream(context) => {
                    self.tx.send(output).await;
                }
                intervention = self.intervention_rx.recv() => {
                    if let Some(intervention) = intervention {
                        context.push(intervention.to_message());
                        let revised_output = self.llm.chat_stream(context).await;
                        self.tx.send(revised_output).await;
                    }
                }
            }
        }
    }
}
```

**追问决策三级门控**：

| 级别 | 方法 | 延迟 | 触发条件 |
|------|------|------|---------|
| Level 1 | 关键词规则匹配 | 微秒级 | 命中信念级别冲突关键词 |
| Level 2 | embedding 余弦相似度 | 毫秒级 | L1 未命中但讨论同一子话题 |
| Level 3 | Flash LLM 廉价判定 | 秒级 | L1/L2 均无法判定 |

**文件**：`rust/possession/src/soul/process.rs`（改造） + 新增 `rust/possession/src/soul/intervention.rs`

---

### 二、语义碰撞检测引擎

**问题**：当前 `cross_detector.rs` 使用纯关键词规则匹配，面对复杂哲学/政治分析时准确性低（~30%）。

**方案**：混合语义碰撞引擎——三路径并行：

```
魂 A token 流 ──► 滑动窗口缓冲（每 N tokens 一个片段）
                       │
                       ├──► 快速路径：关键词规则（微秒级）
                       │         └── 命中 → 碰撞事件
                       │
                       ├──► 嵌入路径：ONNX embedding（毫秒级）
                       │         ├── 片段 → all-MiniLM-L6-v2 编码
                       │         ├── 与魂 B/C 的最近片段做余弦相似度
                       │         ├── 低相似度 + 话题重叠 → 互补/矛盾
                       │         └── 高相似度 + 同一结论 → 冗余抑制
                       │
                       └──► 结构路径：主义主义坐标碰撞（纳秒级）
                                 └── 维度差异 > 阈值 → 立场差异标记
```

**技术选型**：
- **模型**：`all-MiniLM-L6-v2`（384维，~80MB），ONNX 格式
- **运行时**：`ort` crate（ONNX Runtime Rust binding）
- **单次编码延迟**：<5ms
- **依赖**：`ort = "2"`，`tokenizers` 用于文本预处理

**文件**：`rust/possession/src/cross_detector.rs`（重构） + 新增 `rust/possession/src/semantic_collision.rs`

**预期效果**：碰撞检测准确率 30% → 75%

---

### 三、魂记忆图谱

**问题**：设计文档中 `SoulState` 是线性历史，无法表达前提→结论的推导关系和跨魂知识融合。

**方案**：用 `petgraph` 构建有向图记忆结构：

```rust
enum MemoryNode {
    Conclusion { id, content, confidence, premises, timestamp },
    Premise    { id, content, status: Stable | Shaken | Overturned },
    Observation { id, content, source: SelfOutput | PracticeFeedback | OtherSoul },
    BlindSpot  { id, description, discovered_by },
}

enum MemoryEdge {
    Supports     { weight },
    Contradicts  { severity },
    Refines,
    Questions,
    Integrates,  // 跨魂边
}
```

**核心能力**：

| 能力 | 方法 | 说明 |
|------|------|------|
| 矛盾检测 | BFS 图遍历 + 语义相似度 | O(n)，零 LLM 成本 |
| 前提动摇 | DFS 从前提出发沿 Supports 边传播 | 自动标记所有受影响结论为"待审查" |
| 跨魂融合 | Integrates 边连接不同魂的节点 | 本身就是对"综合"的结构化表达 |
| 盲区发现 | 统计某魂无出边的主题区域 | 自动识别方法论盲区 |

**文件**：新增 `rust/possession/src/soul/memory_graph.rs`

**依赖**：`petgraph = "0.6"`

---

### 四、自适应合议拓扑

**问题**：当前合议固定全部并行启动，在简单任务上浪费 LLM 成本。

**方案**：`TopologyPlanner` 根据任务复杂度、魂间多样性、成本约束动态编排：

```rust
enum ConferenceTopology {
    FullMesh            { souls, cross_detect },     // 高复杂度 + 高多样性
    ClusteredParallel   { clusters, intra_synthesis }, // 中复杂度
    SequentialLadder    { soul_chain },              // 有依赖关系
    Oppositional        { camp_a, camp_b },          // 天然对立
    Minimal             { soul },                    // 简单任务，最低成本
}
```

**决策树**：

```
复杂度 < 0.3 + 预算有限 → Minimal
多样性 > 0.7 + 复杂度 > 0.6 → FullMesh + cross_detect
存在天然对立阵营 → Oppositional
默认 → ClusteredParallel
```

**动态调整**：合议进行 30 秒后若零碰撞 + 高语义重叠 → 自动降级拓扑，节省后续 LLM 调用。

**文件**：新增 `rust/possession/src/conference/topology.rs`

**预期效果**：简单任务成本降低 60-80%

---

### 五、魂生态系统

#### 5.1 魂信誉系统

```rust
struct SoulReputation {
    soul_name: String,
    approval_rate: f32,               // 修正提案被批准比例
    contradiction_rate: f32,          // 被判定为错误的比例
    practice_validation_rate: f32,    // 实践验证通过率
    insight_novelty: f32,             // 独特观点比例
    reputation_score: f32,            // 加权总分
}
```

**应用场景**：
- 审查优先级：低信誉魂的输出优先审查
- 辩论权重：高信誉魂的观点权重更高
- 新用户选魂参考
- 零召唤魂自动休眠

#### 5.2 魂杂交——方法论融合

```
魂 A（马克思：历史唯物主义）× 魂 B（费曼：科学还原论）
              ↓
        杂交魂 C（辩证法+科学方法）
              ↓
       综合官分析 A 和 B 的历次方法论碰撞，提取可融合部分，自动生成 summon_prompt
```

**文件**：新增 `rust/possession/src/soul/hybridization.rs`

#### 5.3 魂间直接教学

```
费曼（老师）分析乔布斯（学生）的盲区
  → 按费曼学习法组织教学内容
  → 出题测试
  → 评分 + 反馈 + 修正
```

**文件**：新增 `rust/possession/src/modes/teaching.rs`

---

## 实施路线图

| Phase | 内容 | 文件 | 增益 |
|-------|------|------|------|
| **1** | 干预感知魂进程 + 追问三级门控 | `soul/process.rs` + `soul/intervention.rs` | 碰撞从"事后汇总"变为"实时干预" |
| **2** | ONNX 语义碰撞引擎 | `cross_detector.rs` + `semantic_collision.rs` | 检测准确率 30%→75% |
| **3** | 魂记忆图谱 (petgraph) | `soul/memory_graph.rs` | 自我审计零 LLM 成本 |
| **4** | 自适应合议拓扑 | `conference/topology.rs` | 简单任务成本降 60-80% |
| **5** | 魂信誉 + 魂杂交 + 魂间教学 | 多文件 | 万民幡从工具变为平台 |
| **6** | TUI + Ollama + 魂休眠 | `sb-tui/` + `ollama.rs` | 本地零成本运行 |

---

## 新增依赖一览

| Crate | 用途 | Phase |
|-------|------|-------|
| `ort = "2"` | ONNX Runtime，语义碰撞编码 | 2 |
| `tokenizers` | HuggingFace tokenizer，文本预处理 | 2 |
| `petgraph = "0.6"` | 图数据结构，魂记忆图谱 | 3 |
| `ratatui = "0.28"` | TUI 终端界面 | 6 |
| `tonic` / `prost` | gRPC（可选替代 HTTP） | 1-6 |

---

## 不可妥协的设计约束（始终遵守）

1. **魂上下文隔离** — 每个魂独立 LLM 调用，不共享上下文
2. **审查者隔离** — 幡主审查必须是独立调用
3. **输出原文保全** — 不压缩、改写、概括
4. **计算分层** — 能用规则判断的绝不用 LLM
5. **引擎与界面分离** — 核心库不依赖任何界面框架
