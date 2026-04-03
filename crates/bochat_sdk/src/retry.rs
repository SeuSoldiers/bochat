use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(2),
        }
    }
}

impl RetryPolicy {
    pub fn next_delay(&self, attempt: usize) -> Duration {
        let exp = 1u64 << attempt.min(8);
        let delay = self.base_delay.saturating_mul(exp as u32);
        delay.min(self.max_delay)
    }

    pub fn should_retry_method(method: &reqwest::Method) -> bool {
        matches!(
            *method,
            reqwest::Method::GET | reqwest::Method::HEAD | reqwest::Method::OPTIONS
        )
    }

    pub fn should_retry_status(status: reqwest::StatusCode) -> bool {
        status.is_server_error() || status.as_u16() == 429
    }
}

#[cfg(test)]
mod tests {
    use super::RetryPolicy;

    #[test]
    fn test_idempotent_method_retryable() {
        assert!(RetryPolicy::should_retry_method(&reqwest::Method::GET));
        assert!(RetryPolicy::should_retry_method(&reqwest::Method::HEAD));
        assert!(!RetryPolicy::should_retry_method(&reqwest::Method::POST));
    }

    #[test]
    fn test_next_delay_increases() {
        let p = RetryPolicy::default();
        let d0 = p.next_delay(0);
        let d1 = p.next_delay(1);
        assert!(d1 >= d0);
    }
}
