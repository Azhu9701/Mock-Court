# Business Logic Model — F2: Soul Browser

## 路由结构

```
app/
└── souls/
    ├── page.tsx              # /souls — 魂列表（Server Component）
    ├── loading.tsx           # Suspense 加载态
    ├── error.tsx             # Error boundary
    └── [name]/
        ├── page.tsx          # /souls/[name] — 魂详情（Server Component）
        ├── loading.tsx       # 加载态
        └── not-found.tsx     # 404
```

## 数据流 (Q4: A — Server Components + fetch)

### Soul List 数据流 (Q2: C — 混合模式)

```
SoulListPage (RSC):
  1. fetch('http://127.0.0.1:3096/api/v1/souls') → SoulListEntry[]
  2. 全量数据传给 Client Component: <SoulListClient souls={data} />

SoulListClient (Client Component):
  1. 初始渲染：全量 souls → Card Grid
  2. 用户输入搜索词 → fetch `/api/v1/souls/search?q=xxx`
  3. 搜索结果覆盖 Card Grid
  4. ismism 四维滑块 → 客户端过滤（就近匹配）

筛选逻辑 (Q5: C):
  1. 品级: 直接 filter (soul.grade === selectedGrade)
  2. ismism: 客户端距离计算，按阈值过滤
  3. 关键词: 调用 API search
```

### Soul Detail 数据流

```
SoulDetailPage (RSC):
  并行 fetch:
  1. fetch(`/api/v1/souls/${name}`) → SoulProfile
  2. fetch(`/api/v1/analytics/soul-effectiveness/${name}`) → EffectivenessTrend
  3. 渲染完整 Profile
```

## 组件树

```
SoulListPage (RSC)
├── SoulFilterBar (Client Component)
│   ├── GradeSelect — 品级下拉 (S/A/B/C/D/All)
│   ├── IsmismSliders — 四维范围滑块 (field/ontology/epistemology/teleology)
│   └── SearchInput — 关键词搜索 (debounce 300ms)
├── SoulCardGrid (Client Component)
│   └── SoulCard[] — 魂卡片
│       ├── GradeBadge — 品级徽章 (S~D)
│       ├── SoulName — 魂名
│       ├── IsmismCode — ismism 编码
│       ├── FieldTag — 领域标签
│       └── SummonCount — 召唤次数
└── SoulCount — "共 N 个魂"

SoulDetailPage (RSC)
├── SoulProfileHeader
│   ├── GradeBadge
│   ├── SoulName
│   └── IsmismCode
├── IsmismRadar (Client Component) — SVG radar
├── SoulInfoGrid
│   ├── DomainTags
│   ├── FieldBadge
│   └── CreatedAt
├── PromptPreview — summon_prompt 折叠预览
├── EffectivenessTrend (Client Component)
│   ├── EffectiveRateBar
│   └── TotalCalls
├── PracticeObservations (Client Component)
│   └── ObservationCard[] — 实践记录列表
└── SoulActionBar
    ├── SummonButton — 跳转到 /possess
    ├── EditButton — 编辑 modal
    └── DeleteButton — 确认删除
```

## API 调用汇总

| 页面 | 调用 | 方法 | 缓存策略 |
|------|------|------|----------|
| /souls | `/souls` | RSC fetch | `next.revalidate = 60` |
| /souls (search) | `/souls/search?q=` | Client fetch | 不缓存 |
| /souls/[name] | `/souls/:name` | RSC fetch | `next.revalidate = 60` |
| /souls/[name] | `/soul-effectiveness/:name` | RSC fetch | `next.revalidate = 60` |

## ismism 距离计算 (客户端)

```typescript
function ismismDistance(a: IsmismCode, b: IsmismCode): number {
  return Math.sqrt(
    (a.field - b.field) ** 2 +
    (a.ontology - b.ontology) ** 2 +
    (a.epistemology - b.epistemology) ** 2 +
    (a.teleology - b.teleology) ** 2
  );
}

// 筛选: 以滑块中心值为 target，距离 < threshold 的魂保留
function filterByIsmism(
  souls: SoulListEntry[],
  target: IsmismCode,
  threshold: number
): SoulListEntry[] {
  return souls.filter(s => {
    const code = ismismParse(s.ismism_code);
    return code && ismismDistance(code, target) <= threshold;
  });
}
```
