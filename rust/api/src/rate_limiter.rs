use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    max_tokens: f64,
    refill_rate: f64,
}

impl TokenBucket {
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        TokenBucket {
            tokens: max_tokens,
            last_refill: Instant::now(),
            max_tokens,
            refill_rate,
        }
    }

    fn try_consume(&mut self, now: Instant) -> bool {
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn retry_after_secs(&self) -> f64 {
        if self.tokens >= 1.0 {
            return 0.0;
        }
        let needed = 1.0 - self.tokens;
        (needed / self.refill_rate).ceil().max(1.0)
    }
}

pub struct RateLimiter {
    buckets: Arc<DashMap<String, TokenBucket>>,
    default_max_tokens: f64,
    default_refill_rate: f64,
}

impl RateLimiter {
    pub fn new(requests_per_second: f64, burst_size: f64) -> Self {
        RateLimiter {
            buckets: Arc::new(DashMap::new()),
            default_max_tokens: burst_size,
            default_refill_rate: requests_per_second,
        }
    }

    pub fn check(&self, key: &str) -> (bool, u64) {
        let now = Instant::now();
        let mut bucket = self.buckets.entry(key.to_string()).or_insert_with(|| {
            TokenBucket::new(self.default_max_tokens, self.default_refill_rate)
        });
        let allowed = bucket.try_consume(now);
        let retry_after = if allowed {
            0
        } else {
            bucket.retry_after_secs() as u64
        };
        (allowed, retry_after)
    }

    pub fn cleanup(&self) {
        let now = Instant::now();
        self.buckets.retain(|_, bucket| {
            now.duration_since(bucket.last_refill) < Duration::from_secs(300)
        });
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        RateLimiter {
            buckets: self.buckets.clone(),
            default_max_tokens: self.default_max_tokens,
            default_refill_rate: self.default_refill_rate,
        }
    }
}
