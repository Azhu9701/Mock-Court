use std::collections::HashMap;

use foundation::{IsmismCode, SoulListEntry, SoulMatch, SoulProfile};

pub fn tokenize(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    let mut tokens: Vec<String> = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if is_cjk(c) {
            tokens.push(c.to_string());
            if i + 1 < chars.len() && is_cjk(chars[i + 1]) {
                tokens.push(format!("{}{}", c, chars[i + 1]));
            }
            i += 1;
        } else if c.is_alphanumeric() {
            let mut word = String::new();
            while i < chars.len() && chars[i].is_alphanumeric() {
                word.push(chars[i]);
                i += 1;
            }
            tokens.push(word);
        } else {
            i += 1;
        }
    }

    tokens.sort();
    tokens.dedup();
    tokens
}

fn is_cjk(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c)
        || ('\u{3400}'..='\u{4DBF}').contains(&c)
        || ('\u{F900}'..='\u{FAFF}').contains(&c)
}

pub fn build_inverted_index(profiles: &HashMap<String, SoulProfile>) -> HashMap<String, Vec<String>> {
    let mut index: HashMap<String, Vec<String>> = HashMap::new();

    for (name, profile) in profiles {
        let fields = collect_text_fields(profile);
        for (_, text) in &fields {
            for token in tokenize(text) {
                index.entry(token).or_default().push(name.clone());
            }
        }
    }

    for names in index.values_mut() {
        names.sort();
        names.dedup();
    }

    index
}

fn collect_text_fields(profile: &SoulProfile) -> Vec<(&'static str, String)> {
    vec![
        ("name", profile.name.clone()),
        ("field", profile.field.clone()),
        ("ontology", profile.ontology.clone()),
        ("epistemology", profile.epistemology.clone()),
        ("teleology", profile.teleology.clone()),
        ("ismism_code", profile.ismism_code.clone()),
        ("tags", profile.tags.join(" ")),
        ("summon_prompt", profile.summon_prompt.clone()),
    ]
}

pub fn fulltext_search(
    query: &str,
    profiles: &HashMap<String, SoulProfile>,
    index: &HashMap<String, Vec<String>>,
) -> Vec<SoulMatch> {
    let query_tokens = tokenize(query);
    if query_tokens.is_empty() {
        return vec![];
    }

    let mut scores: HashMap<String, (f64, Vec<String>)> = HashMap::new();

    for token in &query_tokens {
        if let Some(matched_names) = index.get(token) {
            for name in matched_names {
                if let Some(profile) = profiles.get(name) {
                    let (score, matched) = scores.entry(name.clone()).or_insert((0.0, vec![]));
                    let (field_hits, hit_fields) = count_token_hits(profile, token);
                    *score += field_hits;
                    for f in hit_fields {
                        if !matched.contains(&f) {
                            matched.push(f);
                        }
                    }
                }
            }
        }
    }

    let mut results: Vec<SoulMatch> = scores
        .into_iter()
        .filter_map(|(name, (score, matched_fields))| {
            profiles.get(&name).map(|p| SoulMatch {
                entry: SoulListEntry::from(p),
                relevance: score,
                matched_fields,
            })
        })
        .collect();

    results.sort_by(|a, b| {
        let score_a = a.relevance + cold_start_boost(a.entry.summon_count);
        let score_b = b.relevance + cold_start_boost(b.entry.summon_count);
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results
}

/// 冷启动加权：summon_count < 3 的魂获得额外 relevance 加分，打破马太效应。
///
/// summon_count = 0 → +2.0, 1 → +1.3, 2 → +0.7, >=3 → 0
fn cold_start_boost(summon_count: u32) -> f64 {
    match summon_count {
        0 => 2.0,
        1 => 1.3,
        2 => 0.7,
        _ => 0.0,
    }
}

fn count_token_hits(profile: &SoulProfile, token: &str) -> (f64, Vec<String>) {
    let lower_token = token.to_lowercase();
    let mut hits = 0.0;
    let mut fields = vec![];

    let tags_joined = profile.tags.join(" ");

    let field_data: Vec<(&str, &str, f64)> = vec![
        ("name", &profile.name, 5.0),
        ("tags", &tags_joined, 3.0),
        ("ismism_code", &profile.ismism_code, 3.0),
        ("field", &profile.field, 2.0),
        ("ontology", &profile.ontology, 2.0),
        ("epistemology", &profile.epistemology, 2.0),
        ("teleology", &profile.teleology, 2.0),
        ("summon_prompt", &profile.summon_prompt, 1.0),
    ];

    for (field_name, text, weight) in &field_data {
        let count = text.to_lowercase().matches(&lower_token).count() as f64;
        if count > 0.0 {
            hits += count * weight;
            fields.push(field_name.to_string());
        }
    }

    (hits, fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cold_start_boost() {
        assert!((cold_start_boost(0) - 2.0).abs() < 0.001);
        assert!((cold_start_boost(1) - 1.3).abs() < 0.001);
        assert!((cold_start_boost(2) - 0.7).abs() < 0.001);
        assert!((cold_start_boost(3) - 0.0).abs() < 0.001);
        assert!((cold_start_boost(10) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_cold_start_boost_favors_new_souls() {
        // 新魂 (summon=0, relevance=3.0) 应该排在老魂 (summon=10, relevance=4.0) 前面
        let new_score = 3.0 + cold_start_boost(0);  // 5.0
        let old_score = 4.0 + cold_start_boost(10); // 4.0
        assert!(new_score > old_score);
    }

    #[test]
    fn test_cold_start_boost_does_not_override_strong_relevance() {
        // 高相关老魂仍然应该排在低相关新魂前面
        let strong_old = 8.0 + cold_start_boost(10); // 8.0
        let weak_new = 2.0 + cold_start_boost(0);    // 4.0
        assert!(strong_old > weak_new);
    }
}

pub fn nearest_search(
    target: &IsmismCode,
    profiles: &HashMap<String, SoulProfile>,
    limit: Option<usize>,
) -> Vec<SoulMatch> {
    let mut results: Vec<SoulMatch> = profiles
        .iter()
        .filter_map(|(_name, profile)| {
            IsmismCode::try_from(profile.ismism_code.as_str()).ok().map(|code| {
                let distance = target.distance(&code, None);
                let relevance = 1.0 / (1.0 + distance);
                SoulMatch {
                    entry: SoulListEntry::from(profile),
                    relevance,
                    matched_fields: vec!["ismism".into()],
                }
            })
        })
        .collect();

    results.sort_by(|a, b| {
        let score_a = a.relevance + cold_start_boost(a.entry.summon_count);
        let score_b = b.relevance + cold_start_boost(b.entry.summon_count);
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if let Some(n) = limit {
        results.truncate(n);
    }
    results
}
