# Tech Stack Decisions — F1: App Shell

## Core Stack

| 技术 | 版本 | 用途 |
|------|------|------|
| Next.js | 15 (Q1: A) | App Router 框架 |
| React | 19 | UI 组件 |
| TypeScript | 5.x (宽松模式, Q3: B) | 类型系统 |
| pnpm | 9 (Q1: A) | 包管理器 |
| Tailwind CSS | 4 | 样式系统 (Q1: A) |

## UI & Components

| 库 | 用途 |
|----|------|
| shadcn/ui (Q2: A) | Button, Sheet, Tooltip 等基础组件 |
| Lucide React (Q5: A) | 图标库 (按需导入) |
| next-themes | 暗/亮主题切换 (ThemeProvider) |
| class-variance-authority | 组件 variant 管理 (shadcn 依赖) |
| clsx + tailwind-merge | className 条件合并 (shadcn 依赖) |

## 不引入

| 候选 | 理由 |
|------|------|
| Ant Design / MUI | Q2: A — 选择 shadcn/ui |
| Google Fonts | Q5: A — 使用系统字体栈 |
| Zustand / Redux | Q4 (Functional): A — React Context 已足够 |
| framer-motion | F1 Shell 无动画需求 (后续 F3 可加) |
| React Query / SWR | F1 无数据请求 |

## 系统字体栈 (Q5: A)

```css
font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
  "Helvetica Neue", Arial, "Noto Sans SC", sans-serif;
```

## Project Init Command

```bash
pnpm create next-app@latest nextjs --typescript --tailwind --eslint --app --src-dir --import-alias "@/*"
cd nextjs
pnpm dlx shadcn@latest init
```
