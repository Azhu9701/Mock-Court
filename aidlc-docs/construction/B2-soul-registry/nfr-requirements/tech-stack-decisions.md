# Tech Stack Decisions — B2: Soul Registry

## Decisions

### 1. 倒排索引：简单 HashMap

**决策**: 使用 `HashMap<String, Vec<String>>` 实现倒排索引（Q1 答案 A）。

**理由**:
- 24魂规模下，HashMap 倒排索引完全够用
- 零外部依赖，编译快，二进制小
- 查询 O(k)（k=tokens 数），远低于 Tantivy 的初始化和索引开销
- Tantivy 引入 ~10MB 二进制增量，对 24 魂来说是过度工程
- SQLite FTS5 需要在 SQLite 中维护额外表，背离 B2 纯内存定位

### 2. 中文分词：单字 + 双字组合

**决策**: 使用简单单字分词 + 双字 bigram 组合（Q2 答案 A）。

**算法**:
```
fn tokenize(text: &str) -> Vec<String>:
    1. 去掉标点符号和空白
    2. 对每个 CJK 字符：单字作为 token
    3. 对每对相邻 CJK 字符：双字组合作为 token
    4. 对 ASCII 单词：直接作为 token（已由空白分隔）
    5. 对所有 token 转小写并去重
```

**理由**:
- 中文搜索"马克思" → tokens: ["马","克","思","马克","克思","马克思","克思","思"] — 任何合理的子串查询都能命中
- jieba-rs 引入 ~20MB 字典文件，严重超出 B2 轻量定位
- 24魂的小规模语料下，单字+双字组合足以覆盖搜索需求
- 相关度排序（BR3.3）通过字段权重补偿分词精度损失

### 3. 内存策略：全量预加载

**决策**: 启动时一次性加载所有魂到内存（Q3 答案 A）。

**理由**:
- 24魂总内存 < 500KB
- 加载时间 < 100ms
- 所有操作零磁盘 I/O
- 热加载/lazy 的复杂度在此规模下无收益

## Dependencies

B2 `registry` crate 依赖：

```toml
[dependencies]
foundation = { path = "../foundation" }  # Storage trait, models, error types
tokio = { workspace = true }             # async runtime (reload spawn_blocking)
serde = { workspace = true }             # Serialize
serde_json = { workspace = true }        # JSON serialize (API responses)
tracing = { workspace = true }           # Logging
```

**无新增外部依赖**。所有搜索和索引逻辑用 `std::collections` 实现。
