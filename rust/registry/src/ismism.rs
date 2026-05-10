use foundation::{IsmismCode, IsmismStats, SoulListEntry};

pub fn parse_ismism(s: &str) -> Result<IsmismCode, String> {
    IsmismCode::try_from(s)
}

#[allow(dead_code)]
pub fn ismism_distance(a: &IsmismCode, b: &IsmismCode) -> f64 {
    a.distance(b, None)
}

#[allow(dead_code)]
pub fn ismism_distance_weighted(
    a: &IsmismCode,
    b: &IsmismCode,
    weights: (f64, f64, f64, f64),
) -> f64 {
    a.distance(b, Some(weights))
}

pub fn compute_distribution(souls: &[SoulListEntry]) -> IsmismStats {
    let mut stats = IsmismStats::default();
    stats.total_souls = souls.len();

    for soul in souls {
        if let Ok(code) = parse_ismism(&soul.ismism_code) {
            *stats.field_distribution.entry(code.field).or_insert(0) += 1;
            *stats.ontology_distribution.entry(code.ontology).or_insert(0) += 1;
            *stats.epistemology_distribution.entry(code.epistemology).or_insert(0) += 1;
            *stats.teleology_distribution.entry(code.teleology).or_insert(0) += 1;
        }
    }

    stats
}
