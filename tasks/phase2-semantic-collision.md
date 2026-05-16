# Phase 2: ONNX 语义碰撞检测引擎

请根据 `/Users/huyi/Desktop/rust banner/rust-optimization-plan.md` 中 Phase 2 的方案实现。

## 背景
当前 cross_detector.rs 使用纯关键词规则匹配，面对复杂哲学/政治分析时准确率低（~30%）。需升级为三路径并行语义碰撞引擎。

## 工作内容

### 1. 重构 rust/possession/src/cross_detector.rs
- 保留现有关键词规则路径作为"快速路径"
- 添加 CollisionPath 枚举：KeywordPath / EmbeddingPath / StructurePath
- 添加 sliding window buffer（每 N tokens 一个片段，N 可配置，默认 64）
- 添加冗余抑制检测（高相似度 + 同一结论 → 标记冗余）
- 保留所有现有公共 API 兼容性

### 2. 创建 rust/possession/src/semantic_collision.rs
- SemanticFragment 结构体（id, soul_name, text, embedding: Option<Vec<f32>>, ismism_coords: [f32; 4]）
- SlidingWindow 结构体（max_window_size, fragments: VecDeque<SemanticFragment>）
- cosine_similarity(a: &[f32], b: &[f32]) -> f32 纯函数
- 主义主义坐标距离计算：ismism_distance(a: [f32; 4], b: [f32; 4]) -> f32
- CollisionDetected 结构体（collision_type, from_soul, to_soul, confidence: f32, path: CollisionPath, description）
- SemanticCollisionEngine：register_fragment / detect_all / 三路径并行检测
- 嵌入路径用 trait EmbeddingProvider 接口设计，不实际集成 ONNX
- 碰撞类型新增：Redundancy、BlindSpotComplement

### 3. 更新 rust/possession/src/lib.rs 添加 pub mod semantic_collision;

## 约束
- 嵌入路径暂用 trait 接口设计，不实际集成 ONNX
- 结构路径用简单欧几里得距离
- 保持 cross_detector.rs 现有测试通过
- 所有检测方法不阻塞（同步执行）

## 验证
1. cargo check -p possession 通过
2. cross_detector.rs 所有现有测试通过
3. semantic_collision.rs 包含 SlidingWindow, cosine_similarity, ismism_distance 单元测试

请先阅读所有相关源文件，理解现有架构，然后实现代码。完成后自检 cargo check。
