# NFR Design Patterns — B2: Soul Registry

## Pattern 1: In-Memory Index (全量预加载)

**问题**: 如何保证所有魂查询操作零 I/O、< 5ms 延迟？

**方案**: `SoulRegistry` 启动时全量加载，维护内存 HashMap 索引。

```
SoulRegistry::new(store)
  ├── store.list_soul_names()          ──> 文件名列表
  ├── for each name: store.read_soul() ──> SoulProfile
  ├── souls: HashMap<name, SoulProfile>──> 主索引
  └── inverted_index: HashMap<token, Vec<name>> ──> 搜索索引
```

**约束**:
- 所有读操作只访问 HashMap，零 I/O
- 写操作（CRUD）先写 FileStore，成功后更新 HashMap
- `RwLock<souls>` + `RwLock<inverted_index>` 分别加锁，减少竞争

## Pattern 2: Simple Tokenization (单字+双字分词)

**问题**: 中文全文搜索无外部依赖时如何保证召回率？

**方案**: 单字 + 双字 bigram 组合分词。

```
tokenize("马克思") → ["马", "克", "思", "马克", "克思"]
tokenize("实践开口 P1") → ["实", "践", "开", "口", "实践", "践开", "开口", "p1"]
```

**规则**:
- 去掉标点，保留字母/数字
- CJK 字符：单字 + 相邻双字组合
- ASCII：按空白分词，转小写
- 所有 token 去重

**精度分析**: 24魂语料 ~240KB，每魂 ~10KB。双字组合确保"马克思"搜索命中"马克思"、"马克"、"克思"。单字作为回退保证原子匹配。

## Pattern 3: RwLock Concurrent Access

**问题**: 多线程同时读魂列表 + 偶尔写操作的安全并发？

**方案**: `RwLock` 保护两个 HashMap。

```rust
pub struct SoulRegistry {
    store: Arc<dyn Storage>,
    souls: RwLock<HashMap<String, SoulProfile>>,
    inverted_index: RwLock<HashMap<String, Vec<String>>>,
}
```

**规则**:
- `list_souls()`, `get_soul()`, `search_souls()` → `souls.read()` + `inverted_index.read()`
- `create_soul()`, `update_soul()`, `delete_soul()` → 先 FileStore 写 → `souls.write()` + `inverted_index.write()`
- `reload()` → `souls.write()` + `inverted_index.write()`（全量替换）
- 读锁之间不互斥，多读并发
- 写锁与读锁互斥，保证 CRUD 原子性

## Pattern 4: Dual-Write Consistency (继承自 B1)

**问题**: 魂文件写入后内存索引更新失败如何恢复？

**方案**: 继承 B1 的 dual-write 模式。

```
create_soul(profile):
  1. store.write_soul(&profile)      // 文件写入
  2. souls.write().insert(name, profile)  // 内存索引
  3. update_inverted_index(profile)
```

**恢复**: 若步骤 2/3 失败，`reload()` 从文件重建索引。

## Pattern 5: Graceful Degradation (启动降级)

**问题**: 部分魂文件损坏时如何保证服务可用性？

**方案**: 加载时跳过无法解析的魂文件。

```
reload():
  for name in store.list_soul_names():
    match store.read_soul(&name):
      Ok(profile) → 加入索引
      Err(e) → tracing::warn!("Skipping soul {}: {}", name, e), 继续
```

**约束**: 不因单个魂文件损坏而阻塞整个服务启动。
