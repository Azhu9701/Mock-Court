# NFR Requirements — B2: Soul Registry

## Performance

| 指标 | 目标 | 依据 |
|------|------|------|
| `list_souls()` 延迟 | < 1ms | 纯内存 HashMap 遍历（24条记录） |
| `get_soul(name)` 延迟 | < 1ms | 纯内存 HashMap 查寻 |
| `search_souls(query)` 延迟 | < 5ms | 倒排索引 + 相关度计算（24魂，每魂 ~10KB 文本） |
| `nearest_neighbor_search()` 延迟 | < 5ms | 遍历 24 魂的 4D 欧氏距离计算 |
| `reload()` 延迟 | < 100ms | 24 个文件读取 + 解析 + 索引重建 |
| 启动时 SoulRegistry 初始化 | < 100ms | 包含 reload() |

### 分析
- 24 个魂的全量索引 < 240KB，完全在 CPU L2 缓存内
- HashMap 查寻 O(1)，遍历 O(24) ≈ O(1)
- 倒排索引：每魂 ~50-100 token，索引 < 2400 条目

## 内存

| 指标 | 目标 |
|------|------|
| souls HashMap | < 240KB（24 × ~10KB SoulProfile） |
| inverted_index HashMap | < 50KB（~2400 token 条目） |
| Rust String 去重/碎片化 | < 100KB 额外开销 |
| **总内存** | **< 500KB** |

### 决策
全量预加载（Q3 答案 A）：24魂规模下，内存成本可忽略，预加载保证所有操作零 I/O。

## 可靠性

| 要求 | 描述 |
|------|------|
| 启动失败降级 | 若部分魂文件不可解析，记录 warning 跳过，不阻塞启动 |
| 索引恢复 | `reload()` 可从 FileStore 随时重建完整索引 |
| 并发安全 | `RwLock` 保护：多读并发，写操作互斥 |
| 数据一致性 | CRUD 遵循 dual-write：先写文件，成功后更新内存索引 |

## 可维护性

| 要求 | 描述 |
|------|------|
| 无外部依赖 | 倒排索引用 std::collections::HashMap，分词用简单算法 |
| 魂数量扩展 | HashMap 设计支持 O(1) 增删，不设硬上限 |
| 搜索排序可调 | 字段权重常量集中定义，便于调优 |
