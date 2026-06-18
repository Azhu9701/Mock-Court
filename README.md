# Snake Skin

> 仓库名：[soul-banner-lite](https://github.com/Azhu9701/soul-banner-lite)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)](https://www.rust-lang.org)
[![Next.js](https://img.shields.io/badge/Next.js-16-black?logo=next.js)](https://nextjs.org)
[![Docker](https://img.shields.io/badge/Docker-ready-blue?logo=docker)](https://www.docker.com)

**Snake Skin 是一个 AI 模拟仲裁庭系统**——围绕劳动争议等法律场景，同时传唤多个具有独立法律立场的 AI 角色（仲裁法官、原告律师、被告律师、专家证人、劳动者之声），围绕同一案件展开并行庭审、质证交锋与裁决说理。

它不是法律咨询机器人。它的价值在于：**让不同法律立场的 AI 在对抗中暴露各自论证的盲区，让使用者看清自己被夹在什么力量之间。**

---

## 核心概念

| 对比 | 传统法律咨询 | Snake Skin 模拟仲裁庭 |
|------|------------|----------------------|
| 交互模式 | 一问一答 | 多角色并行庭审，实时碰撞 |
| 法律立场 | 单一视角 | 法官、原告、被告、专家、当事人五方同时陈述 |
| 论证方式 | 法条罗列 | 举证质证、争议焦点归纳、裁决说理 |
| 盲区发现 | 无 | 各方论证盲区在对抗中被暴露 |
| 入口 | 输入框 | 案件描述、庭审记录、角色传唤、@mention 都是入口 |

## 基本架构

```
                              ┌──────────────────┐
                              │   审查官（幡主）    │
                              │  庭前质证 → 整合案件 → 开庭 │
                              └────────┬─────────┘
                                       │ refined_case
┌─────────────┐    ┌──────────────┐    │    ┌───────────────┐
│  Next.js 前端│───▶│  Axum API    │───▶│  AI Gateway   │
│  (shadcn/ui) │    │  (端口 3096)  │    │ Claude/GPT/DS │
└─────────────┘    └──────┬───────┘    └───────────────┘
                          │
          ┌───────────────┼───────────────────────┐
          ▼               ▼               ▼               ▼
    ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
    │ Registry │   │Possession│   │   CLI    │   │Foundation│
    │  角色注册表 │   │  庭审引擎 │   │  命令行   │   │  基础设施 │
    └──────────┘   └──────────┘   └──────────┘   └──────────┘
```

## 庭审角色

仓库默认包含 **5 个劳动争议庭审角色**，开箱即用：

| 角色 | 法律立场 | 核心能力 |
|------|---------|---------|
| **仲裁法官** | 居中裁判 | 归纳争议焦点、分配举证责任、检索法条、独立核算、裁决说理 |
| **原告律师** | 代理劳动者 | 权利主张、举证质证、诉求计算、证据清单生成 |
| **被告律师** | 代理用人单位 | 抗辩免责、合规审查、证据质证、经营自主权辩护 |
| **专家证人** | 行业专家 | 专业意见、行业惯例评估、客观中立、利益平衡分析 |
| **劳动者之声** | 当事人陈述 | 第一人称事实还原、真实处境、朴素诉求 |

每个角色有独立的**主义主义四维坐标**（法庭角色/法律立场/论证方法/价值取向）、专属工具权限（如 `search_labor_law`、`calculate_severance`、`generate_evidence_checklist`）和自我声明边界。

> **提示**：目前聚焦**劳动争议仲裁**场景。角色配置和知识库位于 `data/souls/` 和 `data/knowledge/labor-law/`，可扩展至其他法律领域。

## 技术栈

**后端：Rust**
- Web 框架：Axum 0.7 + WebSocket + SSE
- 数据库：SQLite（WAL 模式）+ FTS5 全文搜索
- AI 网关：Claude / OpenAI / DeepSeek / LM Studio 多提供商 + 模型智能路由
- 图数据库：petgraph（法律关系图谱）
- 异步运行时：Tokio

**前端：Next.js 16**
- UI 框架：shadcn/ui + Tailwind CSS
- WebSocket 实时流式通信
- React 19 + TypeScript
- SearXNG / Bing 联网搜索集成
- 网页抓取：Jina Reader fast path + web2llm fallback

## 快速开始

### 前置条件

- [Docker](https://www.docker.com/) 或 [OrbStack](https://orbstack.dev/)（推荐 Docker 模式）
- 或 Rust 1.75+ + Node.js 18+ + pnpm（源码模式）

### 方式一：Docker 一键启动（推荐）

```bash
# 1. 克隆项目
git clone https://github.com/Azhu9701/soul-banner-lite.git
cd soul-banner-lite

# 2. 一键构建并启动（首次约 15-20 分钟编译 Rust）
bash scripts/start-local.sh

# 3. 访问 http://localhost:8088
```

`start-local.sh` 会自动生成 `.env` 文件（从 `deploy/.env.example` 复制），首次启动后编辑 `.env` 填入 API Key。

启动后包含 5 个服务：Caddy（反向代理 + 端口 8088）、API（Rust 后端）、Web（Next.js 前端）、SearXNG（联网搜索）、Cloudflare Tunnel（可选公网访问）。

**常用命令：**

```bash
# 查看日志
docker compose -f docker-compose.local.yml logs -f

# 停止
docker compose -f docker-compose.local.yml down

# 重新构建（代码更新后）
docker compose -f docker-compose.local.yml up --build -d
```

### 方式二：源码安装

```bash
# 1. 环境变量（可选，源码模式使用 .env.example）
cp .env.example .env
# 编辑 .env 填入 LM Studio 模型名或 API Key

# 2. 一键安装
bash install.sh
```

脚本会自动完成：依赖检查、数据目录初始化、后端编译、前端构建、启动脚本生成。

安装完成后用开发模式启动：

```bash
bash start.sh
```

### 运行方式

#### 方式 A：本地算力（LM Studio — 推荐，零 API 费用）

无需云端 API Key，完全使用本地 GPU/CPU 运行大模型。

**1. 安装 LM Studio**

下载地址：https://lmstudio.ai

**2. 加载模型**

打开 LM Studio，在左侧模型浏览器下载或导入模型（推荐 Qwen3.5-14B、DeepSeek-R1-Distill 等中文友好模型）。

**3. 启动本地服务器**

点击左侧「Developer」→ 选择模型 → 启动服务器（默认端口 1234）。

**4. 配置 Snake Skin**

打开 `/models` 页面：
- 选择「LM Studio」
- 填入模型名（如 `qwen/qwen3.5-14b`）
- **源码模式**填入 `http://localhost:1234/v1`，**Docker 模式**填入 `http://host.docker.internal:1234/v1`
- 如有 API Key 则填入（LM Studio 默认无认证）
- 点击「测试」验证连通性
- 点击「设为活跃」切换 provider

**5. 开始使用**

返回首页，描述你的劳动争议案件，系统将自动传唤 5 个角色展开庭审。

#### 方式 B：云端 API（DeepSeek / Claude / OpenAI）

**配置 API Key**

将 API Key 写入 `data/apikeys.json`（此文件已在 `.gitignore` 中排除，不会被提交）：

```json
{
  "deepseek": "your-deepseek-api-key",
  "openai": "your-openai-api-key",
  "claude": "your-claude-api-key"
}
```

或打开 `/models` 页面选择对应 provider 并「设为活跃」。

#### 方式二：手动安装

**前置条件**

- Rust 工具链（1.75+）
- Node.js 18+ 和 pnpm
- 至少一个 AI 提供商的 API Key（DeepSeek / Claude / OpenAI），或本地 LM Studio

**编译后端**

```bash
cargo build --package api --release
```

**编译前端**

```bash
cd nextjs && pnpm install && pnpm build
```

**启动**

```bash
# 终端 1：启动 API 服务（端口 3096）
cd rust && cargo run --package api

# 终端 2：启动前端（端口 3000）
cd nextjs && pnpm start
```

### 访问

- **Docker 模式**：http://localhost:8088（Caddy 统一代理前端和 API）
- **源码开发模式**：前端 http://localhost:3000，API http://127.0.0.1:3096
- API 健康检查：`/api/v1/health`

## 配置

编辑 `config/default.yaml` 自定义路径和行为：

```yaml
data_dir: "./data"
souls_dir: "./data/souls"
archive_dir: "./data/archive"
db_path: "./data/db/app.db"
registry_path: "./data/registry.yaml"
call_records_path: "./data/call-records.yaml"
server_host: "127.0.0.1"
server_port: 3096
nextjs_port: 3000

searxng_url: "http://127.0.0.1:8080"
search_engine: "bing"

rate_limit:
  enabled: true
  requests_per_second: 30
  burst_size: 60

# API 认证 token（也可通过环境变量 WANMINFAN_API_TOKEN 设置）
# api_token: "your-secret-token-here"

# CORS 允许的来源
cors_origins:
  - "http://localhost:3000"
  - "http://localhost:3002"
```

> **注意**：`data/archive/` 和 `data/db/` 在 `.gitignore` 中被忽略，运行时自动生成。克隆后仓库中保留 `.gitkeep` 文件以确保目录结构存在。

## 项目结构

```
├── rust/
│   ├── api/           # Axum HTTP API + WebSocket + SSE
│   ├── cli/           # 命令行工具（角色管理、庭审记录查询等）
│   ├── possession/    # 庭审引擎（合议/辩论/接力/教学等模式）
│   ├── ai-gateway/    # AI 模型网关（Claude/OpenAI/DeepSeek + 路由）
│   ├── registry/      # 角色注册表 + 全文检索 + 冷启动 boost
│   ├── archive/       # 归档 + 成本追踪 + Token 统计
│   └── foundation/    # 基础设施（SQLite/存储/错误处理）
├── nextjs/            # Next.js 前端
├── config/            # 配置文件
│   ├── default.yaml   # 默认配置
│   └── domain.yaml    # 模拟仲裁庭术语定义（角色、庭审流程、裁决风格）
├── data/              # 运行时数据
│   ├── souls/         # 庭审角色定义（仲裁法官、原告律师、被告律师等）
│   ├── knowledge/     # 知识库（labor-law 劳动法知识库）
│   ├── archive/       # 庭审记录归档（.gitignore 忽略）
│   └── db/            # SQLite 数据库（.gitignore 忽略）
├── deploy/            # Docker 部署配置（Caddyfile, SearXNG, docker-compose）
├── scripts/           # 辅助脚本（start-local.sh 等）
├── install.sh         # 源码一键安装脚本
├── start.sh           # 开发模式一键启动
└── .env.example       # 环境变量模板（源码模式）
```

## 关键特性

### 庭审流程与实时干预
- **角色长驻进程**：每个角色作为独立 tokio task，`tokio::select!` 竞态监听干预信号
- **三级追问门控**：L1 关键词规则(微秒) → L2 trigram Jaccard 相似度(毫秒) → L3 Flash LLM 判定(秒)
- **实时干预注入**：碰撞检出后立即通过 intervention 通道打断角色推理，重新生成带冲突上下文的回应

### 语义碰撞检测
- **三路径并行**：关键词规则 + trigram 语义相似度 + 四维坐标距离
- **滑动窗口**：每 N tokens 捕获片段，冗余抑制，盲区互补检测
- **碰撞类型**：矛盾 / 视角差异 / 前提分歧 / 补充挑战 / 冗余 / 盲区互补

### 角色专属工具
每个角色拥有不同的工具权限：

| 工具 | 功能 | 可用角色 |
|------|------|---------|
| `search_labor_law` | 检索劳动法相关法条 | 仲裁法官、原告律师、被告律师、专家证人 |
| `calculate_severance` | 独立核算经济补偿金、赔偿金 | 仲裁法官、原告律师、被告律师、专家证人 |
| `generate_evidence_checklist` | 生成证据清单和举证指引 | 原告律师、被告律师 |
| `WebSearch` | 联网搜索补充信息 | 全部角色 |
| `web_fetch` | 网页抓取，直接注入网页内容 | 全部角色 |

### 裁决机制
- **五步裁决法**：共识确认 → 分歧梳理 → 盲区发现 → 工具性分析 → 裁决说理
- **仲裁法官退庭评议**：庭审阶段不做出最终裁决，所有角色发言完毕后仲裁法官退庭评议，独立做出裁决
- **裁决内容**：争议焦点归纳、事实认定、举证责任分配、法律适用、经济补偿/赔偿金计算、建议调解或诉讼方案

### 角色匹配与推荐
- **全文 + 向量检索**：SQLite FTS5 + 余弦相似度向量检索
- **冷启动 boost**：新角色获得 relevance 加分，打破马太效应
- **四维坐标搜索**：法庭角色/法律立场/论证方法/价值取向
- **多因子混合匹配**：坐标邻近度 + 领域术语命中 + 触发关键词 + 全文相关性 + 实践反馈加权

### 成本与数据
- **Token 消耗追踪**：庭审统计/会话历史/角色效能表三处可见
- **模型智能路由**：根据任务类型自动选择模型和推理强度
- **实时成本统计**：含 DeepSeek 缓存折扣预估

### 庭审效率指数
- **消费 vs 实践**：未走反馈闭环的庭审计为消费型——用 AI 模拟代替实际维权行动，和刷短视频没有本质区别
- **公式**：`(无效 + 部分有效 × 0.5) / 总庭审 × 100` — 数字越高，说明你消费的模拟越多、行动越少
- **三问闭环**：每次庭审结束后直填三问——你会做什么？谁该在场？什么判断错了？
- **设计动机**：不是为了优化数字，是为了让你每次打开模拟仲裁庭时问自己——这次是真的要维权，还是又来消费一场模拟表演？

### 联网搜索与网页抓取
- **Bing 搜索**：开箱即用，默认搜索引擎
- **SearXNG 集成**：自托管元搜索引擎，支持多引擎聚合
- **追问搜索**：追问时可开启联网搜索，搜索结果注入角色上下文
- **网页抓取**：内置 `web_fetch_tool`（Jina Reader fast path + web2llm fallback），直接注入网页内容

### 输出质量控制
- **反剧场式旁白**：system prompt 层面强制约束，严禁第三人称叙事/动作描写/场景表演
- **角色一致性**：角色的 voice 转化为法律论证风格而非戏剧表演——"用角色的法律思维方式思考，不是演角色"
- **法庭发言风格**：深度不等于篇幅，3句能说清楚的问题不需要3页。每个观点都要落到可操作的结论：支持/驳回/需要补充证据/建议调解

### 前端体验
- **@mention 角色传唤**：追问框输入 @ 触发角色名自动补全，已出庭角色优先展示
- **↑ 输入历史**：主输入框和追问框支持 ↑↓ 键回显历史
- **双层加载**：digest 摘要 + 按需展开完整庭审对话
- **消息分叉**：从任意案件描述分叉新庭审，保留历史上下文
- **卡片下载**：角色发言和裁决报告弹窗支持下载 .md
- **思考过程折叠**：深度思考模型的推理链默认一行，点击展开
- **庭审记录**：侧栏和庭审历史支持内联重命名
- **庭审统计**：庭审效率、角色效能、成本追踪可视化

## 社区与贡献

- 提交 Issue：[GitHub Issues](https://github.com/Azhu9701/soul-banner-lite/issues)
- 查看 CI 状态：[Actions](https://github.com/Azhu9701/soul-banner-lite/actions)
- 请阅读 [Issue/PR 模板](.github/) 提交规范反馈

## 法律免责声明

Snake Skin 是**法律模拟辅助工具**，其输出不构成正式法律意见。所有裁决、分析和建议仅供学习和参考，实际维权行动请咨询执业律师或当地劳动仲裁委员会。

## License

MIT
