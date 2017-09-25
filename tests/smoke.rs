extern crate futures;
extern crate futures_timer;

use std::time::{Instant, Duration};

use futures::prelude::*;
use futures_timer::{Timer, Timeout};

fn far_future() -> Instant {
    Instant::now() + Duration::new(5000, 0)
}

#[test]
fn works() {
    let i = Instant::now();
    let dur = Duration::new(0, 100);
    let d = Timeout::new(dur);
    d.wait().unwrap();
    assert!(i.elapsed() > dur);
}

#[test]
fn error_after_inert() {
    let t = Timer::new();
    let handle = t.handle();
    drop(t);
    assert!(Timeout::new_handle(far_future(), handle).poll().is_err());
}

#[test]
fn drop_makes_inert() {
    let t = Timer::new();
    let handle = t.handle();
    let timeout = Timeout::new_handle(far_future(), handle);
    drop(t);
    assert!(timeout.wait().is_err());
}
