# 万民幡 Soul Banner Lite

> 这不是聊天界面——这是多视角同时碰撞的观测窗口。

万民幡是一个**多 AI 人格并行推理系统**。同时召唤多个具有独立世界观的思想家（称为"魂"），让他们围绕同一个问题展开思考、碰撞、辩论与综合。

## 核心概念

| 对比 | 传统聊天界面 | 万民幡 |
|------|------------|--------|
| 交互模式 | 单列时间线 | 多列并行流 |
| 对话方式 | 一问一答 | 一问多答 + 辩证综合 |
| 控制节奏 | 使用者主导 | 魂各自推进，交叉追问实时干预 |
| 历史回溯 | 往上翻 | 每个魂有独立记忆图谱、修正历史、盲区记录 |
| 入口 | 输入框 | 魂状态、知识库检索、历史回溯都是入口 |

## 基本架构

```
                              ┌──────────────────┐
                              │   审查官（幡主）    │
                              │  一审 → 补视角 → 终裁 │
                              └────────┬─────────┘
                                       │ verified_souls
┌─────────────┐    ┌──────────────┐    │    ┌───────────────┐
│  Next.js 前端│───▶│  Axum API    │───▶│  AI Gateway   │
│  (shadcn/ui) │    │  (端口 3096)  │    │  Claude/GPT/DS │
└─────────────┘    └──────┬───────┘    └───────────────┘
                          │
          ┌───────────────┼───────────────────────┐
          ▼               ▼               ▼               ▼
    ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
    │ Registry │   │Possession│   │ Archive  │   │Foundation│
    │  魂注册表 │   │  附体引擎 │   │  归档分析 │   │  基础设施 │
    └──────────┘   └──────────┘   └──────────┘   └──────────┘
```

## 已支持的附体模式

| 模式 | 描述 |
|------|------|
| **单魂** | 单一魂独立输出，适合快速咨询 |
| **合议** | 多魂并行流式输出 + 实时碰撞检测 + 三级门控干预 + 辩证综合 |
| **辩论** | 两列对立 + 中间裁决栏 |
| **接力** | 横向时间轴，阶段卡片传递 |
| **学习** | 魂间教学（费曼学习法） |
| **实践开口** | 方法论实践模式 |

## 内置魂（思想家）

马克思、列宁、毛泽东、邓小平、鲁迅、尼采、黑格尔、费曼、马斯克、黄仁勋、庄子、孔子、胡塞尔、波伏娃、法农、葛兰西、伊本赫勒敦、稻盛和夫、未明子、乔布斯、海绵宝宝、斯大林、Karpathy、Aaron Swartz、祝鹤槐、轮臂……等 26 个预设魂。

每个魂有独立的**主义主义四维坐标**（场域论/存在论/认识论/目的论）、专属召唤咒语、排除场景和自我声明。

## 技术栈

**后端：Rust**
- Web 框架：Axum 0.7 + WebSocket + SSE
- 数据库：SQLite（WAL 模式）+ FTS5 全文搜索
- AI 网关：Claude / OpenAI / DeepSeek 多提供商 + 模型智能路由
- 图数据库：petgraph（魂记忆图谱）
- 异步运行时：Tokio

**前端：Next.js 16**
- UI 框架：shadcn/ui + Tailwind CSS
- WebSocket 实时通信
- React 19 + TypeScript

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
│   ├── api/           # Axum HTTP API + WebSocket + SSE
│   ├── possession/    # 附体引擎（合议/辩论/接力/教学等模式）
│   ├── ai-gateway/    # AI 模型网关（Claude/OpenAI/DeepSeek + 路由）
│   ├── registry/      # 魂注册表 + 全文检索 + 冷启动 boost
│   ├── archive/       # 归档 + 成本追踪 + Token 统计
│   └── foundation/    # 基础设施（SQLite/存储/错误处理）
├── nextjs/            # Next.js 前端
├── config/            # 配置文件
├── data/              # 数据目录（souls/archive/db）
├── scripts/           # 辅助脚本
└── start.sh           # 一键启动
```

## 关键特性

### 魂进程与实时干预
- **魂长驻进程**：每个魂作为独立 tokio task，`tokio::select!` 竞态监听干预信号
- **三级追问门控**：L1 关键词规则(微秒) → L2 trigram Jaccard 相似度(毫秒) → L3 Flash LLM 判定(秒)
- **实时干预注入**：碰撞检出后立即通过 intervention 通道打断魂推理，重新生成带冲突上下文的回应

### 语义碰撞检测
- **三路径并行**：关键词规则 + trigram 语义相似度 + 主义主义坐标距离
- **滑动窗口**：每 N tokens 捕获片段，冗余抑制，盲区互补检测
- **碰撞类型**：矛盾 / 视角差异 / 前提分歧 / 补充挑战 / 冗余 / 盲区互补

### 魂记忆图谱
- 基于 petgraph 的有向图：前提 → 结论的推导链、跨魂 Integrates 融合边
- BFS 矛盾检测（O(n)，零 LLM 成本）、DFS 前提动摇传播
- 支持图合并、盲区发现

### 自适应合议拓扑
- 5 种拓扑：Minimal / ClusteredParallel / FullMesh / Oppositional / SequentialLadder
- 基于魂多样性 + 任务复杂度 + 预算约束的决策树
- 30 秒无碰撞自动降级，节省 LLM 成本

### 魂生态系统
- **信誉系统**：加权评分 0.3A+0.25(1-C)+0.25P+0.2N，驱动审查优先级/辩论权重/休眠判定
- **魂杂交**：四维方法论签名匹配，自动发现互补魂对生成融合 prompt
- **魂间教学**：费曼学习法流程，老师分析盲区 → 出题 → 评分反馈

### 审查官权威机制
- **审查官一审**：算法匹配魂 → 审查官审核（pass/reject + 缺失视角识别）
- **补充匹配**：系统根据缺失视角搜索候选魂补充
- **审查官终裁**：`verified_souls` 作为唯一权威，算法匹配只作参考

### 搜索与匹配
- **全文 + 向量检索**：SQLite FTS5 + 余弦相似度向量检索
- **冷启动 boost**：summon_count < 3 的魂获得 relevance 加分，打破马太效应
- **主义主义坐标**：四维坐标距离搜索

### 成本与数据
- **Token 消耗追踪**：仪表盘/会话历史/魂效能表三处可见
- **模型智能路由**：根据任务类型自动选择模型和推理强度
- **实时成本统计**：含 DeepSeek 缓存折扣预估

### 前端体验
- **↑ 输入历史**：主输入框和追问框支持 ↑↓ 键回显历史
- **卡片下载**：魂回应和综合报告弹窗支持下载 .md
- **WebSocket 流式渲染**：50ms 批量刷新，useMemo 优化

## License

MIT
