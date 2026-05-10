# NFR Requirements — F2: Soul Browser

## Performance

| 指标 | 目标 | 策略 |
|------|------|------|
| 列表页 FCP | < 1.5s | RSC fetch + Suspense streaming |
| 详情页 LCP | < 2s | 并行 fetch (profile + effectiveness) |
| Recharts bundle | < 60KB | 仅 import RadarChart |
| 搜索响应 | debounce 300ms + API < 50ms | 不触发全量渲染 |

## 加载策略 (Q3: C — Streaming SSR)

```
SoulListPage:
  1. RSC 发起 fetch → 流式返回
  2. <Suspense fallback={<SoulGridSkeleton />}>
      <SoulListView souls={data} />
     </Suspense>
  3. 骨架屏：4 个占位卡片（桌面）
  4. Error boundary 捕获 API 错误

SoulDetailPage:
  1. 并行 Promise.all([fetch(profile), fetch(effectiveness)])
  2. Suspense 包裹 IsmismRadar + EffectivenessPanel
  3. not-found.tsx 处理 404
```

## 并发

| 指标 | 目标 |
|------|------|
| 初始全量加载 souls 数量 | ≤ 50 |
| 搜索结果渲染 | 即时（API 返回 < 50ms） |
| ismism 距离计算 | 客户端同步 (< 5ms for 50 souls) |

## 可靠性

| 要求 | 描述 |
|------|------|
| API 离线 | 显示 ErrorBoundary fallback + 重试按钮 |
| 404 处理 | not-found.tsx + 返回列表链接 |
| 网络错误 | try-catch fetch + 友好错误提示 |
