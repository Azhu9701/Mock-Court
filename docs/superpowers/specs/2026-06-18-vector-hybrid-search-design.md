# 向量 Hybrid 搜索设计

- **日期**:2026-06-18
- **状态**:已确认,待实现
- **范围**:`foundation` 新增向量召回 + hybrid 融合;`archive`/`api` 转发层适配;HTTP 接口签名零改动

---

## 1. 背景与目标

### 1.1 现状

rust-agent 的 knowledge 搜索当前依赖 **SQLite FTS5 + trigram tokenizer**(`rust/foundation/src/sqlite.rs:622-683`),对 possession 会话产生的 message 内容做字面全文检索,并配合 `cjk_tokenize`(`sqlite.rs:586-599`)处理中文。

`rust/foundation/src/vector_search.rs` 存在一个 `SimpleVectorIndex` 内存脚手架(线性扫描余弦相似度),但**没有接入任何 embedding 模型,生产无人调用**——是预留的后路。

### 1.2 问题

FTS5 是字面匹配,**无法召回语义相近但字面不同的内容**。典型漏召回:

- "如何让 AI 记住我" vs "长期记忆怎么实现" —— 同义改述,字面零重合
- "persistent memory for LLM" vs "让大模型有记忆" —— 跨语言
- 口语化、错别字、同义替换

possession 会话中 soul 想"回忆过往类似对话"时,纯字面召回会漏掉大量相关历史。

### 1.3 目标

在现有 FTS5 基础上**并联一条向量召回支路**,用 **RRF(Reciprocal Rank Fusion)** 融合两路 top-K 结果,实现教科书级 hybrid 搜索。embedding 走**进程内 ONNX 推理**(`bge-small-zh-v1.5`,512 维),存储复用现有 SQLite 库(**sqlite-vec 扩展**)。

全链路失败时**静默降级到纯 FTS5**,行为与今天完全一致 → 老用户升级零风险。

### 1.4 非目标

- 不替换 FTS5(它是底线,不是被替代方)
- 不改 soul 搜索(`registry/src/search.rs` 倒排索引 + ismism 最近邻目前够用)
- 不引外部向量数据库(qdrant / lancedb / pgvector)
- 不引外部 embedding API(OpenAI / Cohere / Ollama)
- HTTP `/api/knowledge/search` 签名零改动,前端无需适配

---

## 2. 关键决策(已与用户确认)

| # | 决策 | 选择 | 备选与理由 |
|---|------|------|-----------|
| 1 | embedding 来源 | **本地 ONNX(bge-small-zh-v1.5, 512 维)** | 否决 OpenAI API(联网/要钱/破相)、Ollama(多运维负担)。符合项目"单文件 SQLite + 零外部依赖"美学 |
| 2 | 业务线 | **knowledge(聊天消息)hybrid 优先** | souls 数量小,现有倒排索引够用;knowledge 数据持续增长,FTS5 字面漏召回痛点最大 |
| 3 | 索引粒度 | **按单条 message** | 跟 FTS5 表对齐(一行一消息);session 摘要会糊入无关内容,噪声大;写入路径改动最小 |
| 4 | 向量存储 | **SQLite-vec(`vec0` 虚表,同库)** | 跟 FTS5 同库同事务同备份;否决内存线性扫描(重启重灌/规模瓶颈)、独立向量库(运维破相) |
| 5 | 融合策略 | **RRF(k=60)** | BM25 负 rank 与 cosine [−1,1] 量纲不可比,硬归一化是 RAG 调参地狱;RRF 用 rank 不用 score,20 行 Rust 搞定 |
| 6 | 默认开关 | **`vector_search.enabled: false`** | 零回归保证;模型到位 + 手动开启才生效 |

---

## 3. 架构总览

```
                    ┌─────────────────────────────────────────┐
   GET /api/knowledge/search?q=...                  Hybrid Search
   POST /api/knowledge/rebuild        ┌──────────────────────────┐
                │                     │  ArchiveSystem           │
                ▼                     │  .search_knowledge()     │
   ┌────────────────────┐              │     │                    │
   │ routes/knowledge.rs│──── call ────┼─────┤                    │
   └────────────────────┘              │     ├─ FTS5 路径(现有)   │
                                       │     │   search_fts()      │
                                       │     │                     │
                                       │     ├─ 向量路径(新增)    │
                                       │     │   search_vector()   │
                                       │     │     │               │
                                       │     │     ▼               │
                                       │     │   EmbeddingEngine   │
                                       │     │   (ONNX bge-small)  │
                                       │     │     │               │
                                       │     │     ▼               │
                                       │     │   sqlite-vec KNN    │
                                       │     │   (vec0 虚表)        │
                                       │     │                     │
                                       │     └─► RRF Fuse (k=60)   │
                                       │           │               │
                                       │           ▼               │
                                       │       Vec<KnowledgeResult│
                                       └──────────────────────────┘
```

### 3.1 分层归属

严格遵循现有分层(foundation → archive → api):

- **`foundation`**(存储底层 + 推理 + 融合)
  - `SqliteDb` 扩展:vec0 表迁移 + KNN 查询
  - 新增 `EmbeddingEngine`:ONNX 推理封装
  - 新增 `HybridSearcher`:RRF 融合 + 降级控制
- **`archive`**(服务层):`ArchiveSystem::search_knowledge` 内部从"只调 FTS5"升级为"调 HybridSearcher"
- **`api`**(HTTP 层):`routes/knowledge.rs` 签名不变,返回结构不变 → **前端零改动**

### 3.2 核心约束

- 所有新代码进 `foundation`;api/archive 层只做转发,不感知 embedding/向量细节
- `Storage` trait 加一个新方法 `search_knowledge_hybrid`,旧 `search_knowledge` 保留(向后兼容)
- 单条 message → 一条 embedding,主键 `(session_id, seq)`,跟 `knowledge_fts` 对齐
- 向量相关失败一律降级,**FTS5 是底线**

---

## 4. 组件细节

三个新组件,全在 `foundation`,职责单一、可独立测试。

### 4.1 `EmbeddingEngine`(`foundation/src/embedding.rs`,新建)

**职责**:把文本变成 512 维向量。封装 ONNX 推理,对外暴露 `embed` / `embed_batch`。

```rust
pub struct EmbeddingEngine {
    session: ort::Session,            // ONNX runtime session
    tokenizer: tokenizers::Tokenizer, // HF tokenizer (bge-small-zh)
    dim: usize,                       // 512
}

impl EmbeddingEngine {
    pub fn load(model_dir: &Path) -> Result<Self>;   // 启动时加载一次
    pub fn embed(&self, text: &str) -> Result<Embedding>;
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>>;  // 重建索引用
    pub fn dim(&self) -> usize { 512 }
}
```

**关键决策**:

- 模型文件放 `data/models/bge-small-zh-v1.5/`(`model.onnx` + `tokenizer.json`);**不自动下载**,避免运行时网络依赖。首次启动检测缺失则打印 `scripts/download-embedding-model.sh` 指引
- **池化 + L2 归一化在 Rust 里做**(bge 系列要 mean-pool + normalize 才能正确 cosine),不依赖 ONNX 图内是否内置
- `embed` 失败返回 `Err`,由上层决定降级——engine 自己不吞错

### 4.2 `SqliteDb` 向量扩展(改 `foundation/src/sqlite.rs`)

**职责**:vec0 虚表 + KNN 查询,与 FTS5 平级。

**Schema 迁移**(加在 `migrate()`):

```sql
-- 启动时 load sqlite-vec 扩展,然后:
CREATE VIRTUAL TABLE IF NOT EXISTS message_embeddings USING vec0(
    session_id TEXT,
    seq        INTEGER,
    embedding  float[512]
);
```

**新方法**:

```rust
impl SqliteDb {
    fn ensure_vec0(conn: &Connection) -> Result<()>;   // load 扩展 + 建表(类比 ensure_fts5)

    pub fn index_embedding(
        &self, session_id: &str, seq: i64, embedding: &[f32],
    ) -> Result<()>;

    pub fn search_vector(
        &self, query_embedding: &[f32], limit: usize,
    ) -> Result<Vec<VectorHit>>;   // KNN: MATCH + ORDER BY distance

    pub fn rebuild_vector_index(
        &self, embeddings: &[(session_id, seq, Vec<f32>)],
    ) -> Result<usize>;

    pub fn fetch_knowledge_by_keys(
        &self, keys: &[(String, i64)],  // (session_id, seq)
    ) -> Result<Vec<KnowledgeResult>>;  // 批量回表,RRF 融合后补全字段
}

pub struct VectorHit {
    pub session_id: String,
    pub seq: i64,
    pub distance: f32,   // sqlite-vec 返回 L2 距离,RRF 只用 rank
}
```

**关键决策**:

- `sqlite-vec` 用 `rusqlite` 的 `load_extension` API 加载;扩展二进制(`.dylib`/`.so`/`.dll`)随项目分发到 `rust/foundation/extensions/`,不依赖系统装
- KNN 走 `vec0` 标准 `WHERE embedding MATCH ? ORDER BY distance LIMIT ?`,线性扫描(当前数据量级够用,未来可换 IVF)
- embedding 写入**不在 `append_message` 同步做**(避免阻塞消息写入),改成上层 spawn 异步任务

### 4.3 `HybridSearcher`(`foundation/src/hybrid_search.rs`,新建)

**职责**:FTS5 + 向量两路召回,RRF 融合。

```rust
pub struct HybridSearcher {
    db: Arc<SqliteDb>,
    embedding: Arc<EmbeddingEngine>,
}

impl HybridSearcher {
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeResult>> {
        // 1. 并行召回两路 top-K (K = min(limit * 3, 200),召回冗余且有上限)
        // 2. RRF 融合:score = 1/(60+rank_fts+1) + 1/(60+rank_vec+1)
        // 3. 去重(按 session_id+seq)+ 排序 + 取 top-limit
        // 4. 回 messages 表批量补全 → KnowledgeResult
    }
}
```

**RRF 融合实现**(~20 行,纯函数,易测):

```rust
fn rrf_fuse(fts: Vec<VectorHit>, vec_hits: Vec<VectorHit>, limit: usize) -> Vec<(Key, f32)> {
    let mut scores: HashMap<Key, f32> = HashMap::new();
    for (rank, h) in fts.iter().enumerate() {
        *scores.entry(h.key()).or_default() += 1.0 / (60 + rank as f32 + 1.0);
    }
    for (rank, h) in vec_hits.iter().enumerate() {
        *scores.entry(h.key()).or_default() += 1.0 / (60 + rank as f32 + 1.0);
    }
    let mut all: Vec<_> = scores.into_iter().collect();
    all.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    all.into_iter().take(limit).collect()
}
```

**降级逻辑全在这一层**:

- `embedding.embed(query)` 失败 → `warn!` + 向量路返回空,RRF 自然退化纯 FTS5
- `db.search_vector()` 失败 → 同上降级
- 两路都失败(FTS5 也挂)→ 返回 `Err`,HTTP 报 500

### 4.4 `Storage` trait 扩展(`foundation/src/storage.rs`)

加一个方法,不动旧的:

```rust
async fn search_knowledge_hybrid(&self, query: &str, limit: usize)
    -> Result<Vec<KnowledgeResult>>;
```

`api/src/store.rs` 的 impl 调 `HybridSearcher::search`;旧 `search_knowledge` 保留(行为等价,可内部委托 hybrid)。

---

## 5. 数据流

三条数据流:**写入**(实时)、**查询**(实时)、**重建**(离线)。

### 5.1 写入路径(实时,每条新消息)

**触发点**:`append_message`(FTS5 已在此同步写,向量在**同一事务外**异步挂)。

```
possession 写一条 message
        │
        ▼
SqliteDb::append_message(msg)
        │
        ├─[同步] INSERT messages + INSERT knowledge_fts   ← 现有,不动
        │
        └─[异步 spawn] 向量写入(新增)
                │
                ▼
        EmbeddingEngine::embed(msg.content)  (~10ms CPU)
                │
                ▼
        SqliteDb::index_embedding(session_id, seq, vec)
                │
                ▼
        INSERT INTO message_embeddings
                │
                ▼
        [失败?] warn! + 丢弃,不影响消息主流程
```

**关键决策**:

- **异步 spawn,不阻塞消息写入**——消息是用户对话关键路径,embedding 失败不能拖垮对话。失败只记日志,该条暂时无向量(FTS5 仍索引)
- **幂等写入**——vec0 允许重复 insert,写入前 `DELETE WHERE session_id=? AND seq=?` 再 INSERT,防重复 spawn 产生重复向量
- **空/极短消息跳过**——跟 FTS5 现有 `if !msg.content.is_empty()` 一致
- spawn 任务拿 `Arc<EmbeddingEngine>` + `Arc<SqliteDb>`,不持有 request 生命周期

### 5.2 查询路径(实时,HTTP `/api/knowledge/search`)

**调用链(签名零改动)**:

```
GET /api/knowledge/search?q=xxx&limit=20
        │
        ▼
routes/knowledge.rs::search()            ← 不改
        │
        ▼
ArchiveSystem::search_knowledge()        ← 内部改:调 hybrid 而非裸 FTS5
        │
        ▼
HybridSearcher::search(query, limit)     ← 新
        │
        ├─[并行 tokio::join!]─────────────┐
        │                                 │
        ▼                                 ▼
EmbeddingEngine::embed(query)      SqliteDb::search_fts(query, K=60)
        │                                 │   (现有 search_knowledge 逻辑,提取成内部方法)
        ▼                                 │
SqliteDb::search_vector(vec, K=60)        │
        │                                 │
        └─────────────┬───────────────────┘
                      ▼
              RRF 融合(k=60)
                      │
                      ▼
           去重 + top-20 keys
                      │
                      ▼
   SqliteDb::fetch_knowledge_by_keys()  ← 新:批量回表补 content/soul/mode
                      │
                      ▼
              Vec<KnowledgeResult>
                      │
                      ▼
        HTTP 返回(结构跟现在一模一样)
```

**关键决策**:

- **两路并行**(`tokio::join!`)——FTS5 是 IO,embedding 是 CPU,并行压延迟到 max(两路)而非 sum
- **K = min(limit × 3, 200)**(默认 60)——RRF 需召回冗余,但加上限防极端 `limit` 导致无效召回。融合后裁到 limit
- **降级发生在这一层**:embedding/vector 失败 → 向量路空,RRF 数学上等价纯 FTS5
- **回表**用 `SELECT ... FROM messages WHERE (session_id, seq) IN (...)` 批量,避免 N+1
- **高亮行为**:`content_snippet` 在 FTS5 路径带 `<b>` 高亮(FTS5 `snippet()` 产生),向量路回表的消息**不带高亮**(纯 content 截断到 200 字符,跟空 query 分支一致)。这是可接受的代价——hybrid 召回的语义命中本就没有明确的"命中词"可标。前端如需高亮可自行处理,本次不改

### 5.3 重建路径(离线,`POST /api/knowledge/rebuild`)

**触发**:手动调重建 API(与现有 `rebuild_fts` 并列),或首次启用向量时全量灌库。

```
POST /api/knowledge/rebuild
        │
        ▼
ArchiveSystem::rebuild_all()   ← 新方法,串行调 FTS5 + 向量
        │
        ├─[1] SqliteDb::rebuild_fts()             ← 现有
        │
        └─[2] SqliteDb::rebuild_vector_index()
                │
                ▼
        SELECT session_id, seq, content FROM messages WHERE content != ''
                │
                ▼ (流式分批,每批 256 条,控制内存)
        EmbeddingEngine::embed_batch(batch)
                │
                ▼
        事务批量 INSERT INTO message_embeddings
                │
                ▼
        返回总条数 { "indexed_fts": N, "indexed_vector": M }
```

**关键决策**:

- **分批 256 条**——ONNX batch 推理比单条快 5-10x,但全量一次性灌会把 1w 条 embedding 撑爆内存;256 是 CPU/内存平衡点
- **单事务批插**——每批一个 transaction,1w 条 = 40 个事务,比逐条快两个数量级
- **响应结构变**:`{"indexed": N}` → `{"indexed_fts": N, "indexed_vector": M}`。重建是 admin 操作,破兼容可接受
- **不并发**——重建是重 CPU 任务,与在线 embedding spawn 抢 CPU 会拖垮查询;串行最稳

### 5.4 配置开关(放 `config/default.yaml`)

```yaml
# 向量搜索(实验性,默认关,降级到纯 FTS5)
vector_search:
  enabled: false              # true 才加载 ONNX 模型 + 启用向量写入/查询
  model_dir: "./data/models/bge-small-zh-v1.5"
  rebuild_on_startup: false   # 首启/模型升级时全量灌库
```

**`enabled: false` 时**:

- `EmbeddingEngine` 不加载(省内存)
- `append_message` 不 spawn embedding
- `search_knowledge` 直接走旧 FTS5 路径
- **行为跟现在 100% 一致** → 老用户升级零风险

---

## 6. 错误处理

### 6.1 失败矩阵

按"**失败点 × 影响 × 行为**"显式列出,降级边界清晰:

| # | 失败点 | 影响范围 | 行为 | 日志 |
|---|--------|---------|------|------|
| 1 | 启动时 sqlite-vec 扩展 load 失败 | 向量功能整体不可用 | `vector_search.enabled` 强制降级 false,服务正常起 | `error!` + 启动 banner `[vector: DISABLED]` |
| 2 | 启动时 ONNX 模型文件缺失 | 向量功能整体不可用 | 同上;打印 download 脚本指引 | `error!` |
| 3 | `EmbeddingEngine::load` 推理初始化失败 | 同上 | 同上 | `error!` |
| 4 | 写入路径 `embed()` 失败 | 单条消息无向量 | 丢弃,FTS5 仍索引该条 | `warn!`(限频) |
| 5 | 写入路径 `index_embedding()` DB 失败 | 同上 | 同上 | `warn!` |
| 6 | 查询路径 `embed(query)` 失败 | 该次查询退化纯 FTS5 | 向量路返回空,RRF 自然降级 | `warn!` |
| 7 | 查询路径 `search_vector()` DB 失败 | 同上 | 同上 | `warn!` |
| 8 | 查询路径 `search_fts()` 失败 | 该次查询彻底失败 | 返回 `Err` → HTTP 500 | `error!` |
| 9 | 重建路径某批 embed 失败 | 该批跳过,继续下一批 | 计入 `failed` 计数,不中断 | `warn!` |
| 10 | `vector_search.enabled=false` | 全部向量路径不触发 | 等价当前行为 | 无(静默) |

**核心原则**:**FTS5 是底线,向量是增益**。任何向量相关失败都不让"搜索"本身不可用,最坏退化到今天的纯 FTS5。

**启动顺序约定**(澄清 #1/#2/#3 的交互):配置加载后,若 `vector_search.enabled=true`,启动流程**先尝试 load sqlite-vec 扩展 → 再尝试 `EmbeddingEngine::load`**,任一失败则把运行期 `vector_enabled` 标志位翻成 `false`(配置值不变,只是运行期降级),并打 `error!` 日志 + 启动 banner 标注 `[vector: DISABLED]`。之后所有写入/查询路径检查运行期标志位,与 `enabled=false` 行为一致。

**warn! 限频**:写入路径连续失败(模型彻底坏)用简单计数器——每 100 条只打一条日志,带"已抑制 N 条"字段,避免日志风暴。

### 6.2 `FoundationError` 扩展(`foundation/src/error.rs`)

```rust
pub enum FoundationError {
    // ... 现有

    #[error("sqlite-vec extension not available: {0}")]
    VectorExtensionUnavailable(String),

    #[error("embedding model not found at {path}")]
    EmbeddingModelNotFound { path: String },

    #[error("embedding inference failed: {0}")]
    EmbeddingInferenceFailed(String),

    #[error("vector dimension mismatch: expected {expected}, got {got}")]
    VectorDimensionMismatch { expected: usize, got: usize },
}
```

**不新增** `VectorSearchError`(现有枚举)会被废弃——它本来无生产用例,新代码统一用 `FoundationError`,错误类型单一。

---

## 7. 测试策略

分三层,每层都快、不依赖外部模型。

### 7.1 层 1:单元测试(纯函数,秒级)

放各模块 `#[cfg(test)]`,`cargo test` 直接跑。

- **`hybrid_search.rs::rrf_fuse`** — 纯函数测试
  - 两路命中同一 key → 分数叠加(>单路)
  - 一路空 → 退化成另一路排序
  - limit 截断正确
  - rank 从 0 开始,k=60 边界(`1/(60+0+1)` 精确值)
- **`sqlite.rs::cjk_tokenize`** — 现有,保留
- **`embedding.rs`** 池化/归一化数学函数(mean-pool、L2-norm)单独抽函数测,不跑 ONNX

### 7.2 层 2:集成测试(SQLite + vec0 + mock embedding,秒级)

`tests/hybrid_search_integration.rs` —— 用**真的 SQLite + vec0**(sqlite-vec 能 load),embedding 用 **mock**(`MockEmbeddingEngine` 返回确定性伪向量,如文本 hash → 512 维)。

测什么:

- 写入 5 条消息 → 向量表有 5 行
- 查询 → 返回 `Vec<KnowledgeResult>`,字段齐全
- embedding engine 失败(mock 注入 error)→ 返回结果退化为纯 FTS5(用 FTS5 必命中的关键词验证)
- `search_vector` 失败(mock DB 报错)→ 同上降级
- 重建 → `indexed_vector` 计数正确,重建前后查询结果一致

**为什么 mock embedding**:真 ONNX 模型 100MB,CI 跑不起;测试关心的是**融合逻辑 + 降级**,不是 embedding 质量(那是模型的事)。

### 7.3 层 3:真模型 smoke 测试(手动,可选)

`tests/smoke_real_model.rs`,用 `#[ignore]` 标注,本地有模型时 `cargo test -- --ignored` 跑。

- 加载真 bge-small-zh ONNX
- `embed("如何让 AI 记住我")` 与 `embed("长期记忆怎么实现")` 的 cosine > 0.6(语义相近)
- `embed("如何让 AI 记住我")` 与 `embed("今天天气真好")` 的 cosine < 0.3(语义无关)
- 端到端:写几条消息,hybrid 查询能语义召回(同义改述 query)

**不进 CI**,仅开发时验证模型没装错。

---

## 8. 验收标准(Definition of Done)

1. `cargo test` 全绿(层 1 + 层 2)
2. `cargo build --release` 通过
3. `vector_search.enabled=false` 时,所有现有测试 + 行为不变(零回归)
4. `vector_search.enabled=true` + 模型在位时:
   - 写消息 → 向量表新增一行
   - 搜索 → 返回 hybrid 结果
   - 故意删模型文件 → 搜索降级纯 FTS5,不报错
5. 重建 API 返回 `{indexed_fts, indexed_vector}`
6. smoke 测试手动跑过(语义召回验证)

---

## 9. 文件改动清单

| 文件 | 改动 |
|------|------|
| `rust/foundation/src/embedding.rs` | 新建 — ONNX 推理封装 |
| `rust/foundation/src/hybrid_search.rs` | 新建 — RRF 融合 + 降级 |
| `rust/foundation/src/sqlite.rs` | 加 `ensure_vec0` / `index_embedding` / `search_vector` / `rebuild_vector_index` / `fetch_knowledge_by_keys`;`search_knowledge` 拆出 `search_fts` 内部方法 |
| `rust/foundation/src/storage.rs` | trait 加 `search_knowledge_hybrid` |
| `rust/foundation/src/lib.rs` | 导出 `embedding` / `hybrid_search` 模块 |
| `rust/foundation/src/config.rs` | 加 `embedding_model_dir` / `vector_enabled` / `vector_rebuild_on_startup` 配置 |
| `rust/foundation/src/error.rs` | 加 4 个向量相关 error variant;废弃 `VectorSearchError` |
| `rust/foundation/Cargo.toml` | 加 `ort` / `tokenizers` / `rusqlite`(vec0 load_extension) |
| `rust/foundation/extensions/` | 新建目录,放 sqlite-vec `.dylib`/`.so`/`.dll` |
| `rust/api/src/store.rs` | impl `search_knowledge_hybrid`,注入 `HybridSearcher` |
| `rust/api/src/state.rs` | `AppState` 持有 `HybridSearcher`(条件构造) |
| `rust/api/src/main.rs` | 启动时按 `vector_enabled` 条件构造 engine + searcher |
| `rust/archive/src/lib.rs` | `search_knowledge` 内部委托 hybrid(若 searcher 存在) |
| `config/default.yaml` | 加 `vector_search` 配置块 |
| `scripts/download-embedding-model.sh` | 新建 — 拉 bge-small-zh ONNX 模型 |

---

## 10. 风险与边界

- **首次引入 `ort` + `tokenizers` 重依赖**,编译时间会增加(~30s),运行时按需加载,不启用 vector 时无开销
- **sqlite-vec 扩展跨平台分发**:`.dylib`(macOS)/`.so`(Linux)/`.dll`(Windows)随仓库走,放 `rust/foundation/extensions/`。CI 要能找到——这是最可能踩坑的地方,writing-plans 阶段单独列步骤验证
- **WAL 模式下 vec0 并发**:sqlite-vec 跟主库同文件同 WAL,写入并发度受 SQLite 单写者限制。写入路径已异步串行 spawn(共用 channel),不抢锁
- **embedding 模型升级**(如以后换 bge-m3)→ 维度 512→1024,需 drop+rebuild 向量表。`rebuild_vector_index` 已支持全量重建,但 schema 维度硬编码 512,升级需改代码 + 重建。此限制写进文档
- **ONNX 推理线程**:bge-small-zh 单条约 5-15ms CPU,写入路径异步 spawn 不阻塞;但重建路径全量灌库时会占满 CPU,应避免在高峰期触发(文档提示)
