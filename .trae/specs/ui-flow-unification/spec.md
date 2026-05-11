# UI 流程统一化 Spec

## Why
当前应用的讨论流程页面（`possession-entry`、`possess/[sessionId]`、`sessions/[id]`、`sessions`）之间在宽度约束、导航行为、渲染模式上互相不一致。同一个 SessionRunner 组件在不同上下文中外观不同，实时会话和历史详情用了两套完全独立的渲染系统，用户从侧边栏点击会话后进入的页面与进行中的页面体验割裂。

## What Changes
- 统一所有页面的宽度约束和容器样式，消除负边距反模式
- SessionRunner 行为统一：不依赖父组件环境，自身提供一致的返回导航和完成态处理
- 实时会话页和历史详情页共享同一套渲染组件（SoulChatBubble 等）
- PostSessionReview 和 FollowUpInput 在所有页面中位置一致
- 面包屑支持深层路由友好标签

## Impact
- Affected specs: UI Layout (existing), Session Flow (existing)
- Affected code:
  - `nextjs/app/possess/[sessionId]/page.tsx` — 移除负边距，统一容器
  - `nextjs/components/possession-entry.tsx` — 统一 running 阶段的宽度
  - `nextjs/components/session-runner.tsx` — 添加自包含的返回导航和结束态处理
  - `nextjs/app/sessions/[id]/page.tsx` — 使用共享的渲染组件
  - `nextjs/components/breadcrumb.tsx` — 添加深层路由标签映射
  - `nextjs/components/shell-layout.tsx` — 可能添加全屏模式支持

## ADDED Requirements

### Requirement: 统一页面容器宽度
所有流程相关页面 SHALL 使用以下宽度约束：
- 输入/匹配/审查等前序流程阶段：`max-w-2xl mx-auto`
- 实时会话和历史详情：`max-w-5xl mx-auto`

系统中 SHALL NOT 出现通过负边距（`-m-4` 等）抵消 Shell 内边距的 hack。

#### Scenario: 访问实时会话页面
- **WHEN** 用户通过 `http://host/possess/{id}?mode=conference` 访问正在进行中的合议会话
- **THEN** 页面使用 `max-w-5xl mx-auto` 容器，Shell 内边距正常生效
- **AND** 与从 `possession-entry` 流程中进入 running 阶段后的视图宽度一致

#### Scenario: 访问历史会话详情
- **WHEN** 用户通过侧边栏或 `/sessions/{id}` 访问已完成的历史会话
- **THEN** 页面宽度与实时会话页面一致（`max-w-5xl`）

### Requirement: SessionRunner 自包含结束态
SessionRunner 组件 SHALL 在 `status === "done"` 时自行展示 PostSessionReview 和 FollowUpInput，不依赖父组件传入回调控制。

#### Scenario: 会话完成后的闭环
- **WHEN** 实时会话的 WebSocket 收到 `done` 事件
- **THEN** SessionRunner 自动展示「反馈闭环」区域，包含 PostSessionReview 和 FollowUpInput
- **AND** 父组件不需要通过 `onReview` 回调来切换 UI

### Requirement: 侧边栏会话导航智能区分
侧边栏的会话列表 SHALL 根据 `status` 字段智能跳转：
- `active` 状态的会话 → 跳转到 `/possess/{id}?mode={mode}`
- 其他状态的会话 → 跳转到 `/sessions/{id}`

#### Scenario: 点击正在运行的会话
- **WHEN** 用户在侧边栏点击一个 status 为 "active" 的会话
- **THEN** 导航到实时会话页面 `/possess/{id}?mode={mode}`

### Requirement: 深层路由面包屑友好标签
面包屑组件 SHALL 支持从会话 API 获取会话标题，显示在面包屑最后一段。

#### Scenario: 浏览历史会话详情
- **WHEN** 用户访问 `/sessions/abc123`，该会话标题为「资本主义批判」
- **THEN** 面包屑显示为「首页 > 会话历史 > 资本主义批判」

## MODIFIED Requirements

### Requirement: PostSessionReview 位置统一
**原行为**：在 `possession-entry.tsx` 中内联在 SessionRunner 下方，在 `possess/[sessionId]` 中替换整个页面。

**新行为**：PostSessionReview SHALL 在所有场景下作为 SessionRunner 完成后的内联组件展示。

### Requirement: FollowUpInput 复用
**原行为**：`follow-up-input.tsx` 内部包含独立 WebSocket 客户端实现，与 `useWebSocket` hook 重复。

**新行为**：FollowUpInput SHALL 复用 `useWebSocket` hook 或通过共享的 WebSocket 连接获取事件，消除重复代码。历史详情页的 FollowUpInput 行为应与实时会话页一致。

## REMOVED Requirements

### Requirement: possess/[sessionId] 的负边距 hack
**Reason**: 破坏 Shell 布局一致性。
**Migration**: 移除 `-m-4 lg:-m-8`，使用标准的 `max-w-5xl mx-auto`。
