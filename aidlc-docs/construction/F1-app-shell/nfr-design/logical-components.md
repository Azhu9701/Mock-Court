# Logical Components — F1: App Shell

## Component Architecture

```
nextjs/
├── app/
│   ├── layout.tsx              # RootLayout (html > body > Providers > ShellLayout)
│   ├── page.tsx                # "/" → redirect to /souls
│   ├── globals.css             # Tailwind directives + system font
│   └── souls/                  # F2 placeholder
│   │   └── page.tsx            # "魂览 — 敬请期待"
│   ├── possess/                # F3 placeholder
│   │   └── page.tsx            # "附体 — 敬请期待"
│   ├── sessions/               # F4 placeholder
│   │   └── page.tsx            # "会话历史 — 敬请期待"
│   └── analytics/              # F4 placeholder
│       └── page.tsx            # "仪表盘 — 敬请期待"
├── components/
│   ├── providers.tsx           # ThemeProvider + SidebarProvider
│   ├── shell-layout.tsx        # 主布局容器
│   ├── sidebar.tsx             # 侧边栏容器
│   ├── sidebar-logo.tsx        # Logo
│   ├── sidebar-nav.tsx         # 导航组 + NavItem 渲染
│   ├── sidebar-footer.tsx      # 底部操作区
│   ├── nav-item.tsx            # 单个导航项
│   ├── header.tsx              # 顶部栏
│   ├── breadcrumb.tsx          # 面包屑导航
│   ├── theme-toggle.tsx        # 主题切换按钮
│   ├── mobile-menu-button.tsx  # 移动端菜单按钮
│   └── quick-actions.tsx       # 快捷操作按钮
├── contexts/
│   └── sidebar-context.tsx     # SidebarContext + Provider + useSidebar hook
├── config/
│   └── nav.ts                  # navConfig 静态配置数据
├── lib/
│   └── utils.ts                # cn() helper (clsx + twMerge)
├── next.config.ts              # Next.js 配置
├── tailwind.config.ts          # Tailwind 配置
├── tsconfig.json               # TypeScript 配置
└── package.json                # 依赖声明
```

## Component Dependencies

```
RootLayout
├── next-themes (ThemeProvider)
└── SidebarProvider (contexts/sidebar-context)
    └── ShellLayout
        ├── Sidebar
        │   ├── SidebarLogo → Link
        │   ├── SidebarNav → NavItem[] → Link
        │   └── SidebarFooter
        │       ├── ThemeToggle → useTheme()
        │       └── CollapseButton
        ├── Header
        │   ├── MobileMenuButton → useSidebar()
        │   ├── Breadcrumb → usePathname()
        │   └── QuickActions → Link
        └── Main → {children}
```

## Key Interfaces

| Component | Key Hook | Description |
|-----------|----------|-------------|
| RootLayout | — | HTML shell + Providers |
| Providers | — | Context 注入层 |
| ShellLayout | — | flex 布局容器 |
| Sidebar | `useSidebar()`, `usePathname()` | 导航侧边栏 |
| ThemeToggle | `useTheme()` | 暗/亮切换 |
| Breadcrumb | `usePathname()` | 路径面包屑 |
| NavItem | `usePathname()` | 激活状态检测 |
| QuickActions | — | 快捷入口链接 |

## External Dependencies

```json
{
  "dependencies": {
    "next": "^15.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "next-themes": "^0.4.0",
    "lucide-react": "^0.460.0",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.5.0"
  },
  "devDependencies": {
    "typescript": "^5.0.0",
    "tailwindcss": "^4.0.0",
    "@tailwindcss/postcss": "^4.0.0"
  }
}
```
