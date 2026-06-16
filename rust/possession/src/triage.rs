use foundation::{DomainProfile, PossessionMode};
use crate::{EntryType, PossessionInput};

const TRIGGER_THRESHOLD: u32 = 2;

// Multi-stage markers — domain-independent structural patterns
const STAGE_MARKERS: &[&str] = &[
    "第一阶段", "第二阶段", "第三阶段", "第一步", "第二步", "第三步",
    "首先", "其次", "最后", "P1", "P2", "P3", "1.", "2.", "3.",
];

pub fn triage(input: &PossessionInput, domain: &DomainProfile) -> EntryType {
    // Explicit mode override from user
    if let Some(ref mode) = input.mode {
        return match mode {
            PossessionMode::Single => EntryType::Single,
            PossessionMode::Conference => EntryType::Conference,
            PossessionMode::Debate => EntryType::Debate,
            PossessionMode::Relay => EntryType::Relay,
            PossessionMode::Learn => EntryType::Learn,
            PossessionMode::PracticeOpening => EntryType::PracticeOpening,
        };
    }

    let task = &input.task;
    let soul_count = input.souls.len();
    let has_topic = input.topic.is_some();
    let tm = &domain.trigger_markers;

    // Practice: keyword hits + first-person prefix bonus
    let mut practice_score = count_matches(task, &tm.practice);
    if task.starts_with("我") { practice_score += 1; }
    practice_score = practice_score.min(3);

    // Learn: keyword hits + question-mark bonus (only when not practice-oriented)
    let mut learn_score = count_matches(task, &tm.learn);
    if practice_score == 0 && (task.ends_with('?') || task.ends_with('？')) {
        learn_score += 1;
    }
    learn_score = learn_score.min(3);

    // Debate: keyword hits + explicit 2-soul duel detection
    let mut debate_score = count_matches(task, &tm.debate);
    if soul_count == 2 { debate_score += 1; }
    debate_score = debate_score.min(3);

    // Relay: keyword hits + "从...到" pattern + multi-stage detection
    let mut relay_score = count_matches(task, &tm.relay);
    if task.contains("从") && task.contains("到") { relay_score += 1; }
    relay_score = relay_score.min(3);

    if practice_score >= TRIGGER_THRESHOLD {
        EntryType::PracticeOpening
    } else if debate_score >= TRIGGER_THRESHOLD || (soul_count == 2 && has_topic) {
        EntryType::Debate
    } else if learn_score >= TRIGGER_THRESHOLD && soul_count <= 1 {
        EntryType::Learn
    } else if relay_score >= TRIGGER_THRESHOLD || has_multi_stage(task) {
        EntryType::Relay
    } else if soul_count >= 2 {
        EntryType::Conference
    } else if soul_count == 1 || practice_score == 1 {
        EntryType::Single
    } else {
        EntryType::Single
    }
}

fn count_matches(task: &str, markers: &[String]) -> u32 {
    let mut score = 0u32;
    for m in markers {
        if task.contains(m.as_str()) {
            score += 1;
            if score >= 3 { break; }
        }
    }
    score
}

fn has_multi_stage(task: &str) -> bool {
    STAGE_MARKERS.iter().filter(|m| task.contains(*m)).count() >= 2
}
