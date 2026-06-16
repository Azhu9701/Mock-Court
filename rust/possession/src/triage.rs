use foundation::{DomainProfile, PossessionMode};
use crate::{EntryType, PossessionInput};

const TRIGGER_THRESHOLD: u32 = 2;

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

    let tm = &domain.trigger_markers;

    let practice_score = score_markers(task, &tm.practice).min(3);
    let learn_score = score_markers(task, &tm.learn).min(3);
    let debate_score = {
        let mut s = score_markers(task, &tm.debate);
        if soul_count == 2 { s += 1; }
        s.min(3)
    };
    let relay_score = score_markers(task, &tm.relay).min(3);

    if practice_score >= TRIGGER_THRESHOLD {
        EntryType::PracticeOpening
    } else if debate_score >= TRIGGER_THRESHOLD {
        EntryType::Debate
    } else if learn_score >= TRIGGER_THRESHOLD && soul_count <= 1 {
        EntryType::Learn
    } else if relay_score >= TRIGGER_THRESHOLD {
        EntryType::Relay
    } else if soul_count >= 2 {
        EntryType::Conference
    } else {
        EntryType::Single
    }
}

fn score_markers(task: &str, markers: &[String]) -> u32 {
    let mut score = 0u32;
    for m in markers {
        if task.contains(m.as_str()) {
            score += 1;
            if score >= 3 { break; }
        }
    }
    score
}
