# 万民幡 Rust 版设计指导

> **前提**：不再复刻 Claude Code 版的限制——用 Rust 的能力重新想象万民幡可以是什么。

---

## 一、定位跃迁

| | Claude Code 版 | Rust 版 |
|---|---|---|
| 本质 | 脚本化工作流（编排 Claude Code 平台能力） | 独立软件系统（自有架构） |
| 魂 | 一次性 spawn，用完即弃 | 长驻进程，有记忆连续性 |
| 合议 | 串行等待→一次性综合 | 流式碰撞，实时交叉 |
| 计算 | LLM 调用为主，脚本辅助 | 确定性规则优先，LLM 只在需要判断时调用 |
| 反馈闭环 | 人工操作（改 YAML / 写 MD） | 系统内生驱动（魂自我审计 → 自动修正提案） |
| 使用者 | 单人 | 多人独立上下文 + 共享魂框架修正 |
| 存储 | 模拟 Obsidian vault（Markdown 文件树） | 嵌入式数据库 + 全文检索 + 向量语义检索 |
| 界面 | Claude Code 会话 | CLI / TUI / Web / Bot 独立切换 |

---

## 二、核心能力

### 2.1 魂是长驻进程

```
魂 A (tokio task) ── mpsc channel ──► 持续监听议题
                   ── 内部状态     ──► 上次对此类问题的判断
                   ── 自我审计     ──► 发现与历史矛盾 → 自动修正声明
```

每个魂是一个 `tokio` task，靠 channel 接收任务。不是每次附体重新解释「你是谁」，而是魂在启动时加载一次 summon_prompt，之后维持自己的上下文状态。内存里记住上次的结论，下次对话接着上次说。

```rust
struct SoulProcess {
    profile: SoulProfile,
    state: SoulState,           // 内部状态：最近 N 次输出摘要、活跃修正提案
    rx: mpsc::Receiver<Task>,   // 接收议题
    tx: mpsc::Sender<Output>,   // 流式输出
    llm: Arc<dyn LlmClient>,
}

impl SoulProcess {
    async fn run(mut self) {
        while let Some(task) = self.rx.recv().await {
            // 不是从零开始，而是带着 state 里的记忆一起推理
            let context = self.build_context(&task);
            let output = self.llm.chat_stream(context).await;
            self.state.record(task, &output);
            self.tx.send(output).await;
        }
    }
}
```

### 2.2 流式合议

不是「等所有人说完再综合」——而是在魂产出 token 的过程中实时交叉：

```
魂 A ── token stream ──┐
魂 B ── token stream ──┼──► 实时交叉检测器 ──► 矛盾/互补 ──► 动态注入追问
魂 C ── token stream ──┘
```

交叉检测器监听各魂的输出流，当检测到可碰撞的点（A 说 X，B 说了非 X；或 A 的盲区恰好是 B 的领域），立刻生成追问注入到相关魂的 prompt 中。

```rust
struct StreamingConference {
    souls: Vec<SoulProcess>,
    detector: CrossDetector,    // 实时检测矛盾/互补/盲区
    synthesizer: Synthesizer,   // 持续更新综合视图
}

impl StreamingConference {
    async fn run(&mut self, task: Task) -> Synthesis {
        // 1. 广播 task 到所有魂
        // 2. 每条 token stream 进入 detector
        // 3. detector 发现碰撞 → 生成追问 → 注入目标魂
        // 4. 所有魂流结束后 → synthesizer 输出最终综合
    }
}
```

这比「完整输出→综合官消化」多了一个维度：碰撞是实时发生的，不能事后追认。追问的方向本身是合议产出的一部分。

### 2.3 计算分层：LLM 只做非它不可的事

```
零 LLM 成本（确定性规则）：
  ├── 入口分流判定
  ├── 魂关键词/领域预筛选
  ├── 连续消费性使用检测
  ├── 失败条件触发
  ├── 魂休眠/唤醒判定
  └── 共识/分歧骨架提取

LLM 调用（需要判断力）：
  ├── 魂分析输出（核心成本）
  ├── 幡主审查（仅在 match 置信度 < 0.3）
  ├── 辩证综合（五步法）
  └── 交叉追问生成（仅在 detector 发现碰撞时）
```

每次合议的 LLM 调用次数是动态的——理想情况下只调 N+1 次（N 魂分析 + 1 综合官），没有碰撞就不触发追问。

### 2.4 魂自我进化

这不是人工改 YAML，而是魂内生驱动的自我修正：

```rust
impl SoulProcess {
    /// 每次输出后自动执行
    fn post_output_audit(&mut self, task: &Task, output: &Output) -> Option<Revision> {
        // 1. 对比本次输出与历史 — 有没有自我矛盾？
        if let Some(contradiction) = self.state.detect_contradiction(output) {
            return Some(Revision::from_contradiction(contradiction));
        }

        // 2. 检查是否触及已知盲区
        if let Some(blind_spot) = self.profile.touching_blind_spot(task, output) {
            return Some(Revision::blind_spot_alert(blind_spot));
        }

        // 3. 框架前提是否被实践数据动摇？
        if let Some(shaken) = self.state.check_premise_shaken(task, output) {
            return Some(Revision::premise_shaken(shaken));
        }

        None
    }
}
```

修正提案自动提交幡主审查，审查通过后合并到 summon_prompt。整个过程不是「使用者决定要不要修正」，而是「魂自己发现问题 → 提交审查 → 审查通过自动生效」。

### 2.5 魂间直接对话

不只是主编排器居中调度——魂可以直接和另一个魂对话：

```rust
// 辩论模式：两魂直接交换输出流
debate(soul_a, soul_b, topic).await;

// 学习模式：两对立魂互读互审
soul_a.request_review(soul_b.id(), review_request).await;
soul_b.request_review(soul_a.id(), review_request).await;

// 召唤协作：魂 A 发现需要魂 B 的领域知识
// 在流式输出中声明 → 编排器检测 → 自动 spawn B → 注入 B 的回应到 A 的后续推理
```

主编排器只做三件事：路由、记录、存档。魂之间的交互是去中心化的。

### 2.6 多使用者 + 共享魂修正

```
┌──────────────────────────────────────┐
│              魂 Engine               │
│  ┌────┐ ┌────┐ ┌────┐ ┌──────────┐  │
│  │列宁│ │鲁迅│ │费曼│ │辩证综合官│  │
│  └──┬─┘ └──┬─┘ └──┬─┘ └────┬─────┘  │
│     │       │       │         │        │
│     └───────┴───────┴─────────┘        │
│                    │                   │
│     ┌──────────────┼──────────────┐    │
│     ▼              ▼              ▼    │
│  ┌──────┐     ┌──────┐     ┌──────┐   │
│  │用户 A │     │用户 B │     │用户 C │   │
│  │独立   │     │独立   │     │独立   │   │
│  │上下文 │     │上下文 │     │上下文 │   │
│  └──────┘     └──────┘     └──────┘   │
└──────────────────────────────────────┘

用户 A 的实践反馈 → 魂修正 → 用户 B 下次调用同一魂自动受益
用户 B 的召唤记录不暴露给用户 A ——上下文隔离
```

同一套魂，多人独立使用，但魂的框架修正全员共享。每次修正带来源标记（来自哪个用户的实践数据），但不暴露用户的具体内容。

### 2.7 脱离 Obsidian，用真正的数据库

不再模拟文件树。存档层是结构化的：

```sql
-- 魂输出不是文件，是可查询的记录
CREATE TABLE soul_outputs (
    id UUID,
    soul_name TEXT,
    mode TEXT,           -- single / conference / debate / relay / practice_opening
    task TEXT,
    content TEXT,        -- 完整输出（不压缩）
    metadata JSONB,
    embedding VECTOR(1536),  -- 语义检索
    created_at TIMESTAMP
);

-- 魂修正历史
CREATE TABLE soul_revisions (
    id UUID,
    soul_name TEXT,
    trigger TEXT,        -- contradiction / blind_spot / premise_shaken / practice_feedback
    proposal TEXT,
    status TEXT,         -- pending / approved / rejected
    source_user UUID,
    created_at TIMESTAMP
);
```

全文检索（tantivy）+ 向量检索（pgvector 或 qdrant），你可以问：「过去三个月，哪些合议反复触及同一个盲区？」

### 2.8 界面与引擎分离：Next.js 前端

Rust 核心是库（`soul-banner-core`），暴露 gRPC API。Next.js 前端通过 API Routes 桥接 gRPC：

```
soul-banner-core (Rust lib)
  └── gRPC server (tonic)
        │
        │  gRPC stream / unary
        │
  ┌─────▼──────────────────────────────────┐
  │        Next.js 前端                      │
  │                                         │
  │  RSC（服务端渲染）                        │
  │  ├── 魂列表 / 战绩 / 知识库               │
  │  └── 数据从 gRPC 直取，不经过客户端        │
  │                                         │
  │  Client Components（浏览器端）            │
  │  ├── 合议页（多列 SSE 流式面板）           │
  │  ├── 辩论页 / 魂详情页                    │
  │  └── EventSource 接收 SSE               │
  │                                         │
  │  API Routes                             │
  │  ├── /api/confer → gRPC stream → SSE   │
  │  └── /api/* → gRPC unary → JSON        │
  └─────────────────────────────────────────┘
        │
        │  辅助界面
  ┌─────▼──────────┐
  │  CLI (clap)    │  ← 脚本/自动化/SSH
  │  TUI (ratatui) │  ← 本地重度使用，零序列化开销
  └────────────────┘
```

Rust 核心库不关心界面——它只暴露 gRPC API。Next.js 负责 Web 体验，CLI/TUI 和核心库同进程运行。

---

## 三、架构

```
┌──────────────────────────────────────────────────────┐
│                   soul-banner-core                     │
│                                                       │
│  ┌──────────┐  ┌──────────┐  ┌───────────────────┐   │
│  │ 入口分流  │  │ 魂匹配器  │  │ 模型路由器          │   │
│  │ (纯规则)  │  │ (TF-IDF) │  │ (能力感知+成本优化) │   │
│  └──────────┘  └──────────┘  └───────────────────┘   │
│                                                       │
│  ┌──────────────────────────────────────────────┐    │
│  │              魂进程管理器                       │    │
│  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────────┐    │    │
│  │  │ 魂 A  │ │ 魂 B  │ │ 魂 C  │ │ 幡主审查  │    │    │
│  │  │ task │ │ task │ │ task │ │   task    │    │    │
│  │  └──┬───┘ └──┬───┘ └──┬───┘ └────┬─────┘    │    │
│  │     └────────┼────────┼──────────┘           │    │
│  │              ▼        ▼                       │    │
│  │         ┌──────────────┐                      │    │
│  │         │ 流式交叉检测器 │                      │    │
│  │         └──────┬───────┘                      │    │
│  │                ▼                               │    │
│  │         ┌──────────────┐                      │    │
│  │         │  辩证综合官   │                      │    │
│  │         └──────────────┘                      │    │
│  └──────────────────────────────────────────────┘    │
│                                                       │
│  ┌──────────┐  ┌──────────┐  ┌───────────────────┐   │
│  │ 审计引擎  │  │ 知识库    │  │ 魂修正引擎         │   │
│  │ (失败条件)│  │ (全文+向量)│  │ (self_audit →     │   │
│  │          │  │          │  │  revision_proposal)│   │
│  └──────────┘  └──────────┘  └───────────────────┘   │
│                                                       │
│  ┌──────────────────────────────────────────────┐    │
│  │                LLM 抽象层                      │    │
│  │  OpenAI 兼容 │ Anthropic │ Ollama │ 其他…      │    │
│  └──────────────────────────────────────────────┘    │
│                                                       │
│  ┌──────────────────────────────────────────────┐    │
│  │                存储层                           │    │
│  │  SQLite/Postgres │ tantivy (全文) │ pgvector   │    │
│  └──────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────┘
                          │
                          │ gRPC
          ┌───────────────┼───────────────┐
          ▼               ▼               ▼
     ┌────────┐     ┌────────┐     ┌────────────┐
     │   CLI  │     │  TUI   │     │  Web / Bot │
     └────────┘     └────────┘     └────────────┘
```

---

## 四、源文件结构

```
rust-soul-banner/
├── Cargo.toml
├── soul-banner-core/             # 核心库（纯 Rust，不绑定界面）
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # 公开 API
│       ├── triage.rs             # 入口分流（零 LLM）
│       ├── matcher.rs            # 魂匹配（TF-IDF + 规则）
│       ├── soul/
│       │   ├── mod.rs
│       │   ├── profile.rs        # SoulProfile 结构 + YAML 解析
│       │   ├── process.rs        # 魂长驻进程（tokio task + channel）
│       │   ├── spawner.rs        # 魂生命周期管理
│       │   └── self_audit.rs     # 魂自我审计 → 修正提案
│       ├── modes/
│       │   ├── mod.rs
│       │   ├── single.rs         # 单魂附体
│       │   ├── conference.rs     # 流式合议 + 交叉检测
│       │   ├── debate.rs         # 两魂直接对话
│       │   ├── relay.rs          # 接力
│       │   ├── practice_opening.rs # 实践开口
│       │   └── user_study.rs     # 使用者学习
│       ├── conference/
│       │   ├── mod.rs
│       │   ├── cross_detector.rs # 实时交叉检测器
│       │   └── synthesizer.rs    # 辩证综合官
│       ├── review/
│       │   ├── mod.rs
│       │   ├── banner_master.rs  # 幡主审查
│       │   └── revision_engine.rs # 魂修正提案 → 审查 → 合并
│       ├── registry.rs           # 魂注册表
│       ├── call_records.rs       # 召唤记录 + 有效性
│       ├── audit.rs              # 失败条件自动检测
│       ├── knowledge.rs          # 全文检索 + 向量检索 + 卡片管理
│       ├── model_router.rs       # 模型能力感知 + 成本优化路由
│       ├── prompt/
│       │   ├── mod.rs
│       │   ├── builder.rs        # Prompt 组装
│       │   └── templates.rs      # 各模式 prompt 模板
│       ├── llm/
│       │   ├── mod.rs
│       │   ├── client.rs         # LlmClient trait
│       │   ├── openai.rs         # OpenAI 兼容
│       │   ├── anthropic.rs      # Anthropic
│       │   └── ollama.rs         # 本地模型
│       ├── storage/
│       │   ├── mod.rs
│       │   ├── db.rs             # SQLite/Postgres
│       │   ├── fulltext.rs       # tantivy 全文索引
│       │   └── vector.rs         # pgvector / qdrant
│       └── multi_user.rs         # 多使用者隔离 + 共享魂修正
├── sb-cli/                       # CLI（clap）
│   ├── Cargo.toml
│   └── src/main.rs
├── sb-tui/                       # TUI（ratatui，可选）
│   ├── Cargo.toml
│   └── src/main.rs
├── sb-server/                    # gRPC server 入口
│   ├── Cargo.toml
│   └── src/main.rs
├── frontend/                     # Next.js 前端
│   ├── package.json
│   ├── next.config.js
│   ├── tailwind.config.ts
│   └── src/
│       ├── app/
│       │   ├── layout.tsx
│       │   ├── page.tsx                    # 首页
│       │   ├── confer/[id]/page.tsx        # 合议页 (Client)
│       │   ├── debate/[id]/page.tsx        # 辩论页 (Client)
│       │   ├── souls/[name]/page.tsx       # 魂详情 (RSC + Client)
│       │   ├── knowledge/page.tsx          # 知识库
│       │   └── api/
│       │       ├── confer/route.ts         # gRPC stream → SSE
│       │       ├── souls/route.ts
│       │       └── knowledge/route.ts
│       ├── components/
│       │   ├── SoulPanel.tsx               # 魂流式面板
│       │   ├── SoulGrid.tsx                # 多列面板容器
│       │   ├── CollisionFeed.tsx           # 碰撞通知流
│       │   ├── SynthesisPanel.tsx          # 辩证综合面板
│       │   ├── DebateColumn.tsx            # 辩论列
│       │   ├── RulingPanel.tsx             # 裁决面板
│       │   ├── RevisionTimeline.tsx        # 修正时间线
│       │   └── BlindSpotHeatmap.tsx        # 盲区热力图
│       └── lib/
│           ├── grpc-client.ts              # gRPC 客户端封装
│           └── sse.ts                      # SSE 事件解析
├── data/
│   ├── registry.yaml
│   └── souls/
│       ├── 列宁.md
│       ├── 毛泽东.md
│       └── ...
└── docs/
    ├── rust-port-guide.md
    └── ui-design.md
```

---

## 五、gRPC API（核心库对外接口）

```protobuf
service SoulBanner {
    // 附体模式
    rpc PossessSingle(PossessRequest) returns (stream SoulEvent);
    rpc Confer(ConferRequest) returns (stream ConferenceEvent);
    rpc Debate(DebateRequest) returns (stream DebateEvent);
    rpc Relay(RelayRequest) returns (stream RelayEvent);
    rpc PracticeOpening(PracticeOpeningRequest) returns (PracticeOpeningResponse);

    // 管理
    rpc ListSouls(ListSoulsRequest) returns (ListSoulsResponse);
    rpc CollectSoul(CollectSoulRequest) returns (CollectSoulResponse);
    rpc RefineSoul(RefineSoulRequest) returns (RefineSoulResponse);
    rpc ReviewSoul(ReviewSoulRequest) returns (ReviewSoulResponse);
    rpc DismissSoul(DismissSoulRequest) returns (DismissSoulResponse);

    // 查询
    rpc GetStats(GetStatsRequest) returns (StatsResponse);
    rpc SearchKnowledge(SearchKnowledgeRequest) returns (SearchKnowledgeResponse);
    rpc QueryCallRecords(QueryCallRecordsRequest) returns (QueryCallRecordsResponse);
}

message ConferenceEvent {
    oneof event {
        SoulToken soul_token = 1;        // 魂的流式 token
        CrossDetection collision = 2;    // 交叉检测到碰撞
        SoulOutput soul_output = 3;      // 单个魂完成
        Synthesis synthesis = 4;         // 最终综合
    }
}
```

所有模式返回 stream——界面可以实时渲染魂的思考过程。

---

## 六、CLI

```bash
# 附体
sb possess --soul 列宁 "分析具身智能产业格局"
sb confer "AI 时代的工厂生产关系"
sb debate "AI 解放还是异化劳动" --a 马克思 --b 费曼
sb relay "厂二代转型路线" --souls 列宁,毛泽东,鲁迅,邓小平
sb open "我的工厂正在引入机器人替代工人"
sb study "黑格尔辩证法"

# 魂管理
sb list                      # 幡中有什么魂（含休眠/活跃/零召唤标记）
sb collect "庄子"            # 收魂
sb refine "庄子"             # 炼化
sb review "庄子"             # 审查
sb dismiss "庄子"            # 散魂
sb soul-status "列宁"        # 单魂详情：最近输出、修正历史、盲区记录

# 运维
sb stats                     # 幡中战绩
sb audit                     # 失败条件检查
sb knowledge "关键词"        # 知识库检索
sb check                     # 交叉校验

# 流式模式（终端实时看 token 流）
sb confer --stream "AI 时代的工厂生产关系"
```

---

## 七、MVP 范围

### 第一阶段：最小可用（魂是独立 API 调用）

- [ ] `SoulProfile` 加载（YAML/Markdown 解析）
- [ ] `LlmClient` trait + OpenAI 兼容实现
- [ ] 单魂附体（单 API 调用，streaming 输出）
- [ ] 多魂合议（并行调用 + 等待全部完成后辩证综合）
- [ ] 输出落盘（SQLite + Markdown 双写）
- [ ] 召唤记录 + 有效性评分
- [ ] CLI（clap）

**这个阶段已经比 Claude Code 版好**：不绑平台、可混合模型、流式输出。

### 第二阶段：魂进程化 + 流式碰撞

- [ ] 魂长驻进程（tokio task + channel）
- [ ] 流式合议 + 交叉检测器
- [ ] 幡主审查（独立调用）
- [ ] 魂匹配引擎（TF-IDF + 规则）
- [ ] 入口分流（纯规则）
- [ ] 辩论/接力模式

### 第三阶段：自我进化

- [ ] 魂自我审计（post_output_audit）
- [ ] 自动修正提案 → 幡主审查 → 合并
- [ ] 失败条件自动检测
- [ ] 实践开口模式
- [ ] 使用者学习模式

### 第四阶段：多用户 + 知识层

- [ ] 多用户上下文隔离
- [ ] 共享魂修正
- [ ] 全文检索（tantivy）
- [ ] 向量检索
- [ ] 知识卡片自动提取
- [ ] 魂休眠机制

### 第五阶段：界面层

- [ ] gRPC API server
- [ ] TUI（ratatui 实时流渲染）
- [ ] 多 provider（Anthropic / Ollama）
- [ ] 模型能力感知路由
- [ ] Web UI

---

## 八、不可妥协的设计约束

1. **魂上下文隔离** — 每个魂独立 LLM 调用，不共享上下文
2. **审查者隔离** — 幡主审查必须是独立调用，不能和魂分析在同一上下文中
3. **输出原文保全** — 不压缩、改写、概括
4. **计算分层** — 能用规则判断的绝不用 LLM
5. **引擎与界面分离** — 核心库不依赖任何界面框架，通过 gRPC 对外暴露

---

---

## 九、围绕 DeepSeek V4 的架构优化

> DeepSeek V4 有三个对万民幡来说是结构性利好的特性：自动上下文缓存（80-92% 折扣）、1M 上下文窗口、跨轮思考保留。

### 9.1 上下文缓存：万民幡的天然适配

万民幡的使用模式天然适合 DeepSeek 的自动前缀缓存：

```
每次魂 spawn 的 messages 结构：

[system prompt: 魂方法论 + 人格 + 坐标]  ← 静态，跨调用不变，100% 缓存命中
[user: 任务描述 + 时代背景 + 约束]          ← 同一次合议中 N 个魂共享，N-1 次命中
[user: 本魂职责 + 输出路径]                ← 魂间不同，不命中
```

**优化策略**：

```rust
impl PromptBuilder {
    /// 为 DeepSeek 缓存优化的 message 顺序
    fn build_cache_optimized(&self, soul: &SoulProfile, ctx: &TaskContext) -> Vec<Message> {
        vec![
            // 第1条：魂的完整 summon_prompt（静态 —— 每次调用都一样）
            // → 首次加载后，后续所有对该魂的调用 100% 缓存命中
            Message::system(&soul.summon_prompt),

            // 第2条：共享任务上下文（同一次合议中所有魂相同）
            // → 第一个魂加载后，其余 N-1 个魂全部命中
            Message::user(&ctx.shared_task_context()),

            // 第3条：本魂特有指令（动态 —— 各魂不同，不期望命中）
            Message::user(&format!(
                "## 你的职责\n{}\n\n## 输出路径\n{}",
                soul.role_in_conference(),
                ctx.output_path_for(soul.name())
            )),
        ]
    }
}
```

**成本估算（5魂合议）**：

| 场景 | 无缓存 | DeepSeek V4 缓存优化后 |
|------|--------|----------------------|
| 5魂 × 2万token summon_prompt | 每次全价 | 首魂全价，4魂命中（80-92%折扣） |
| 共享任务上下文（3K token） | 5次全价 | 1次全价，4次命中 |
| 合计（Flash） | ~$0.35 | ~$0.08（约 **4.4x 降幅**） |
| 合计（Pro） | ~$4.35 | ~$0.82（约 **5.3x 降幅**） |

对于高频使用的魂（列宁已被召唤47次），summon_prompt 缓存命中率接近 100%。

### 9.2 1M 上下文窗口：重新定义「综合」

之前辩证综合官能看到的上下文受限于模型窗口。现在 1M token 意味着：

**魂自我审计可以对比完整历史**：

```rust
impl SoulProcess {
    async fn deep_self_audit(&self, db: &Database) -> AuditReport {
        // 拉取本魂最近 20 次输出的全文（不压缩、不摘要）
        let recent_outputs = db.recent_outputs(&self.name, 20).await;
        // 拉取所有相关审查报告
        let reviews = db.reviews_for(&self.name).await;
        // 拉取所有 practice_observations
        let observations = db.observations_for(&self.name).await;

        // 全部注入 prompt —— 1M 窗口下这不是问题
        let prompt = format!(
            "{}\n\n## 你的历史输出（全文）\n{}\n\n## 审查报告\n{}\n\n## 实践观察\n{}",
            SELF_AUDIT_TEMPLATE,
            recent_outputs.join("\n---\n"),
            reviews.join("\n---\n"),
            observations.join("\n---\n")
        );

        self.llm.chat().system(&self.summon_prompt).user(prompt).send().await
    }
}
```

**辩证综合官读入完整合议历史**：

不只是当前合议的 N 个魂输出——而是连同相关知识库条目、最近 3 次同类合议的综合报告、所有参与魂的修正历史——全部注入。综合官不再「盲审」，它有完整的上下文纵深。

**知识库检索不再截断**：

搜索命中 50 个相关文档？以前只能取 top 3 摘要。现在全塞进去——tantivy 全文排在前面，向量检索补充语义相关的，综合官自己判断哪些相关。

### 9.3 跨轮思考保留：魂长驻进程的加速器

DeepSeek V4 的 Interleaved Thinking 特性：在多轮工具调用中，推理链不被清空。

这和万民幡的「魂长驻进程」设计完美吻合：

```
魂进程生命周期（利用跨轮思考保留）：

启动 → 加载 summon_prompt → 进入待命
  │
  ├─ 第1次任务：完整推理（Think High）
  │   └─ 推理链保留 ← DeepSeek 自动保留
  │
  ├─ 第2次任务：推理链已有上下文
  │   └─ 跳过重复推理，直接基于上次结论展开
  │
  ├─ 第3次任务：发现与第1次矛盾
  │   └─ 推理链中有完整历史 → 自动触发自我修正声明
  │
  └─ ...
```

传统 API 每次调用是全新上下文——魂每次都从零开始推理同一个背景。DeepSeek V4 的跨轮保留意味着魂可以在多次调用间维持连贯思维链，这是「魂有记忆」的技术基础。

```rust
// 利用跨轮思考保留
impl SoulProcess {
    async fn run_with_continuity(&mut self) {
        let mut conversation: Vec<Message> = vec![
            Message::system(&self.summon_prompt),
        ];

        while let Some(task) = self.rx.recv().await {
            // 不需要重新发送 summon_prompt —— DeepSeek 保留了推理链
            // 只需发送增量任务
            conversation.push(Message::user(&task.prompt));

            let response = self.llm.chat()
                .messages(&conversation)
                .reasoning_effort(if task.complexity > 0.7 {
                    ReasoningEffort::ThinkHigh
                } else {
                    ReasoningEffort::Think   // 保留推理链，但力度降低
                })
                .send()
                .await;

            conversation.push(Message::assistant(&response.content));
            // 推理链自动保留，下一个 task 受益

            // 定期清理旧消息防止上下文膨胀
            if conversation.len() > 20 {
                // 保留 system + 最近 10 轮
                conversation.drain(1..conversation.len() - 20);
            }
        }
    }
}
```

### 9.4 模型路由器：Flash vs Pro 的分工

```rust
enum DeepSeekModel {
    Flash,          // 高性价比，低延迟
    ProThink,       // Strong reasoning
    ProThinkHigh,   // Deep reasoning
    ProThinkMax,    // 1M context + max reasoning
}

struct ModelRouter {
    flash: LlmClient,  // deepseek-v4-flash
    pro: LlmClient,    // deepseek-v4-pro
}

impl ModelRouter {
    fn route(&self, task: &RoutingRequest) -> (DeepSeekModel, ReasoningEffort) {
        match task.kind {
            // 零 LLM 成本 —— 路由器根本不调用
            RoutingKind::Triage | RoutingKind::Match => unreachable!(),

            // Flash 足够 —— 简单魂、简单查询
            RoutingKind::SimpleSoul       // 海绵宝宝、乔布斯
            | RoutingKind::StatsQuery
            | RoutingKind::KnowledgeSearch => (DeepSeekModel::Flash, ReasoningEffort::NonThink),

            // Pro + 标准推理 —— 核心分析魂
            RoutingKind::CoreAnalysis     // 列宁、毛泽东、鲁迅
            | RoutingKind::Debate => (DeepSeekModel::Pro, ReasoningEffort::Think),

            // Pro + 深度推理 —— 需要严格判断
            RoutingKind::BannerMasterReview
            | RoutingKind::SecondReview
            | RoutingKind::SoulRefinement => (DeepSeekModel::Pro, ReasoningEffort::ThinkHigh),

            // Pro + 最大推理 —— 最复杂的综合任务
            RoutingKind::Synthesis
            | RoutingKind::SoulSelfAudit => (DeepSeekModel::Pro, ReasoningEffort::ThinkMax),
        }
    }
}
```

### 9.5 结构化输出：综合报告直接落盘

DeepSeek V4 的 JSON Schema 遵循率 > 99%。辩证综合官可以直接输出结构化数据：

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
struct SynthesisOutput {
    consensus: Vec<ConsensusItem>,
    divergence: Vec<DivergenceItem>,
    blind_spots: Vec<BlindSpotItem>,
    principal_contradiction: Contradiction,
    action_program: Vec<ActionItem>,
}

// 综合官调用时强制 JSON 输出
let synthesis: SynthesisOutput = llm.chat()
    .system(SYNTHESIZER_PROMPT)
    .user(synthesis_prompt)
    .response_format(ResponseFormat::JsonSchema)
    .send()
    .await
    .parse()?;

// 直接写入数据库，不需要从 Markdown 中解析五步法
db.insert_synthesis(synthesis).await;
```

不需要从一段 Markdown 文本中用正则提取「共识」「分歧」「盲区」——综合官输出直接是结构化数据。前端渲染、知识库索引、盲区热力图都省去了解析步骤。

### 9.6 成本模型

按 DeepSeek V4 定价和缓存策略，预估一次 5魂合议的成本：

```
模型路由：
  列宁（Pro Think）    ：summon_prompt 缓存命中  $0.145 × 0.02M = $0.003
  鲁迅（Pro Think）    ：summon_prompt 缓存命中  $0.003
  费曼（Pro Think）    ：summon_prompt 缓存命中  $0.003
  Karpathy（Flash）    ：summon_prompt 缓存命中  $0.028 × 0.015M = $0.0004
  未明子（Pro Think）  ：summon_prompt 缓存命中  $0.003

  共享任务上下文（3K）  ：首个 Pro 全价 + 其余命中  约 $0.005
  各魂输出（平均 2K）   ：Pro 输出 $3.48/M × 0.008M × 4 = $0.111
                         Flash 输出 $0.28/M × 0.002M = $0.001

  辩证综合官（Pro ThinkMax，32K 输入 + 3K 输出）：
                        输入 $1.74/M × 0.032M = $0.056
                        输出 $3.48/M × 0.003M = $0.010

  合计：~$0.20 / 次 5魂合议
```

对比 Claude Code 版（所有魂走 Opus，无缓存优化）的 ~$2-3/次，成本降低约 **10-15 倍**。

---

*「法有穷时，行无竟处。」—— DeepSeek V4 让这面旗帜的日常使用成本降低了一个数量级。*
