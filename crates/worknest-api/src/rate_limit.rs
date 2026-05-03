//! Rate limiting middleware for auth endpoints

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// In-memory sliding window rate limiter
#[derive(Clone)]
pub struct RateLimiter {
    /// Map of IP addresses to request timestamps
    requests: Arc<Mutex<HashMap<IpAddr, Vec<Instant>>>>,
    /// Maximum number of requests allowed in the window
    max_requests: usize,
    /// Time window duration
    window: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `max_requests` - Maximum requests per window per IP
    /// * `window_secs` - Window duration in seconds
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    /// Check if a request from the given IP should be allowed.
    /// Returns true if allowed, false if rate limited.
    pub async fn check(&self, ip: IpAddr) -> bool {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();

        let timestamps = requests.entry(ip).or_default();

        // Remove expired timestamps
        timestamps.retain(|t| now.duration_since(*t) < self.window);

        if timestamps.len() >= self.max_requests {
            false
        } else {
            timestamps.push(now);
            true
        }
    }

    /// Clean up expired entries to prevent memory growth.
    /// Should be called periodically.
    pub async fn cleanup(&self) {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();

        requests.retain(|_, timestamps| {
            timestamps.retain(|t| now.duration_since(*t) < self.window);
            !timestamps.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_rate_limiter_allows_under_limit() {
        let limiter = RateLimiter::new(3, 60);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        assert!(limiter.check(ip).await);
        assert!(limiter.check(ip).await);
        assert!(limiter.check(ip).await);
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(2, 60);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        assert!(limiter.check(ip).await);
        assert!(limiter.check(ip).await);
        assert!(!limiter.check(ip).await); // Should be blocked
    }

    #[tokio::test]
    async fn test_rate_limiter_different_ips() {
        let limiter = RateLimiter::new(1, 60);
        let ip1 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let ip2 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));

        assert!(limiter.check(ip1).await);
        assert!(!limiter.check(ip1).await); // ip1 blocked
        assert!(limiter.check(ip2).await); // ip2 still allowed
    }

    #[tokio::test]
    async fn test_rate_limiter_cleanup() {
        let limiter = RateLimiter::new(10, 60);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        limiter.check(ip).await;
        limiter.cleanup().await;

        // After cleanup, entries still within window should remain
        let requests = limiter.requests.lock().await;
        assert!(requests.contains_key(&ip));
    }
}
