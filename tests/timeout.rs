extern crate futures;
extern crate futures_timer;

use std::time::{Instant, Duration};

use futures::executor::block_on;
use futures_timer::Delay;

#[test]
fn smoke() {
    let dur = Duration::from_millis(10);
    let start = Instant::now();
    let timeout = Delay::new(dur);
    block_on(timeout).unwrap();
    assert!(start.elapsed() >= (dur / 2));
}

#[test]
fn two() {
    let dur = Duration::from_millis(10);
    let timeout = Delay::new(dur);
    block_on(timeout).unwrap();
    let timeout = Delay::new(dur);
    block_on(timeout).unwrap();
}
