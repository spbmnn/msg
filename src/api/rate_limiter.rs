use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{interval, Duration};

pub struct RateLimiter {
    sem: Arc<Semaphore>,
    jh: tokio::task::JoinHandle<()>,
}

impl RateLimiter {
    pub fn new(duration: Duration, capacity: usize) -> Self {
        let sem = Arc::new(Semaphore::new(capacity));
        let jh = tokio::spawn({
            let sem = sem.clone();
            let mut interval = interval(duration);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            async move {
                loop {
                    interval.tick().await;

                    if sem.available_permits() < capacity {
                        sem.add_permits(1);
                    }
                }
            }
        });

        Self { jh, sem }
    }

    pub async fn acquire(&self) {
        let permit = self.sem.acquire().await.unwrap();
        permit.forget();
    }
}

impl Drop for RateLimiter {
    fn drop(&mut self) {
        self.jh.abort();
    }
}
