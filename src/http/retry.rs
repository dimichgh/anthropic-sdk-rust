use std::time::{Duration, Instant};
// Note: backoff crate available for more complex scenarios
use crate::types::{AnthropicError, Result};

/// Advanced retry policy with configurable strategies
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub multiplier: f64,
    /// Whether to add jitter to delays
    pub jitter: bool,
    /// Maximum total time to spend retrying
    pub max_elapsed_time: Option<Duration>,
    /// Specific error conditions to retry on
    pub retry_conditions: Vec<RetryCondition>,
}

/// Conditions under which to retry a request
#[derive(Debug, Clone, PartialEq)]
pub enum RetryCondition {
    /// Retry on network timeouts
    Timeout,
    /// Retry on connection failures
    ConnectionError,
    /// Retry on specific HTTP status codes
    HttpStatus(u16),
    /// Retry on rate limiting (429)
    RateLimit,
    /// Retry on server errors (5xx)
    ServerError,
    /// Retry on authentication errors (401)
    AuthenticationError,
    /// Retry on all retriable errors
    All,
}

/// Retry executor that applies policies
#[derive(Debug)]
pub struct RetryExecutor {
    policy: RetryPolicy,
}

/// Result of a retry execution
#[derive(Debug)]
pub enum RetryResult<T> {
    /// Operation succeeded
    Success(T),
    /// Operation failed after all retries
    Failed(AnthropicError),
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
            max_elapsed_time: Some(Duration::from_secs(60)),
            retry_conditions: vec![
                RetryCondition::Timeout,
                RetryCondition::ConnectionError,
                RetryCondition::RateLimit,
                RetryCondition::ServerError,
            ],
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy with exponential backoff
    pub fn exponential() -> Self {
        Self::default()
    }

    /// Set maximum number of retries
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set initial delay
    pub fn initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set maximum delay
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set backoff multiplier
    pub fn multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Enable or disable jitter
    pub fn jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Set maximum elapsed time
    pub fn max_elapsed_time(mut self, max_elapsed: Duration) -> Self {
        self.max_elapsed_time = Some(max_elapsed);
        self
    }

    /// Set retry conditions
    pub fn retry_conditions(mut self, conditions: Vec<RetryCondition>) -> Self {
        self.retry_conditions = conditions;
        self
    }

    /// Check if an error should be retried
    pub fn should_retry(&self, error: &AnthropicError) -> bool {
        for condition in &self.retry_conditions {
            match condition {
                RetryCondition::All => return true,
                RetryCondition::Timeout => {
                    if matches!(error, AnthropicError::Timeout) {
                        return true;
                    }
                }
                RetryCondition::ConnectionError => {
                    if matches!(error, AnthropicError::NetworkError(_)) {
                        return true;
                    }
                }
                RetryCondition::HttpStatus(code) => {
                    if let AnthropicError::HttpError { status, .. } = error {
                        if status == code {
                            return true;
                        }
                    }
                }
                RetryCondition::RateLimit => {
                    if let AnthropicError::HttpError { status, .. } = error {
                        if *status == 429 {
                            return true;
                        }
                    }
                }
                RetryCondition::ServerError => {
                    if let AnthropicError::HttpError { status, .. } = error {
                        if *status >= 500 && *status < 600 {
                            return true;
                        }
                    }
                }
                RetryCondition::AuthenticationError => {
                    if let AnthropicError::HttpError { status, .. } = error {
                        if *status == 401 {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Calculate next delay using exponential backoff
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.initial_delay.as_millis() as f64;
        let delay_ms = base_delay * self.multiplier.powi(attempt as i32);
        let delay = Duration::from_millis(delay_ms as u64);
        
        let delay = std::cmp::min(delay, self.max_delay);
        
        if self.jitter {
            self.add_jitter(delay)
        } else {
            delay
        }
    }

    fn add_jitter(&self, delay: Duration) -> Duration {
        // Simple jitter implementation without external dependencies
        let jitter_range = delay.as_millis() as f64 * 0.1; // 10% jitter
        let jitter = (std::ptr::addr_of!(self) as usize % 100) as f64 / 100.0 * jitter_range;
        let jittered_ms = (delay.as_millis() as f64 + jitter) as u64;
        Duration::from_millis(jittered_ms)
    }
}

impl RetryExecutor {
    /// Create a new retry executor
    pub fn new(policy: RetryPolicy) -> Self {
        Self { policy }
    }

    /// Execute an operation with retry logic
    pub async fn execute<T, F, Fut>(&self, operation: F) -> RetryResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let start_time = Instant::now();
        let mut last_error = None;

        for attempt in 0..=self.policy.max_retries {
            // Check max elapsed time
            if let Some(max_elapsed) = self.policy.max_elapsed_time {
                if start_time.elapsed() >= max_elapsed {
                    break;
                }
            }

            match operation().await {
                Ok(result) => {
                    return RetryResult::Success(result);
                }
                Err(error) => {
                    last_error = Some(error.clone());

                    // Check if we should retry
                    if attempt < self.policy.max_retries && self.policy.should_retry(&error) {
                        let delay = self.policy.calculate_delay(attempt);
                        tracing::debug!(
                            "Request failed (attempt {}/{}): {}. Retrying in {:?}",
                            attempt + 1,
                            self.policy.max_retries + 1,
                            error,
                            delay
                        );
                        tokio::time::sleep(delay).await;
                    } else {
                        break;
                    }
                }
            }
        }

        RetryResult::Failed(last_error.unwrap_or_else(|| {
            AnthropicError::Other("Unknown error in retry executor".to_string())
        }))
    }

    /// Get retry policy
    pub fn get_policy(&self) -> &RetryPolicy {
        &self.policy
    }
}

/// Helper function to create a retry executor with default policy
pub fn default_retry() -> RetryExecutor {
    RetryExecutor::new(RetryPolicy::default())
}

/// Helper function to create a retry executor for API calls
pub fn api_retry() -> RetryExecutor {
    RetryExecutor::new(
        RetryPolicy::exponential()
            .max_retries(3)
            .initial_delay(Duration::from_millis(500))
            .max_delay(Duration::from_secs(30))
            .retry_conditions(vec![
                RetryCondition::RateLimit,
                RetryCondition::ServerError,
                RetryCondition::Timeout,
                RetryCondition::ConnectionError,
            ])
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_should_retry() {
        let policy = RetryPolicy::default();
        
        assert!(policy.should_retry(&AnthropicError::Timeout));
        assert!(policy.should_retry(&AnthropicError::HttpError {
            status: 429,
            message: "Rate limited".to_string(),
        }));
        assert!(policy.should_retry(&AnthropicError::HttpError {
            status: 500,
            message: "Server error".to_string(),
        }));
        assert!(!policy.should_retry(&AnthropicError::InvalidApiKey));
    }

    #[test]
    fn test_delay_calculation() {
        let policy = RetryPolicy::exponential()
            .initial_delay(Duration::from_millis(100))
            .multiplier(2.0)
            .jitter(false);
        
        assert_eq!(policy.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(400));
    }

    #[tokio::test]
    async fn test_retry_executor_success() {
        let policy = RetryPolicy::exponential().max_retries(2);
        let executor = RetryExecutor::new(policy);
        
        let result = executor.execute(|| async {
            Ok::<i32, AnthropicError>(42)
        }).await;
        
        match result {
            RetryResult::Success(value) => assert_eq!(value, 42),
            _ => panic!("Expected success"),
        }
    }

    #[tokio::test]
    async fn test_retry_executor_failure() {
        let policy = RetryPolicy::exponential()
            .max_retries(1)
            .initial_delay(Duration::from_millis(1));
        let executor = RetryExecutor::new(policy);
        
        let result = executor.execute(|| async {
            Err::<i32, AnthropicError>(AnthropicError::InvalidApiKey)
        }).await;
        
        match result {
            RetryResult::Failed(_) => {},
            _ => panic!("Expected failure"),
        }
    }
}
