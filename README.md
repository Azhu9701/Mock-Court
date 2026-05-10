# 万民幡 Soul Banner Lite

> 这不是聊天界面——这是多视角同时碰撞的观测窗口。

万民幡是一个**多 AI 人格并行推理系统**。同时召唤多个具有独立世界观的思想家（称为"魂"），让他们围绕同一个问题展开思考、碰撞、辩论与综合。

## 核心概念

| 对比 | 传统聊天界面 | 万民幡 |
|------|------------|--------|
| 交互模式 | 单列时间线 | 多列并行流 |
| 对话方式 | 一问一答 | 一问多答 + 辩证综合 |
| 控制节奏 | 使用者主导 | 魂各自推进，交叉追问自动触发 |
| 历史回溯 | 往上翻 | 每个魂有独立时间轴、修正历史、盲区记录 |
| 入口 | 输入框 | 魂状态、知识库检索、历史回溯都是入口 |

## 基本架构

```
┌─────────────┐    ┌──────────────┐    ┌───────────────┐
│  Next.js 前端│───▶│  Axum API    │───▶│  AI Gateway   │
│  (shadcn/ui) │    │  (端口 3096)  │    │  Claude/GPT/DS │
└─────────────┘    └──────┬───────┘    └───────────────┘
                          │
          ┌───────────────┼───────────────┐
          ▼               ▼               ▼
    ┌──────────┐   ┌──────────┐   ┌──────────┐
    │ Registry │   │Possession│   │ Archive  │
    │  魂注册表 │   │  附体引擎 │   │  归档分析 │
    └──────────┘   └──────────┘   └──────────┘
```

## 已支持的附体模式

| 模式 | 描述 |
|------|------|
| **单魂** | 单一魂独立输出，适合快速咨询 |
| **合议** | 多魂同时输出，流式交叉检测，实时碰撞，辩证综合 |
| **辩论** | 两列对立 + 中间裁决栏 |
| **接力** | 横向时间轴，阶段卡片传递 |
| **学习** | 魂教学模式 |
| **实践开口** | 方法论实践模式 |

## 内置魂（思想家）

马克思、列宁、毛泽东、邓小平、鲁迅、尼采、黑格尔、费曼、马斯克、黄仁勋、庄子、孔子、胡塞尔、波伏娃、法农、葛兰西、伊本赫勒敦、稻盛和夫、未明子、乔布斯、海绵宝宝……等 27 个预设魂。

每个魂有独立的**主义主义四维坐标**（场域论/存在论/认识论/目的论）、专属召唤咒语、排除场景和自我声明。

## 技术栈

**后端：Rust**
- Web 框架：Axum 0.7 + WebSocket
- 数据库：SQLite（WAL 模式）+ FTS5 全文搜索
- AI 网关：Claude / OpenAI / DeepSeek 多提供商
- 异步运行时：Tokio

**前端：Next.js**
- UI 框架：shadcn/ui + Tailwind CSS
- WebSocket 实时通信
- 响应式布局

## 快速开始

### Windows 用户（MSI 安装包）

1. 从 [GitHub Releases](https://github.com/Azhu9701/soul-banner-lite/releases) 下载 `万民幡-Setup.msi`
2. 双击安装，按向导操作
3. 安装完成后，从开始菜单启动「万民幡」
4. 首次启动前，编辑安装目录下的 `data/apikeys.json`，填入 API Key
5. 浏览器自动打开 http://localhost:3000

### macOS / Linux 用户

#### 前置条件

- Rust 工具链（1.75+）
- Node.js 18+ 和 pnpm
- 至少一个 AI 提供商的 API Key（DeepSeek / Claude / OpenAI）

### 配置 API Key

将 API Key 写入 `data/apikeys.json`（此文件已在 `.gitignore` 中排除）：

```json
{
  "deepseek": "your-deepseek-api-key",
  "openai": "your-openai-api-key",
  "claude": "your-claude-api-key"
}
```

### 启动

```bash
# 一键启动（API + 前端）
bash start.sh
```

或分别启动：

```bash
# 启动 API 服务（端口 3096）
cargo run -p api --release

# 启动前端（端口 3000）
cd nextjs && pnpm dev
```

### 访问

- 前端：http://localhost:3000
- API：http://127.0.0.1:3096
- API 健康检查：http://127.0.0.1:3096/api/v1/health

## 配置

编辑 `config/default.yaml` 自定义路径和行为：

```yaml
data_dir: "./data"
souls_dir: "./data/souls"
archive_dir: "./data/archive"
db_path: "./data/wanminfan.db"
server_host: "127.0.0.1"
server_port: 3096
nextjs_port: 3000
```

## 项目结构

```
├── rust/
│   ├── api/           # Axum HTTP API + WebSocket
│   ├── possession/    # 附体引擎（合议/辩论/接力等模式）
│   ├── ai-gateway/    # AI 模型网关（Claude/OpenAI/DeepSeek）
│   ├── registry/      # 魂注册表 + 全文检索
│   ├── archive/       # 归档 + 成本追踪 + 分析
│   └── foundation/    # 基础设施（SQLite/存储/错误处理）
├── nextjs/            # Next.js 前端
├── config/            # 配置文件
├── data/              # 数据目录（souls/archive/db）
├── scripts/           # 辅助脚本
└── start.sh           # 一键启动
```

## 关键特性

- **流式交叉检测**：多魂并行输出时实时检测碰撞（矛盾/互补/盲区），自动生成追问注入
- **魂长驻进程**：每个魂作为独立 tokio task，支持状态管理和跨轮记忆保留
- **魂自我审计**：输出后自动检测自我矛盾、边界违反、前提动摇，生成修正提案
- **模型智能路由**：根据任务类型自动选择模型和推理强度（Flash/Pro/Think High）
- **成本透明化**：实时 token 消耗统计、预估费用（含 DeepSeek 缓存折扣）
- **全文 + 向量检索**：SQLite FTS5 全文搜索 + 余弦相似度向量检索

## License

MIT
