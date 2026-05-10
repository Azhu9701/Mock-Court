# Functional Design Plan — F2: Soul Browser

## Plan Steps

- [x] Step 1: 创建 `domain-entities.md` — 页面状态类型
- [x] Step 2: 创建 `business-logic-model.md` — 组件树 + API 调用流程
- [x] Step 3: 创建 `business-rules.md` — 筛选/搜索/CRUD 规则
- [x] Step 4: 创建 `frontend-components.md` — 组件 Props/State

## Design Questions

### Q1: 魂列表展示方式
- [Answer]: B — 卡片网格

### Q2: 搜索方式
- [Answer]: C — 混合（初全量 + 搜索调 API）

### Q3: 魂详情页
- [Answer]: A — 完整 Profile + ismism 雷达图 + 统计 + 实践记录

### Q4: 数据请求策略
- [Answer]: A — Server Components + fetch (RSC)

### Q5: 筛选维度
- [Answer]: C — 品级 + ismism 四维滑块 + 关键词
