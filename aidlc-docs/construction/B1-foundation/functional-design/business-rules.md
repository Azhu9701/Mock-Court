# Business Rules — B1: Foundation

## Data Integrity

### R1: Soul Profile 必填字段
`name`, `ismism_code`, `summon_prompt` 为必填。缺少任一字段 → 拒绝写入。
`ismism_code` 格式：`/\d-\d-\d-\d/`，不匹配 → 拒绝写入。

### R2: Soul Name 唯一性
`name` 为唯一标识。新建/炼化时检查同名魂是否存在。已存在 → 若为升级操作则覆盖，否则返回 Conflict 错误。

### R3: 存档文件命名
`存档路径格式`：`archive/{YYYY}/{MM}/{DD}/{session_id}/{soul_name|synthesis|verdict}.md`
日期派生自 session.created_at，禁止手动指定日期。

### R4: 落盘先于呈现
魂输出写入文件系统 → 写入 call-records → 才能通知前端可展示。
若任一写入失败 → 整个 session 标记为 inconsistent，不通知前端。

### R5: 写入原子性
`record_call` 同时写 SQLite + call-records.yaml。任一失败 → 回滚另一侧（两阶段提交）。

### R6: Session 状态转换
```
active ──► completed     (所有魂输出完成 + 存档完成)
active ──► inconsistent  (存档写入失败)
completed ──► archived   (用户手动归档)
```
不允许 completed → active 逆向转换。

### R7: Message 序列号
同一 session 内 message.seq 严格递增。并发写入时使用 SQLite 事务保证序号唯一。

## Validation

### V1: PossessionMode 枚举值
合法值：`single`, `conference`, `debate`, `relay`, `learn`, `practice_opening`。
非法值 → 拒绝创建 session。

### V2: Effectiveness 枚举值
合法值：`effective`, `partial`, `invalid`。
非法值 → 拒绝写入 call_records。

### V3: SoulGrade 枚举值
合法值：`S`, `A`, `B`, `C`, `D`。
非法值 → 拒绝写入 SoulProfile。

### V4: MessageRole 枚举值
合法值：`user`, `soul`, `synthesis`, `system`。
`soul` role 必须同时提供 `soul_name`，否则拒绝。

## Storage Rules

### S1: 分层数据访问
- 列表查询 → SQLite registry_cache 或 YAML registry.yaml（只读摘要字段）
- 魂详情 → 文件系统 souls/{name}.md（完整档案）
- 禁止在列表查询时加载完整 SoulProfile

### S2: 文件系统并发安全
读写 souls/ 目录时使用文件锁（`flock`），防止多请求同时写同一魂档案。

### S3: SQLite 连接池
单连接写、多连接读。写操作使用 WAL 模式，读操作不阻塞写。

## Import Rules (从现有万民幡导入)

### I1: 导入源路径
从环境变量或配置读取源路径：`WANMINFAN_SOURCE` → `souls/` + `registry.yaml` + `call-records.yaml`。

### I2: 格式兼容
- `souls/{name}.yaml` → 转换为 `souls/{name}.md`（YAML frontmatter + 提取 summon_prompt 为 Markdown body）
- `registry.yaml` → 直接复制 + 写入 SQLite 索引
- `call-records.yaml` → 解析后写入 SQLite call_records 表

### I3: 导入验证
导入后运行验证：魂文件数 = registry 条目数，call-records 条目数与源数据一致。

## Edge Cases

### E1: 空魂列表
Registry 无任何魂 → API 返回空列表（非错误），UI 显示引导创建第一个魂。

### E2: 存档目录不存在
写入存档前自动创建目录树（`create_dir_all`）。

### E3: SQLite 文件不存在
首次启动时自动创建 `wanminfan.db` 并运行所有 `CREATE TABLE IF NOT EXISTS`。

### E4: 大型魂档案
单个 soul YAML 超过 100KB → 记录警告日志，不拒绝加载。超过 1MB → 拒绝加载。

### E5: 日期分组边界
跨日存档（session 从 23:59 开始）→ 以 session.created_at 日期为准，同 session 内所有文件在同一日期目录下。
