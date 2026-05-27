use std::time::Duration;

/// Retry policy for retryable HTTP requests.
///
/// 可重试 HTTP 请求使用的重试策略。
///
/// The SDK currently retries only safe/idempotent methods such as `GET`.
///
/// 目前 SDK 只会对 `GET` 等安全/幂等方法执行自动重试。
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
    /// Compute the next exponential backoff delay.
    ///
    /// 计算下一次指数退避等待时间。
    pub fn next_delay(&self, attempt: usize) -> Duration {
        let exp = 1u64 << attempt.min(8);
        let delay = self.base_delay.saturating_mul(exp as u32);
        delay.min(self.max_delay)
    }

    /// Whether the HTTP method is safe to retry automatically.
    ///
    /// 当前 HTTP 方法是否适合自动重试。
    pub fn should_retry_method(method: &reqwest::Method) -> bool {
        matches!(
            *method,
            reqwest::Method::GET | reqwest::Method::HEAD | reqwest::Method::OPTIONS
        )
    }

    /// Whether the response status should trigger a retry.
    ///
    /// 当前响应状态码是否应该触发重试。
    pub fn should_retry_status(status: reqwest::StatusCode) -> bool {
        status.is_server_error() || status.as_u16() == 429
    }
}

#[cfg(test)]
mod tests {
    use super::RetryPolicy;
    use reqwest::{Method, StatusCode};
    use std::time::Duration;

    #[test]
    fn test_idempotent_method_retryable() {
        assert!(RetryPolicy::should_retry_method(&Method::GET));
        assert!(RetryPolicy::should_retry_method(&Method::HEAD));
        assert!(RetryPolicy::should_retry_method(&Method::OPTIONS));
        assert!(!RetryPolicy::should_retry_method(&Method::POST));
        assert!(!RetryPolicy::should_retry_method(&Method::PUT));
        assert!(!RetryPolicy::should_retry_method(&Method::PATCH));
        assert!(!RetryPolicy::should_retry_method(&Method::DELETE));
    }

    #[test]
    fn test_next_delay_increases() {
        let p = RetryPolicy::default();
        let d0 = p.next_delay(0);
        let d1 = p.next_delay(1);
        assert!(d1 >= d0);
    }

    #[test]
    fn test_next_delay_respects_max_delay_cap() {
        let p = RetryPolicy {
            max_attempts: 5,
            base_delay: Duration::from_millis(300),
            max_delay: Duration::from_millis(700),
        };
        assert_eq!(p.next_delay(0), Duration::from_millis(300));
        assert_eq!(p.next_delay(1), Duration::from_millis(600));
        assert_eq!(p.next_delay(2), Duration::from_millis(700));
        assert_eq!(p.next_delay(20), Duration::from_millis(700));
    }

    #[test]
    fn test_should_retry_status_boundaries() {
        assert!(RetryPolicy::should_retry_status(StatusCode::TOO_MANY_REQUESTS));
        assert!(RetryPolicy::should_retry_status(StatusCode::INTERNAL_SERVER_ERROR));
        assert!(RetryPolicy::should_retry_status(StatusCode::BAD_GATEWAY));
        assert!(!RetryPolicy::should_retry_status(StatusCode::BAD_REQUEST));
        assert!(!RetryPolicy::should_retry_status(StatusCode::UNAUTHORIZED));
        assert!(!RetryPolicy::should_retry_status(StatusCode::NOT_FOUND));
    }
}
