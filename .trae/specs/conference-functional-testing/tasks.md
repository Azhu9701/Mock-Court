# Tasks

- [x] Task 1: Rust 后端 CrossDetector 边界测试补充
  - [x] 补充 `test_empty_buffer_no_collision` 测试
  - [x] 补充 `test_same_soul_no_self_collision` 测试
  - [x] 补充 `test_multiple_collision_types` 测试
  - 验证：`cargo test -p possession` 全部通过

- [ ] Task 2: Rust 后端单魂模式集成测试
  - [ ] 创建 mock GatewayRegistry 辅助函数（用于返回预设 LLM 响应）
  - [ ] 添加 `test_single_soul_success` 测试
  - [ ] 添加 `test_single_soul_self_audit_alert` 测试
  - 验证：`cargo test -p possession` 全部通过

- [ ] Task 3: Rust 后端合议模式集成测试
  - [ ] 添加 `test_conference_two_souls_success` 测试（mock gateway 返回预设输出）
  - [ ] 添加 `test_conference_soul_not_found` 测试
  - [ ] 添加 `test_conference_timeout` 测试
  - 验证：`cargo test -p possession` 全部通过

- [ ] Task 4: Rust 后端辩论模式集成测试
  - [ ] 添加 `test_debate_two_souls_success` 测试
  - 验证：`cargo test -p possession` 全部通过

- [x] Task 5: Next.js 前端测试基础设施
  - [x] 安装 vitest、@testing-library/react、@testing-library/jest-dom
  - [x] 创建 `nextjs/vitest.config.ts` 配置文件
  - [x] 创建 `nextjs/__tests__/` 目录和示例测试
  - [x] 编写 ConferenceView 组件的简单渲染测试
  - 验证：`cd nextjs && npx vitest run` 全部通过

# Task Dependencies
- Task 2、3、4 共享 mock GatewayRegistry 辅助函数，Task 2 先完成 mock 基础设施，Task 3、4 可并行进行
- Task 5（前端）独立于 Task 1-4（Rust 后端），可并行进行
