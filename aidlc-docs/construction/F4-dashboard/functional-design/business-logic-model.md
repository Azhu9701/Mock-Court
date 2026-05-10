# Business Logic Model — F4: Dashboard

## 页面结构

```
/analytics (Dashboard)
├── StatsOverviewRow (RSC, 4 cards)
├── ModeBarChart (Client, Recharts)
├── SoulEffectivenessTable (RSC)
├── AlertSection (RSC)
│   ├── UnsummonedAlerts
│   └── LowEffectivenessAlerts
└── RecentSessions (RSC)

/sessions (历史)
├── SessionTimeline (RSC + Client)
│   ├── TimelineDateGroup[]
│   │   └── TimelineItem[]
│   └── SessionDetailDialog
```

## 数据流 (Q4 继承 F2: RSC + fetch)

```
Dashboard (RSC):
  并行 fetch:
  1. /analytics/summon-stats → StatsOverview + ModeChart
  2. /analytics/mode-distribution → ModeChart
  3. /analytics/unsummoned → Alerts
  4. /analytics/low-effectiveness → Alerts
  5. /sessions?limit=10 → RecentSessions

SessionTimeline (RSC):
  1. /sessions → SessionSummary[]
  2. 按日期分组 → TimelineDateGroup[]
```

## 组件树

### Dashboard (/analytics)

```
StatsOverviewRow
├── StatCard (总召唤次数, BarChart3 icon)
├── StatCard (魂参与率, Users icon)
├── StatCard (有效率, CheckCircle icon)
└── StatCard (活跃警报, AlertTriangle icon)

ModeBarChart (Recharts BarChart)
└── 6 种模式柱状图

SoulEffectivenessTable
└── Table rows: soul_name + effective_rate bar + call_count

AlertSection
├── UnsummonedList (AlertTriangle icon, red)
└── LowEffectivenessList (TrendingDown icon, yellow)

RecentSessions
└── SessionTimeline (limit 10)
    └── TimelineItem: date + title + mode badge + status
```

### Sessions (/sessions)

```
SessionTimeline
└── DateGroups[]
    ├── "今天" / "昨天" / "2024-05-09" label
    └── TimelineItem[]
        ├── SessionModeBadge
        ├── SessionTitle → Link /sessions/:id
        ├── MessageCount + CreatedAt
        └── SessionStatusBadge
```
