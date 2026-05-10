# Tech Stack Decisions — F2: Soul Browser

## New Dependencies

| 库 | 用途 | 决策依据 |
|----|------|----------|
| recharts | IsmismRadar 雷达图 (Q1: B) | React 原生，RadarChart 内置 |
| react-hook-form | 表单状态管理 (Q2: C) | shadcn/ui Form 依赖 |
| zod | Schema 验证 (Q2: C) | 类型安全 + 运行时校验 |
| @hookform/resolvers | RHF + Zod 桥接 (Q2: C) | shadcn/ui Form 依赖 |

## Inherited from F1

| 库 | 用途 |
|----|------|
| next 15 | App Router + RSC |
| tailwindcss 4 | 样式 |
| shadcn/ui | Button, Select, Dialog, Input, Form, Skeleton |
| lucide-react | 图标 |
| clsx + tailwind-merge | cn() helper |

## shadcn/ui 组件（需添加）

```bash
pnpm dlx shadcn@latest add select dialog input form skeleton badge
```

## 不引入

| 候选 | 理由 |
|------|------|
| D3.js | Q1: B — 选择 Recharts |
| TanStack Query | Q4 (Functional): A — 使用 RSC fetch |
| nuqs | URL searchParams 管理 — 后续可加，当前手动 |

## Project Init

```bash
cd nextjs
pnpm add recharts react-hook-form zod @hookform/resolvers
pnpm dlx shadcn@latest add select dialog input form skeleton badge
```
