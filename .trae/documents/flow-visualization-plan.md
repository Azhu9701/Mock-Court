# 流程可视化优化 + 使用者参与强制化

## 当前问题

### 可视化方面
1. **阶段指示器单调**：仅一个垂直步骤列表（classifying → matching → reviewing → adjusting → starting），无进度百分比、无预估步骤数、无过渡动画
2. **progressLine 单一**：只有一行蓝色背景文字，信息密度低
3. **log 面板扁平**：纯文本列表，无分类筛选、无折叠
4. **running 阶段 SessionRunner 显示不连贯**：ProcessTimeline 只在 streaming/done 时出现，WaitingSoulsView 无魂到达提示
5. **魂到达时无视觉高潮**：`soul_started` 只是往 processSteps 数组追加一条记录

### 使用者参与方面
6. **「使用者预设」默认折叠**：judgment/worry/unknown 藏在可折叠面板里，多数用户不会点开
7. **无强制校验**：不填也能直接发起讨论，后端的 judgment/worry/unknown 形同虚设
8. **无完成度反馈**：三个字段没有「已填写」vs「待填写」的视觉区分

---

## 实施方案

### 第一步：重做阶段指示器 — 水平步进器 + 进度条

**文件：** `nextjs/components/possession-entry.tsx`（阶段 UI 部分）

改造现有垂直阶段列表为**水平步进器**（horizontal stepper），位于"附体流程"卡片中：

```
[入口分流] ─── [匹配魂] ─── [审查] ─── [调整] ─── [启动]
   ✅          ⏳ 进行中       ○           ○          ○
```

- 当前步骤：蓝色填充 + 脉冲动画 + Loader2 图标
- 已完成步骤：绿色填充 + CheckCircle2 对勾 + 耗时（如 "1.2s"）
- 待处理步骤：灰色描边空圆
- 步骤之间用连线连接，已完成连线变绿色
- 顶部加进度条：`2/5 步骤 · 匹配魂中...`
- 每个步骤下方有小字说明（如"入口分流中…"显示为"分析任务类型"）

### 第二步：执行日志升级 — 分组 + 级别过滤

**文件：** `nextjs/components/possession-entry.tsx`（log panel 部分）

- 添加日志类型过滤器按钮：全部 / 关键 / 魂匹配 / 审查
- 日志条目按类型着色（info=灰色、success=绿色、warning=琥珀色、error=红色）
- 自动折叠同类型连续条目（减少视觉噪音）
- 添加「复制日志」按钮

### 第三步：魂到达视觉高潮

**文件：** `nextjs/components/session-runner.tsx`

在 `WaitingSoulsView` 中，当魂 `soul_started` 时：
- 列表中弹出 pulse 动画标记"已到达"
- 匹配魂列表卡片逐个点亮（从灰色到彩色），展示到达顺序
- 在 `ProcessTimeline` 中正在进行的步骤添加呼吸动画边框

### 第四步：使用者预设强制化

**文件：** `nextjs/components/possession-entry.tsx`

改动：
1. **面板默认展开**：`showPresets` 初始值从 `false` → `true`
2. **judgment 设为必填**：标注红色星号 `*`
3. **启动按钮逻辑**：`!task.trim()` → `!task.trim() || !judgment.trim()`
4. **未填判断时按钮禁用**：按钮下方显示"请填写你的判断后再开始"提示
5. **worry 和 unknown 为推荐项**：不填可启动，但按钮旁显示"推荐填写"徽章
6. **三个字段完成后显示勾选状态**：每个 Textarea 右上角显示 CheckCircle2 绿色对勾
7. **折叠面板标题动态显示**：
   - 全未填：`使用者预设 ⚠️ 请填写判断`
   - 部分填写：`使用者预设（2/3 完成）`
   - 全部填写：`使用者预设 ✅`

### 第五步：running 阶段过渡优化

**文件：** `nextjs/components/session-runner.tsx` + `nextjs/components/possession-entry.tsx`

1. 从"启动"阶段到 running 阶段添加一个过渡动画面板（0.5s 淡入）
2. WaitingSoulsView 中添加有趣的文案轮播（如"正在召唤马克思之魂…"、"列宁正在思考…"）
3. 魂到达时显示 avater 弹入动画（使用 CSS `animate-in zoom-in`）

### 第六步：完成后总结卡片

**文件：** `nextjs/components/possession-entry.tsx`

在 `sessionDone` 触发后，SessionRunner 下方显示一个总结卡片：
- 总耗时
- 参与魂数量
- 综合报告字数
- PostSessionReview 在卡片内显示，完成后自动折叠为追问输入

---

## 涉及文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `nextjs/components/possession-entry.tsx` | **重写阶段 UI** | 水平步进器 + 日志升级 + 使用者预设强制化 + 过渡动画 + 总结卡片 |
| `nextjs/components/session-runner.tsx` | 修改 | 魂到达动画 + WaitingSoulsView 升级 + ProcessTimeline 呼吸边框 |
| `nextjs/components/follow-up-input.tsx` | 不修改 | 已在上次修复中完善 |

## 交互变化对比

### 之前
```
[折叠的"使用者预设"]  ← 多数用户忽略
[Textarea: 输入问题]
[搜索背景资料（可选）]
[开始讨论 按钮]  ← 不填预设也能点

↓ 点击后

[垂直步骤列表] [日志面板]  ← 视觉单调
↓
[SessionRunner 流式]  ← 无魂到达动画
↓
[会话完成 → 反馈闭环 → 追问]
```

### 之后
```
[使用者预设 ⚠️ 请填写判断]  ← 展开，judgment 有红色星号
├ 你的判断 *  [___________]  ← 必填
├ 你的担忧    [___________]  ← 推荐
└ 未知领域    [___________]  ← 推荐
[Textarea: 输入问题]
[搜索背景资料（可选）]
[我想问]  ← 判断未填时灰色禁用，下方有提示

↓ 点击后

[████████░░░░░░░░] 2/5 · 匹配魂中...
[入口分流 ✅1.2s] ─ [匹配魂 ⏳] ─ [审查 ○] ─ [调整 ○] ─ [启动 ○]
[日志：全部 ▼ | 📋]  ← 可筛选可复制

↓ running 阶段

[马克思 ✅ 已到达] [列宁 🌀 思考中] [未明子 ○ 等待中]
← ProcessTimeline 呼吸动画

↓ 完成后

┌─ 流程总结 ─────────────┐
│ ⏱ 42s  ·  3 魂  ·  1,240 字 │
└────────────────────────┘
[PostSessionReview 4 步反馈]
[追问输入]
```
