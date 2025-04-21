use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::sleep;

/// Centralized API rate limiter to enforce e621's 2 requests/sec cap.
#[derive(Clone)]
pub struct ApiLimiter {
    semaphore: Arc<Semaphore>,
    delay: Duration,
}

impl ApiLimiter {
    pub fn new(max_concurrent: usize, delay: Duration) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            delay,
        }
    }

    /// Run a future under limit control.
    pub async fn run<T, F>(&self, task: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let _permit = self.semaphore.acquire().await.expect("semaphore poisoned!");
        let result = task.await;
        sleep(self.delay).await;
        result
    }
}

use once_cell::sync::Lazy;

pub static API_LIMITER: Lazy<ApiLimiter> = Lazy::new(|| {
    ApiLimiter::new(1, Duration::from_millis(750)) // Technically the limit is 2/sec, might as well give some wiggle room
});
