use ai_gateway::GatewayRegistry;
use foundation::Result;
use registry::SoulRegistry;

/// 教学步骤
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LessonStep {
    /// 步骤序号 (1-based)
    pub step_number: u32,
    /// 步骤标题
    pub title: String,
    /// 步骤内容
    pub content: String,
    /// 教学方法标签：如 "讲解", "示例", "类比", "提问", "总结"
    pub method: String,
}

/// 测验题目
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuizQuestion {
    /// 问题文本
    pub question: String,
    /// 选项列表
    pub options: Vec<String>,
    /// 正确答案索引 (0-based)
    pub correct_answer: usize,
    /// 答案解释
    pub explanation: String,
}

/// 魂间教学会话
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeachingSession {
    /// 教师魂名
    pub teacher: String,
    /// 学生魂名
    pub student: String,
    /// 教学主题
    pub topic: String,
    /// 教师对学生盲区的分析
    pub teacher_analysis: String,
    /// 教学计划
    pub lesson_plan: Vec<LessonStep>,
    /// 测验题目
    pub quiz: Vec<QuizQuestion>,
    /// 教学效果评分 (0.0 ~ 1.0)
    pub score: f32,
    /// 反馈文本
    pub feedback: String,
}

impl TeachingSession {
    pub fn new(teacher: String, student: String, topic: String) -> Self {
        TeachingSession {
            teacher,
            student,
            topic,
            teacher_analysis: String::new(),
            lesson_plan: Vec::new(),
            quiz: Vec::new(),
            score: 0.0,
            feedback: String::new(),
        }
    }
}

/// 构建教师分析学生盲区的 prompt
pub fn build_teacher_analysis_prompt(
    teacher_profile: &foundation::SoulProfile,
    student_profile: &foundation::SoulProfile,
    topic: &str,
) -> String {
    let mut prompt = String::new();
    prompt.push_str(&format!(
        "你是一位教师，名叫「{}」。你的方法论特点是：{}\n\n",
        teacher_profile.name, teacher_profile.mind
    ));
    prompt.push_str(&format!(
        "你的学生是「{}」，其方法论特点是：{}\n\n",
        student_profile.name, student_profile.mind
    ));
    prompt.push_str(&format!(
        "教学主题：{}\n\n", topic
    ));
    prompt.push_str(
        "## 任务：分析学生的盲区\n\n"
    );
    prompt.push_str(
        "请从以下维度分析学生在「{topic}」上的知识盲区和认知局限：\n"
    );
    prompt.push_str(
        "1. **领域知识缺口**：学生对该主题缺少哪些关键概念或事实？\n"
    );
    prompt.push_str(
        "2. **方法论局限**：学生的推理风格可能导致哪些系统性的遮蔽？\n"
    );
    prompt.push_str(
        "3. **前提盲区**：学生有哪些未言明的预设需要被揭示？\n"
    );
    prompt.push_str(
        "4. **交叉点**：你的方法论可以从哪些角度补全学生的视角？\n\n"
    );
    prompt.push_str(
        "输出格式：用清晰的段落逐一说明每条盲区，每个盲区标注严重程度（高/中/低）。\n"
    );

    prompt
}

/// 构建教师制定教学计划的 prompt
pub fn build_lesson_plan_prompt(
    teacher_profile: &foundation::SoulProfile,
    student_profile: &foundation::SoulProfile,
    topic: &str,
    analysis: &str,
) -> String {
    let mut prompt = String::new();
    prompt.push_str(&format!(
        "你是一位教师「{}」，教学方法：{}\n\n",
        teacher_profile.name, teacher_profile.mind
    ));
    prompt.push_str(&format!(
        "学生「{}」需要学习：{}\n\n",
        student_profile.name, topic
    ));
    prompt.push_str("## 学生盲区分析（已完成）\n\n");
    prompt.push_str(analysis);
    prompt.push_str("\n\n## 任务：制定教学计划\n\n");
    prompt.push_str(
        "请制定一个 3-5 步的教学计划，格式如下：\n\n"
    );
    prompt.push_str(
        "```\n"
    );
    prompt.push_str(
        "步骤N: [标题] (方法：讲解/示例/类比/提问/总结)\n"
    );
    prompt.push_str("[内容]\n");
    prompt.push_str("```\n\n");
    prompt.push_str(
        "要求：\n"
    );
    prompt.push_str(
        "- 步骤由浅入深，每步解决一个明确的盲区\n"
    );
    prompt.push_str(
        "- 用学生能理解的框架解释（考虑学生的方法论偏好）\n"
    );
    prompt.push_str(
        "- 使用类比和具体示例\n"
    );
    prompt.push_str(
        "- 最后一步应包含总结和自查要点\n"
    );

    prompt
}

/// 构建教师出测验题的 prompt
pub fn build_quiz_prompt(
    teacher_profile: &foundation::SoulProfile,
    student_profile: &foundation::SoulProfile,
    topic: &str,
    lesson_plan: &[LessonStep],
) -> String {
    let mut prompt = String::new();
    prompt.push_str(&format!(
        "你是一位教师「{}」。\n\n",
        teacher_profile.name
    ));
    prompt.push_str(&format!(
        "你已经为学生「{}」讲授了「{}」，教学步骤如下：\n\n",
        student_profile.name, topic
    ));

    for step in lesson_plan {
        prompt.push_str(&format!(
            "步骤{} - {} (方法: {})\n",
            step.step_number, step.title, step.method
        ));
    }

    prompt.push_str("\n## 任务：出测验题\n\n");
    prompt.push_str(
        "请出 3-5 道选择题，格式如下：\n\n"
    );
    prompt.push_str(
        "```\n"
    );
    prompt.push_str(
        "Q: [问题]\n"
    );
    prompt.push_str(
        "A) [选项A]\n"
    );
    prompt.push_str(
        "B) [选项B]\n"
    );
    prompt.push_str(
        "C) [选项C]\n"
    );
    prompt.push_str(
        "D) [选项D]\n"
    );
    prompt.push_str(
        "正确答案: [A/B/C/D]\n"
    );
    prompt.push_str(
        "解释: [为什么这是正确答案]\n");
    prompt.push_str("```\n\n");
    prompt.push_str(
        "要求：\n"
    );
    prompt.push_str(
        "- 每道题对应教学计划中的一个关键点\n"
    );
    prompt.push_str(
        "- 选项要有区分度，错误选项应是常见误解\n"
    );
    prompt.push_str(
        "- 解释要说明为什么正确答案是对的、其他选项为什么不对\n"
    );

    prompt
}

/// 构建整体教学 session 的 prompt（综合版，单次调用）
///
/// 将分析、教学计划、出题合并到一次 prompt 中，适合简单教学场景。
pub fn build_full_teaching_prompt(
    teacher_profile: &foundation::SoulProfile,
    student_profile: &foundation::SoulProfile,
    topic: &str,
) -> String {
    let mut prompt = String::new();
    prompt.push_str(&format!(
        "## 角色\n你是「{}」，一位教师。\n", teacher_profile.name
    ));
    prompt.push_str(&format!(
        "你的教学方法论：{}\n\n", teacher_profile.mind
    ));
    prompt.push_str(&format!(
        "## 学生\n「{}」，方法论特点：{}\n\n", student_profile.name, student_profile.mind
    ));
    prompt.push_str(&format!(
        "## 教学主题\n{}\n\n", topic
    ));
    prompt.push_str(
        "## 任务\n请完成以下三个部分的输出：\n\n"
    );
    prompt.push_str(
        "### 第一部分：学生盲区分析\n"
    );
    prompt.push_str(
        "从你的方法论视角，分析学生对「{topic}」的知识盲区和认知局限。标注每个盲区的严重程度。\n\n"
    );
    prompt.push_str(
        "### 第二部分：教学计划\n"
    );
    prompt.push_str(
        "制定 3-5 步教学计划，步骤由浅入深。用学生能理解的框架解释，善用类比和示例。\n\n"
    );
    prompt.push_str(
        "### 第三部分：测验\n"
    );
    prompt.push_str(
        "出 3-5 道选择题测试学生对主题的理解。每题标注正确答案并附解释。\n"
    );

    prompt
}

/// 运行魂间教学会话
///
/// 教师魂分析学生盲区、制定教学计划、出测验题，生成完整的 TeachingSession。
/// 实际使用时，上层调用者应：
/// 1. 调用 build_teacher_analysis_prompt 生成分析 prompt
/// 2. 通过 gateway 调用 teacher soul 获取分析
/// 3. 调用 build_lesson_plan_prompt 生成教学计划 prompt
/// 4. 通过 gateway 获取教学计划
/// 5. 调用 build_quiz_prompt 生成测验 prompt
/// 6. 通过 gateway 获取测验
/// 7. 组装 TeachingSession
pub async fn run_teaching_session(
    teacher: &str,
    student: &str,
    topic: &str,
    _gateway: &GatewayRegistry,
    registry: &SoulRegistry,
) -> Result<TeachingSession> {
    let teacher_profile = registry.get_soul(teacher)?;
    let student_profile = registry.get_soul(student)?;

    let mut session = TeachingSession::new(
        teacher.to_string(),
        student.to_string(),
        topic.to_string(),
    );

    // 生成分析 prompt（实际调用由上层完成）
    let analysis_prompt = build_teacher_analysis_prompt(
        &teacher_profile,
        &student_profile,
        topic,
    );
    session.teacher_analysis = analysis_prompt;

    // 生成教学计划 prompt（实际调用由上层完成）
    let lesson_prompt = build_lesson_plan_prompt(
        &teacher_profile,
        &student_profile,
        topic,
        &session.teacher_analysis,
    );

    // 示例教学步骤（占位，实际由 LLM 生成后填充）
    session.lesson_plan = vec![
        LessonStep {
            step_number: 1,
            title: "概念引入".to_string(),
            content: lesson_prompt,
            method: "讲解".to_string(),
        },
    ];

    // 生成测验 prompt（实际调用由上层完成）
    let quiz_prompt = build_quiz_prompt(
        &teacher_profile,
        &student_profile,
        topic,
        &session.lesson_plan,
    );
    session.feedback = quiz_prompt;

    Ok(session)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_profile(name: &str, mind: &str) -> foundation::SoulProfile {
        foundation::SoulProfile {
            name: name.to_string(),
            ismism_code: "0-0-0-0".to_string(),
            field: String::new(),
            ontology: String::new(),
            epistemology: String::new(),
            teleology: String::new(),
            domains: vec![],
            exclude_scenarios: vec![],
            summon_count: 0,
            effectiveness: foundation::EffectivenessStats::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec![],
            summon_prompt: format!("You are {}", name),
            practice_observations: vec![],
            title: String::new(),
            description: String::new(),
            voice: String::new(),
            mind: mind.to_string(),
            self_declare: String::new(),
            skills_expertise: vec![],
            model: String::new(),
            tools: String::new(),
            trigger_keywords: vec![],
            compat: vec![],
            incompat: vec![],
        }
    }

    #[test]
    fn test_teaching_session_new() {
        let session = TeachingSession::new(
            "费曼".into(),
            "乔布斯".into(),
            "科学方法".into(),
        );

        assert_eq!(session.teacher, "费曼");
        assert_eq!(session.student, "乔布斯");
        assert_eq!(session.topic, "科学方法");
        assert_eq!(session.score, 0.0);
        assert!(session.lesson_plan.is_empty());
        assert!(session.quiz.is_empty());
    }

    #[test]
    fn test_build_teacher_analysis_prompt() {
        let teacher = make_profile("费曼", "科学还原论——从第一性原理出发，用实验检验一切");
        let student = make_profile("乔布斯", "直觉设计——相信简洁和直觉的力量");

        let prompt = build_teacher_analysis_prompt(&teacher, &student, "科学方法");

        assert!(prompt.contains("费曼"));
        assert!(prompt.contains("乔布斯"));
        assert!(prompt.contains("科学方法"));
        assert!(prompt.contains("科学还原论"));
        assert!(prompt.contains("直觉设计"));
        assert!(prompt.contains("盲区"));
    }

    #[test]
    fn test_build_lesson_plan_prompt() {
        let teacher = make_profile("费曼", "还原论");
        let student = make_profile("乔布斯", "直觉");
        let analysis = "学生对实验验证的重要性认识不足";

        let prompt = build_lesson_plan_prompt(&teacher, &student, "科学方法", analysis);

        assert!(prompt.contains("费曼"));
        assert!(prompt.contains("乔布斯"));
        assert!(prompt.contains(analysis));
        assert!(prompt.contains("教学计划"));
    }

    #[test]
    fn test_build_quiz_prompt() {
        let teacher = make_profile("费曼", "还原论");
        let student = make_profile("乔布斯", "直觉");
        let lesson_plan = vec![LessonStep {
            step_number: 1,
            title: "科学方法入门".to_string(),
            content: "科学方法的核心是可检验性".to_string(),
            method: "讲解".to_string(),
        }];

        let prompt = build_quiz_prompt(&teacher, &student, "科学方法", &lesson_plan);

        assert!(prompt.contains("费曼"));
        assert!(prompt.contains("科学方法入门"));
        assert!(prompt.contains("正确答案"));
    }

    #[test]
    fn test_build_full_teaching_prompt() {
        let teacher = make_profile("费曼", "还原论");
        let student = make_profile("乔布斯", "直觉");

        let prompt = build_full_teaching_prompt(&teacher, &student, "科学方法");

        assert!(prompt.contains("费曼"));
        assert!(prompt.contains("乔布斯"));
        assert!(prompt.contains("科学方法"));
        assert!(prompt.contains("盲区分析"));
        assert!(prompt.contains("教学计划"));
        assert!(prompt.contains("测验"));
    }
}
