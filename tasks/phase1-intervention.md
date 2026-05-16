# Phase 1: 干预感知的魂长驻进程 + 三级追问门控

请根据 `/Users/huyi/Desktop/rust banner/rust-optimization-plan.md` 中 Phase 1 的方案实现。

## 背景
当前魂进程(process.rs)在合议模式下使用一次性 API 调用，碰撞检测结果只能事后汇总给综合官，魂在推理过程中无法被实时干预。

## 工作内容

### 1. 修改 rust/possession/src/soul/process.rs
- 新增 Intervention 枚举（ContradictionQuestion, BlindSpotRedirect, DeepenRequest）
- 在 SoulProcess 的 run() 方法中用 tokio::select! 实现推理与干预的竞态
- 当魂在流式推理时收到 intervention，将 intervention 转为 context message 注入，重新启动推理
- 添加 intervention_rx 通道
- 修改 SoulProcess::new() 增加 intervention channel 参数
- 修改 SoulProcessManager 以支持干预通道注册
- 保持现有功能不变

### 2. 创建 rust/possession/src/soul/intervention.rs
- InterventionGate 结构体，三级门控：L1关键词规则匹配 → L2 embedding余弦相似度 → L3 Flash LLM判定
- InterventionDecision 枚举：NoAction / InjectQuestion / Redirect / DeepenRequest
- gate() 方法按 L1 → L2 → L3 级联尝试
- 定义信念冲突关键词表

### 3. 更新 rust/possession/src/soul/mod.rs 添加 pub mod intervention;

## 约束
- 魂上下文隔离，计算分层（能用规则绝不用LLM）
- 匹配现有代码风格，保持 API 兼容
- 所有新类型派生 Debug, Clone

## 验证
1. cargo check -p possession 通过
2. process.rs 中 tokio::select! 正确引入 intervention 分支
3. intervention.rs 三级门控逻辑完整

请先阅读所有相关源文件，理解现有架构，然后实现代码。完成后自检 cargo check。
