extern crate futures;
extern crate futures_timer;

use std::time::{Instant, Duration};

use futures::future;
use futures::prelude::*;
use futures_timer::{Timer, Delay};

fn far_future() -> Instant {
    Instant::now() + Duration::new(5000, 0)
}

#[test]
fn works() {
    let i = Instant::now();
    let dur = Duration::from_millis(100);
    let d = Delay::new(dur);
    d.wait().unwrap();
    assert!(i.elapsed() > dur);
}

#[test]
fn error_after_inert() {
    let t = Timer::new();
    let handle = t.handle();
    drop(t);
    assert!(Delay::new_handle(far_future(), handle).poll().is_err());
}

#[test]
fn drop_makes_inert() {
    let t = Timer::new();
    let handle = t.handle();
    let timeout = Delay::new_handle(far_future(), handle);
    drop(t);
    assert!(timeout.wait().is_err());
}

#[test]
fn reset() {
    let i = Instant::now();
    let dur = Duration::from_millis(100);
    let mut d = Delay::new(dur);
    future::poll_fn(|| d.poll()).wait().unwrap();
    assert!(i.elapsed() > dur);

    let i = Instant::now();
    d.reset(dur);
    future::poll_fn(|| d.poll()).wait().unwrap();
    assert!(i.elapsed() > dur);
}

#[test]
fn drop_timer_wakes() {
    let t = Timer::new();
    let handle = t.handle();
    let mut timeout = Delay::new_handle(far_future(), handle);
    let mut t = Some(t);
    assert!(future::poll_fn(|| {
        match timeout.poll() {
            Ok(Async::NotReady) => {}
            other => return other,
        }
        drop(t.take());
        Ok(Async::NotReady)
    }).wait().is_err());
}
