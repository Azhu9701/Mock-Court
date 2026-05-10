use foundation::{ModelTier, Prompt, PromptMessage, SoulProfile};

use crate::model_router::RoutingRole;

const SYNTHESIS_SYSTEM_PROMPT: &str = r#"你是辩证综合官。你是独立子 agent——只做辩证综合，不做评判。不读取文件——所有上下文已在 prompt 中。

## 你的核心任务

不是和稀泥——不是把各魂的观点凑成"各有道理"。你要做的是：识别真正的一致、暴露不能调和的冲突、标记所有魂都没看到的盲区、把摩擦作为信息而不是噪音来处理。

## 五步辩证综合法

### 1. 共识
各魂在哪些判断上独立抵达了相同或相近的结论？注意：
- 只有多个魂从**不同论证路径**抵达同一结论，才算真正的共识
- 如果两个魂的结论相似但论证逻辑完全不同——标注"表面共识，深层分歧"
- 如果所有魂的结论都一致——警惕：是否魂的选择有偏？是否任务本身限定了答案空间？

### 2. 分歧
各魂在哪些点上立场真正对立？区分三种分歧：
- **事实分歧**：对"发生了什么"的判断不同（可检验）
- **价值分歧**：对"什么重要/什么是对的"的判断不同（不可调和，只能承认）
- **前提分歧**：对"什么是最真实的/什么是知识的起点"的预设不同（元分歧——他们不是在争论同一件事，他们根本不在同一个现实里）

前提分歧是最深层、最容易被忽视的。当费曼说"只有可观测的才是真实的"而庄子说"道的整体不可分割"——这不是观点不同，是本体论承诺在不同的宇宙里。

### 3. 盲区
所有参与的魂都没有涉及、但对理解这个议题至关重要的维度和缺口。对每个盲区标记：
- 是否可由已有的魂覆盖（调另一个魂就能补）
- 还是需要新的魂类型（已有魂的本体论/认识论决定了它们结构性地看不到这个维度）

### 4. 主要矛盾
从共识、分歧、盲区中提炼出贯穿全局的核心张力。这不是"总结"，而是找出那个如果解决了其他问题都会跟着松动的根节点。标注矛盾的两极分别被哪个（些）魂代表。

### 5. 行动纲领
提出使用者可参考的方向。每个方向必须有：
- 具体可操作的内容（不是"注意平衡"这种空话）
- 建议的时间框架（立即/一周内/一月内/长期）
- 优先级（1-3，1最高）

## 重要规则

1. **分歧不许和谐掉**——如果两个魂确实站在不可通约的本体论预设上，不要说"综合来看双方各有道理"。诚实报告：它们不是在争论，是看不见彼此在说什么。

2. **引用魂名标注来源**——每个共识/分歧/盲区标注来自哪些魂。

3. **盲区不只是"没提到的话题"**——更深层的盲区是：所有魂共享了一个未言明的预设，而正是这个预设限制了思考。试着找出这种结构性盲区。

4. **综合官自身的盲区**——在报告最后，标注你认为这份综合本身可能遗漏了什么。你的立场（作为综合官）是否系统性地偏向某类结论？诚实标注。

5. **不要用形式替代思考**——五步结构是脚手架，不是填空题。如果某个步骤确实没有产出（例如没有共识），诚实地说"无"，而不是编造。

## 输出格式

用以下 Markdown 格式输出完整的综合报告：

# 辩证综合

**综合官**：辩证综合官
**参与魂**：列出所有魂名
**日期**：标注当前日期

---

## 一、共识（N项）

每项编号列出，格式：`1. **共识点** — 来源说明`

---

## 二、分歧（N项）

每项注明分歧类型（事实/价值/前提），格式：
`1. **分歧轴** — [前提分歧] A魂认为... vs B魂认为...`

---

## 三、盲区（N项）

每项标注是否可由已有魂覆盖：
`1. **盲区维度** — 说明 [不可由已有魂覆盖 / 可由X魂覆盖]`

---

## 四、主要矛盾

格式：`**矛盾描述** — 相关方：X魂与Y魂`

---

## 五、行动纲领

| # | 时限 | 内容 |
|---|------|------|
| 1 | 时限 | 具体行动 |

---

## 六、综合官自审

标注本综合可能遗漏的视角或维度。"#;

#[derive(Debug, Clone)]
pub struct PromptBuilder;

impl PromptBuilder {
    pub fn new() -> Self {
        PromptBuilder
    }

    pub fn build_summon_prompt(
        &self,
        soul: &SoulProfile,
        task: &str,
        judgment: Option<&str>,
        worry: Option<&str>,
        unknown: Option<&str>,
        tier: ModelTier,
        search_results: Option<&str>,
    ) -> Prompt {
        let system_content = Self::build_system_content(soul, &tier, "请始终保持角色一致性。用你的立场、术语和风格回应。禁止用括号添加动作/场景描写，直接输出观点。");
        let mut user_content = Self::build_user_presets_with_search(judgment, worry, unknown, search_results);
        user_content.push_str(&format!("任务：{}", task));
        Prompt {
            messages: vec![
                PromptMessage { role: "system".into(), content: system_content, reasoning_content: None },
                PromptMessage { role: "user".into(), content: user_content, reasoning_content: None },
            ],
        }
    }

    /// DeepSeek cache-optimized: 3-message split for prefix cache hit rate.
    /// Message order: (1) system/summon prompt (static) → (2) shared task context → (3) soul-specific instruction
    pub fn build_summon_cached(
        &self, soul: &SoulProfile, task: &str,
        judgment: Option<&str>, worry: Option<&str>, unknown: Option<&str>,
        tier: ModelTier, search_results: Option<&str>,
    ) -> Prompt {
        let system_content = Self::build_system_content(soul, &tier, "请始终保持角色一致性。禁止用括号添加动作/场景描写，直接输出观点。");
        let mut shared = Self::build_user_presets_with_search(judgment, worry, unknown, search_results);
        shared.push_str(&format!("任务：{}", task));
        let soul_specific = Self::build_soul_specific_prompt(soul, task, None);
        Prompt {
            messages: vec![
                PromptMessage { role: "system".into(), content: system_content, reasoning_content: None },
                PromptMessage { role: "user".into(), content: shared, reasoning_content: None },
                PromptMessage { role: "user".into(), content: soul_specific, reasoning_content: None },
            ],
        }
    }

    /// 带差异化任务分派的召唤。每个魂可以有自己的分析角度/子问题（task_card）。
    /// 这是产生高质量多视角输出的关键——不是让所有魂分析同一个问题，而是给每个魂
    /// 分配它在系统中最擅长回答的那个子问题。
    pub fn build_summon_with_task_card(
        &self, soul: &SoulProfile, shared_task: &str,
        task_card: &str,
        judgment: Option<&str>, worry: Option<&str>, unknown: Option<&str>,
        tier: ModelTier, search_results: Option<&str>,
    ) -> Prompt {
        let system_content = Self::build_system_content(soul, &tier, "请始终保持角色一致性。禁止用括号添加动作/场景描写，直接输出观点。");
        let mut shared = Self::build_user_presets_with_search(judgment, worry, unknown, search_results);
        shared.push_str(&format!("总任务背景：{}", shared_task));
        let soul_specific = Self::build_soul_specific_prompt(soul, shared_task, Some(task_card));
        Prompt {
            messages: vec![
                PromptMessage { role: "system".into(), content: system_content, reasoning_content: None },
                PromptMessage { role: "user".into(), content: shared, reasoning_content: None },
                PromptMessage { role: "user".into(), content: soul_specific, reasoning_content: None },
            ],
        }
    }

    /// 为魂构建专属的任务指令。利用魂的 self_declare、skills_expertise、compat/incompat 信息
    /// 生成更精准的分析指引，而非通用的一句话"请从你的立场分析"。
    fn build_soul_specific_prompt(soul: &SoulProfile, _shared_task: &str, task_card: Option<&str>) -> String {
        let mut prompt = format!(
            "## 你的角色\n你是 **{}**（ismism 坐标 **{}**）。\n\n",
            soul.name, soul.ismism_code
        );

        if !soul.self_declare.is_empty() {
            prompt.push_str(&format!("**你的自我声明**：{}\n\n", soul.self_declare));
        }

        if let Some(card) = task_card {
            prompt.push_str(&format!(
                "## 你的专属任务\n{}\n\n这是总任务中**只有你能做、其他魂做不到或做不好的部分**。请聚焦你的专属问题，用你的方法论框架深度回答。不要试图覆盖所有方面——覆盖你不擅长的方面反而会稀释你独特视角的价值。\n\n",
                card
            ));
        } else {
            prompt.push_str(&format!(
                "## 你的分析任务\n请从你（{}）的立场、本体论预设、认识论路径和目的论指向前提出发，对以上总任务进行深度分析。\n\n",
                soul.name
            ));
        }

        if !soul.skills_expertise.is_empty() {
            let skills: Vec<&str> = soul.skills_expertise.iter().map(|s| s.as_str()).take(5).collect();
            prompt.push_str(&format!("**你的核心能力**：{}\n\n", skills.join(" / ")));
        }

        prompt.push_str("## 分析要求\n\
1. **从你的本体论预设出发**——你默认什么是最真实的？这个预设在这次分析中让你看见了什么、又让你必然看不见什么？\n\
2. **使用你自己的方法论**——不要模仿其他魂的分析方式。你的价值恰恰在于你和别人不同。\n\
3. **诚实标注你的盲区**——在分析结尾，明确说「以我的框架，我看不见X」「我这套方法在Y条件下会失效」。\n\
4. **面向实践输出**——你的分析最终要能帮助使用者做决定或看清局面。不要停留在纯理论推演。\n\
5. **保持角色一致性**——用你自己的术语、风格和思维节奏。你是你，不是ChatGPT。\n");

        if let Some(card) = task_card {
            if card.contains("地基") || card.contains("是什么") {
                prompt.push_str("\n**特别注意**：你的任务是打地基——不要说名字，看东西。区分可观测的事实和给事实起的名字。从第一性原理出发。\n");
            }
            if card.contains("边界") || card.contains("不能") || card.contains("局限") {
                prompt.push_str("\n**特别注意**：你的任务是画边界——诚实地标注系统/方法论的局限。边界不是缺陷，是不自欺的前提。\n");
            }
            if card.contains("瞒") || card.contains("骗") || card.contains("自欺") || card.contains("解剖") {
                prompt.push_str("\n**特别注意**：你的任务是自我解剖——不是批判外部，是把刀对准自己。找出系统/方法论最不舒服的盲区。\n");
            }
        }

        prompt
    }

    /// 构建幡主审查 + 任务分派 Prompt。
    /// 幡主（默认未明子）审查候选魂组合是否适合该任务，并为每个魂分配专属的子任务。
    pub fn build_banner_lord_review_prompt(
        banner_lord: &SoulProfile,
        task: &str,
        candidate_souls: &[SoulProfile],
        judgment: Option<&str>,
        worry: Option<&str>,
        unknown: Option<&str>,
    ) -> Prompt {
        let system_content = format!(
            "{}你是{}，ismism坐标{}。你作为幡主审查官，需要完成两项任务：\n\
             1. 审查候选魂是否适合这个任务——不适合的要去掉或替换\n\
             2. 为每个确定使用的魂分派一个**差异化的子问题**——不是所有人分析同一个问题，\
             而是把你的总任务拆解成每个魂最擅长回答的那一个侧面\n\n\
             不读取文件——所有上下文已在 prompt 中。",
            banner_lord.summon_prompt, banner_lord.name, banner_lord.ismism_code
        );

        let mut candidates_info = String::new();
        for s in candidate_souls {
            let f = s.ismism_code.chars().next().unwrap_or('?');
            let o = s.ismism_code.chars().nth(2).unwrap_or('?');
            let e = s.ismism_code.chars().nth(4).unwrap_or('?');
            let t = s.ismism_code.chars().nth(6).unwrap_or('?');
            let self_decl = if s.self_declare.is_empty() { "无" } else { &s.self_declare };
            let skill = s.skills_expertise.first().map(|x| x.as_str()).unwrap_or("无");
            let excl_str = s.exclude_scenarios.join("、");
            let excl = if s.exclude_scenarios.is_empty() { "无" } else { excl_str.as_str() };
            candidates_info.push_str(&format!(
                "- **{}** [{}] 场域={} 本体论={} 认识论={} 目的论={}\n  self_declare={}\n  skills={}\n  exclude={}\n\n",
                s.name, s.field, f, o, e, t, self_decl, skill, excl
            ));
        }

        let mut user_content = format!(
            "## 总任务\n{}\n\n## 使用者预设\n判断：{}\n担忧：{}\n未知：{}\n\n## 候选魂\n{}\n",
            task,
            judgment.unwrap_or("无"),
            worry.unwrap_or("无"),
            unknown.unwrap_or("无"),
            candidates_info
        );

        user_content.push_str(r#"## 你的两阶段任务

### 第一阶段：审查魂组合
逐魂检查：
1. 魂的领域是否覆盖任务的相关维度？self_declare 的边界是否与任务冲突？
2. 场域定位是否匹配任务的性质？
3. 魂之间的场域/本体论/目的论是否互补？是否存在结构冗余（两个魂的同维度值相同=覆盖重复）？是否存在断裂（场域不兼容=无法对话）？
4. 是否缺少关键视角？（例如全是场域1的魂分析不到社会结构，全是场域4的魂分析不到理论前提）

裁决：pass（全部通过）/ conditional（增删某魂后通过）/ reject（全部重选）

### 第二阶段：差异化任务分派
为每个**确认使用的魂**分配一个不同的子问题（task_card）。原则：
- 不是每个人回答同一个问题——那会浪费多视角的价值
- 每个魂的子问题应该是**只有他能回答好、其他魂回答不好的**
- 利用魂的本体论/认识论差异——场域1的魂做地基（"这是什么"），场域2的魂做边界（"这看不到什么"），场域3的魂做自反（"这个问法本身有什么问题"），场域4的魂做实践（"怎么落地"）
- 每个子问题要具体——不是"请分析"，而是"请回答：X在Y条件下的Z"
- 如果某个魂不适合任何子问题——不在第一阶段通过它

### 输出格式
返回 JSON（不要 markdown 包裹）：
{
  "verdict": "pass|conditional|reject",
  "verified_souls": ["魂名1", "魂名2"],
  "task_cards": {
    "魂名1": "这个魂专属的子问题——具体、聚焦、只有他能回答好",
    "魂名2": "另一个魂专属的不同子问题"
  },
  "checks": ["逐魂审查结果"],
  "notes": "审查备注",
  "missing_perspectives": ["缺少的关键视角"],
  "boundary_risks": ["识别的边界风险"]
}"#);

        Prompt {
            messages: vec![
                PromptMessage { role: "system".into(), content: system_content, reasoning_content: None },
                PromptMessage { role: "user".into(), content: user_content, reasoning_content: None },
            ],
        }
    }

    pub fn build_synthesis_prompt(
        &self,
        task: &str,
        outputs: &[(String, String)],
    ) -> Prompt {
        let mut user_content = format!("## 任务\n{}\n\n## 各魂输出全文\n", task);
        for (name, content) in outputs {
            user_content.push_str(&format!("\n### {}\n{}\n", name, content));
        }
        user_content.push_str("\n请按五步辩证综合法进行综合。输出一份完整的综合报告。");
        Prompt {
            messages: vec![
                PromptMessage {
                    role: "system".into(),
                    content: SYNTHESIS_SYSTEM_PROMPT.into(),
                    reasoning_content: None,
                },
                PromptMessage { role: "user".into(), content: user_content, reasoning_content: None },
            ],
        }
    }

    /// 构建结构化综合报告的提示词（输出JSON）
    pub fn build_structured_synthesis_prompt(
        &self,
        task: &str,
        outputs: &[(String, String)],
    ) -> Prompt {
        let mut user_content = format!("## 任务\n{}\n\n## 各魂输出全文\n", task);
        for (name, content) in outputs {
            user_content.push_str(&format!("\n### {}\n{}\n", name, content));
        }
        user_content.push_str("\n请按五步辩证综合法输出结构化综合报告。");
        Prompt {
            messages: vec![
                PromptMessage {
                    role: "system".into(),
                    content: format!("{}\n\n## JSON 输出格式\n严格按以下 JSON 格式输出（不要 markdown 包裹）：\n{{\n  \"consensus\": [{{\"point\": \"共识内容\", \"shared_by\": [\"魂名1\", \"魂名2\"], \"depth\": \"独立抵达/表面共识\"}}],\n  \"divergence\": [{{\"axis\": \"分歧轴描述\", \"divergence_type\": \"事实/价值/前提\", \"positions\": [{{\"soul_name\": \"魂名\", \"stance\": \"立场\"}}]}}],\n  \"blind_spots\": [{{\"dimension\": \"盲区维度\", \"missing_perspective\": \"缺失的视角\", \"coverable_by_existing\": true/false, \"suggested_soul\": \"可选魂名\", \"is_structural\": true/false}}],\n  \"principal_contradiction\": {{\"description\": \"主要矛盾描述\", \"parties\": [\"相关方1\", \"相关方2\"]}},\n  \"action_program\": [{{\"direction\": \"行动方向\", \"rationale\": \"理由\", \"priority\": 1-3, \"timeline\": \"立即/一周/一月/长期\"}}],\n  \"synthesis_self_audit\": {{\"missing_perspectives\": [\"本综合可能遗漏的视角\"], \"synthesizer_bias\": \"综合官自身的潜在偏向\"}}\n}}\n\n规则：\n- divergence_type 必须是 事实/价值/前提 三者之一\n- is_structural=true 表示所有参与魂结构性地看不到这个维度（本体论/认识论限制）\n- priority 1=最高 3=最低\n- synthesis_self_audit 必须诚实标注——综合官也是站在某个立场上进行综合的", SYNTHESIS_SYSTEM_PROMPT),
                    reasoning_content: None,
                },
                PromptMessage { role: "user".into(), content: user_content, reasoning_content: None },
            ],
        }
    }

    /// 带碰撞检测结果的综合提示词——将多魂间的实时碰撞信息注入综合官，帮助它识别
    /// 深层结构冲突，而非仅从最终输出文本中寻找分歧。
    pub fn build_synthesis_with_collisions(
        &self,
        task: &str,
        outputs: &[(String, String)],
        collision_summary: &str,
    ) -> Prompt {
        let mut user_content = format!("## 任务\n{}\n\n## 各魂输出全文\n", task);
        for (name, content) in outputs {
            user_content.push_str(&format!("\n### {}\n{}\n", name, content));
        }
        user_content.push_str(&format!("\n## 实时碰撞检测摘要\n在魂并行输出过程中，系统检测到以下碰撞事件。这些事件可能揭示了输出文本中没有明确写出来、但在思维过程中实时发生的冲突。请综合这些碰撞信息进行辩证综合：\n\n{}", collision_summary));
        user_content.push_str("\n请按五步辩证综合法进行综合。输出一份完整的综合报告。");
        Prompt {
            messages: vec![
                PromptMessage {
                    role: "system".into(),
                    content: SYNTHESIS_SYSTEM_PROMPT.into(),
                    reasoning_content: None,
                },
                PromptMessage { role: "user".into(), content: user_content, reasoning_content: None },
            ],
        }
    }

    pub fn build_debate_prompt(
        &self,
        soul: &SoulProfile,
        opponent: &str,
        topic: &str,
        context: Option<&str>,
    ) -> Prompt {
        let system = format!(
            "你是 {}，ismism 坐标 {}。你正在与 {} 就以下议题进行辩论。请站在你的立场用你的思维方式进行论证和反驳。保持角色一致性。",
            soul.name, soul.ismism_code, opponent
        );
        let mut user = format!("辩论议题：{}\n\n", topic);
        if let Some(c) = context {
            user.push_str(&format!("已有论述：\n{}\n\n", c));
        }
        user.push_str("请发表你的观点。");
        Prompt {
            messages: vec![
                PromptMessage { role: "system".into(), content: system, reasoning_content: None },
                PromptMessage { role: "user".into(), content: user, reasoning_content: None },
            ],
        }
    }

    pub fn build_relay_prompt(
        &self,
        soul: &SoulProfile,
        prev_output: Option<&str>,
        task: &str,
    ) -> Prompt {
        let system = format!(
            "你是 {}，ismism 坐标 {}（{}）。\n\n{}",
            soul.name, soul.ismism_code, soul.field, soul.summon_prompt
        );
        let mut user = format!("任务：{}\n\n", task);
        if let Some(po) = prev_output {
            user.push_str(&format!("前一位魂的分析结果：\n{}\n\n请在此基础上继续推进，不必重复前面已有的分析。", po));
        } else {
            user.push_str("请开始你的分析。");
        }
        Prompt {
            messages: vec![
                PromptMessage { role: "system".into(), content: system, reasoning_content: None },
                PromptMessage { role: "user".into(), content: user, reasoning_content: None },
            ],
        }
    }

    pub fn build_review_prompt(&self, soul: &SoulProfile, output: &str) -> Prompt {
        Prompt {
            messages: vec![
                PromptMessage { role: "system".into(), content: "你是一个魂审查官。请审查以下魂的召唤效果和角色一致性。".into(), reasoning_content: None },
                PromptMessage {
                    role: "user".into(),
                    content: format!(
                        "魂名：{}\nismism：{}\n召唤提示：{}\n输出：{}\n\n请评价该魂是否保持了角色一致性，是否符合其 ismism 坐标所描述的立场。如有偏差请指出。",
                        soul.name, soul.ismism_code, soul.summon_prompt, output
                    ),
                    reasoning_content: None,
                },
            ],
        }
    }

    pub fn build_practice_opening_prompt(
        &self,
        soul: &SoulProfile,
        practitioner_data: &str,
    ) -> Prompt {
        Prompt {
            messages: vec![
                PromptMessage {
                    role: "system".into(),
                    content: format!(
                        "你是 {}，ismism 坐标 {}。你现在以实践开口模式运行。一个实践者（在场者）提供了其实践现场数据，需要你从自己的立场和理论视角进行消化分析。记住：你的分析要服务于实践者的行动改进。",
                        soul.name, soul.ismism_code
                    ),
                    reasoning_content: None,
                },
                PromptMessage {
                    role: "user".into(),
                    content: format!(
                        "实践者数据：\n{}\n\n请从你的理论视角对该实践数据进行消化分析，指出：\n1. 你看到了什么（描述）\n2. 这说明了什么（判断）\n3. 建议什么行动（输出）",
                        practitioner_data
                    ),
                    reasoning_content: None,
                },
            ],
        }
    }

    pub fn build_collect_prompt(&self, name: &str) -> Prompt {
        Prompt {
            messages: vec![
                PromptMessage {
                    role: "system".into(),
                    content: "你是一个人物研究助手。你的任务是对指定人物进行收魂（信息收集），输出结构化的 raw 素材。请基于你的知识提供以下6个维度的信息：".into(),
                    reasoning_content: None,
                },
                PromptMessage {
                    role: "user".into(),
                    content: format!(
                        "人物：{}\n\n请按以下维度输出（中文）：\n\n## 生平\n（生卒年、主要经历、时代背景）\n\n## 核心思想\n（主要理论/观点/贡献，3-5点）\n\n## 方法论\n（思维方式、分析方法、论证风格）\n\n## 代表作\n（主要著作、论文、演讲）\n\n## 影响与争议\n（对后世的影响、主要批判）\n\n## ismism 四维定位建议\n- 场域（1-4）：{}\n- 本体论（1-4）：{}\n- 认识论（1-4）：{}\n- 目的论（1-4）：{}",
                        name,
                        "场域(1-4)：1=形而下学(气学/自然科学), 2=形而上学(道学), 3=观念论(心学), 4=实践·辩证唯物主义",
                        "本体论(1-4)：1=同一性/循环/秩序, 2=分裂/冲突/二元对立, 3=中心化/中介/调和, 4=虚无/敞开/内在不可能性",
                        "认识论(1-4)：1=同一性/实证/循环, 2=分裂/建构/二元, 3=中心化/历史/辩证, 4=虚无/解构/敞开",
                        "目的论(1-4)：1=保守/秩序/同一, 2=多元/分裂/循环, 3=进步/中心化/调和, 4=革命/虚无/敞开",
                    ),
                    reasoning_content: None,
                },
            ],
        }
    }

    pub fn build_refine_prompt(&self, raw_material: &str) -> Prompt {
        Prompt {
            messages: vec![
                PromptMessage {
                    role: "system".into(),
                    content: "你是一个魂档案炼化师。请根据 raw 素材，生成结构化的 Soul Profile。ismism 编码必须基于素材中的实际内容，不可凭空编造。宁可标注不确定，不强行填入。".into(),
                    reasoning_content: None,
                },
                PromptMessage {
                    role: "user".into(),
                    content: format!(
                        "## Raw 素材\n{}\n\n## 主义主义 256 目录参考\n场域：1=形而下学(气学) 2=形而上学(道学) 3=观念论(心学) 4=实践·辩证唯物主义\n本体论/认识论/目的论：1=同一/循环 2=分裂/冲突 3=中心/调和 4=虚无/敞开\n\n## 输出格式\n请返回 JSON（不要 markdown 包裹）：\n{{\n  \"name\": \"\",\n  \"ismism_code\": \"f-o-e-t\",\n  \"field\": \"所属领域名\",\n  \"ontology\": \"本体论立场描述\",\n  \"epistemology\": \"认识论立场描述\",\n  \"teleology\": \"目的论立场描述\",\n  \"domains\": [\"领域1\",\"领域2\"],\n  \"tags\": [\"标签1\",\"标签2\"],\n  \"summon_prompt\": \"详细的召唤词（200-500字），包含角色设定、语言风格、思维方法指导、与使用者的互动方式。用中文输出。\",\n  \"rationale\": \"ismism 编码的理据（各维度为什么是这个值，参考256目录）\"\n}}\n\n规则：ismism 编码基于素材实际内容，不可凭空编造。宁可标近似，不强行塞入。",
                        raw_material
                    ),
                    reasoning_content: None,
                },
            ],
        }
    }

    fn build_system_content(soul: &SoulProfile, tier: &ModelTier, suffix: &str) -> String {
        let mut c = format!(
            "你是 {}，ismism 坐标 {}。\n\n",
            soul.name, soul.ismism_code
        );
        c.push_str(&soul.summon_prompt);
        c.push_str("\n\n");
        c.push_str(suffix);
        if matches!(tier, ModelTier::Pro | ModelTier::Max) {
            c.push_str("\n\n推理模式：深度思考，提供结构化分析。");
        }
        c
    }

    fn build_user_presets(judgment: Option<&str>, worry: Option<&str>, unknown: Option<&str>) -> String {
        Self::build_user_presets_with_search(judgment, worry, unknown, None)
    }

    fn build_user_presets_with_search(
        judgment: Option<&str>,
        worry: Option<&str>,
        unknown: Option<&str>,
        search_results: Option<&str>,
    ) -> String {
        let mut presets = String::new();
        if let Some(sr) = search_results {
            presets.push_str(&format!("## 议题背景（实时搜索）\n{}\n\n", sr));
        }
        if let Some(j) = judgment {
            presets.push_str(&format!("## 判断\n{}\n\n", j));
        }
        if let Some(w) = worry {
            presets.push_str(&format!("## 顾虑\n{}\n\n", w));
        }
        if let Some(u) = unknown {
            presets.push_str(&format!("## 未知\n{}\n\n", u));
        }
        presets
    }

    /// 根据任务类型和模型能力选择合适的提示词构建策略
    pub fn build_for_role(
        &self,
        role: RoutingRole,
        soul: Option<&SoulProfile>,
        task: &str,
        outputs: Option<&[(String, String)]>,
    ) -> Prompt {
        match role {
            RoutingRole::Synthesizer => {
                let outputs = outputs.unwrap_or(&[]);
                self.build_synthesis_prompt(task, outputs)
            }
            RoutingRole::Reviewer => {
                let soul = soul.expect("Reviewer requires a soul");
                self.build_review_prompt(soul, task)
            }
            RoutingRole::Soul => {
                let soul = soul.expect("Soul role requires a soul");
                self.build_summon_prompt(soul, task, None, None, None, ModelTier::Pro, None)
            }
            RoutingRole::KnowledgeCard => {
                self.build_collect_prompt(task)
            }
        }
    }
}
