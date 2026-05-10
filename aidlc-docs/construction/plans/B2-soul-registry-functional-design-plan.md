# Functional Design Plan — B2: Soul Registry

## Plan Checklist

- [x] Generate `domain-entities.md` — SoulSummary, SoulMatch, IsmismStats 等注册中心专用类型
- [x] Generate `business-logic-model.md` — SoulRegistry CRUD, 搜索算法, ismism 解析
- [x] Generate `business-rules.md` — 魂唯一性, ismism 校验, 搜索排序规则
- [x] B2 does NOT include frontend → skip frontend-components.md

## Unit Context

B2 Soul Registry 是魂的注册中心，依赖于 `foundation` crate 的 `FileStore`/`Storage`。它提供：
- 魂的加载、列表、详情查询
- ismism 四维坐标解析与过滤搜索
- 全文搜索（关键词、编码、品级）
- Registry 管理和 ismism 分布统计

## Design Questions

### Question 1: ismism 坐标过滤语义
当用户按 ismism 坐标过滤魂列表时（如"找 field≥3 且 ontology≥2 的魂"），过滤语义应该是什么？

A) 精确匹配 — 每个维度必须等于指定值
B) 阈值过滤 — 每个维度 >= 指定值（适合查找"在某维度上足够强"的魂）
C) 组合条件 — 支持 >=、<=、=、!= 的混合条件
D) 最近邻搜索 — 给定目标坐标，按 4D 欧氏距离排序返回最近的魂
E) Other (please describe after [Answer]: tag below)

[Answer]: D

### Question 2: 全文搜索范围
SoulRegistry 的全文搜索（`search_souls(query)`）应该搜索魂档案的哪些内容？

A) 仅搜索魂名 — 精确/模糊匹配魂名
B) 魂名 + 标签 — 搜索 name + tags
C) 魂名 + 领域描述 — 搜索 name + field/ontology/epistemology/teleology 文本
D) 全量搜索 — 搜索 name + 所有元数据 + summon_prompt 全文
E) Other (please describe after [Answer]: tag below)

[Answer]: D

### Question 3: SoulSummary 字段
列表视图（`list_souls`）返回的 SoulSummary 应包含哪些字段？

A) 最小信息 — name, ismism_code, grade
B) 标准信息 — name, ismism_code, grade, field, tags, summon_count
C) 完整统计 — name, ismism_code, grade, field, tags, summon_count, effectiveness, updated_at
D) Other (please describe after [Answer]: tag below)

[Answer]: B

### Question 4: ismism 坐标编码解析
ismism 编码（如 "4-1-4-3"）如何存储和解析？`IsmismCode` 结构应该如何建模？

A) 字符串存储 + 按需解析 — 存为 "4-1-4-3"，解析时 split("-") 得到 [f, o, e, t]
B) 结构化存储 — 四元组 struct IsmismCode { field: u8, ontology: u8, epistemology: u8, teleology: u8 }
C) 双模式 — 既存字符串也存结构化字段，搜索用结构化，展示用字符串
D) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 5: SoulRegistry 的魂管理边界
SoulRegistry 应该暴露哪些魂管理操作？收魂/炼化/审查是否需要通过 SoulRegistry？

A) 只读 + 重新加载 — list, get, search, reload（魂文件由 FileStore 直接管理）
B) 只读 + 轻量管理 — 上述 + 启用/禁用魂 + 更新 registry 元数据
C) 完整 CRUD — 上述 + create/update/delete 魂（封装 FileStore 的全部写入操作）
D) Other (please describe after [Answer]: tag below)

[Answer]: C
