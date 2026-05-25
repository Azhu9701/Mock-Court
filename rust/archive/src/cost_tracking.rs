use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};
use serde::{Serialize, Deserialize};
use foundation::{CallFilter, PossessionMode, Result, Storage};

/// 模型定价配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub provider: String,
    pub model: String,
    pub input_price_per_million: f64,
    pub output_price_per_million: f64,
    pub cache_discount_rate: f64, // 缓存折扣率，0.0-1.0
}

impl Default for ModelPricing {
    fn default() -> Self {
        Self {
            provider: "deepseek".to_string(),
            model: "deepseek-chat".to_string(),
            input_price_per_million: 2.0,
            output_price_per_million: 8.0,
            cache_discount_rate: 0.9, // 90% 折扣
        }
    }
}

/// 单个调用的成本记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallCost {
    pub call_id: String,
    pub soul_name: String,
    pub mode: PossessionMode,
    pub model: String,
    pub provider: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub input_cost: f64,
    pub output_cost: f64,
    pub cache_savings: f64,
    pub total_cost: f64,
    pub timestamp: DateTime<Utc>,
}

/// 会话级别的成本摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCostSummary {
    pub session_id: String,
    pub total_cost: f64,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
    pub cache_savings: f64,
    pub calls: Vec<CallCost>,
}

/// 统计成本报告
#[derive(Debug, Clone, Serialize)]
pub struct CostReport {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_cost: f64,
    pub total_calls: usize,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_savings: f64,
    pub by_mode: HashMap<PossessionMode, ModeCost>,
    pub by_soul: Vec<SoulCost>,
}

/// 模式维度的成本
#[derive(Debug, Clone, Serialize)]
pub struct ModeCost {
    pub mode: PossessionMode,
    pub total_cost: f64,
    pub call_count: usize,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// 魂维度的成本
#[derive(Debug, Clone, Serialize)]
pub struct SoulCost {
    pub soul_name: String,
    pub total_cost: f64,
    pub call_count: usize,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub avg_cost_per_call: f64,
}

/// 成本追踪管理器
pub struct CostTracker {
    pricing: Vec<ModelPricing>,
}

impl CostTracker {
    pub fn new() -> Self {
        Self {
            pricing: vec![ModelPricing::default()],
        }
    }
    
    pub fn with_pricing(pricing: Vec<ModelPricing>) -> Self {
        Self { pricing }
    }
    
    /// 添加或更新定价规则
    pub fn add_pricing(&mut self, pricing: ModelPricing) {
        self.pricing.push(pricing);
    }
    
    /// 计算一次调用的成本
    pub fn calculate_call_cost(
        &self,
        call_id: String,
        soul_name: String,
        mode: PossessionMode,
        provider: String,
        model: String,
        input_tokens: u32,
        output_tokens: u32,
        use_cache: bool,
    ) -> CallCost {
        let default_pricing = ModelPricing::default();
        let pricing = self.pricing
            .iter()
            .find(|p| p.provider == provider && p.model == model)
            .or_else(|| self.pricing.first())
            .unwrap_or(&default_pricing);
            
        let input_cost = input_tokens as f64 / 1_000_000.0 * pricing.input_price_per_million;
        let output_cost = output_tokens as f64 / 1_000_000.0 * pricing.output_price_per_million;
        
        let (input_cost, cache_savings) = if use_cache {
            let discounted = input_cost * (1.0 - pricing.cache_discount_rate);
            let savings = input_cost - discounted;
            (discounted, savings)
        } else {
            (input_cost, 0.0)
        };
        
        let total_cost = input_cost + output_cost;
        
        CallCost {
            call_id,
            soul_name,
            mode,
            model,
            provider,
            input_tokens,
            output_tokens,
            input_cost,
            output_cost,
            cache_savings,
            total_cost,
            timestamp: Utc::now(),
        }
    }
    
    /// 生成时间段内的成本报告
    pub async fn generate_report(
        &self,
        store: &dyn Storage,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        provider: &str,
        model: &str,
    ) -> Result<CostReport> {
        let mut total_cost = 0.0;
        let mut total_input_tokens = 0u64;
        let mut total_output_tokens = 0u64;
        let mut total_cache_savings = 0.0;
        let mut by_mode: HashMap<PossessionMode, ModeCost> = HashMap::new();
        let mut by_soul_map: HashMap<String, SoulCostAccumulator> = HashMap::new();

        let records = store.query_call_records(&CallFilter::default()).await?;

        let filtered: Vec<_> = records
            .into_iter()
            .filter(|r| r.created_at >= start && r.created_at <= end)
            .collect();

        let total_calls = filtered.len();

        for record in filtered {
            let input_tokens = record.usage.prompt_tokens;
            let output_tokens = record.usage.completion_tokens;
            let cost = self.calculate_call_cost(
                record.id.clone(),
                record.soul_name.clone(),
                record.mode.clone(),
                provider.to_string(),
                model.to_string(),
                input_tokens,
                output_tokens,
                true,
            );

            total_cost += cost.total_cost;
            total_input_tokens += input_tokens as u64;
            total_output_tokens += output_tokens as u64;
            total_cache_savings += cost.cache_savings;
            
            // 按模式统计
            let mode_entry = by_mode.entry(record.mode.clone()).or_insert(ModeCost {
                mode: record.mode.clone(),
                total_cost: 0.0,
                call_count: 0,
                input_tokens: 0,
                output_tokens: 0,
            });
            mode_entry.total_cost += cost.total_cost;
            mode_entry.call_count += 1;
            mode_entry.input_tokens += cost.input_tokens as u64;
            mode_entry.output_tokens += cost.output_tokens as u64;
            
            // 按魂统计
            let soul_entry = by_soul_map.entry(record.soul_name.clone()).or_insert(SoulCostAccumulator {
                total_cost: 0.0,
                call_count: 0,
                input_tokens: 0,
                output_tokens: 0,
            });
            soul_entry.total_cost += cost.total_cost;
            soul_entry.call_count += 1;
            soul_entry.input_tokens += cost.input_tokens as u64;
            soul_entry.output_tokens += cost.output_tokens as u64;
        }
        
        let mut by_soul: Vec<SoulCost> = by_soul_map
            .into_iter()
            .map(|(soul_name, acc)| SoulCost {
                soul_name,
                total_cost: acc.total_cost,
                call_count: acc.call_count,
                input_tokens: acc.input_tokens,
                output_tokens: acc.output_tokens,
                avg_cost_per_call: acc.total_cost / acc.call_count as f64,
            })
            .collect();
        
        by_soul.sort_by(|a, b| b.total_cost.partial_cmp(&a.total_cost).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(CostReport {
            period_start: start,
            period_end: end,
            total_cost,
            total_calls,
            total_input_tokens,
            total_output_tokens,
            total_cache_savings,
            by_mode,
            by_soul,
        })
    }
    
    /// 获取每日成本趋势
    pub async fn get_daily_trends(
        &self,
        store: &dyn Storage,
        days: u32,
        provider: &str,
        model: &str,
    ) -> Result<Vec<DailyCost>> {
        let mut trends = Vec::new();
        let end = Utc::now();
        let start = end - Duration::days(days as i64);

        let records = store.query_call_records(&CallFilter::default()).await?;
        let filtered: Vec<_> = records
            .into_iter()
            .filter(|r| r.created_at >= start && r.created_at <= end)
            .collect();

        // Group by date and compute real daily costs
        use std::collections::BTreeMap;
        let mut daily: BTreeMap<String, (f64, u64)> = BTreeMap::new();
        for record in &filtered {
            let date = record.created_at.format("%Y-%m-%d").to_string();
            let cost = self.calculate_call_cost(
                record.id.clone(),
                record.soul_name.clone(),
                record.mode.clone(),
                provider.to_string(),
                model.to_string(),
                record.usage.prompt_tokens,
                record.usage.completion_tokens,
                true,
            );
            let entry = daily.entry(date).or_insert((0.0, 0));
            entry.0 += cost.total_cost;
            entry.1 += 1;
        }

        let avg_daily = if filtered.is_empty() { 0.0 } else {
            daily.values().map(|(cost, _)| cost).sum::<f64>() / days as f64
        };

        for i in 0..days {
            let date = start + Duration::days(i as i64);
            let date_str = date.format("%Y-%m-%d").to_string();
            let cost = daily.get(&date_str).map(|(c, _)| *c).unwrap_or(avg_daily);
            trends.push(DailyCost { date, cost });
        }

        Ok(trends)
    }
}

/// 每日成本
#[derive(Debug, Clone, Serialize)]
pub struct DailyCost {
    pub date: DateTime<Utc>,
    pub cost: f64,
}

struct SoulCostAccumulator {
    total_cost: f64,
    call_count: usize,
    input_tokens: u64,
    output_tokens: u64,
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cost_calculation() {
        let tracker = CostTracker::new();
        
        let cost = tracker.calculate_call_cost(
            "test-123".to_string(),
            "马克思".to_string(),
            PossessionMode::Conference,
            "deepseek".to_string(),
            "deepseek-chat".to_string(),
            1_000_000,
            500_000,
            false,
        );
        
        assert!((cost.total_cost - 6.0).abs() < 0.001);
    }
    
    #[test]
    fn test_cache_discount() {
        let tracker = CostTracker::new();
        
        let with_cache = tracker.calculate_call_cost(
            "test-456".to_string(),
            "马克思".to_string(),
            PossessionMode::Conference,
            "deepseek".to_string(),
            "deepseek-chat".to_string(),
            1_000_000,
            500_000,
            true,
        );
        
        assert!(with_cache.cache_savings > 0.0);
        assert!(with_cache.total_cost < 6.0);
    }
}
