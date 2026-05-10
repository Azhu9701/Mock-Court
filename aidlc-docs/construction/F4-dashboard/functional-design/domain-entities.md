# Domain Entities — F4: Dashboard

## Dashboard State

```typescript
interface DashboardState {
  summonStats: SummonStats | null;
  modeDistribution: Record<string, number>;
  unsummonedAlerts: SoulAlert[];
  lowEffectiveness: BoundaryReview[];
  isLoading: boolean;
}
```

## API Types

```typescript
// /api/v1/analytics/summon-stats
interface SummonStats {
  total_calls: number;
  unique_souls_called: number;
  total_souls_available: number;
  by_mode: Record<string, number>;
  by_soul: SoulCallStats[];
}

interface SoulCallStats {
  soul_name: string;
  call_count: number;
  effective_count: number;
  partial_count: number;
  invalid_count: number;
}

// /api/v1/analytics/unsummoned?threshold_days=30
interface SoulAlert {
  soul_name: string;
  alert_type: 'NeverSummoned' | 'UnsummonedLongDuration';
  detail: string;
}

// /api/v1/analytics/low-effectiveness?threshold=0.3
interface BoundaryReview {
  soul_name: string;
  effective_rate: number;
  total_calls: number;
  recommendation: string;
}

// /api/v1/sessions
interface SessionSummary {
  id: string;
  title: string;
  mode: string;
  status: string;
  created_at: string;
  message_count: number;
}

// /api/v1/sessions/:id
interface SessionDetail {
  session: Session;
  messages: Message[];
}
```

## Relations

```
/analytics (Dashboard)
├── StatsOverview — 总览卡片 (4 指标)
├── ModeDistributionChart — Recharts BarChart
├── SoulEffectivenessTable — 魂有效性排行
├── AlertPanel — 未召唤/低效告警
└── SessionTimeline — 最近会话时间线

/sessions (会话历史)
├── SessionTimeline — 完整时间线
└── SessionDetailModal — 点击查看详情
```
