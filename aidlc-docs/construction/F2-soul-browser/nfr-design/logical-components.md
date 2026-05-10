# Logical Components — F2: Soul Browser

## File Structure

```
app/souls/
├── page.tsx                    # SoulListPage (RSC + Suspense)
├── loading.tsx                 # Skeleton grid fallback
├── error.tsx                   # ErrorBoundary + retry
└── [name]/
    ├── page.tsx                # SoulDetailPage (RSC, parallel fetch)
    ├── loading.tsx             # Detail skeleton
    └── not-found.tsx           # 404 page

components/
├── soul-card.tsx               # 魂卡片
├── soul-card-grid.tsx          # 卡片网格容器
├── soul-filter-bar.tsx         # 筛选栏 (Client)
├── soul-filter-bar.tsx         # (品级 + ismism + 搜索)
├── grade-badge.tsx             # 品级徽章
├── ismism-radar.tsx            # Recharts RadarChart (Client)
├── soul-prompt.tsx             # 召唤词折叠展开
├── effectiveness-panel.tsx     # 有效性趋势 (Client)
├── practice-observations.tsx   # 实践记录 (Client)
├── edit-soul-dialog.tsx        # 编辑表单 (shadcn Form)
├── delete-soul-confirm-dialog.tsx
└── summon-button.tsx           # 召唤按钮

config/
└── soul-filter.ts              # ismism 常量 + 距离计算
```

## Component Dependencies

```
SoulListPage (RSC)
└── Suspense fallback={SoulGridSkeleton}
    └── SoulListView (Client)
        ├── SoulFilterBar
        │   ├── Select (grade)
        │   ├── IsmismSliders (4x range)
        │   └── Input (search, debounce)
        └── SoulCardGrid
            └── SoulCard[] → Link

SoulDetailPage (RSC)
├── SoulProfileCard (Server)
├── Suspense fallback={Skeleton}
│   ├── IsmismRadar (Client, Recharts)
│   ├── EffectivenessPanel (Client)
│   └── PracticeObservations (Client)
├── SoulPrompt (Server)
└── SoulActionBar (Client)
    ├── SummonButton → Link
    ├── EditSoulDialog (shadcn Form + RHF + Zod)
    └── DeleteConfirmDialog
```

## External Dependencies

```
F2 Soul Browser
├── next (RSC + App Router)
├── recharts (RadarChart, PolarGrid, PolarAngleAxis, PolarRadiusAxis, Radar)
├── react-hook-form (useForm, FormProvider)
├── zod + @hookform/resolvers (schema validation)
├── shadcn/ui (Select, Dialog, Input, Form, Skeleton, Badge, Button)
└── lucide-react (Search, Filter, Edit, Trash, Play)
```
