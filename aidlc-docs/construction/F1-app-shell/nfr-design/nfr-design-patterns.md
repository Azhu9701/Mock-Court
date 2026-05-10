# NFR Design Patterns — F1: App Shell

## Pattern 1: Theme FOUC Prevention (闪烁防护)

**问题**: next-themes 默认在客户端 hydration 后设置 theme，导致暗色/亮色闪烁。

**方案**: 在 `<head>` 中内联同步 script，在 DOM 构建前设置 `document.documentElement.className`。

```html
<!-- app/layout.tsx -->
<script dangerouslySetInnerHTML={{
  __html: `
    (function() {
      try {
        var t = localStorage.getItem('theme');
        var d = document.documentElement;
        if (t === 'dark' || (!t && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
          d.classList.add('dark');
        } else {
          d.classList.remove('dark');
        }
      } catch(e) {}
    })();
  `
}} />
```

**关键**: 必须在 `<body>` 渲染前执行，使用同步 IIFE，try-catch 包裹防止异常阻断渲染。

## Pattern 2: Responsive Sidebar (响应式侧边栏)

**问题**: 桌面/平板/手机需要不同的侧边栏行为。

**方案**: CSS 断点 + React state + conditional rendering。

```
Desktop (≥1024px):
  Sidebar: fixed left, w-60, translate-x based on collapsed state
  Main: ml-60 (shifts with sidebar)

Tablet (768-1023px):
  Sidebar: collapsed by default, overlay on toggle
  Main: ml-0 full width

Mobile (<768px):
  Sidebar: Sheet component (shadcn/ui)
  Hamburger button in Header
```

**状态管理**:
```typescript
const [collapsed, setCollapsed] = useState(() => {
  if (typeof window === 'undefined') return false;
  return localStorage.getItem('sidebar-collapsed') === 'true';
});
const [mobileOpen, setMobileOpen] = useState(false);
```

## Pattern 3: Context Split (上下文分离)

**问题**: 单一大 Context 更新会导致整个组件树 re-render。

**方案**: ThemeProvider 和 SidebarProvider 分离，各自独立更新。

```typescript
// ThemeProvider (from next-themes) — 仅主题相关组件订阅
// SidebarContext — 仅侧边栏相关组件订阅
<ThemeProvider attribute="class" defaultTheme="system" enableSystem>
  <SidebarContext.Provider value={sidebarValue}>
    {children}
  </SidebarContext.Provider>
</ThemeProvider>
```

**关键**: Theme toggle button 只订阅 ThemeContext, Sidebar collapse button 只订阅 SidebarContext。互不触发对方的重渲染。

## Pattern 4: Nav Active State (导航激活检测)

**问题**: 基于当前路径高亮对应导航项。

**方案**: `usePathname()` + 前缀匹配 + 子路由展开。

```typescript
function isActive(item: NavItem, pathname: string): boolean {
  if (item.href === '/') return pathname === '/';
  return pathname.startsWith(item.href);
}

// 子路由自动展开
function shouldExpand(item: NavItem, pathname: string): boolean {
  return item.children?.some(child => isActive(child, pathname)) ?? false;
}
```

## Pattern 5: Sticky Layout (固定布局)

**问题**: 长内容滚动时保持 Header 和 Sidebar 可见。

**方案**: CSS `position: sticky` + flexbox `overflow-hidden` on body。

```css
/* body: h-screen overflow-hidden */
/* Sidebar: fixed, h-screen */
/* Header: sticky top-0 */
/* Main: flex-1 overflow-y-auto */
```

**关键**: 使用 `flex-1 overflow-y-auto` 让主内容区独立滚动，不触发整体页面滚动。

## Pattern 6: Icon Loading (按需图标)

**问题**: 图标库（Lucide）包含 1000+ 图标，全量引入导致 bundle 膨胀。

**方案**: 静态映射表 + 按需 import。

```typescript
import { Home, Users, Brain, History, BarChart3 } from 'lucide-react';

const iconMap: Record<string, React.ComponentType<{className?: string}>> = {
  home: Home,
  users: Users,
  brain: Brain,
  history: History,
  'bar-chart': BarChart3,
};

function NavIcon({ name }: { name: string }) {
  const Icon = iconMap[name];
  return Icon ? <Icon className="h-4 w-4" /> : null;
}
```

Tree-shaking 自动移除未使用的图标。
