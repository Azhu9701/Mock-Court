# Frontend Components — F2: Soul Browser

## Component Hierarchy

```
SoulListPage (RSC)
├── PageHeader ("魂览")
└── SoulListView (Client)
    ├── SoulFilterBar
    │   ├── GradeSelect (Select: S/A/B/C/D/全部)
    │   ├── IsmismSliders (4 x Range Slider)
    │   └── SearchInput (Input + debounce)
    ├── SoulCount ("共 N 个魂")
    ├── SoulCardGrid
    │   └── SoulCard[]
    └── SoulGridSkeleton (loading)

SoulDetailPage (RSC)
├── PageHeader + Breadcrumb
├── SoulProfileCard
│   ├── GradeBadge
│   ├── SoulName + IsmismCode
│   └── DomainTags + FieldBadge
├── IsmismRadar (SVG)
├── SoulPrompt (Collapsible)
├── EffectivenessPanel (Client)
│   └── EffectivenessBar (progress bar)
├── PracticeObservations (Client)
│   └── ObservationCard[]
└── SoulActionBar
    ├── SummonButton
    ├── EditSoulDialog (Modal)
    └── DeleteConfirmDialog
```

## Component Definitions

### SoulCard

```typescript
interface SoulCardProps {
  soul: SoulListEntry;
}
```

- 点击跳转到 `/souls/${soul.name}`
- `data-testid="soul-card-{name}"`
- 显示: GradeBadge, name, ismism_code, field tag, summon_count

### GradeBadge

```typescript
interface GradeBadgeProps {
  grade: SoulGrade;
}
```

- 颜色映射: S=gold, A=blue, B=green, C=gray, D=red
- `data-testid="grade-badge"`
- 圆形徽章，显示品级字母

### SoulFilterBar

```typescript
interface SoulFilterBarProps {
  initialGrade: string | null;
  initialIsmism: IsmismCode | null;
  initialQuery: string;
  onFilterChange: (filters: FilterState) => void;
}
```

- GradeSelect: shadcn/ui Select 组件
- IsmismSliders: 4 个 range input (1-4)，步长 0.5
- SearchInput: shadcn/ui Input + debounce 300ms
- URL searchParams 同步 (nuqs 或手动)

### IsmismRadar

```typescript
interface IsmismRadarProps {
  ismismCode: string; // "1-2-3-3"
}
```

- 纯 SVG 实现（无第三方图表库）
- 四轴：field(领域), ontology(本体论), epistemology(认识论), teleology(目的论)
- 值域 1-4，雷达点 + 填充
- `data-testid="ismism-radar"`

### SoulPrompt

```typescript
interface SoulPromptProps {
  prompt: string;
}
```

- 默认显示前 200 字符 + 省略号
- "展开"按钮显示全文
- 使用 Markdown 渲染（或纯 pre-wrap）

### EffectivenessPanel

```typescript
interface EffectivenessPanelProps {
  trend: EffectivenessTrend;
}
```

- 三段进度条: effective (green) / partial (yellow) / invalid (red)
- 百分比标注
- 总调用次数显示
- `data-testid="effectiveness-panel"`

### PracticeObservations

```typescript
interface PracticeObservationsProps {
  observations: PracticeObservation[];
}
```

- 默认显示 5 条，"展开全部"按钮
- ObservationCard: date + revision_type badge + observation text
- `data-testid="practice-observations"`

### EditSoulDialog

```typescript
interface EditSoulDialogProps {
  soul: SoulProfile;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSaved: () => void;  // router.refresh()
}
```

- shadcn/ui Dialog + Form
- 可编辑字段: grade, ismism_code, field, domains, tags, summon_prompt
- PUT `/api/v1/souls/:name`
- 成功后调用 `onSaved()` 刷新

### DeleteConfirmDialog

```typescript
interface DeleteConfirmDialogProps {
  soulName: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDeleted: () => void;  // router.push('/souls')
}
```

- DELETE `/api/v1/souls/:name`
- 确认文字: "魂曰：[name] 将被散离，此操作不可撤回"
- 成功后跳转到 /souls

### SummonButton

```typescript
interface SummonButtonProps {
  soulName: string;
}
```

- shadcn/ui Button + Play icon
- href = `/possess?preset=single&souls[]=${soulName}`
- `data-testid="summon-btn-{name}"`

## Files Created

```
app/souls/
├── page.tsx                    # SoulListPage (RSC)
├── loading.tsx                 # Skeleton grid
├── error.tsx                   # Error boundary
└── [name]/
    ├── page.tsx                # SoulDetailPage (RSC)
    ├── loading.tsx             # Skeleton
    └── not-found.tsx           # 404

components/
├── soul-card.tsx
├── soul-card-grid.tsx
├── soul-filter-bar.tsx
├── grade-badge.tsx
├── grade-select.tsx
├── ismism-sliders.tsx
├── search-input.tsx
├── soul-profile-card.tsx
├── ismism-radar.tsx
├── soul-prompt.tsx
├── effectiveness-panel.tsx
├── practice-observations.tsx
├── edit-soul-dialog.tsx
├── delete-soul-confirm-dialog.tsx
└── summon-button.tsx

config/
└── soul-filter.ts              # ismism 常量 + 距离计算 helper
```
