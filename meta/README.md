# Snake Skin Framework — Meta

这里是 Snake Skin 多 Agent 并行推理框架的工程手册。用它快速搭建多 AI Agent 协同处理用户任务的前后端 Web 应用。

## 这是什么

一套 Rust (Axum) + Next.js 的前后端组合，核心能力：

- **多 Agent 并行调度**：同时唤出多个 AI，各自独立推理，实时流式输出
- **碰撞检测与综合**：自动检测不同 Agent 输出的矛盾/互补/盲区，生成辩证综合
- **领域可切换**：换一套 `domain.yaml` 就能从法律顾问切到医疗诊断、金融分析、客服系统
- **Agent 自动创建**：输入自然语言描述，自动生成 Agent 配置（Collect）；基于反馈迭代优化（Refine）
- **记忆图谱**：每个 Agent 拥有独立记忆拓扑，支持跨轮引用和上下文关联

## 怎么开始

```bash
# 1. 用脚手架生成新项目
snake init my-project --domain custom

# 2. 填入 API Key
cd my-project && cp .env.example .env
# 编辑 .env，填入至少一个 AI 提供商的 Key

# 3. 启动
cargo run -p api &
cd nextjs && pnpm dev

# 4. 打开浏览器
open http://localhost:3000
```

## 目录

| 路径 | 用途 |
|------|------|
| [OVERVIEW.md](./OVERVIEW.md) | 框架全景：能做什么、适用场景、核心概念 |
| [docs/architecture/](./docs/architecture/) | 系统架构：数据流、分层模型、接口契约 |
| [docs/guides/](./docs/guides/) | 开发指南：快速上手、自定义领域、添加工具/模型 |
| [docs/reference/](./docs/reference/) | 参考手册：API 路由、WS 事件、配置项、类型索引 |
| [templates/](./templates/) | 脚手架模板：domain.yaml 骨架、Agent 定义示例 |
| [roadmap.md](./roadmap.md) | 演进路线图 |
