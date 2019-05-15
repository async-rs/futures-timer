#![feature(async_await)]
use std::time::{Duration, Instant};

use futures::future;
use futures::prelude::*;
use futures_timer::{Delay, Timer};
use std::task::Poll;

fn far_future() -> Instant {
    Instant::now() + Duration::new(5000, 0)
}

#[runtime::test]
async fn works() {
    let i = Instant::now();
    let dur = Duration::from_millis(100);
    let d = Delay::new(dur).await;
    assert!(dbg!(i.elapsed() > dur));
}

// #[runtime::test]
// async fn error_after_inert() {
//     let t = Timer::new();
//     let handle = t.handle();
//     drop(t);
//     assert!(Delay::new_handle(far_future(), handle).poll().is_err());
// }

// #[runtime::test]
// async fn drop_makes_inert() {
//     let t = Timer::new();
//     let handle = t.handle();
//     let timeout = Delay::new_handle(far_future(), handle);
//     drop(t);
//     let res = timeout.await;
//     assert!(res.is_err());
// }

// #[test]
// fn reset() {
//     let i = Instant::now();
//     let dur = Duration::from_millis(100);
//     let mut d = Delay::new(dur);
//     future::poll_fn(|| d.poll()).wait().unwrap();
//     assert!(i.elapsed() > dur);

//     let i = Instant::now();
//     d.reset(dur);
//     future::poll_fn(|| d.poll()).wait().unwrap();
//     assert!(i.elapsed() > dur);
// }

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
