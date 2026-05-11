# Tasks

## [x] Task 1: 统一页面容器宽度
- **Priority**: P0
- **Depends On**: None
- **Description**:
  - 移除 `possess/[sessionId]/page.tsx` 的负边距 `-m-4 lg:-m-8`
  - 所有流程页面统一宽度约束：`max-w-5xl mx-auto`
  - 确保 SessionRunner 在 `possession-entry` running 阶段和 `possess/[sessionId]` 独立页面中外观一致
- **Test Requirements**:
  - `human-judgment`: 检查两个路径下的 SessionRunner 渲染效果是否一致
  - `human-judgment`: 确认移动端和桌面端宽度正常

## [x] Task 2: SessionRunner 自包含结束态与返回导航
- **Priority**: P0
- **Depends On**: Task 1
- **Description**:
  - SessionRunner 组件内部自行处理 `status === "done"` 时的 PostSessionReview + FollowUpInput 展示
  - 移除 `onReview` prop，改为组件内部管理完成态闭环
  - SessionRunner 添加顶部返回链接（`<Link href="/possess">`），不依赖父组件
- **Test Requirements**:
  - `human-judgment`: 验证会话完成后能正常展示 review 和 follow-up
  - `human-judgment`: 验证返回链接正常工作

## [x] Task 3: 侧边栏智能导航
- **Priority**: P0
- **Depends On**: None
- **Description**:
  - 修改 `sidebar-sessions.tsx`，根据会话 `status` 决定跳转目标
  - `status === "active"` 或 `status === "running"` → `/possess/{id}?mode={mode}`
  - 其他状态 → `/sessions/{id}`
- **Test Requirements**:
  - `human-judgment`: 验证 active 会话点击后进入实时页面

## [x] Task 4: 面包屑深层路由标签
- **Priority**: P1
- **Depends On**: None
- **Description**:
  - 创建 BreadcrumbContext + BreadcrumbSetter 组件
  - 修改 `breadcrumb.tsx`，支持从 Context 获取会话标题作为面包屑最后一段
  - 在 `sessions/[id]` 页面通过 BreadcrumbSetter 传入会话标题
- **Test Requirements**:
  - `human-judgment`: 验证访问 `/sessions/{id}` 时面包屑显示会话标题而非 raw ID

## [x] Task 5: 统一 PostSessionReview 行为
- **Priority**: P1
- **Depends On**: Task 2
- **Description**:
  - 修改 `possess/[sessionId]/page.tsx`，移除 `showReview` 状态和整页替换逻辑
  - PostSessionReview 改由 SessionRunner 内部管理
- **Test Requirements**:
  - `human-judgment`: 验证 review 完成后用户不会被迫跳转到其他页面

## [x] Task 6: FollowUpInput WebSocket 复用
- **Priority**: P2
- **Depends On**: Task 2
- **Description**:
  - FollowUpInput 在 SessionRunner 的完成态中作为子组件使用
  - 历史详情页（`sessions/[id]`）中 FollowUpInput 保持现有行为（通过 API 发送追问）
  - 实时会话完成态中的 FollowUpInput 同样通过 API 发送追问，保持一致性
- **Test Requirements**:
  - `human-judgment`: 验证追问功能在实时会话和历史详情中均可正常工作

# Task Dependencies
- Task 2 depends on Task 1
- Task 5 depends on Task 2
- Task 6 depends on Task 2
- Task 3 and Task 4 have no dependencies, can be done in parallel
