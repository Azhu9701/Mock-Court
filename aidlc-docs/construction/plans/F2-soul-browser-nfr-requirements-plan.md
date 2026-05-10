# NFR Requirements Plan — F2: Soul Browser

## Plan Steps

- [x] Step 1: 创建 `nfr-requirements.md` — 性能/加载策略
- [x] Step 2: 创建 `tech-stack-decisions.md` — 图表库/表单库选择

## Design Questions

### Q1: IsmismRadar 图表实现
Q3 确定展示 ismism 四维雷达图。实现方式？
- [Answer]: B — Recharts

### Q2: 魂编辑表单方案
- [Answer]: C — shadcn/ui Form (RHF + Zod)

### Q3: 列表加载/空态策略
- [Answer]: C — Streaming SSR (Suspense)
