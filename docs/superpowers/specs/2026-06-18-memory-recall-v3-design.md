# 记忆援引系统设计 (v3)

- **日期**:2026-06-18
- **状态**:设计草案,待 review
- **依赖**:v2.1 向量 hybrid 搜索 spec(`2026-06-18-vector-hybrid-search-design.md`)—— 本 spec 是其上层应用
- **范围**:把 v2.1 的向量 hybrid 搜索暴露给 soul(possession)和综合官(conference),让模拟法庭能"援引过往判例"

---

## 1. 背景与目标

### 1.1 现状(v2.1 之后)

v2.1 建好了向量 hybrid 搜索的基础设施:
- `EmbeddingClient`(BGE-M3 via LMStudio)+ 两张 vec0 表(message/card)+ 跨表 RRF
- `HybridSearcher::search(query, limit) -> Vec<HybridResult>`
- `/api/knowledge/search` 端点(给前端用)

**但搜索对 soul 和综合官完全不可见**——基础设施建好了,法庭里却没人用它。这是当前最大的浪费:knowledge 搜索能查,但只有人类去前端手点查询时才有用;soul 在 possession 时完全不知道过往对话,综合官在 conference 时也看不到历史判例。

### 1.2 问题

模拟法庭的核心痛点:**每场审判都是孤立的**。

- soul 在 possession 时只看到当前 task + 预设,**对"这个议题以前审过、结论是什么"完全无知**——同样的问题反复审,无人发现
- 综合官在 conference 合议时,只看到本场各 soul 的输出,**无法援引历史 KnowledgeCard(判例)**——积累产生不了复利
- v2.1 已经把"可语义召回过往对话 + 判例卡片"做出来了,但**没接进思维链**

### 1.3 目标

把 v2.1 的向量 hybrid 搜索接入思维链,**两条路径**:

1. **工具路径(A)**:新增 `recall_memory` 工具,soul 通过 `Memory` capability 主动调用——soul 自己判断"我需要回忆相关历史"时触发。**低延迟成本(只在需要时调)、高可控**
2. **注入路径(B)**:conference 综合时,自动把 top-K 相关历史塞进 synthesis prompt——综合官无感获得"过往判例"。**始终生效、覆盖合成质量**

### 1.4 非目标

- 不做 possession 单魂的自动注入(路径 C)——`single.rs` 已有 `search_results`/`facts` 通道被 web_search 占用,先不混淆;留给 v4 视效果再决定
- 不做"热点议题聚类""盲点自动检测"等上层分析——那是 v5+ 的语义分析层,本 spec 只做"能召回 + 能注入"
- 不改 web_search 工具行为
- 不改 prompt 的领域模板(`domain.yaml`)——注入点在 PromptBuilder 代码层,不在配置层

---

## 2. 关键决策

| # | 决策 | 选择 | 理由 |
|---|------|------|------|
| 1 | 集成方式 | **A 工具 + B 注入 双路径** | A 给 soul 主动控制(省延迟),B 给综合官被动增益(保覆盖);两者互补,不是二选一 |
| 2 | 工具名 / capability | `recall_memory` / `Memory` | 对齐现有 `web_search`/`WebSearch` 命名风格;soul 在 `tools` 字段写 `Memory` 即可开启 |
| 3 | 工具召回来源 | **messages + cards(同 v2.1 HybridSearcher)** | 复用现成跨表 RRF,不另起炉灶;工具只是 HybridSearcher 的薄包装 |
| 4 | 注入位置(B 路径) | **新增 `build_synthesis_with_recalled` 方法**,作为 `build_synthesis_with_collisions` 的兄弟 | prompt.rs 已有 collisions 注入的成熟范式可镜像;不动现有方法签名 |
| 5 | 注入内容 | **top-5 HybridResult,渲染为 `## 历史回响` section** | 5 条够综合官参考又不撑爆上下文;section 标题明确区分"本场输出"vs"历史判例" |
| 6 | 召回 query(B 路径) | **task 原文** | 综合时只有 task 是稳定的语义锚;soul 输出本场已有,不该作为召回 query |
| 7 | 工具调用轮次配额 | **归入"非编码工具"桶(5 轮)** | recall 是低频、单次的检索,不需要 20 轮;对齐 `max_tool_rounds_for_tools` 现有分桶(`tools.rs:40-52`) |
| 8 | 降级策略 | **与 v2.1 一致:FTS5 兜底** | HybridSearcher 内部已处理,工具/注入层不感知失败,失败返回空字符串 |
| 9 | 是否限流 | **工具路径:全局速率限(默认 20 次/分钟)**;注入路径:无限制(每场 conference 1 次) | 防止 soul 把 recall 当拐杖无限调用撑爆成本;per-session 限流需改 ToolHandler trait 传 session_id(见 §4.1.1),成本高收益低,采用全局速率替代 |
| 10 | 触发开关 | **`vector_search.memory_recall: { tool: bool, synthesis_inject: bool }`** | 默认全 true(假设 v2.1 已启用);关掉可分别禁用两条路径 |

---

## 3. 架构总览

```
┌─────────────────────────────────────────────────────────────────┐
│  v3 = v2.1 HybridSearcher 的两个上层消费者                         │
└─────────────────────────────────────────────────────────────────┘

  路径 A:soul 主动召回(工具)                路径 B:综合官被动注入
  ─────────────────────────                 ──────────────────────
  possession single/conference                conference synthesis
        │                                            │
        ▼                                            ▼
  soul LLM 决定调用工具                 conference.rs:244 注入点
        │                                            │
        ▼                                            ▼
  recall_memory 工具              build_synthesis_with_recalled()
        │                                            │
        └──────────────┬─────────────────────────────┘
                       ▼
            HybridSearcher::search()      ← v2.1 已建好
                       │
                       ▼
            Vec<HybridResult>
                       │
            ┌──────────┴──────────┐
            ▼                     ▼
      渲染成 markdown         渲染成 ## 历史回响
      塞回 tool message       塞进 synthesis user message
```

### 3.1 分层归属

- **`foundation`** — 无新代码(复用 v2.1 HybridSearcher)
- **`possession`** — 无新代码(工具 dispatch loop 现成,`stream.rs:252` + `conference.rs:474`)
- **`api`** — 新增 `memory_recall_tool.rs`(工具实现);改 `main.rs`(注册)
- **`ai-gateway`** — `prompt.rs` 加 `build_synthesis_with_recalled` 方法
- **`possession/modes/conference.rs`** — synthesis 调用点加一行 recall + 改调用新方法

### 3.2 核心约束

- 工具与注入共用同一个 `HybridSearcher` 实例(单例,Arc 共享)
- 召回结果渲染**统一格式**:区分 `source=message`(庭审记录)vs `source=card`(过往判决)
- 两条路径都**不阻塞主流程**——recall 失败返回空,工具/注入照常进行
- soul capability 与综合官注入**独立开关**,可单开一端

---

## 4. 组件细节

### 4.1 路径 A:`MemoryRecallTool`(`rust/api/src/memory_recall_tool.rs`,新建)

**职责**:soul 可调用的工具,包装 HybridSearcher。镜像 `web_search_tool.rs` 的结构。

```rust
use std::sync::Arc;
use async_trait::async_trait;
use foundation::hybrid_search::{HybridSearcher, HybridResult};
use possession::tools::{ToolHandler, ToolDefinition};

pub struct MemoryRecallTool {
    searcher: Arc<HybridSearcher>,
    max_calls_per_session: usize,   // 默认 3,防滥用
    call_counts: Arc<DashMap<String, usize>>,  // session_id → 已调用次数
}

impl MemoryRecallTool {
    pub fn new(searcher: Arc<HybridSearcher>) -> Self {
        Self { searcher, max_calls_per_session: 3, call_counts: Default::default() }
    }
}

#[async_trait]
impl ToolHandler for MemoryRecallTool {
    fn name(&self) -> &str { "recall_memory" }

    fn description(&self) -> &str {
        "检索过往会话的记忆:相关对话记录(庭审)与合议产出的知识卡片(判例)。\
         用于在回答前回忆'这个议题以前审过吗、结论是什么'。\
         每场会话限调用 3 次。"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "要回忆的议题/关键词。用自然语言描述,语义相近即可命中。"
                },
                "limit": {
                    "type": "integer",
                    "description": "返回条数,默认 5",
                    "default": 5
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, arguments: &str) -> foundation::Result<String> {
        // 1. 解析参数
        let args: RecallArgs = serde_json::from_str(arguments)
            .unwrap_or_default();
        let limit = args.limit.unwrap_or(5).min(10);

        // 2. 限流检查(需要 session_id —— 通过约定:工具调用前 soul 上下文注入)
        //    见 §4.1.1 关于 session_id 来源
        // 3. 调 HybridSearcher
        let results = self.searcher.search(&args.query, limit).await
            .unwrap_or_default();   // 失败降级空
        // 4. 渲染 markdown
        Ok(render_results(&results))
    }
}

fn render_results(results: &[HybridResult]) -> String {
    if results.is_empty() {
        return "未找到相关历史记录。".to_string();
    }
    let mut out = String::from("## 召回的历史记录\n\n");
    for (i, r) in results.iter().enumerate() {
        let source_label = match r.source.as_str() {
            "card" => format!("📜 判例卡 [{}]", r.card_id.as_deref().unwrap_or("?")),
            _ => format!("💬 庭审记录 [{}]", r.session_id),
        };
        out.push_str(&format!(
            "### {}. {}\n**来源**:{} | **时间**:{}\n**摘要**:{}\n\n",
            i + 1, r.task_summary, source_label, r.created_at, r.content_snippet
        ));
    }
    out
}
```

#### 4.1.1 session_id 与限流的处理

工具的 `execute(arguments)` 只拿到模型给的 JSON,**没有 session 上下文**。这是现有 ToolHandler trait 的硬限制(`tools.rs:54-72`)。

**方案**:给 `MemoryRecallTool` 加一个 per-session 上下文注入机制 —— **不扩展 trait**,而是在 `run_tool_loop`(`stream.rs:252`)进入前,通过工具构造时的 session-scoped wrapper 注入。

具体做法(避免改 trait):
- `MemoryRecallTool` 内部用 `tokio::task_local!` 或一个 `Arc<RwLock<Option<String>>>` "当前 session_id" 槽
- `run_tool_loop` 进入时设置该槽,退出时清空
- 工具 execute 时读该槽做限流计数

**更简单的替代(采用)**:**放弃 per-session 限流,改成全局速率限制**——用 `governor` 或简单的 `AtomicU32` "近 N 秒内最多 M 次"。哲学讨论场景不会高频调用,全局速率足够防滥用。砍掉 session_id 跟踪,**降低复杂度**。

### 4.2 路径 B:conference synthesis 注入

**改动两处**:

#### 4.2.1 `prompt.rs` 新增方法(`ai-gateway/src/prompt.rs:601` 之后,作为 `build_synthesis_with_collisions` 的兄弟)

```rust
/// 综合官 prompt,附加召回的历史判例。
/// siblings: build_synthesis_prompt(无碰撞无历史)、build_synthesis_with_collisions(有碰撞无历史)
pub fn build_synthesis_with_recalled(
    &self,
    task: &str,
    outputs: &[(String, String)],
    collisions: Option<&str>,
    recalled: &[RecalledItem],
) -> Prompt {
    let mut user_content = format!("## 任务\n{}\n\n## 各魂输出全文\n", task);
    for (name, content) in outputs {
        user_content.push_str(&format!("\n### {}\n{}\n", name, content));
    }
    if let Some(c) = collisions.filter(|s| !s.is_empty()) {
        user_content.push_str(&format!("\n## 立场碰撞\n{}\n", c));
    }
    if !recalled.is_empty() {
        user_content.push_str("\n## 历史回响\n");
        user_content.push_str("以下是过往会话中与本次议题相关的记录与判例,供参考(非本场证据,可援引可质疑):\n\n");
        for item in recalled {
            user_content.push_str(&format!(
                "- **[{}]** {}({})\n",
                item.kind, item.summary, item.time
            ));
        }
    }
    user_content.push_str(&self.domain.synthesis_closing.as_deref().unwrap_or(""));
    let system_content = self.domain.synthesis_system_prompt.clone();
    Prompt { messages: vec![
        PromptMessage { role: "system".into(), content: system_content, ..Default::default() },
        PromptMessage { role: "user".into(), content: user_content, ..Default::default() },
    ]}
}

pub struct RecalledItem {
    pub kind: &'static str,    // "庭审" | "判例"
    pub summary: String,
    pub time: String,
}
```

#### 4.2.2 `conference.rs` 调用点(`possession/src/modes/conference.rs:244` 附近)

```rust
// 现有(line 206-243 不变):synthesis_outputs, collision_summary 组装

// v3 新增:召回历史判例
let recalled: Vec<RecalledItem> = if memory_inject_enabled {
    let hits = memory.searcher.search(task, 5).await.unwrap_or_default();
    hits.into_iter().map(|h| RecalledItem {
        kind: if h.source == "card" { "判例" } else { "庭审" },
        summary: format!("{}: {}", h.task_summary, h.content_snippet),
        time: h.created_at,
    }).collect()
} else { vec![] };

let synthesis_prompt = prompt_builder.build_synthesis_with_recalled(
    task, &synthesis_outputs,
    if collision_summary.is_empty() { None } else { Some(&collision_summary) },
    &recalled,
);
```

conference.rs 需要持有 `memory: ConferenceMemory { searcher: Arc<HybridSearcher>, inject_enabled: bool }`,通过 `PossessionEngine` 字段传入(见 §4.4)。

### 4.3 capability 注册与解析

#### 4.3.1 `tools.rs` 加别名(`possession/src/tools.rs:26-38` 的 `resolve_tool_name`)

```rust
"Memory" | "Recall" | "记忆" => "recall_memory".to_string(),
```

放在 `"WebSearch" | "WebFetch" | "Search"` 那行之后。

#### 4.3.2 工具注册(`api/src/main.rs:192-212`)

```rust
// 在 WebSearchTool 注册之后
// 注意:看"运行期标志位"(v2.1 §6.3),不是配置值——
// 配置 enabled=true 但 probe_dim 失败时,hybrid_searcher 仍为 None
if let Some(ref searcher) = hybrid_searcher {
    if config.vector_search.memory_recall.tool_enabled {
        engine.tool_registry_mut().register(Arc::new(
            memory_recall_tool::MemoryRecallTool::new(searcher.clone())
        ));
    }
}
```

注意:**只在 v2.1 向量搜索启用 + HybridSearcher 构造成功时注册**——否则 soul 写了 `Memory` 也调不到(工具不存在,filter_definitions 自然过滤掉)。

### 4.4 searcher 的注入路径

注入方式取决于 `PossessionEngine` 当前如何持有共享依赖(需在实现阶段核实其字段结构,本 spec 不预设字段名)。原则:

- `api/src/state.rs`:`AppState` 持有 `Option<Arc<HybridSearcher>>`(v2.1 已加)+ `memory_recall` 配置
- 路径 B 的 conference 流程需要拿到 searcher——通过 `PossessionEngine` 构造时从 AppState 注入,或 conference 调用时作为参数显式传入(二选一,实现时择优)
- 路径 A 的工具注册在 `main.rs`,直接拿 AppState 的 searcher,不经 PossessionEngine
- `searcher=None` 或对应 `*_enabled=false` 时,两条路径各自跳过,**优雅降级**

---

## 5. 数据流

### 5.1 路径 A:soul 主动召回

```
possession single mode
        │
        ▼
soul profile.tools 含 "Memory"
        │
        ▼
resolve_tool_name → "recall_memory"
        │
        ▼
filter_definitions 包含 recall_memory
        │
        ▼
run_tool_loop(stream.rs:252)
        │
        ▼
soul LLM 输出 tool_call: recall_memory(query="自由意志与决定论")
        │
        ▼
ToolRegistry::execute(dispatch)
        │
        ▼
MemoryRecallTool::execute
        │
        ▼
HybridSearcher::search("自由意志与决定论", 5)
        │
        ├─[并行] embed(query) + search_messages_fts + search_cards_fts + 2x vec KNN
        ▼
跨表 RRF → top-5 HybridResult
        │
        ▼
render_results → markdown
        │
        ▼
作为 role:"tool" 消息塞回 history(stream.rs:362)
        │
        ▼
下一轮 soul LLM 看到 ## 召回的历史记录,据此回应
```

**关键决策**:
- soul **自主决定**何时召回——多数议题不需要,避免无谓延迟
- 工具结果作为标准 tool message 回灌,**不改 prompt 结构**
- 失败返回空字符串(渲染为"未找到相关历史记录"),soul 自然忽略

### 5.2 路径 B:综合官被动注入

```
conference stage 1 完成 → 各 soul 输出收集完毕
        │
        ▼
conference.rs:244 附近
        │
        ▼
[v3 新增] memory.searcher.search(task, 5)
        │
        ▼
top-5 HybridResult → Vec<RecalledItem>
        │
        ▼
prompt_builder.build_synthesis_with_recalled(
    task, synthesis_outputs, collisions, recalled
)
        │
        ▼
synthesis user message 包含:
  ## 任务
  ## 各魂输出全文
  ## 立场碰撞(若有)
  ## 历史回响 ← 新增 section
  ## 综合要求
        │
        ▼
综合官 LLM 在 ThinkMax 下综合,参考历史判例
        │
        ▼
蒸馏 KnowledgeCard(走 v2.1 的 index_knowledge_card 路径)
```

**关键决策**:
- 注入 query 用 **task 原文**——task 是稳定语义锚;soul 输出是本场产物不该当 query
- 注入发生在**综合前**——综合官看到的是"本场各魂输出 + 历史回响",天然形成对比
- **每场 conference 固定一次召回**(无 rate limit),成本可控

### 5.3 配置开关(扩展 v2.1 的 `vector_search` 块)

```yaml
vector_search:
  enabled: false
  embedding:
    base_url: "http://localhost:1234"
    model: "bge-m3"
    # ... (v2.1 不变)
  memory_recall:
    tool_enabled: true          # 路径 A:soul 工具。false 则不注册 recall_memory
    synthesis_inject: true      # 路径 B:综合官注入。false 则 conference 不召回
    synthesis_top_k: 5          # 注入条数
    tool_rate_limit_per_min: 20 # 路径 A 全局速率限制(每分钟,防 soul 滥用)
```

**`vector_search.enabled=false` 时**:`memory_recall` 整块失效(无 searcher),与 v2.1 行为一致。
**`vector_search.enabled=true` 但运行期 probe 失败降级**(见 v2.1 §6.3):HybridSearcher 为 None,两条路径同样失效——优雅降级,不报错。

---

## 6. 错误处理

延续 v2.1 原则:**召回是增益,失败不阻塞主流程**。

| # | 失败点 | 影响范围 | 行为 |
|---|--------|---------|------|
| 1 | 工具 execute 时 searcher 失败 | 该次工具调用无结果 | 返回"未找到相关历史记录"字符串,工具 loop 继续 |
| 2 | conference 注入时 searcher 失败 | 该场综合无历史回响 | `unwrap_or_default()` 得空 vec,`build_synthesis_with_recalled` 正常渲染(无历史回响 section),综合照常进行 |
| 3 | `vector_search.enabled=false` | 两条路径全关 | 工具不注册;soul 写了 Memory 也调不到;conference 跳过召回。等价当前行为 |
| 4 | `memory_recall.tool_enabled=false` | 仅路径 A 关 | 工具不注册,路径 B 不受影响 |
| 5 | `memory_recall.synthesis_inject=false` | 仅路径 B 关 | conference 跳过召回,路径 A 不受影响 |

**核心原则**:**v3 失败的最坏情况 = 回到 v2.1 行为**(搜索还在,只是 soul/综合官用不上)。绝不会让 possession 或 conference 本身失败。

---

## 7. 测试策略

### 7.1 层 1:单元测试(纯函数)

- `memory_recall_tool::render_results`:
  - 空结果 → "未找到相关历史记录"
  - 混合 message + card → 正确标注来源标签
- `prompt.rs::build_synthesis_with_recalled`:
  - 空 recalled → 无 `## 历史回响` section
  - 有 recalled → section 存在且格式正确
  - 同时有 collisions + recalled → 两个 section 共存
  - 渲染不破坏 prefix-cache 友好性(system 消息不变)

### 7.2 层 2:集成测试(mock searcher)

`tests/memory_recall_integration.rs` —— `MockHybridSearcher` 注入确定性结果。

- 工具路径:
  - soul profile.tools="Memory" → tool_registry 含 recall_memory
  - 执行 execute → 调用 MockSearcher → 返回 markdown
  - MockSearcher 报错 → 返回"未找到"
- 注入路径:
  - conference 流程跑通,MockSearcher 返回 2 条 → synthesis user message 含 `## 历史回响`
  - synthesis_inject=false → user message 不含该 section

### 7.3 层 3:真模型 smoke(手动,`#[ignore]`)

- 写 3 条历史 message + 1 张 card(语义相关)
- possession 带 Memory 的 soul,问相关问题 → soul 应该调用 recall_memory 并引用历史
- conference 跑一场 → 综合输出应体现对历史回响的参考

**不进 CI**(依赖真 LMStudio + BGE-M3)。

---

## 8. 验收标准(DoD)

1. `cargo test` 全绿(含 v2.1 现有测试 + v3 新测试)
2. `cargo build --release` 通过
3. `vector_search.enabled=false`:行为与 v2.1 完全一致(零回归)
4. `enabled=true` + `tool_enabled=true`:
   - soul profile 写 `Memory` → 该 soul possession 时能调用 recall_memory
   - 工具返回 markdown,含来源标注
   - searcher 失败 → 工具返回"未找到",不报错
5. `enabled=true` + `synthesis_inject=true`:
   - conference 综合时,synthesis prompt 含 `## 历史回响`
   - searcher 失败 → 无该 section,综合照常完成
6. smoke 测试:soul 确实会主动调用 recall(不是必然,但模型应理解工具描述)
7. smoke 测试:综合官输出体现历史参考(定性,需人工判断)

---

## 9. 文件改动清单

| 文件 | 改动 |
|------|------|
| `rust/api/src/memory_recall_tool.rs` | **新建** — 工具实现,镜像 web_search_tool |
| `rust/api/src/main.rs:192-212` | 注册 MemoryRecallTool(条件注册) |
| `rust/possession/src/tools.rs:26-38` | `resolve_tool_name` 加 `"Memory"\|"Recall"\|"记忆" => "recall_memory"` |
| `rust/ai-gateway/src/prompt.rs:601` | 新增 `build_synthesis_with_recalled` + `RecalledItem` |
| `rust/possession/src/modes/conference.rs:244` | synthesis 前召回 + 改调新方法 |
| `rust/possession/src/lib.rs` | conference 入口持有/接收 `Option<Arc<HybridSearcher>>`(实现时择优字段 vs 参数注入) |
| `rust/api/src/state.rs` | (若 v2.1 已加 searcher 字段则无改动;否则加 `Option<Arc<HybridSearcher>>`) |
| `config/default.yaml` | `vector_search` 块加 `memory_recall` 子配置 |
| `data/souls/*.md` | 演示用:给 1-2 个 soul 的 frontmatter `tools:` 加 `Memory`(可选) |

**总改动量预估**:~250 行新代码 + ~30 行修改。**不动 foundation 层**(HybridSearcher 复用 v2.1)。

---

## 10. 风险与边界

- **soul 可能不用工具**——LLM 不一定理解何时该 recall。缓解:工具 description 写明确("在回答前回忆'这个议题以前审过吗'");长期看可在 summon_prompt 里教 soul 用。验收 #6 接受"不是必然调用"
- **综合官注入可能引入噪声**——top-5 召回未必都相关,可能干扰综合。缓解:prompt 里明确"非本场证据,可援引可质疑";top_k 可调
- **延迟**——路径 A 在 soul 决策路径上加一次 HTTP recall(~100-500ms);路径 B 在综合前加一次(固定)。LMStudio loopback,可接受;若敏感可关 tool/synthesis_inject
- **成本**——每场 conference 多一次 BGE-M3 embed + 2 路 KNN;每场 possession 视 soul 调用次数。低频,可控
- **向 v4/v5 演进**——本 spec 的 RecallItem 渲染是"扁平列表";未来可做语义聚类(把多条召回聚成"反复讨论的议题")、盲点检测(某 soul 在某类议题总被召回却无有效论证)。这是 v5 方向,本 spec 不实施
- **session 限流的取舍**——§4.1.1 选择全局速率而非 per-session,牺牲了精确限流换简单;若发现滥用,可在 v3.1 引入 task_local session_id

---

## 附录:与 v2.1 的关系

v2.1 建的是**"能搜"**(基础设施),v3 建的是**"能用"**(接入思维链):

| 层 | v2.1 | v3 |
|----|------|----|
| 搜索能力 | HybridSearcher 跨表 RRF | 复用,不改 |
| 数据写入 | message/card embedding + FTS | 复用,不改 |
| 查询入口 | `/api/knowledge/search`(人类用) | **新增**:工具(soul 用)+ 注入(综合官用) |
| 消费者 | 前端人类用户 | **新增**:soul LLM、综合官 LLM |

**v3 是 v2.1 的纯上层应用,不动 v2.1 任何已建好的代码**。v2.1 没接 v3 也能独立运行(人类用搜索),v3 没有v2.1 不能运行(依赖 searcher)。
