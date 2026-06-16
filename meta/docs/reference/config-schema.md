# 配置项与默认值

## default.yaml

```yaml
# 数据目录
data_dir: "./data"
souls_dir: "./data/souls"
agents_dir: "./data/agents"          # Agent 定义文件目录
archive_dir: "./data/archive"
db_path: "./data/app.db"

# 其他路径
registry_path: "./data/registry.yaml"
call_records_path: "./data/call-records.yaml"

# 服务器
server_host: "127.0.0.1"
server_port: 3001
nextjs_port: 3000

# 搜索引擎
searxng_url: "http://127.0.0.1:8080"
search_engine: "bing"                # bing | duckduckgo | searxng

# 限流
rate_limit:
  enabled: true
  requests_per_second: 30
  burst_size: 60

# CORS
cors_origins:
  - "http://localhost:3000"

# Agent 热加载
watch_agents: false                  # 监控 data/agents/ 变化
watch_interval: 5                    # 扫描间隔（秒）

# 缓存
cache:
  enabled: true
  ttl_seconds: 3600

# 连接池
connection_pool:
  max_connections: 10
  idle_timeout_seconds: 30
```

## domain.yaml

```yaml
domain:
  name: "custom"
  icon: "🤖"
  system_name: "智囊团"
  agent_noun: "专家"
  user_title: "用户"
  synthesis_verb: "综合分析"

dimensions:
  - id: "category"
    label: "领域"
    description: "专业领域分类"
    values: ["技术", "商业", "设计", "运营"]
    weight: 1.0

synthesis:
  template: |
    你是一位系统协调人。
    ...（Handlebars 模板）...

collect_intro: |
  我将帮助你创建一位新的专家。
  ...

trigger_markers:
  single: ["简单", "快速", "一句话"]
  conference: ["分析", "综合", "多角度"]
  debate: ["辩论", "对立", "正方"]
  relay: ["步骤", "流程", "阶段"]
  learn: ["学习", "教我", "解释"]
  practice: ["实践", "操作", "执行"]

memory:
  default_share: "session"           # none | session | all
  max_turns: 20
  persist: true
```

## .env

```env
# AI 提供商
OPENAI_API_KEY=sk-xxxxx
CLAUDE_API_KEY=sk-ant-xxxxx
DEEPSEEK_API_KEY=sk-xxxxx

# 本地模型
LMSTUDIO_URL=http://localhost:1234/v1
LMSTUDIO_MODEL=qwen/qwen3.6-27b

# 中转站
AI_RELAY_URL=https://your-relay.example.com
AGENT_PROXY_KEY=your-key

# 可选：Web 搜索
SEARXNG_URL=http://127.0.0.1:8080

# 可选：认证
API_TOKEN=your-secret-token
NEXT_PUBLIC_API_TOKEN=your-secret-token
```

## Agent 定义文件 (Markdown frontmatter)

```yaml
---
name: "agent-name"                   # 必填：唯一标识
title: "显示标题"                     # 可选
description: "简短描述"               # 可选
model: "deepseek-chat"               # 可选：推荐模型
tools:                               # 可选：可用工具
  - "tool_name"
trigger_keywords:                    # 推荐：触发关键词
  - "关键词"
dimensions:                          # 推荐：坐标维度
  category: "技术"
system_prompt: |                     # 推荐：系统 Prompt
  ...
compat:                              # 可选：兼容 Agent
  - "other-agent"
incompat:                            # 可选：不兼容 Agent
  - "another-agent"
domains:                             # 可选：擅长领域
  - "领域标签"
voice: "语气描述"                    # 可选
mind: "思维模式"                      # 可选
---
```
