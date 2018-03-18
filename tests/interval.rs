extern crate futures;
extern crate futures_timer;

use std::time::{Instant, Duration};

use futures::prelude::*;
use futures::executor::block_on;
use futures_timer::Interval;

#[test]
fn single() {
    let dur = Duration::from_millis(10);
    let start = Instant::now();
    let interval = Interval::new(dur);
    block_on(interval.take(1).collect()).unwrap();
    assert!(start.elapsed() >= dur);
}

#[test]
fn two_times() {
    let dur = Duration::from_millis(10);
    let start = Instant::now();
    let interval = Interval::new(dur);
    let result = block_on(interval.take(2).collect()).unwrap();
    assert!(start.elapsed() >= dur*2);
    assert_eq!(result, vec![(), ()]);
}
