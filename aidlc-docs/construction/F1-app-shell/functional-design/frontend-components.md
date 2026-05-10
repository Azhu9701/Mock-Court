# Frontend Components — F1: App Shell

## Component Hierarchy

```
RootLayout
└── Body
    └── Providers (ThemeProvider + SidebarProvider)
        └── ShellLayout
            ├── Sidebar
            │   ├── SidebarLogo
            │   ├── SidebarNav
            │   │   └── NavItem[] (recursive)
            │   └── SidebarFooter
            │       ├── ThemeToggle
            │       └── CollapseButton
            ├── Header
            │   ├── Breadcrumb
            │   ├── MobileMenuButton
            │   └── QuickActions
            │       └── NewPossessionButton
            └── Main
                └── {children}
```

## Component Definitions

### RootLayout

```typescript
// app/layout.tsx
export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="zh-CN" suppressHydrationWarning>
      <head>
        {/* 防闪烁 inline script */}
        <Script id="theme-init" strategy="beforeInteractive">
          {themeInitScript}
        </Script>
      </head>
      <body className="antialiased">
        <Providers>
          <ShellLayout>{children}</ShellLayout>
        </Providers>
      </body>
    </html>
  );
}
```

**Props**: 无 (Next.js 自动注入 children)

### Providers

```typescript
// components/providers.tsx
function Providers({ children }) {
  return (
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
      <SidebarProvider>
        {children}
      </SidebarProvider>
    </ThemeProvider>
  );
}
```

### ShellLayout

```typescript
// components/shell-layout.tsx
function ShellLayout({ children }) {
  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar />
      <div className="flex flex-1 flex-col overflow-hidden">
        <Header />
        <main className="flex-1 overflow-y-auto p-4 lg:p-8">
          {children}
        </main>
      </div>
    </div>
  );
}
```

**State**: 无自有状态，从 Context 读取

### Sidebar

```typescript
// components/sidebar.tsx
function Sidebar() {
  const { collapsed } = useSidebar();
  const pathname = usePathname();

  return (
    <aside
      data-testid="app-sidebar"
      className={cn(
        "fixed inset-y-0 left-0 z-30 flex w-60 flex-col border-r bg-background transition-all duration-200",
        collapsed && "-translate-x-full lg:translate-x-0 lg:w-0 lg:overflow-hidden"
      )}
    >
      <SidebarLogo />
      <SidebarNav items={navConfig.groups} currentPath={pathname} />
      <SidebarFooter />
    </aside>
  );
}
```

**Props**: 无 (从 Context + usePathname 读取)

### SidebarLogo

```typescript
// components/sidebar-logo.tsx
function SidebarLogo() {
  return (
    <Link href="/" className="flex h-14 items-center gap-2 border-b px-4">
      <Flag className="h-6 w-6 text-primary" />
      <span className="text-lg font-bold">万民幡</span>
    </Link>
  );
}
```

**Props**: 无

### SidebarNav

```typescript
// components/sidebar-nav.tsx
function SidebarNav({ items, currentPath }: SidebarNavProps) {
  return (
    <nav className="flex-1 overflow-y-auto p-2" aria-label="主导航">
      {items.map((group, i) => (
        <div key={i} className="mb-4">
          <h3 className="mb-1 px-3 text-xs font-semibold text-muted-foreground">
            {group.label}
          </h3>
          {group.items.map((item) => (
            <NavItem key={item.key} item={item} active={currentPath.startsWith(item.href)} />
          ))}
        </div>
      ))}
    </nav>
  );
}
```

**Props**:
```typescript
interface SidebarNavProps {
  items: NavGroup[];
  currentPath: string;
}
```

### NavItem

```typescript
// components/nav-item.tsx
function NavItem({ item, active }: { item: NavItem; active: boolean }) {
  const Icon = dynamicIcon(item.icon);
  return (
    <Link
      href={item.href}
      data-testid={`nav-${item.key}`}
      className={cn(
        "flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors",
        active
          ? "bg-primary/10 text-primary font-medium"
          : "text-muted-foreground hover:bg-muted hover:text-foreground"
      )}
    >
      <Icon className="h-4 w-4" />
      <span>{item.label}</span>
      {item.badge != null && <Badge>{item.badge}</Badge>}
    </Link>
  );
}
```

### SidebarFooter

```typescript
// components/sidebar-footer.tsx
function SidebarFooter() {
  return (
    <div className="flex items-center justify-between border-t p-2">
      <ThemeToggle />
      <CollapseButton />
    </div>
  );
}
```

### ThemeToggle

```typescript
// components/theme-toggle.tsx
function ThemeToggle() {
  const { theme, setTheme } = useTheme();
  return (
    <Button
      variant="ghost"
      size="icon"
      data-testid="theme-toggle"
      onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
      aria-label="切换主题"
    >
      <Sun className="h-4 w-4 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
      <Moon className="absolute h-4 w-4 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
    </Button>
  );
}
```

### Header

```typescript
// components/header.tsx
function Header() {
  return (
    <header className="flex h-14 items-center gap-4 border-b bg-background px-4 lg:px-8">
      <MobileMenuButton />
      <Breadcrumb />
      <div className="flex-1" />
      <QuickActions />
    </header>
  );
}
```

**Props**: 无 (从 Context 读取面包屑)

### Breadcrumb

```typescript
// components/breadcrumb.tsx
function Breadcrumb() {
  const pathname = usePathname();
  const segments = pathname.split('/').filter(Boolean);
  return (
    <nav data-testid="breadcrumb" aria-label="面包屑">
      <ol className="flex items-center gap-1 text-sm text-muted-foreground">
        <li><Link href="/">首页</Link></li>
        {segments.map((seg, i) => (
          <li key={i} className="flex items-center gap-1">
            <ChevronRight className="h-3 w-3" />
            <span className="capitalize">{seg}</span>
          </li>
        ))}
      </ol>
    </nav>
  );
}
```

### QuickActions

```typescript
// components/quick-actions.tsx
function QuickActions() {
  return (
    <Link href="/possess" data-testid="new-possession-btn">
      <Button size="sm">
        <Plus className="mr-1 h-4 w-4" />
        开始附体
      </Button>
    </Link>
  );
}
```

## Context Definitions

### SidebarContext

```typescript
// contexts/sidebar-context.tsx
interface SidebarContextType {
  collapsed: boolean;
  toggle: () => void;
  mobileOpen: boolean;
  setMobileOpen: (open: boolean) => void;
}

function SidebarProvider({ children }: { children: React.ReactNode }) {
  const [collapsed, setCollapsed] = useState(() => {
    if (typeof window === 'undefined') return false;
    return localStorage.getItem('sidebar-collapsed') === 'true';
  });

  const toggle = useCallback(() => {
    setCollapsed(prev => {
      localStorage.setItem('sidebar-collapsed', String(!prev));
      return !prev;
    });
  }, []);

  // ...
}
```

### ThemeContext (使用 next-themes)

```typescript
// next-themes 的 ThemeProvider 已提供 useTheme() hook:
// { theme, setTheme, resolvedTheme, systemTheme }
// 无需自定义 Context
```

## State Summary

| 状态 | 来源 | 持久化 |
|------|------|--------|
| theme | next-themes ThemeProvider | localStorage |
| sidebar.collapsed | SidebarContext | localStorage |
| sidebar.mobileOpen | SidebarContext | 无 (session only) |
| currentPath | usePathname() | URL (reactively) |
| navConfig | 静态常量 | 无 |

## 图标映射 (Lucide React)

```typescript
const iconMap: Record<string, LucideIcon> = {
  home: Home,
  users: Users,
  brain: Brain,
  history: History,
  'bar-chart': BarChart3,
  settings: Settings,
  // ...
};

function dynamicIcon(name: string): LucideIcon {
  return iconMap[name] || Circle;
}
```
