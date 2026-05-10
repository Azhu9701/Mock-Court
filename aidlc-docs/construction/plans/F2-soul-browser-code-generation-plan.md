# Code Generation Plan — F2: Soul Browser

## Plan Steps

### Dependencies
- [ ] Step 1: 安装依赖 `pnpm add recharts react-hook-form zod @hookform/resolvers` + shadcn 组件

### Supporting Files
- [ ] Step 2: 创建 `config/soul-filter.ts` — ismism 解析 + 距离计算 + 常量
- [ ] Step 3: 创建 API helper `lib/api.ts` — fetch wrapper for B6 API

### Shared Components
- [ ] Step 4: 创建 `components/grade-badge.tsx`
- [ ] Step 5: 创建 `components/soul-card.tsx` + `soul-card-grid.tsx`
- [ ] Step 6: 创建 `components/soul-filter-bar.tsx` (Client)
- [ ] Step 7: 创建 `components/ismism-radar.tsx` (Client, Recharts)
- [ ] Step 8: 创建 `components/soul-prompt.tsx` + `effectiveness-panel.tsx` + `practice-observations.tsx`
- [ ] Step 9: 创建 `components/edit-soul-dialog.tsx` + `delete-soul-confirm-dialog.tsx` + `summon-button.tsx`

### Pages
- [ ] Step 10: 创建 `app/souls/page.tsx` + `loading.tsx` + `error.tsx`
- [ ] Step 11: 创建 `app/souls/[name]/page.tsx` + `loading.tsx` + `not-found.tsx`

### Verification
- [ ] Step 12: `pnpm build` — 0 errors
