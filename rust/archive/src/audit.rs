use foundation::{CallFilter, Effectiveness, FailureAlert, FailureAlertType, Result, Storage};

/// 审计引擎 — 检测召唤记录中的失败条件
pub struct AuditEngine;

impl AuditEngine {
    /// 检测单个魂的失败条件：
    /// - 连续3次部分有效 → BoundaryReview
    /// - 累计3次无效 → Suspension
    pub async fn check_soul(store: &dyn Storage, soul_name: &str) -> Result<Vec<FailureAlert>> {
        let records = store
            .query_call_records(&CallFilter {
                soul_name: Some(soul_name.to_string()),
                ..Default::default()
            })
            .await?;

        let mut alerts = Vec::new();

        // 连续3次部分有效检测
        let mut partial_streak = 0u32;
        for r in &records {
            if matches!(r.effectiveness, Effectiveness::Partial) {
                partial_streak += 1;
                if partial_streak >= 3 {
                    alerts.push(FailureAlert {
                        soul_name: soul_name.to_string(),
                        alert_type: FailureAlertType::BoundaryReview,
                    });
                    break;
                }
            } else {
                partial_streak = 0;
            }
        }

        // 累计3次无效检测
        let invalid_count = records
            .iter()
            .filter(|r| matches!(r.effectiveness, Effectiveness::Invalid))
            .count();
        if invalid_count >= 3 {
            alerts.push(FailureAlert {
                soul_name: soul_name.to_string(),
                alert_type: FailureAlertType::Suspension,
            });
        }

        Ok(alerts)
    }

    /// 检测所有魂的失败条件
    pub async fn check_all(store: &dyn Storage) -> Result<Vec<FailureAlert>> {
        let all_records = store
            .query_call_records(&CallFilter::default())
            .await?;

        // 按魂分组
        let mut soul_names: Vec<String> = all_records
            .iter()
            .map(|r| r.soul_name.clone())
            .collect();
        soul_names.sort();
        soul_names.dedup();

        let mut all_alerts = Vec::new();
        for name in soul_names {
            let alerts = Self::check_soul(store, &name).await?;
            all_alerts.extend(alerts);
        }
        Ok(all_alerts)
    }
}
