extern crate futures;
extern crate futures_timer;

use std::time::{Instant, Duration};

use futures::prelude::*;
use futures_timer::Timeout;

#[test]
fn smoke() {
    let dur = Duration::from_millis(10);
    let start = Instant::now();
    let timeout = Timeout::new(dur);
    timeout.wait().unwrap();
    assert!(start.elapsed() >= (dur / 2));
}

#[test]
fn two() {
    let dur = Duration::from_millis(10);
    let timeout = Timeout::new(dur);
    timeout.wait().unwrap();
    let timeout = Timeout::new(dur);
    timeout.wait().unwrap();
}
