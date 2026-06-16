# 演进路线图

## Phase 1 · 当前阶段

**目标**: 完成 `meta/` 文档体系 + 模板文件 + CLI `init` 命令骨架。

**产出**:
- [ ] `meta/docs/` 完整文档体系（架构 + 指南 + 参考）
- [ ] `meta/templates/` 模板文件（domain.yaml 骨架、Agent 定义骨架、3 个领域示例）
- [ ] `rust/cli` 扩展 `snake init` 子命令（目录生成 + 模板渲染）
- [ ] 去领域化改造清单中的代码重构（prompt.rs、modes/*.rs、triage.rs 等）
- [ ] 前端组件 `agent-*` 别名导出

## Phase 2 · Crate 独立发布

**目标**: 框架核心模块作为独立 crate 发布到 crates.io。

**计划 crate**:
- `agent-gateway` — AI 提供商统一网关
- `agent-engine` — 多 Agent 调度引擎
- `agent-tools` — 工具注册与调用
- `agent-memory` — 记忆图谱
- `agent-foundation` — 共享类型与存储抽象

## Phase 3 · 依赖远程化

**目标**: `snake init` 生成的 Cargo.toml 从 crates.io 拉取依赖，而非本地 path。

**工作**:
- 确保各 crate 的 semver 兼容策略
- `snake init` 生成项目时写入正确的版本号
- 发布 crates.io 后的兼容性测试

## Phase 4 · 生态建设

**目标**: 插件市场，社区可发布和复用 Agent/领域/模式。

**工作**:
- Agent 定义注册表（类似 npm registry）
- 领域配置市场
- 自定义模式插件接口
- `snake install <agent-package>` 命令
