# UI 流程统一化 验证清单

## 容器与宽度验证
- [x] `possess/[sessionId]/page.tsx` 中不存在 `-m-` 负边距
- [x] 访问 `/possess/{id}?mode=conference` 页面宽度与从 `possession-entry` 进入 running 阶段后一致
- [x] 移动端（375px 宽度）下所有流程页面内容正常显示，无横向滚动
- [x] `/sessions/{id}` 页面宽度与 `/possess/{id}` 一致

## 导航验证
- [x] 侧边栏点击 status 为 "active" 的会话 → 跳转到 `/possess/{id}?mode={mode}`
- [x] 侧边栏点击 status 为 "completed" 的会话 → 跳转到 `/sessions/{id}`
- [x] SessionRunner 内有可用的返回链接，能回到讨论入口
- [x] `/sessions/{id}` 页面有返回链接，能回到会话历史列表

## 面包屑验证
- [x] 访问 `/sessions/{id}` 时，面包屑最后一段显示会话标题（非 raw ID）
- [x] 访问 `/possess/{id}` 时，面包屑友好显示
- [x] 访问 `/possess` 时，面包屑显示「首页 > 讨论」

## 完成态验证
- [x] 会话完成后，SessionRunner 自身展示 PostSessionReview 和 FollowUpInput
- [x] review 和 follow-up 完成后用户留在当前页面，不被强制跳转
- [x] 会话统计（魂数量、耗时等）在完成后正确显示

## 追问验证
- [x] 实时会话页面的追问功能正常工作
- [x] 历史详情页面的追问功能正常工作
- [x] 追问产生的消息在两次请求间保持一致（刷新后仍可见）

## 代码质量验证
- [x] 无 TypeScript 编译错误
- [x] 无 ESLint 错误
