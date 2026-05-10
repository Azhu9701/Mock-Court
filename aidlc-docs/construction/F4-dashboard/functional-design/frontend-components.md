# Frontend Components — F4: Dashboard

## Key Components

### StatCard

```typescript
interface StatCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  icon: string;
}
```

- 4 个并排卡片，响应式 2-4 列
- 图标 + 数值 + 标题
- `data-testid="stat-card-{key}"`

### ModeBarChart

```typescript
interface ModeBarChartProps {
  data: Record<string, number>;
}
```

- Recharts `<BarChart>` + `<Bar>` + `<XAxis>` + `<YAxis>`
- 6 色柱状图，标签中文
- Client Component
- `data-testid="mode-bar-chart"`

### SoulEffectivenessTable

```typescript
interface SoulEffectivenessTableProps {
  stats: SoulCallStats[];
}
```

- 表格：魂名 | 调用次数 | 有效率进度条 | 有效/部分/无效
- 按 effective_rate 降序
- `data-testid="effectiveness-table"`

### AlertPanel

```typescript
interface AlertPanelProps {
  unsummoned: SoulAlert[];
  lowEffectiveness: BoundaryReview[];
}
```

- 两列：左 未召唤告警 (红)，右 低效告警 (黄)
- 无告警 → "✓ 一切正常"
- `data-testid="alert-panel"`

### SessionTimeline

```typescript
interface SessionTimelineProps {
  sessions: SessionSummary[];
}
```

- 按日期分组渲染
- TimelineDateGroup: 日期标题 + TimelineItem[]
- TimelineItem: 模式徽章 + 标题 + 消息数 + 时间
- `data-testid="session-timeline"`

### SessionDetailDialog

```typescript
interface SessionDetailDialogProps {
  session: SessionDetail | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}
```

- Modal 显示 session.messages 列表
- 各消息角色标记 (user/soul/synthesis)
- `data-testid="session-detail-dialog"`

## Files

```
app/analytics/
└── page.tsx                    # Dashboard (RSC)

app/sessions/
├── page.tsx                    # 会话历史 (RSC)
└── [id]/
    └── page.tsx                # 会话详情

components/
├── stat-card.tsx
├── mode-bar-chart.tsx
├── soul-effectiveness-table.tsx
├── alert-panel.tsx
├── session-timeline.tsx
└── session-detail-dialog.tsx
```
