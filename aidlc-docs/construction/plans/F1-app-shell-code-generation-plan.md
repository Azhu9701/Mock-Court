# Code Generation Plan — F1: App Shell

## Unit Context

- **Path**: `nextjs/` — Next.js 15 App Router
- **Type**: 纯前端 shell（布局+导航），无数据请求
- **Dependencies**: next, react, next-themes, lucide-react, shadcn/ui

## Plan Steps

### Project Setup
- [x] Step 1: `pnpm create next-app` 初始化 + 安装 shadcn/ui + next-themes + lucide-react
- [x] Step 2: `nextjs/lib/utils.ts` 已由 shadcn 自动创建
- [x] Step 3: 创建 `nextjs/config/nav.ts` — navConfig 静态配置

### Context & Providers
- [x] Step 4: 创建 `nextjs/contexts/sidebar-context.tsx` — SidebarContext
- [x] Step 5: 创建 `nextjs/components/providers.tsx` — Providers 组合

### Layout Shell
- [x] Step 6: 更新 `nextjs/app/globals.css` — 系统字体 + 主题变量
- [x] Step 7: 更新 `nextjs/app/layout.tsx` — RootLayout + FOUC script
- [x] Step 8: 更新 `nextjs/app/page.tsx` — 首页 redirect → /souls

### Components
- [x] Step 9: 创建 shell 组件: `shell-layout.tsx` + `sidebar.tsx`
- [x] Step 10: 创建导航组件: `sidebar-logo.tsx` + `sidebar-nav.tsx` + `nav-item.tsx` + `sidebar-footer.tsx`
- [x] Step 11: 创建 header 组件: `header.tsx` + `breadcrumb.tsx` + `theme-toggle.tsx` + `mobile-menu-button.tsx` + `quick-actions.tsx`

### Placeholder Pages
- [x] Step 12: 创建占位页面: `souls/`, `possess/`, `sessions/`, `analytics/`

### Verification
- [x] Step 13: `pnpm build` — 0 errors, 6 routes static generated

## Files Created/Modified

| 文件 | 说明 |
|------|------|
| `config/nav.ts` | 导航配置 (4 项 + 图标映射) |
| `contexts/sidebar-context.tsx` | 侧边栏状态 Context |
| `components/providers.tsx` | ThemeProvider + SidebarProvider |
| `components/shell-layout.tsx` | 主布局 (Sidebar + Header + Main) |
| `components/sidebar.tsx` | 侧边栏容器 |
| `components/sidebar-logo.tsx` | 万民幡 Logo |
| `components/sidebar-nav.tsx` | 导航组渲染 |
| `components/nav-item.tsx` | 单个导航项 (active 高亮) |
| `components/sidebar-footer.tsx` | 底部 (主题切换 + 折叠按钮) |
| `components/header.tsx` | 顶部栏 |
| `components/breadcrumb.tsx` | 面包屑导航 |
| `components/theme-toggle.tsx` | 暗/亮切换 |
| `components/mobile-menu-button.tsx` | 移动端菜单按钮 |
| `components/quick-actions.tsx` | 新建附体按钮 |
| `app/globals.css` | 系统字体 + 主题 CSS 变量 (modified) |
| `app/layout.tsx` | RootLayout + FOUC script (modified) |
| `app/page.tsx` | 首页 redirect (modified) |
| `app/souls/page.tsx` | 占位 |
| `app/possess/page.tsx` | 占位 |
| `app/sessions/page.tsx` | 占位 |
| `app/analytics/page.tsx` | 占位 |

**总计**: 21 files, ~300 lines
