use crate::EntryType;
use crate::PossessionInput;

const TRIGGER_THRESHOLD: u32 = 2;

const CONCRETE_MARKERS: &[&str] = &[
    "我的", "我公司", "我工厂", "我们", "最近", "昨天", "今天", "上周", "正在", "上次",
];

const FIRST_PERSON_MARKERS: &[&str] = &[
    "我做了", "我经历过", "我的项目", "我遇到", "我观察到",
];

const UNSEARCHABLE_MARKERS: &[&str] = &[
    "车间", "产线", "出货", "供应商", "工单", "来料", "客户", "竞品", "报价", "招投标",
];

const LEARNING_MARKERS: &[&str] = &[
    "学习", "了解", "是什么", "怎么理解", "教我", "解释一下",
];

const THEORY_MARKERS: &[&str] = &[
    "理论", "概念", "思想", "哲学", "方法论",
];

const DEBATE_MARKERS: &[&str] = &[
    "还是", "要么", "或者", "利弊", "优劣", "权衡", "两难", "选择", "取舍",
];

const OPPOSITION_MARKERS: &[&str] = &[
    "对立", "矛盾", "冲突", "辩论", "争论",
];

const PATH_MARKERS: &[&str] = &[
    "路线", "路径", "路线图", "阶段", "步骤", "流程",
];

const SEQUENCE_MARKERS: &[&str] = &[
    "然后", "接着", "下一步", "之后", "最终",
];

const STAGE_MARKERS: &[&str] = &[
    "第一阶段", "第二阶段", "第三阶段", "第一步", "第二步", "第三步",
    "首先", "其次", "最后", "P1", "P2", "P3", "1.", "2.", "3.",
];

fn any_contains(text: &str, markers: &[&str]) -> bool {
    markers.iter().any(|m| text.contains(m))
}

pub fn triage(input: &PossessionInput) -> EntryType {
    if let Some(ref mode) = input.mode {
        return match mode {
            foundation::PossessionMode::Single => EntryType::Single,
            foundation::PossessionMode::Conference => EntryType::Conference,
            foundation::PossessionMode::Debate => EntryType::Debate,
            foundation::PossessionMode::Relay => EntryType::Relay,
            foundation::PossessionMode::Learn => EntryType::Learn,
            foundation::PossessionMode::PracticeOpening => EntryType::PracticeOpening,
        };
    }

    let task = &input.task;
    let soul_count = input.souls.len();
    let has_topic = input.topic.is_some();

    let practice_score = practice_presence_score(task);
    let learn_score = learning_intent_score(task);
    let debate_score = debate_intent_score(task, soul_count);
    let relay_score = relay_intent_score(task);

    if practice_score >= TRIGGER_THRESHOLD {
        EntryType::PracticeOpening
    } else if debate_score >= TRIGGER_THRESHOLD || (soul_count == 2 && has_topic) {
        EntryType::Debate
    } else if learn_score >= TRIGGER_THRESHOLD && soul_count <= 1 {
        EntryType::Learn
    } else if relay_score >= TRIGGER_THRESHOLD || task_has_multi_stage(task) {
        EntryType::Relay
    } else if soul_count >= 2 {
        EntryType::Conference
    } else if soul_count == 1 || practice_score == 1 {
        EntryType::Single
    } else {
        EntryType::Single
    }
}

fn practice_presence_score(task: &str) -> u32 {
    let mut score = 0u32;
    if task.starts_with("我") { score += 1; }
    if any_contains(task, CONCRETE_MARKERS) { score += 1; }
    if any_contains(task, FIRST_PERSON_MARKERS) { score += 1; }
    if any_contains(task, UNSEARCHABLE_MARKERS) { score += 1; }
    score.min(3)
}

fn learning_intent_score(task: &str) -> u32 {
    let mut score = 0u32;
    if any_contains(task, LEARNING_MARKERS) { score += 1; }
    if any_contains(task, THEORY_MARKERS) { score += 1; }
    if practice_presence_score(task) == 0 && (task.ends_with('?') || task.ends_with('？')) {
        score += 1;
    }
    score.min(3)
}

fn debate_intent_score(task: &str, soul_count: usize) -> u32 {
    let mut score = 0u32;
    if any_contains(task, DEBATE_MARKERS) { score += 1; }
    if soul_count == 2 { score += 1; }
    if any_contains(task, OPPOSITION_MARKERS) { score += 1; }
    score.min(3)
}

fn relay_intent_score(task: &str) -> u32 {
    let mut score = 0u32;
    if any_contains(task, PATH_MARKERS) { score += 1; }
    if any_contains(task, SEQUENCE_MARKERS) { score += 1; }
    if task.contains("从") && task.contains("到") { score += 1; }
    score.min(3)
}

fn task_has_multi_stage(task: &str) -> bool {
    STAGE_MARKERS.iter().filter(|m| task.contains(*m)).count() >= 2
}
