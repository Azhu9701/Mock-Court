# Business Logic Model — F1: App Shell

## 组件树 (Component Tree)

```
RootLayout (app/layout.tsx)
└── Providers
    ├── ThemeProvider        — Q4: A (React Context)
    ├── SidebarProvider      — 侧边栏折叠状态
    └── ShellLayout
        ├── Sidebar
        │   ├── Logo           — 万民幡 brand
        │   ├── NavGroup       — 主导航 (魂览/附体/会话/仪表盘)
        │   └── NavFooter      — 主题切换 + 折叠按钮
        ├── Header
        │   ├── Breadcrumb     — 自动面包屑
        │   └── QuickActions   — 快捷新建附体按钮
        └── Main
            └── {children}     — Next.js page slot
```

## 路由结构 (Next.js App Router)

```
app/
├── layout.tsx              # RootLayout + Providers
├── page.tsx                # "/" → redirect to /souls
├── souls/
│   ├── page.tsx            # /souls (F2)
│   └── [name]/
│       └── page.tsx        # /souls/[name] (F2)
├── possess/
│   ├── page.tsx            # /possess (F3)
│   └── [sessionId]/
│       └── page.tsx        # /possess/[sessionId] (F3)
├── sessions/
│   ├── page.tsx            # /sessions (F4)
│   └── [id]/
│       └── page.tsx        # /sessions/[id] (F4)
└── analytics/
    └── page.tsx            # /analytics (F4)
```

## 数据流

### Theme 切换流程 (Q3: C)

```
1. 用户点击主题切换按钮
2. ThemeContext.toggleTheme() → update state
3. localStorage.setItem('theme', newValue)
4. <html class="dark"> / <html class="light"> 切换
5. Tailwind dark: variant 自动响应
6. 初始加载: localStorage → system preference fallback
```

### Sidebar 折叠流程

```
1. 用户点击折叠按钮
2. SidebarContext.toggle()
3. Sidebar width: 240px ↔ 0px (transition 200ms)
4. MainContent margin-left 同步调整
5. 移动端 (<768px): 折叠时 Sidebar 隐藏, MainContent full-width
6. localStorage.setItem('sidebar-collapsed', value)
```

### 导航激活状态

```
1. 使用 Next.js usePathname() 获取当前路径
2. 匹配 NavConfig 中的 href 前缀
3. 匹配项高亮 (bg-primary/10 text-primary)
4. 父级自动展开子菜单
```

## Provider 组合

```typescript
// app/providers.tsx
function Providers({ children }) {
  return (
    <ThemeProvider defaultTheme="system">
      <SidebarProvider>
        {children}
      </SidebarProvider>
    </ThemeProvider>
  );
}
```

## API 集成

F1 App Shell 不直接调用任何 API 端点（纯布局层）。子页面 (F2-F4) 各自调用 B6 API。
