# NFR Requirements — B1: Foundation

## Performance

### P1: SQLite 读写性能
- Session list 查询 < 10ms（1000 sessions 规模）
- Message append < 5ms
- Soul profile read（~20KB file）< 20ms

### P2: 文件系统操作
- Soul 档案写入 < 50ms
- 存档目录自动创建 < 5ms
- Registry YAML 解析（24 souls）< 30ms

### P3: 并发能力
- SQLite WAL 模式，读写不互斥
- 多 session 并发写入互不阻塞
- 文件写入使用 fsync 保证持久化

## Reliability

### R1: 数据持久化
- 所有写入必须 fsync 后才返回成功
- SQLite 使用 `PRAGMA synchronous = FULL`
- 存档文件写入使用原子写入（先写 tmp 再 rename）

### R2: 错误恢复
- 数据库损坏 → 从 YAML 重新构建 SQLite
- 魂档案损坏 → 跳过该魂，记录错误日志，不影响其他魂加载
- registry.yaml 损坏 → 启动失败，提示修复

### R3: 数据完整性
- call_records 双写（SQLite + YAML），定期交叉校验
- 存档文件完整性校验（文件数、内容非空检查）

## Availability

### A1: 本地应用
- 单进程运行，无分布式协调
- 启动时间 < 2 秒
- 优雅关闭（完成进行中的写入后退出）

### A2: 监控
- 通过日志输出关键指标（启动时间、查询延迟、错误率）
- SQLite 定期 VACUUM（每周一次）

## Maintainability

### M1: 代码结构
- Storage trait 是唯一数据访问点
- 所有 SQL 集中在 foundation crate
- 无硬编码路径 — 全部来自 Config

### M2: 数据可移植
- Export: 导出 YAML registry + souls/ 目录 + archive/ 目录
- Import: 导入现有万民幡 souls/ 和 registry.yaml
- 数据库可随时从 YAML 重建

## Constraints

- SQLite 单文件大小限制（默认 2TB — 本地使用足够）
- 文件系统路径最大 4096 bytes（macOS/Linux 兼容）
- YAML 文件最大 1MB（单魂档案上限）
- 无外部服务依赖（纯本地运行）
