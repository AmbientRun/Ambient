use std::time::Duration;
/// Represents an abstract point in time
pub use std::time::Instant;

pub fn schedule_wakeup<F: 'static + Send + FnOnce()>(dur: Duration, callback: F) {
    tokio::spawn(async move {
        tokio::time::sleep(dur).await;
        callback()
    });
}

pub use tokio::time::{sleep, sleep_until};
