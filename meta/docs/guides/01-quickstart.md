# 快速上手：5 分钟跑起来

## 前置条件

- Rust 工具链 1.75+
- Node.js 18+ 和 pnpm
- 至少一个 AI 提供商的 API Key

## Step 1：生成项目

```bash
snake init my-project --domain custom
cd my-project
```

## Step 2：配置 API Key

```bash
cp .env.example .env
```

编辑 `.env`，至少填一个：

```env
OPENAI_API_KEY=sk-xxxxx
CLAUDE_API_KEY=sk-ant-xxxxx
DEEPSEEK_API_KEY=sk-xxxxx
LMSTUDIO_URL=http://localhost:1234/v1    # 可选：本地模型
```

## Step 3：创建第一个 Agent

```bash
mkdir -p data/agents
```

创建 `data/agents/assistant.md`：

```yaml
---
name: "通用助手"
model: "deepseek-chat"
tools: []
trigger_keywords: ["帮助", "问题", "怎么"]
system_prompt: |
  你是一个通用助手，请简洁清晰地回答用户问题。
---
```

## Step 4：启动服务

```bash
# 终端 1：启动 API（端口 3001）
cargo run -p api

# 终端 2：启动前端（端口 3000）
cd nextjs && pnpm dev
```

## Step 5：验证

浏览器打开 `http://localhost:3000`：

1. 进入侧边栏 → 看到你的 Agent "通用助手"
2. 点击「开始附体」→ 输入问题 → 选择 Agent → 开始
3. 观察流式输出和最终回复

## 下一步

- [定义自己的领域](./02-domain-config.md)
- [创建更多 Agent](./03-create-agent.md)
- [添加自定义工具](./06-add-tool.md)
