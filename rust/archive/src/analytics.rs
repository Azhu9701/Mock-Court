use std::collections::HashMap;

use foundation::{
    CallFilter, Effectiveness, PossessionMode, Result, SessionFilter, Storage,
};

use crate::{
    AlertType, BoundaryReview, EffectivenessTrend, Period, PleasureStats, SoulAlert, SoulCallStats, SummonStats,
};

pub async fn compute_summon_stats(store: &dyn Storage, period: &Period) -> Result<SummonStats> {
    let records = store
        .query_call_records(&CallFilter::default())
        .await?;

    let total_souls_available = store.list_soul_names().await?.len();

    let filtered: Vec<_> = records
        .into_iter()
        .filter(|r| r.created_at >= period.start && r.created_at <= period.end)
        .collect();

    let total_calls = filtered.len();
    let total_tokens: u64 = filtered.iter().map(|r| r.usage.total_tokens as u64).sum();
    let mut soul_names = std::collections::HashSet::new();
    let mut by_mode: HashMap<PossessionMode, usize> = HashMap::new();
    let mut by_soul_map: HashMap<String, SoulCallStats> = HashMap::new();

    for r in &filtered {
        soul_names.insert(r.soul_name.clone());
        *by_mode.entry(r.mode.clone()).or_insert(0) += 1;

        let entry = by_soul_map
            .entry(r.soul_name.clone())
            .or_insert_with(|| SoulCallStats {
                soul_name: r.soul_name.clone(),
                call_count: 0,
                effective_count: 0,
                partial_count: 0,
                invalid_count: 0,
                total_tokens: 0,
            });
        entry.call_count += 1;
        entry.total_tokens += r.usage.total_tokens as u64;
        match r.effectiveness {
            Effectiveness::Effective => entry.effective_count += 1,
            Effectiveness::Partial => entry.partial_count += 1,
            Effectiveness::Invalid => entry.invalid_count += 1,
        }
    }

    let mut by_soul: Vec<SoulCallStats> = by_soul_map.into_values().collect();
    by_soul.sort_by_key(|s| std::cmp::Reverse(s.call_count));

    Ok(SummonStats {
        total_calls,
        unique_souls_called: soul_names.len(),
        total_souls_available,
        total_tokens,
        by_mode,
        by_soul,
        period_start: period.start,
        period_end: period.end,
    })
}

pub async fn compute_soul_effectiveness(
    store: &dyn Storage,
    soul: &str,
) -> Result<EffectivenessTrend> {
    let records = store
        .query_call_records(&CallFilter {
            soul_name: Some(soul.to_string()),
            ..Default::default()
        })
        .await?;

    let total = records.len();
    let effective = records
        .iter()
        .filter(|r| matches!(r.effectiveness, Effectiveness::Effective))
        .count();
    let partial = records
        .iter()
        .filter(|r| matches!(r.effectiveness, Effectiveness::Partial))
        .count();
    let invalid = records
        .iter()
        .filter(|r| matches!(r.effectiveness, Effectiveness::Invalid))
        .count();

    let rate = if total > 0 {
        effective as f64 / total as f64
    } else {
        0.0
    };

    Ok(EffectivenessTrend {
        soul_name: soul.to_string(),
        total_calls: total,
        effective,
        partial,
        invalid,
        effective_rate: rate,
    })
}

pub async fn compute_mode_distribution(
    store: &dyn Storage,
) -> Result<HashMap<PossessionMode, usize>> {
    let sessions = store
        .list_sessions(&SessionFilter::default())
        .await?;

    let mut dist = HashMap::new();
    for s in &sessions {
        *dist.entry(s.mode.clone()).or_insert(0) += 1;
    }
    Ok(dist)
}

pub async fn detect_unsummoned_souls_impl(
    store: &dyn Storage,
    threshold_days: u32,
) -> Result<Vec<SoulAlert>> {
    let records = store
        .query_call_records(&CallFilter::default())
        .await?;

    let threshold = chrono::Utc::now() - chrono::Duration::days(threshold_days as i64);

    let mut called_souls: HashMap<String, chrono::DateTime<chrono::Utc>> = HashMap::new();
    for r in &records {
        let entry = called_souls
            .entry(r.soul_name.clone())
            .or_insert(r.created_at);
        if r.created_at > *entry {
            *entry = r.created_at;
        }
    }

    let soul_names = store.list_soul_names().await?;
    let mut alerts = Vec::new();

    for name in &soul_names {
        match called_souls.get(name) {
            None => {
                alerts.push(SoulAlert {
                    soul_name: name.clone(),
                    alert_type: AlertType::NeverSummoned,
                    detail: format!("魂魄 {} 从未被召唤", name),
                });
            }
            Some(last_called) if *last_called < threshold => {
                let days = (chrono::Utc::now() - *last_called).num_days();
                alerts.push(SoulAlert {
                    soul_name: name.clone(),
                    alert_type: AlertType::UnsummonedLongDuration,
                    detail: format!("魂魄 {} 已 {} 天未被召唤", name, days),
                });
            }
            _ => {}
        }
    }

    Ok(alerts)
}

pub async fn detect_low_effectiveness_impl(
    store: &dyn Storage,
    threshold: f64,
) -> Result<Vec<BoundaryReview>> {
    let records = store
        .query_call_records(&CallFilter::default())
        .await?;

    let mut by_soul: HashMap<String, (usize, usize)> = HashMap::new();
    for r in &records {
        let entry = by_soul.entry(r.soul_name.clone()).or_insert((0, 0));
        entry.0 += 1;
        if matches!(r.effectiveness, Effectiveness::Effective) {
            entry.1 += 1;
        }
    }

    let mut reviews: Vec<BoundaryReview> = by_soul
        .into_iter()
        .filter_map(|(name, (total, effective))| {
            if total < 5 {
                return None;
            }
            let rate = effective as f64 / total as f64;
            if rate < threshold {
                Some(BoundaryReview {
                    soul_name: name,
                    effective_rate: rate,
                    total_calls: total,
                    threshold,
                    recommendation: "请进行实践审查，考虑修正召唤参数或散魂".to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    reviews.sort_by(|a, b| {
        a.effective_rate
            .partial_cmp(&b.effective_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(reviews)
}

/// 蛇皮统计 — 从 session_reviews + 全体会话聚合实践闭环数据
/// 核心逻辑：没有走反馈闭环的会话 = 消费型会话
pub async fn compute_pleasure_stats(store: &dyn Storage) -> Result<PleasureStats> {
    let reviews = store.get_recent_reviews(500).await?;
    let sessions = store.list_sessions(&SessionFilter::default()).await?;

    // 已走反馈闭环的 session_id → review
    let review_map: std::collections::HashMap<String, &foundation::SessionReview> = reviews
        .iter()
        .map(|r| (r.session_id.clone(), r))
        .collect();

    let mut effective = 0usize;
    let mut partial = 0usize;
    let mut invalid = 0usize;
    let mut wasted_tokens: u64 = 0;
    let mut total_tokens: u64 = 0;

    for session in &sessions {
        let tokens = session.total_tokens as u64;
        total_tokens += tokens;

        match review_map.get(&session.id) {
            Some(review) => {
                // 走了闭环：按 effectiveness 判定
                match review.effectiveness.as_str() {
                    "effective" => effective += 1,
                    "partial" => {
                        partial += 1;
                        wasted_tokens += tokens;
                    }
                    _ => {
                        invalid += 1;
                        wasted_tokens += tokens;
                    }
                }
            }
            None => {
                // 没走闭环：消费型会话
                invalid += 1;
                wasted_tokens += tokens;
            }
        }
    }

    let total_reviewed = effective + partial + invalid;
    let pleasure_index = if total_reviewed > 0 {
        ((invalid as f64 + partial as f64 * 0.5) / total_reviewed as f64) * 100.0
    } else {
        0.0
    };

    let waste_ratio = if total_tokens > 0 {
        wasted_tokens as f64 / total_tokens as f64
    } else {
        0.0
    };

    Ok(PleasureStats {
        pleasure_index,
        effective_sessions: effective,
        partial_sessions: partial,
        invalid_sessions: invalid,
        total_reviewed,
        wasted_tokens,
        total_tokens,
        waste_ratio,
    })
}
