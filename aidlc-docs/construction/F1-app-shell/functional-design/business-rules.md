# Business Rules — F1: App Shell

## 1. 布局规则 (Q2: A)

| 规则 | 描述 |
|------|------|
| BR1.1 | 桌面端 (≥1024px): 侧边栏固定在左侧，主内容区居右 |
| BR1.2 | 平板端 (768-1023px): 侧边栏默认折叠，点击展开为浮层 |
| BR1.3 | 移动端 (<768px): 侧边栏为底部抽屉 (Sheet) |
| BR1.4 | 侧边栏展开宽度 240px，折叠后为 0（不是 mini 模式） |
| BR1.5 | 主内容区最大宽度 1280px，居中显示 |

## 2. 主题规则 (Q3: C)

| 规则 | 描述 |
|------|------|
| BR2.1 | 支持三种模式：light / dark / system |
| BR2.2 | system 模式跟随操作系统偏好（prefers-color-scheme media query） |
| BR2.3 | 主题状态持久化到 localStorage key = `theme` |
| BR2.4 | 首次加载默认使用 system 模式 |
| BR2.5 | 主题切换无闪烁：在 `<head>` 中内联 script 读取 localStorage 设置 `document.documentElement.className` |
| BR2.6 | Tailwind `dark:` variant 用于暗色样式 |

## 3. 导航规则 (Q5: B)

| 规则 | 描述 |
|------|------|
| BR3.1 | 主导航分组：「核心」(魂览/附体) + 「回顾」(会话历史/仪表盘) |
| BR3.2 | 当前激活的导航项高亮（基于 usePathname 前缀匹配） |
| BR3.3 | 附体入口始终保持可见（核心功能） |
| BR3.4 | 导航图标使用 Lucide React 图标库 |
| BR3.5 | 侧边栏折叠时仅显示图标（tooltip 显示文字） |

## 4. 响应式规则

| 规则 | 描述 |
|------|------|
| BR4.1 | 断点：sm=640, md=768, lg=1024, xl=1280 |
| BR4.2 | 侧边栏折叠按钮在 lg 以下自动隐藏（无侧边栏可折叠） |
| BR4.3 | 移动端 Header 显示汉堡菜单按钮 |
| BR4.4 | 内容区 padding：桌面 32px, 平板 24px, 手机 16px |

## 5. 性能规则

| 规则 | 描述 |
|------|------|
| BR5.1 | 侧边栏和 Header 使用 `position: sticky` 不随内容滚动 |
| BR5.2 | 导航配置为静态数据（不请求 API），零网络延迟 |
| BR5.3 | 主题切换避免 re-render 整个树（Context split） |
| BR5.4 | 图标按需导入（`import { Home } from 'lucide-react'`），不打包全量 |

## 6. 可访问性规则

| 规则 | 描述 |
|------|------|
| BR6.1 | 侧边栏 `<nav>` 使用 `aria-label="主导航"` |
| BR6.2 | 主题切换按钮使用 `aria-label="切换主题"` |
| BR6.3 | 所有交互元素添加 `data-testid`（自动化测试友好） |
| BR6.4 | 支持键盘导航（Tab / Enter / Escape） |
