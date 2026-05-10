# Business Rules — F2: Soul Browser

## 1. 魂列表渲染规则 (Q1: B — 卡片网格)

| 规则 | 描述 |
|------|------|
| BR1.1 | 桌面端 4 列网格，平板 2 列，手机 1 列 |
| BR1.2 | 卡片展示：品级徽章 + 魂名 + ismism 编码 + 领域 + 召唤次数 |
| BR1.3 | 品级徽章颜色：S=gold, A=blue, B=green, C=gray, D=red |
| BR1.4 | 卡片 hover 时轻微上浮 (transform: translateY(-2px)) |
| BR1.5 | 点击卡片进入 `/souls/[name]` 详情页 |
| BR1.6 | 无筛选结果时显示 EmptyState |

## 2. 搜索规则 (Q2: C — 混合)

| 规则 | 描述 |
|------|------|
| BR2.1 | 初始加载：RSC fetch 全量 souls，客户端渲染 |
| BR2.2 | 关键词搜索：debounce 300ms 后调用 `GET /souls/search?q=` |
| BR2.3 | 搜索结果按 relevance 降序排列 |
| BR2.4 | 清空搜索框 → 恢复全量列表 |
| BR2.5 | 搜索 loading 态使用 SkeletonCard 占位 |

## 3. 筛选规则 (Q5: C — 品级 + ismism + 关键词)

| 规则 | 描述 |
|------|------|
| BR3.1 | 品级筛选：单选下拉 (S/A/B/C/D/全部) |
| BR3.2 | ismism 四维滑块：每维范围 1-4，默认 [1,4]（不筛选） |
| BR3.3 | ismism 筛选逻辑：以滑块中心为 target，距离 ≤ 阈值 (默认 2.0) |
| BR3.4 | 筛选是客户端操作，不触发 API 请求 |
| BR3.5 | 多个筛选条件 AND 逻辑 |
| BR3.6 | 筛选状态编码在 URL searchParams（可分享链接） |

## 4. 魂详情页规则 (Q3: A — 完整 Profile)

| 规则 | 描述 |
|------|------|
| BR4.1 | `/souls/[name]` 不存在时显示 404 页面 |
| BR4.2 | ismism 雷达图：四维 SVG 雷达（field/ontology/epistemology/teleology） |
| BR4.3 | summon_prompt 默认折叠，点击展开全文（可能很长） |
| BR4.4 | 实践记录列表默认展示最近 5 条，可展开全部 |
| BR4.5 | Effectiveness 用进度条显示 effective / partial / invalid 比例 |
| BR4.6 | "召唤此魂" 按钮跳转到 `/possess?preset=single&souls[]=[name]` |

## 5. 数据缓存规则 (Q4: A — RSC)

| 规则 | 描述 |
|------|------|
| BR5.1 | RSC fetch 默认 `revalidate = 60s`（增量静态再生） |
| BR5.2 | CRUD 操作后使用 `router.refresh()` 刷新 RSC 数据 |
| BR5.3 | 搜索不缓存（客户端 fetch） |

## 6. 交互规则

| 规则 | 描述 |
|------|------|
| BR6.1 | 删除魂需二次确认 (Dialog) |
| BR6.2 | 编辑魂通过 Modal 表单，提交后刷新列表 |
| BR6.3 | 所有可点击元素添加 `data-testid` |
| BR6.4 | 页面标题：`<title>` 显示 "魂览 — 万民幡" |
