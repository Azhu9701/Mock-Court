# Snake Skin

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)](https://www.rust-lang.org)
[![Next.js](https://img.shields.io/badge/Next.js-16-black?logo=next.js)](https://nextjs.org)
[![Docker](https://img.shields.io/badge/Docker-ready-blue?logo=docker)](https://www.docker.com)

> 思辨不等于思考。消费观点是最精致的奶头乐。

Snake Skin 是一个**多 AI 人格并行推理系统**——同时召唤多个具有独立世界观的思想家（称为"魂"），围绕同一个问题展开碰撞与综合。但它深知自己的危险：**辩证AI本身就是最高级的奶头乐**。看马克思和尼采吵架、庄子消解黑格尔的体系——这个过程太爽了，爽到你以为自己"想明白了"，其实你只是消费了一场表演。

蛇皮指数存在的意义就是：**用量化数据逼你承认——你是在蜕皮，还是在换一层更精致的皮。**

## 核心概念

| 对比 | 传统聊天界面 | Snake Skin |
|------|------------|------------|
| 交互模式 | 单列时间线 | 多列并行流 |
| 对话方式 | 一问一答 | 一问多答 + 辩证综合 |
| 控制节奏 | 使用者主导 | 魂各自推进，交叉追问实时干预 |
| 历史回溯 | 往上翻 | 每个魂有独立记忆图谱、修正历史、盲区记录 |
| 反馈闭环 | 无 | 蛇皮指数跟踪思辨消费 vs 行动转化 |
| 入口 | 输入框 | 魂状态、知识库检索、历史回溯、@mention 都是入口 |

## 基本架构

```
                              ┌──────────────────┐
                              │   审查官（幡主）    │
                              │  反问 → 整合议题 → 放行 │
                              └────────┬─────────┘
                                       │ refined_task
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
- AI 网关：Claude / OpenAI / DeepSeek / LM Studio 多提供商 + 模型智能路由
- 图数据库：petgraph（魂记忆图谱）
- 异步运行时：Tokio

**前端：Next.js 16**
- UI 框架：shadcn/ui + Tailwind CSS
- WebSocket 实时流式通信
- React 19 + TypeScript
- SearXNG 联网搜索集成

## 快速开始

### Windows 用户（MSI 安装包）

1. 从 [GitHub Releases](https://github.com/Azhu9701/soul-banner-lite/releases) 下载 `万民幡-Setup.msi`
2. 双击安装，按向导操作
3. 安装完成后，从开始菜单启动「万民幡」
4. 首次启动前，编辑安装目录下的 `data/apikeys.json`，填入 API Key
5. 浏览器自动打开 http://localhost:3000

### macOS / Linux 用户

#### 方式一：Docker 一键启动（推荐）

**前置条件**：[Docker](https://www.docker.com/) 或 [OrbStack](https://orbstack.dev/)

```bash
# 1. 克隆项目
git clone https://github.com/Azhu9701/soul-banner-lite.git
cd soul-banner-lite

# 2. 一键构建并启动（首次约 15-20 分钟编译 Rust）
bash scripts/start-local.sh

# 3. 访问
#    http://localhost:8088
```

启动后包含 5 个服务：Caddy（反向代理 + 端口 8088）、API（Rust 后端）、Web（Next.js 前端）、SearXNG（联网搜索）、Cloudflare Tunnel（可选公网访问）。

如需配置 AI 提供商，复制环境变量模板并编辑：

```bash
cp deploy/.env.example .env
# 编辑 .env 填入 API Key 或 LM Studio 地址
```

然后重启容器：

```bash
docker compose -f docker-compose.local.yml restart api
```

**常用命令：**

```bash
# 查看日志
docker compose -f docker-compose.local.yml logs -f

# 停止
docker compose -f docker-compose.local.yml down

# 重新构建（代码更新后）
docker compose -f docker-compose.local.yml up --build -d
```

#### 方式二：源码安装

```bash
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

返回首页，点击「我想问」即可使用本地模型进行附体推理。

### 本地模型选择与深度表现

模型选择直接影响魂的分析深度。核心指标不是总参数量，而是**每 token 激活参数量**——MoE 模型（如 qwen3.6-35b-a3b，激活 3B）的知识虽宽但推理浅，密集模型（如 qwen3.6-27b，激活 27B）在同一 prompt 下能产出更深的辩证反转。

以下测试使用 Marx 魂回答「滴滴外卖平台说自己是技术服务商不是雇主，你怎么看？」，温度 0.9，深度协议一致：

| 模型 | 类型 | 激活参数 | 字数 | 内存需求 | 深度协议 | 推荐场景 |
|------|------|---------|------|---------|---------|---------|
| qwen3-4b | 密集 | 4B | ~975 | ~4GB | L4-L5 缺失，自相矛盾 | 不推荐—深度协议失效 |
| qwen3.6-35b-a3b | MoE | 3B | 500-700 | ~20GB | L4 浅，重复凑数 | 不推荐 |
| **qwen3.5-9b** | 密集 | **9B** | **~1300** | **~6GB** | **六层全到 ✓** | **入门首选** |
| **qwen3.6-27b** | 密集 | **27B** | **~1600** | **~16GB** | **六层全到 + 自我打断 + 更深层分析** | **最优深度** |
| DeepSeek V4 / Claude Opus | 云端 | — | ~1500 | — | **六层全到 + 自我打断 + 自审** | **追求极致时** |

关键发现：

- **4B 密集是深度协议的下界**。4B 能产出文本但不能产出辩证思考——L4（辩证反转）和 L5（历史地平）结构性缺失，模型记不住自己前面写了什么所以末段与首段自相矛盾。往下不建议跑。
- **9B 密集 > 35B MoE（3B active）**。总参数量是虚的，每 token 参与推理的参数量决定分析深度。9B 密集在 L1-L3（现象还原、机制拆解、前提追问）上不输 27B，每层论证完整、无重复注水。
- **27B 密集追上云端的关键：L4 拆成两步。** 把辩证反转拆为"推到极端"+"你必须实际打断自己的论证"，27B 从 1300 字涨到 1600 字，做到了论证中段自我打断并从打断里长出更深一层分析——这个能力之前只有云端有。两步指令让本地模型执行了它原本有能力但不会主动做的动作。
- **去掉「每段不超过3句话」规则后，本地模型篇幅涨 35%+**。模型自己分段能力足够，不需要指令代管。当前深度协议已移除该限制。
- **max_output_tokens 不是瓶颈**。8000 tokens 对 1500-2000 字的中文分析绰绰有余，实际输出由模型推理能力和 prompt 结构决定。

硬件参考：

| 目标 | 方案 | 参考价 |
|------|------|--------|
| 跑 9B 密集（推荐入门） | Mac Mini M4 16GB 或 16GB 笔记本 | ¥3,500-5,000 |
| 跑 27B 密集（追求深度） | Mac Mini M4 32GB 或二手 RTX 3090 24GB | ¥4,500-6,500 |
| 零成本立即用 | LM Studio 云端代理 或 DeepSeek API | ¥0（API 按量付费） |

#### 方式 B：云端 API（DeepSeek / Claude / OpenAI）

**配置 API Key**

将 API Key 写入 `data/apikeys.json`（此文件已在 `.gitignore` 中排除）：

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
├── deploy/            # Docker 部署配置（Caddyfile, SearXNG, docker-compose）
├── scripts/           # 辅助脚本（start-local.sh 等）
├── install.sh         # 源码一键安装脚本
└── start.sh           # 开发模式一键启动
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

### 入场审讯与工具性提取
- **反问不是审判**：审查官围绕议题追问只有使用者才能提供的具体事实——走了几个、谁说的、卡在哪个环节
- **没有裁决**：使用者回答即放行，不再用 LLM 判断"是否可以入场"
- **议题整合**：使用者的回答被 LLM 融进原始议题，生成更完整的任务描述供合议使用
- **构成性追问**：当使用者出现"没办法""一直是这样"时，追问"这个规则是谁制定的？你在替谁说话？"
- **审查官终裁**：终裁阶段仍由审查官审核魂组合、分派差异化子任务，`verified_souls` 为唯一权威

### 搜索与匹配
- **全文 + 向量检索**：SQLite FTS5 + 余弦相似度向量检索
- **冷启动 boost**：summon_count < 3 的魂获得 relevance 加分，打破马太效应
- **主义主义坐标**：四维坐标距离搜索
- **工具意识分数**：`Ismism(40%) + 工具意识(15%) + 领域(20%) + 关键词(10%) + FT(15%)`——self_declare 越具体、"我不做"边界越清楚的魂越靠前。承认工具性不是示弱，是拆解构成自身的力量的第一步
- **多因子混合匹配**：ismism 坐标邻近度 + 领域术语命中 + 触发关键词 + 全文相关性 + 实践反馈加权

### 成本与数据
- **Token 消耗追踪**：蛇皮统计/会话历史/魂效能表三处可见
- **模型智能路由**：根据任务类型自动选择模型和推理强度
- **实时成本统计**：含 DeepSeek 缓存折扣预估

### 蛇皮指数 — 思辨消费跟踪
- **消费 vs 实践**：未走反馈闭环的会话计为消费型——用 AI 辩证代替自己的思考，和刷短视频没有本质区别
- **公式**：`(无效 + 部分有效 × 0.5) / 总会话 × 100` — 数字越高，说明你消费的思辨越多、行动越少
- **三问闭环**：每个会话结束后直填三问——你会做什么？谁该在场？什么判断错了？
- **蛇皮层级**：厚重(≥70) / 中等(40-69) / 较薄(15-39) / 接近蜕皮(<15)
- **设计动机**：蛇皮指数不是为了优化数字，是为了让你每次打开 Snake Skin 时问自己——这次是真的要蜕皮，还是又来换一层更好看的皮？

### 魂召唤与 @mention
- **推荐魂召唤**：综合官推荐补充魂，点击"直接加入合议"即作为子 agent 输出（非辩证综合）
- **@mention**：追问输入框输入 `@魂名` 触发自动补全，选中后该魂以完整身份直接回复
- **入场审讯**：讨论启动前审查官反问，使用者的回答被整合进议题描述，注入魂共用上下文

### 辩证综合与工具性分析
- **五步辩证综合法**：共识 → 分歧 → 盲区 → 工具性分析 → 行动纲领
- **工具性分析**：各魂的发言暴露了使用者在这个议题里被夹在什么力量之间——Ta 服务谁的利益，又被谁的利益压制？哪个魂把使用者当成"有处境的人"来分析，哪个魂当成"有观点的人"来回应？
- **结构性预设承认**：每个魂的召唤 prompt 要求：如果你察觉到自己正在被某个结构性预设支配，把它说出来——"说出'我是被这样构成的'的同时，你已经开始拆解那个构成你的力量"

### 联网搜索
- **SearXNG 集成**：自托管元搜索引擎，支持多引擎聚合
- **追问搜索**：追问时可开启联网搜索，搜索结果注入魂的上下文
- **独立搜索页**：通过 SearXNG 搜索互联网资源

### 输出质量控制
- **反剧场式旁白**：system prompt 层面强制约束，严禁第三人称叙事/动作描写/场景表演
- **角色一致性**：魂的 voice 转化为分析风格而非戏剧表演——"用角色的思维方式思考，不是演角色"

### 前端体验
- **@mention 魂召唤**：追问框输入 @ 触发魂名自动补全，已参会魂优先展示
- **↑ 输入历史**：主输入框和追问框支持 ↑↓ 键回显历史
- **双层加载**：digest 摘要（5-10 observation）+ 按需展开完整对话
- **消息分叉**：从任意用户消息分叉新会话，保留历史上下文
- **卡片下载**：魂回应和综合报告弹窗支持下载 .md
- **思考过程折叠**：深度思考模型的推理链默认一行，点击展开
- **侧栏优化**：模式色点替代图标，紧凑布局，操作按钮 hover 弹出
- **WebSocket 流式渲染**：50ms 批量刷新，useMemo 优化
- **会话重命名**：侧栏和会话历史支持内联重命名

## License

MIT
