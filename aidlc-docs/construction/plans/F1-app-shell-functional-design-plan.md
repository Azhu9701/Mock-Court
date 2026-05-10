# Functional Design Plan — F1: App Shell

## Plan Steps

- [x] Step 1: 创建 `domain-entities.md` — 布局/导航/主题类型
- [x] Step 2: 创建 `business-logic-model.md` — 组件树 + 路由结构
- [x] Step 3: 创建 `business-rules.md` — 布局规则/响应式/主题切换
- [x] Step 4: 创建 `frontend-components.md` — 组件层级 + Props/State

## Design Questions

### Q1: CSS/Styling 方案
- [Answer]: A — Tailwind CSS

### Q2: 布局结构
- [Answer]: A — 侧边栏 + 主内容区

### Q3: 暗色/亮色主题
- [Answer]: C — 暗色/亮色切换，跟随系统偏好

### Q4: 状态管理
- [Answer]: A — React Context

### Q5: 导航入口
- [Answer]: B — 魂览/附体/会话历史/仪表盘 4 项
