# NFR Requirements — F1: App Shell

## Performance (Q4: A — Lighthouse 90+)

| 指标 | 目标 | 策略 |
|------|------|------|
| FCP (First Contentful Paint) | < 1.5s | 无外部字体，Tailwind purged CSS |
| LCP (Largest Contentful Paint) | < 2.5s | 静态 Shell 无数据请求 |
| TBT (Total Blocking Time) | < 200ms | 最小 JS bundle (shadcn 按需打包) |
| CLS (Cumulative Layout Shift) | < 0.1 | sticky sidebar + 固定 header 高度 |
| Accessibility | 90+ | aria-label + keyboard nav |
| Best Practices | 90+ | HTTPS, no deprecated APIs |
| SEO | 90+ | 静态 title/description (本地使用非关键) |

## 浏览器支持

| 浏览器 | 最低版本 |
|--------|---------|
| Chrome | 90+ |
| Safari | 15+ |
| Firefox | 90+ |
| Edge | 90+ |

## 响应式

| 断点 | 宽度 | 行为 |
|------|------|------|
| Desktop | ≥1024px | 侧边栏展开 (240px) + 主内容区 |
| Tablet | 768-1023px | 侧边栏默认折叠 |
| Mobile | <768px | Sheet 式抽屉菜单 |

## 构建

| 指标 | 目标 |
|------|------|
| 初始 JS bundle (Shell) | < 100KB gzipped |
| 初始 CSS bundle | < 20KB gzipped |
| next build 时间 | < 30s |
| Dev server 启动 | < 5s |

## 可靠性

| 要求 | 描述 |
|------|------|
| 主题持久化 | localStorage + inline script 防闪烁 |
| 侧边栏状态持久化 | localStorage restore |
| 离线可用 | 不需要（本地 API 依赖） |
| 优雅降级 | JS 禁用时显示基础 HTML 导航 |
