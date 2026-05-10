# Domain Entities — F1: App Shell

## Layout Types

### SidebarState — 侧边栏状态

```typescript
interface SidebarState {
  collapsed: boolean;
  width: number;        // 展开: 240px, 折叠: 0px
  mobileOpen: boolean;  // 移动端抽屉开关
}
```

### ThemeState — 主题状态

```typescript
type Theme = 'light' | 'dark' | 'system';

interface ThemeState {
  theme: Theme;          // 当前设置
  resolved: 'light' | 'dark';  // 实际渲染的主题（system 已解析）
}
```

### NavItem — 导航项

```typescript
interface NavItem {
  key: string;
  label: string;
  href: string;
  icon: string;         // Lucide icon name
  badge?: number;       // 角标数字
  children?: NavItem[]; // 子导航
}
```

### NavGroup — 导航配置

```typescript
interface NavConfig {
  groups: {
    label: string;
    items: NavItem[];
  }[];
}
```

## Navigation Configuration (Q5: B — 4 项)

```
魂览     → /souls       (list, search, detail, manage)
附体     → /possess     (mode selection, active sessions)
会话历史 → /sessions    (history list, session replay)
仪表盘   → /analytics   (stats, alerts, effectiveness)
```

## Relations

```
App Shell
├── ThemeProvider (React Context)
│   ├── theme: Theme
│   ├── resolved: 'light' | 'dark'
│   └── toggleTheme()
├── SidebarProvider (React Context)
│   ├── collapsed: boolean
│   ├── toggle()
│   └── mobileOpen: boolean
├── Sidebar
│   ├── NavLogo
│   ├── NavGroups (4 组导航入口)
│   └── NavFooter (主题切换 + 折叠按钮)
├── MainContent
│   ├── Header (面包屑 + 快捷操作)
│   └── Content (Next.js children)
└── MobileDrawer (响应式)
```
