# Phase 5: 魂信誉系统 + 魂杂交 + 魂间直接教学

请根据 `/Users/huyi/Desktop/rust banner/rust-optimization-plan.md` 中 Phase 5 的方案实现。

## 背景
万民幡需要从工具进化为平台。需要信誉系统评估魂质量、魂杂交融合方法论、魂间教学让魂互相学习。

## 工作内容

### 1. 创建 rust/possession/src/soul/reputation.rs
- SoulReputation 结构体：soul_name, approval_rate, contradiction_rate, practice_validation_rate, insight_novelty, reputation_score, total_contributions, last_evaluated
- ReputationManager：HashMap<String, SoulReputation>
- 方法：record_approval / record_contradiction / record_practice_validation / record_insight
- recalculate(soul_name)：加权公式 0.3*approval + 0.25*(1-contradiction) + 0.25*practice + 0.2*novelty
- get_top_souls(n) / get_bottom_souls(n) / should_review / get_debate_weight / get_dormant_souls
- ReputationEvent 枚举

### 2. 创建 rust/possession/src/soul/hybridization.rs
- HybridizationCandidate：parent_a, parent_b, methodology_overlap, complementary_strengths, fusion_prompt, estimated_novelty
- MethodologySignature：reasoning_style, evidence_preference, temporal_focus, abstraction_level
- HybridizationEngine：analyze_methodology / find_compatible_pairs / generate_fusion_prompt / compatibility_score

### 3. 创建 rust/possession/src/modes/teaching.rs
- TeachingSession：teacher, student, topic, teacher_analysis, lesson_plan(Vec<LessonStep>), quiz(Vec<QuizQuestion>), score, feedback
- run_teaching_session(teacher, student, topic, gateway, registry) → TeachingSession
- 教学 prompt 构建辅助函数

### 4. 更新模块声明
- soul/mod.rs 添加 pub mod reputation; pub mod hybridization;
- modes/mod.rs 添加 pub mod teaching;

## 约束
- 信誉系统所有计算确定性的，不依赖 LLM
- 杂交引擎核心是方法论签名匹配，不是 LLM 调用
- 匹配现有代码风格，所有类型派生 Debug, Clone, Serialize, Deserialize

## 验证
1. cargo check -p possession 通过
2. reputation.rs 包含计算和排序单元测试
3. hybridization.rs 包含 compatibility_score 单元测试
4. teaching.rs 结构体定义完整

请先阅读所有相关源文件，理解现有架构，然后实现代码。完成后自检 cargo check。
