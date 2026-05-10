# Domain Entities — F2: Soul Browser

## Page State Types

### SoulListState — 魂列表页状态

```typescript
interface SoulListState {
  souls: SoulListEntry[];        // 全量魂列表（Q2: C 初始加载）
  isLoading: boolean;
  gradeFilter: SoulGrade | null; // Q5: C — 品级筛选
  ismismFilter: IsmismCode | null; // Q5: C — ismism 四维滑块
  searchQuery: string;           // Q5: C — 关键词
  searchResults: SoulMatch[] | null; // API 搜索返回
  isSearching: boolean;
}
```

### IsmismSliderValues — 四维滑块

```typescript
interface IsmismSliderValues {
  field: [number, number];       // 1-4 范围
  ontology: [number, number];
  epistemology: [number, number];
  teleology: [number, number];
}
```

### SoulDetailState — 魂详情页状态

```typescript
interface SoulDetailState {
  profile: SoulProfile | null;
  isLoading: boolean;
  effectiveness: EffectivenessTrend | null;
  showingAllObservations: boolean;
}
```

## API Response Types (from B6)

```typescript
// GET /api/v1/souls
interface SoulListEntry {
  name: string;
  ismism_code: string;
  grade: 'S' | 'A' | 'B' | 'C' | 'D';
  field: string;
  tags: string[];
  summon_count: number;
}

// GET /api/v1/souls/search?q=
interface SoulMatch {
  entry: SoulListEntry;
  relevance: number;
  matched_fields: string[];
}

// GET /api/v1/souls/:name
interface SoulProfile {
  name: string;
  ismism_code: string;
  field: string;
  ontology: string;
  epistemology: string;
  teleology: string;
  grade: SoulGrade;
  domains: string[];
  tags: string[];
  summon_prompt: string;
  summon_count: number;
  effectiveness: EffectivenessStats;
  created_at: string;
  updated_at: string;
  practice_observations: PracticeObservation[];
}

// GET /api/v1/analytics/soul-effectiveness/:name
interface EffectivenessTrend {
  soul_name: string;
  total_calls: number;
  effective: number;
  partial: number;
  invalid: number;
  effective_rate: number;
}
```

## Relations

```
/souls (SoulListPage)
├── SoulFilterBar (品级下拉 + ismism 四维滑块 + 搜索框)
└── SoulCardGrid
    └── SoulCard[] → Link → /souls/[name]

/souls/[name] (SoulDetailPage)
├── SoulProfileCard (基本信息 + 领域标签)
├── IsmismRadar (四维雷达图)
├── EffectivenessChart (有效性趋势)
├── PromptPreview (召唤词预览)
├── PracticeObservations (实践记录列表)
├── SoulActions (召唤/编辑/删除按钮)
└── Link → /possess?preset={name}
```
