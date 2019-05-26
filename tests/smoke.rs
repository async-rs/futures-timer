#![feature(async_await)]
use std::time::{Duration, Instant};

use futures_timer::{Delay, Timer};

use std::error::Error;
use std::pin::Pin;

fn far_future() -> Instant {
    Instant::now() + Duration::new(5000, 0)
}

#[runtime::test]
async fn works() {
    let i = Instant::now();
    let dur = Duration::from_millis(100);
    let _d = Delay::new(dur).await;
    assert!(i.elapsed() > dur);
}

#[runtime::test]
async fn error_after_inert() {
    let t = Timer::new();
    let handle = t.handle();
    drop(t);
    let res = Delay::new_handle(far_future(), handle).await;
    assert!(res.is_err());
}

#[runtime::test]
async fn drop_makes_inert() {
    let t = Timer::new();
    let handle = t.handle();
    let timeout = Delay::new_handle(far_future(), handle);
    drop(t);
    let res = timeout.await;
    assert!(res.is_err());
}

#[runtime::test]
async fn reset() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let i = Instant::now();
    let dur = Duration::from_millis(100);
    let mut d = Delay::new(dur);

    // Allow us to re-use a future
    Pin::new(&mut d).await?;

    assert!(i.elapsed() > dur);

    let i = Instant::now();
    d.reset(dur);
    d.await?;
    assert!(i.elapsed() > dur);
    Ok(())
}

// #[test]
// fn drop_timer_wakes() {
//     let t = Timer::new();
//     let handle = t.handle();
//     let mut timeout = Delay::new_handle(far_future(), handle);
//     let mut t = Some(t);
//     assert!(future::poll_fn(|| {
//         match timeout.poll() {
//             Ok(Poll::Pending) => {}
//             other => return other,
//         }
//         drop(t.take());
//         Ok(Poll::Pending)
//     })
//     .wait()
//     .is_err());
// }
