# Logical Components — B2: Soul Registry

## Component Architecture

```
SoulRegistry (src/lib.rs)
├── IndexManager (内部模块)
│   ├── souls: RwLock<HashMap<String, SoulProfile>>
│   ├── inverted_index: RwLock<HashMap<String, Vec<String>>>
│   └── rebuild() / index_one() / deindex_one()
├── SearchEngine (src/search.rs)
│   ├── fulltext_search(query)    → Vec<SoulMatch>
│   ├── nearest_search(target)    → Vec<SoulMatch>
│   └── relevance_score()         → f64
├── Tokenizer (内部函数, src/search.rs)
│   └── tokenize(text)            → Vec<String>
└── IsmismUtils (src/ismism.rs)
    ├── ismism_distance(a, b)     → f64
    └── parse_ismism(code)        → Result<IsmismCode>
```

## Component: SoulRegistry (`src/lib.rs`)

**职责**: 魂注册中心入口，封装加载、查询、管理操作。

```
struct SoulRegistry {
    store: Arc<dyn Storage>,
    souls: RwLock<HashMap<String, SoulProfile>>,
    inverted_index: RwLock<HashMap<String, Vec<String>>>,
}

impl SoulRegistry {
    // 生命周期
    new(store: Arc<dyn Storage>) -> Result<Self>
    reload() -> Result<()>

    // 查询
    list_souls(filter: &IsmismFilter) -> Vec<SoulListEntry>
    get_soul(name: &str) -> Result<SoulProfile>
    search_souls(query: &str) -> Vec<SoulMatch>
    get_ismism_distribution() -> IsmismStats

    // 管理 (Q5: 完整 CRUD)
    create_soul(profile: SoulProfile) -> Result<()>
    update_soul(profile: SoulProfile) -> Result<()>
    delete_soul(name: &str) -> Result<()>
}
```

## Component: SearchEngine (`src/search.rs`)

**职责**: 全文搜索和最近邻搜索逻辑。

```
// 倒排索引查询 + 相关度排序
fn fulltext_search(
    query: &str,
    souls: &HashMap<String, SoulProfile>,
    inverted_index: &HashMap<String, Vec<String>>,
) -> Vec<SoulMatch>

// 4D 欧氏距离搜索
fn nearest_search(
    target: &IsmismCode,
    souls: &HashMap<String, SoulProfile>,
) -> Vec<SoulMatch>
```

**相关度算法**:
```
relevance(name, query) = Σ (token_field_weight × hit_count)

字段权重:
  name:        5.0
  tags:        3.0
  field/ontology/epistemology/teleology: 2.0
  summon_prompt: 1.0
  ismism_code: 3.0
  grade:       2.0

最终排序: relevance 降序 → summon_count 降序
```

## Component: Tokenizer (`src/search.rs`, 私有函数)

**职责**: 中文文本分词。

```
fn tokenize(text: &str) -> Vec<String> {
    // 1. 转小写
    // 2. 分离 CJK 字符和 ASCII 单词
    // 3. CJK: 收集单字 + 相邻双字组合
    // 4. ASCII: 按空白分词
    // 5. 去重
}

fn is_cjk(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c)   // CJK Unified
        || ('\u{3400}'..='\u{4DBF}').contains(&c)  // CJK Ext-A
}
```

## Component: IsmismUtils (`src/ismism.rs`)

**职责**: ismism 坐标解析和距离计算。

```
// 解析 "4-1-4-3" → IsmismCode { field:4, ontology:1, epistemology:4, teleology:3 }
fn parse_ismism(s: &str) -> Result<IsmismCode, String>

// 加权欧氏距离
fn distance(a: &IsmismCode, b: &IsmismCode, weights: Option<(f64,f64,f64,f64)>) -> f64
```

**距离归一化**: `relevance = 1.0 / (1.0 + distance)`
- 距离为 0 → relevance = 1.0（完全匹配）
- 最大可能距离 ≈ 6.0（四个维度各差 3）→ relevance ≈ 0.14

## File Structure

```
rust/registry/
├── Cargo.toml
└── src/
    ├── lib.rs          # SoulRegistry struct + 公共 API
    ├── search.rs       # SearchEngine + Tokenizer
    └── ismism.rs       # IsmismUtils (parse, distance, distribution)
```
