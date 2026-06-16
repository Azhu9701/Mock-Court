# REST API 路由表

全部 API 挂载在 `/api/v1` 前缀下。

## 健康检查

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/health` | 服务健康检查 |

## Agent 管理 (`/souls`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/souls` | 获取 Agent 列表，支持 `?filter=` 过滤 |
| GET | `/souls/search?q=` | 全文搜索 Agent |
| GET | `/souls/:name` | 获取指定 Agent 详情 |
| POST | `/souls` | 创建新 Agent（JSON body） |
| PUT | `/souls/:name` | 更新 Agent |
| DELETE | `/souls/:name` | 删除 Agent |
| POST | `/souls/reload` | 重新加载全部 Agent 文件 |
| POST | `/souls/collect` | AI 自动生成 Agent 配置 |
| POST | `/souls/refine` | 基于反馈优化 Agent |
| GET | `/souls/:name/revisions` | 查看 Agent 修改历史 |
| PUT | `/souls/apply-refine` | 应用优化版本 |
| POST | `/souls/auto-create` | 自动创建 Agent（WS 进度） |
| GET | `/souls/ismism/distribution` | 坐标分布统计 |

## 会话执行 (`/possess`)

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/possess/analyze` | 分析任务 → SSE 流式返回推荐 |
| POST | `/possess` | 启动附体会话 |
| POST | `/possess/interrogate` | 启动审查官审讯流 |
| POST | `/possess/interrogate/:gate_id/respond` | 提交审讯响应 |
| POST | `/possess/ocr` | OCR 文件识别 |
| GET | `/possess/:session_id` | 获取会话状态 |

## 会话管理 (`/sessions`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/sessions` | 获取会话列表，支持 `?status=&mode=` |
| GET | `/sessions/:id` | 获取会话详情（含全部消息） |
| DELETE | `/sessions/:id` | 删除会话 |
| POST | `/sessions/batch-delete` | 批量删除 |
| PUT | `/sessions/:id/rename` | 重命名会话 |
| PUT | `/sessions/:id/fork` | Fork 会话 |
| GET | `/sessions/:id/digest` | 获取会话摘要 |
| POST | `/sessions/:id/distill` | 生成会话蒸馏 |
| GET | `/sessions/:id/export/markdown` | 导出 Markdown |
| DELETE | `/sessions/:id/messages/:seq` | 删除指定序号之后的消息 |
| GET | `/sessions/:id/review` | 获取会后回顾 |
| POST | `/sessions/:id/review` | 保存会后回顾 |

## 分析统计 (`/analytics`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/analytics/summon-stats` | Agent 召唤统计 |
| GET | `/analytics/effectiveness/:agent` | Agent 效能趋势 |
| GET | `/analytics/mode-distribution` | 模式使用分布 |
| GET | `/analytics/alerts/unsummoned` | 长期未使用的 Agent |
| GET | `/analytics/alerts/low-effectiveness` | 低效能 Agent |

## 归档管理 (`/archive`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/archive/verify/:session_id` | 验证归档完整性 |
| POST | `/archive/export` | 导出会话归档 |
| GET | `/archive/export/:task_id` | 获取导出任务状态 |

## 知识库 (`/knowledge`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/knowledge/search?q=` | 搜索知识库 |
| POST | `/knowledge/rebuild` | 重建 FTS5 全文索引 |
| GET | `/knowledge/topics` | 获取知识主题列表 |
| GET | `/knowledge/cards` | 获取知识卡片 |

## 配置管理 (`/config`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/config` | 获取当前配置 |
| GET | `/config/domain` | 获取当前领域配置 |
| POST | `/config/domain` | 热切换领域 |
| GET | `/config/models/default` | 获取默认模型配置 |
| PUT | `/config/models/default` | 设置默认模型 |
| GET | `/config/providers` | 获取提供商配置 |
| PUT | `/config/providers/:provider` | 更新提供商配置（URL/Key/Model） |

## SearXNG 搜索 (`/searxng`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/searxng/search?q=` | Web 搜索 |
| GET | `/searxng/topic-search?q=` | 主题搜索 |

## API Key 管理 (`/apikey`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/apikey` | 列出 API Key（脱敏） |
| POST | `/apikey` | 创建新 Key |
| DELETE | `/apikey/:id` | 删除 Key |

## 认证 (`/auth`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/auth/me` | 获取当前用户信息 |
| POST | `/auth/login` | 登录 |
| POST | `/auth/logout` | 登出 |

## 通用约定

- **请求头**: `Content-Type: application/json`，`Authorization: Bearer <token>`
- **分页**: `?limit=20&offset=0`
- **错误响应**:
  ```json
  { "error": { "code": "AGENT_NOT_FOUND", "message": "Agent 'xxx' not found" } }
  ```
- **成功响应**: 直接返回数据或 `{ "data": ... }`
