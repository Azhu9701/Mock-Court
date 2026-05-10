# Business Logic Model — B2: Soul Registry

## SoulRegistry — 魂注册中心

`SoulRegistry` 是魂数据的运行时索引。它在 `FileStore` 之上构建内存索引，提供高性能搜索和过滤。

### 数据结构

```rust
pub struct SoulRegistry {
    /// 底层文件存储（只读引用）
    store: Arc<dyn Storage>,
    /// 内存索引：魂名 → SoulProfile
    souls: RwLock<HashMap<String, SoulProfile>>,
    /// 全文搜索索引：词 → 魂名列表
    inverted_index: RwLock<HashMap<String, Vec<String>>>,
}
```

### 初始化流程

```
SoulRegistry::new(store: Arc<dyn Storage>)
  1. reload()  — 从 FileStore 全量加载
     ├── store.list_soul_names()             → 获取所有魂文件列表
     ├── 对每个魂名: store.read_soul(name)   → 解析 SoulProfile
     ├── 构建 souls HashMap                  → 内存索引
     └── 构建 inverted_index HashMap         → 全文搜索倒排索引
```

### 倒排索引构建

```
build_inverted_index(profiles: &[SoulProfile]) -> HashMap<String, Vec<String>>
  对每个 profile:
    1. 将 name 分词 (按空格/CJK 字符) → 索引到该魂
    2. 将 field/ontology/epistemology/teleology 文本分词 → 索引到该魂
    3. 将 tags 逐个 → 索引到该魂
    4. 将 summon_prompt 分词 → 索引到该魂
    5. 将 ismism_code 整体作为 token → 索引到该魂
    6. 将 grade 整体作为 token → 索引到该魂
```

### 搜索算法

#### fulltext_search(query: &str) → Vec<SoulMatch>

```
1. 将 query 分词为 tokens
2. 对每个 token，查询 inverted_index 获取匹配的魂名集合
3. 取所有匹配魂名的并集
4. 对每个匹配魂计算相关度:
   relevance = Σ(token 在 profile 各字段中的命中数 × 字段权重)
   字段权重: name=5.0, tags=3.0, field/ontology/epistemology/teleology=2.0, prompt=1.0
5. 按 relevance 降序排序
6. 返回 Vec<SoulMatch>
```

#### nearest_neighbor_search(target: &IsmismCode) → Vec<SoulMatch>

```
1. 遍历所有 souls
2. 将每个魂的 ismism_code 解析为 IsmismCode
3. 计算与 target 的加权欧氏距离
4. 距离 → 相关度: relevance = 1.0 / (1.0 + distance)
   最大相关度 1.0 (距离为0), 随距离衰减
5. 按 relevance 降序排序
6. 返回 Vec<SoulMatch>
```

#### list_souls(filter: &IsmismFilter) → Vec<SoulListEntry>

```
1. 如果 filter.nearest 有值 → nearest_neighbor_search(nearest.target)
2. 否则:
   - 遍历 souls HashMap
   - 如果 filter.grade 有值 → 过滤品级
   - 返回所有匹配魂的 SoulListEntry
3. 按 summon_count 降序排序
```

### 主要方法

```
// 魂查询
list_souls(filter: &IsmismFilter) -> Vec<SoulListEntry>
get_soul(name: &str) -> Result<SoulProfile>
search_souls(query: &str) -> Vec<SoulMatch>

// Registry 管理
reload() -> Result<()>
get_ismism_distribution() -> IsmismStats

// 魂管理 (Q5: 完整 CRUD)
create_soul(profile: SoulProfile) -> Result<()>
update_soul(profile: SoulProfile) -> Result<()>
delete_soul(name: &str) -> Result<()>
```

## 冷启动优化

应用启动时，`SoulRegistry::new()` 一次性加载所有 24 个魂到内存。
24 个 SoulProfile（每个 ~2KB 不含 prompt 正文，~10KB 含）总共 ≤ 240KB，完全适合内存。
倒排索引估计 ~50KB（中文分词后的 token 集合）。

## 数据流

```
HTTP Request (/api/souls)
  → API Layer (B6)
    → SoulRegistry.list_souls(filter)
      → 读内存 HashMap → 过滤 → Vec<SoulListEntry>
    → 序列化 JSON
  → HTTP Response

WebSocket (合议模式选魂)
  → Possession Engine (B4)
    → SoulRegistry.search_souls("哲学")
      → 倒排索引查询 → 相关度排序 → Vec<SoulMatch>
    → 返回匹配魂列表
```
