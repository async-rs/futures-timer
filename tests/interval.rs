#![feature(async_await)]

use std::time::{Duration, Instant};
use std::error::Error;

use futures::prelude::*;
use futures_timer::Interval;

#[runtime::test]
async fn single() -> Result<(), Box<dyn Error + Send + Sync + 'static>>{
    let dur = Duration::from_millis(10);
    let start = Instant::now();
    let interval = Interval::new(dur);
    interval.take(1).collect::<Vec<()>>().await;
    assert!(start.elapsed() >= dur);
    Ok(())
}

#[runtime::test]
async fn two_times() -> Result<(), Box<dyn Error + Send + Sync + 'static>>{
    let dur = Duration::from_millis(10);
    let start = Instant::now();
    let interval = Interval::new(dur);
    let result = interval.take(2).collect::<Vec<()>>().await;
    assert!(start.elapsed() >= dur * 2);
    assert_eq!(result, vec![(), ()]);
    Ok(())
}
