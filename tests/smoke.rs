#![feature(futures_api, pin)]

extern crate futures;
extern crate futures_timer;
#[macro_use] extern crate pin_utils;

use std::time::{Instant, Duration};
use std::pin::Pin;

use futures::{future, Poll};
use futures::prelude::*;
use futures::executor::block_on;
use futures_timer::{Timer, Delay};

fn far_future() -> Instant {
    Instant::now() + Duration::new(5000, 0)
}

#[test]
fn works() {
    let i = Instant::now();
    let dur = Duration::from_millis(100);
    let d = Delay::new(dur);
    block_on(d).unwrap();
    assert!(i.elapsed() > dur);
}

#[test]
fn error_after_inert() {
    let t = Timer::new();
    let handle = t.handle();
    drop(t);
    assert!(block_on(Delay::new_handle(far_future(), handle)).is_err());
}

#[test]
fn drop_makes_inert() {
    let t = Timer::new();
    let handle = t.handle();
    let timeout = Delay::new_handle(far_future(), handle);
    drop(t);
    assert!(block_on(timeout).is_err());
}

#[test]
fn reset() {
    let i = Instant::now();
    let dur = Duration::from_millis(100);
    let mut d = Delay::new(dur);
    pin_mut!(d);
    block_on(future::poll_fn(|lw| d.as_mut().poll(lw))).unwrap();
    assert!(i.elapsed() > dur);

    let i = Instant::now();
    d.reset(dur);
    block_on(future::poll_fn(|lw| d.as_mut().poll(lw))).unwrap();
    assert!(i.elapsed() > dur);
}

#[test]
fn drop_timer_wakes() {
    let t = Timer::new();
    let handle = t.handle();
    let timeout = Delay::new_handle(far_future(), handle);
    pin_mut!(timeout);
    let mut t = Some(t);
    assert!(block_on(future::poll_fn(|lw| {
        match timeout.as_mut().poll(lw) {
            Poll::Pending => {}
            other => return other,
        }
        drop(t.take());
        Poll::Pending
    })).is_err());
}
