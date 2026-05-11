# 合议等功能测试 Spec

## Why
当前系统核心附体功能（合议、辩论、单魂）缺乏系统化的自动化测试覆盖。Rust 后端仅 `cross_detector`、`self_audit`、`model_router` 等少数模块有单元测试，核心业务流程（`conference.rs`、`debate.rs`、`single.rs`）完全没有测试。Next.js 前端完全没有测试基础设施。需要建立测试体系确保核心功能稳定可用。

## What Changes
- Rust 后端：为核心附体模式添加单元测试和集成测试（使用 mock LLM gateway）
- 为 `conference.rs` 的 `run()` 函数添加集成测试，验证并行调度、交叉检测、综合流程
- 为 `debate.rs` 的 `run()` 函数添加集成测试
- 为 `single.rs` 的 `run()` 函数添加集成测试，包含自审流程
- 补充 `cross_detector.rs` 的边界测试用例
- Next.js 前端：引入 vitest + @testing-library/react 测试基础设施

## Impact
- Affected specs: 无（新增测试能力，不影响现有功能规范）
- Affected code:
  - `rust/possession/src/modes/conference.rs` — 添加 `#[cfg(test)]` 测试模块
  - `rust/possession/src/modes/debate.rs` — 添加 `#[cfg(test)]` 测试模块
  - `rust/possession/src/modes/single.rs` — 添加 `#[cfg(test)]` 测试模块
  - `rust/possession/src/cross_detector.rs` — 补充边界测试用例
  - `nextjs/package.json` — 添加 vitest / testing-library 依赖
  - `nextjs/` — 创建 vitest 配置文件和测试目录

## ADDED Requirements

### Requirement: Rust 后端合议模式集成测试
系统 SHALL 在 `conference.rs` 中添加集成测试，使用 mock GatewayRegistry 验证合议模式的完整流程。

#### Scenario: 合议模式正常启动并完成
- **WHEN** 调用 `conference::run()` 并传入 2 个魂和 mock gateway（返回预设输出）
- **THEN** 返回的 `Vec<SoulOutput>` 包含 2 个输出
- **AND** 每个输出的 `soul_name` 和 `content` 与 mock 预设一致
- **AND** 不会超时或 panic

#### Scenario: 合议模式魂查询失败
- **WHEN** 调用 `conference::run()` 传入一个不存在的魂名
- **THEN** 系统发送 `SoulError` 事件
- **AND** 不 panic，继续处理其他魂

#### Scenario: 合议模式超时
- **WHEN** mock gateway 模拟长时间响应（>300 秒）
- **THEN** 系统在 `SOUL_TIMEOUT_SECS` 后中止所有魂
- **AND** 发送超时 SystemMessage 事件

### Requirement: Rust 后端辩论模式集成测试
系统 SHALL 在 `debate.rs` 中添加集成测试，验证辩论模式的并行辩论流程。

#### Scenario: 辩论模式正常完成
- **WHEN** 调用 `debate::run()` 传入 2 个魂和 mock gateway
- **THEN** 返回 2 个 `SoulOutput`，每个 content 与 mock 预设一致
- **AND** 两个魂的输出被存储为 `PossessionMode::Debate`

### Requirement: Rust 后端单魂模式集成测试
系统 SHALL 在 `single.rs` 中添加集成测试，验证单魂模式及自审流程。

#### Scenario: 单魂模式正常完成
- **WHEN** 调用 `single::run()` 传入 1 个魂和 mock gateway（不触发自审告警）
- **THEN** 返回的 `SoulOutput` 的 `soul_name` 和 `content` 与预设一致
- **AND** 输出被存储为 `PossessionMode::Single`

#### Scenario: 单魂模式自审告警
- **WHEN** 魂的输出触发 `SelfAudit` 告警（如内容包含矛盾）
- **THEN** 系统发送 SystemMessage 包含审计告警

### Requirement: CrossDetector 边界测试补充
系统 SHALL 在 `cross_detector.rs` 的现有测试基础上补充边界测试。

#### Scenario: 空缓冲区碰撞检测
- **WHEN** 所有魂的缓冲区为空
- **THEN** `detect_collisions()` 返回空列表

#### Scenario: 同魂不同文本不产生碰撞
- **WHEN** 同一魂连续输出不同内容
- **THEN** 不与自身产生碰撞

#### Scenario: 多碰撞类型同时检测
- **WHEN** 两个魂的输出同时触发 `Contradiction` 和 `PerspectiveDifference`
- **THEN** 两种碰撞都被正确检测

### Requirement: Next.js 前端测试基础设施
系统 SHALL 为 Next.js 前端引入 vitest 测试框架。

#### Scenario: 运行前端测试
- **WHEN** 执行 `npx vitest run`
- **THEN** 测试正常运行并输出结果
- **AND** 测试框架不要求真实浏览器环境

#### Scenario: 组件渲染测试
- **WHEN** 编写 ConferenceView 组件的简单渲染测试
- **THEN** 组件在测试环境中正常渲染，不抛出错误
