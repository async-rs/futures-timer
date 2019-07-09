use std::time::{Duration, Instant};

use futures_timer::Delay;

#[runtime::test]
async fn smoke() {
    let dur = Duration::from_millis(10);
    let start = Instant::now();
    Delay::new(dur).await;
    assert!(start.elapsed() >= (dur / 2));
}

#[runtime::test]
async fn two() {
    let dur = Duration::from_millis(10);
    Delay::new(dur).await;
    Delay::new(dur).await;
}
