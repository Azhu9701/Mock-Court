# Business Rules — B2: Soul Registry

## 1. 魂唯一性约束

| 规则 | 描述 | 触发时机 |
|------|------|----------|
| BR1.1 | 魂名全局唯一，创建时检查 `souls` HashMap 中是否已存在同名魂 | create_soul() |
| BR1.2 | 魂名不能为空字符串 | create_soul() / update_soul() |
| BR1.3 | 更新魂时，如果改名，须检查新名是否已被占用。如果改名导致旧文件名与现有魂冲突，先删除旧文件再写新文件 | update_soul() |
| BR1.4 | 删除魂时，同时清理：`souls/{name}.md` 文件 + registry.yaml 中对应条目 + 内存索引 | delete_soul() |

## 2. ismism 编码校验

| 规则 | 描述 | 触发时机 |
|------|------|----------|
| BR2.1 | ismism_code 必须是 "f-o-e-t" 格式，4 个数字用 "-" 分隔，每个值必须在 [1,4] 范围内 | 所有接受 IsmismCode 的操作 |
| BR2.2 | 无效 ismism_code 的魂仍可加载，但在 ismism 搜索/过滤中被跳过（视为无坐标） | 搜索/过滤 |

## 3. 搜索规则

| 规则 | 描述 |
|------|------|
| BR3.1 | 全文搜索不区分大小写，中文按单字 + 常用词组分词 |
| BR3.2 | 空查询字符串返回空结果（不返回全部魂），最小查询长度 1 字符 |
| BR3.3 | 搜索结果默认按相关度降序，相关度相同则按 summon_count 降序 |
| BR3.4 | 最近邻搜索：魂没有有效 ismism_code 时，排在结果末尾（相关度赋 0） |
| BR3.5 | 最近邻搜索默认不限制返回数量。如果 filter 中有 grade 过滤，在距离排序前先过滤品级 |
| BR3.6 | ismism 距离计算使用等权（各维度权重 1.0），后续可通过 IsmismSearch.weights 自定义权重 |

## 4. 数据一致性

| 规则 | 描述 |
|------|------|
| BR4.1 | CRUD 操作遵循 dual-write 原则：先写文件（FileStore），成功后更新内存索引 |
| BR4.2 | 如果文件写入成功但内存索引更新失败，下次 reload() 可恢复一致性 |
| BR4.3 | `reload()` 从 FileStore 全量重载，丢弃当前内存索引后重建 |
| BR4.4 | `get_ismism_distribution()` 直接从内存 souls HashMap 计算，无需访问磁盘 |

## 5. 错误处理

| 场景 | 错误类型 | 处理 |
|------|----------|------|
| 查找不存在的魂 | `FoundationError::SoulNotFound(name)` | 返回 404 |
| 创建已存在的魂 | `FoundationError::DuplicateSoul(name)` | 返回 409 |
| ismism_code 解析失败 | 不报错，标记为无坐标 | 搜索时跳过 |
| 魂文件损坏（解析失败） | 记录 warning 日志 | 跳过该文件，继续加载其他魂 |
| 文件系统错误 | `FoundationError::Io(err)` | 返回 500 |
