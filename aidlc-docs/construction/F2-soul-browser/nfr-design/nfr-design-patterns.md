# NFR Design Patterns — F2: Soul Browser

## Pattern 1: Streaming RSC + Suspense (Q3: C)

**问题**: 魂列表全量数据可能延迟（API cold start），需要流畅的加载体验。

**方案**: Next.js RSC Streaming + Suspense boundary。

```
// app/souls/page.tsx
export default async function SoulListPage() {
  return (
    <div>
      <h1>魂览</h1>
      <Suspense fallback={<SoulGridSkeleton />}>
        <SoulListAsync />
      </Suspense>
    </div>
  );
}

async function SoulListAsync() {
  const souls = await fetchSouls();
  return <SoulListView souls={souls} />;
}
```

**关键**: `fetchSouls()` 在 RSC 中直接调用 B6 API，利用 Next.js 自动流式传输。

## Pattern 2: Hybrid Search (Q2: C — 混合)

**问题**: 同时支持全量浏览和 API 搜索。

**方案**: 初始 RSC 全量数据 + 客户端条件触发 API。

```
<ClientComponent souls={initialData}>
  // 初始渲染: 全量 initialData
  // onSearch(query): fetch API → setSearchResults
  // isSearching ? searchResults : filterLocal(initialData)
</ClientComponent>
```

**数据流**:
1. RSC: `fetch(/api/v1/souls)` → `SoulListEntry[]` (cached 60s)
2. Client: 用户输入 → `fetch(/api/v1/souls/search?q=)` → `SoulMatch[]`
3. 清空搜索 → 恢复 initialData

## Pattern 3: URL State Sync

**问题**: 筛选条件需要支持分享和浏览器导航。

**方案**: `useSearchParams()` + 手动同步。

```
// 读取初始筛选状态
const searchParams = useSearchParams();
const grade = searchParams.get('grade');
const ismism = searchParams.get('ismism');
const query = searchParams.get('q');

// 筛选变化 → 更新 URL
function onFilterChange(filters: FilterState) {
  const params = new URLSearchParams();
  if (filters.grade) params.set('grade', filters.grade);
  if (filters.ismism) params.set('ismism', filters.ismism);
  if (filters.query) params.set('q', filters.query);
  router.replace(`/souls?${params.toString()}`);
}
```

## Pattern 4: Ismism Distance Filtering

**问题**: 四维滑块筛选需要客户端实时计算。

**方案**: 预解析 ismism_code → 欧氏距离排序。

```typescript
function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

function isWithinRange(
  code: IsmismCode,
  ranges: IsmismSliderValues
): boolean {
  return (
    code.field >= ranges.field[0] && code.field <= ranges.field[1] &&
    code.ontology >= ranges.ontology[0] && code.ontology <= ranges.ontology[1] &&
    code.epistemology >= ranges.epistemology[0] && code.epistemology <= ranges.epistemology[1] &&
    code.teleology >= ranges.teleology[0] && code.teleology <= ranges.teleology[1]
  );
}
```

## Pattern 5: Optimistic Delete

**问题**: 删除魂后需要即时 UI 反馈。

**方案**: 乐观更新 + router.refresh。

```
async function onDelete(name: string) {
  // 1. 乐观：立即从列表移除
  setOptimistic(souls.filter(s => s.name !== name));
  // 2. API 调用
  await fetch(`/api/v1/souls/${name}`, { method: 'DELETE' });
  // 3. 刷新 RSC 缓存
  router.refresh();
}
```

## Pattern 6: Recharts RadarChart

**问题**: ismism 四维数据可视化。

**方案**: Recharts `<RadarChart>` + 自定义 domain。

```tsx
const data = [
  { dimension: '领域', value: code.field, fullMark: 4 },
  { dimension: '本体论', value: code.ontology, fullMark: 4 },
  { dimension: '认识论', value: code.epistemology, fullMark: 4 },
  { dimension: '目的论', value: code.teleology, fullMark: 4 },
];

<RadarChart data={data}>
  <PolarGrid />
  <PolarAngleAxis dataKey="dimension" />
  <PolarRadiusAxis domain={[0, 4]} />
  <Radar dataKey="value" fill="var(--primary)" fillOpacity={0.2} />
</RadarChart>
```

**关键**: 使用 CSS 变量 `var(--primary)` 适配暗/亮主题。
