# Business Rules — F4: Dashboard

## 1. 仪表盘布局 (Q1: B — 5 模块)

| 规则 | 描述 |
|------|------|
| BR1.1 | 顶部：4 个统计卡片横向排列 |
| BR1.2 | 第二行：模式分布柱状图 + 魂有效性表格 (左右 50:50) |
| BR1.3 | 第三行：告警面板 (未召唤 + 低效) |
| BR1.4 | 底部：最近 10 条会话时间线 |

## 2. 统计卡片规则

| 规则 | 描述 |
|------|------|
| BR2.1 | 总召唤：`total_calls`，副标题 "全部历史" |
| BR2.2 | 魂参与率：`unique_souls_called / total_souls_available * 100%` |
| BR2.3 | 有效率：从所有魂聚合 `effective/total` |
| BR2.4 | 活跃警报：未召唤+低效告警总数 |

## 3. 图表规则 (Q2: A — Recharts)

| 规则 | 描述 |
|------|------|
| BR3.1 | 模式分布用 Recharts BarChart，6 根柱 |
| BR3.2 | 柱颜色按模式：single=blue, conference=purple, debate=orange, relay=green, learn=teal, practice_opening=red |
| BR3.3 | 图表跟随主题 (dark/light CSS 变量) |
| BR3.4 | Chart 组件为 Client Component (`"use client"`) |

## 4. 告警规则

| 规则 | 描述 |
|------|------|
| BR4.1 | 未召唤告警 (NeverSummoned/UnsummonedLongDuration) 显示红色标记 |
| BR4.2 | 低效告警显示黄色标记 + 有效率百分比 |
| BR4.3 | 单击告警项跳转到对应魂详情 `/souls/:name` |
| BR4.4 | 无告警时显示 "✓ 一切正常" |

## 5. 会话时间线规则 (Q3: B)

| 规则 | 描述 |
|------|------|
| BR5.1 | 按日期分组：今天/昨天/具体日期 |
| BR5.2 | 时间线项目：模式徽章 + 标题 + 消息数 + 时间 |
| BR5.3 | 单击导航到 `/sessions/:id` (详情页或弹窗) |
| BR5.4 | 按 created_at 倒序 |
| BR5.5 | 滚动加载更多 (limit=50, offset 分页) |

## 6. 数据加载

| 规则 | 描述 |
|------|------|
| BR6.1 | Dashboard 数据 RSC fetch，revalidate=60s |
| BR6.2 | Session 列表初始加载 50 条 |
| BR6.3 | 告警数据实时，不缓存 |
