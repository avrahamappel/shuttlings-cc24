use std::time::Duration;

use leaky_bucket::RateLimiter;

fn build_rate_limiter() -> RateLimiter {
    RateLimiter::builder()
        .max(5)
        .initial(5)
        .interval(Duration::from_secs(1))
        .build()
}

pub struct Bucket {
    rate_limiter: RateLimiter,
}

impl Bucket {
    pub fn new() -> Self {
        let rate_limiter = build_rate_limiter();
        Self { rate_limiter }
    }

    pub fn get_milk(&self) -> bool {
        self.rate_limiter.try_acquire(1)
    }

    pub fn refill(&mut self) {
        self.rate_limiter = build_rate_limiter();
    }
}
