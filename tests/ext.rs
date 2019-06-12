#![feature(async_await)]

use std::io;
use std::time::Duration;
use std::task::Poll;
use std::thread;

use futures::future::poll_fn;
use futures::channel::mpsc::*;
use futures_timer::{*, ext::TimeoutError};
use futures::TryStreamExt as TryStreamExt03;

type TestResult = ::std::result::Result<(), TimeoutError<()>>;

#[runtime::test]
async fn future_timeout() -> TestResult {
    // Never completes
    let long_future = poll_fn::<Result<(), ()>, _>(|_| {
        Poll::Pending
    });

    let res = long_future.timeout(Duration::from_millis(100)).await;
    assert!(res.unwrap_err().is_elapsed());
    Ok(())
}

#[runtime::test]
async fn future_doesnt_timeout() -> TestResult {
    // Never completes
    let short_future = futures::future::ready(Ok(()));
    short_future.timeout(Duration::from_millis(100)).await?;
    Ok(())
}

#[runtime::test]
async fn future_error() -> TestResult {
    let error_future = futures::future::ready(Err::<(), _>(()));
    let res = error_future.timeout(Duration::from_millis(100)).await;
    assert!(res.unwrap_err().is_inner());
    Ok(())
}

#[runtime::test]
async fn stream_timeout() -> TestResult {
    let (mut tx, rx) = unbounded::<Result<u8, ()>>();

    thread::spawn(move || {
        for i in 0..10_u8 {
            tx.start_send(Ok(i)).unwrap();
            thread::sleep(Duration::from_millis(100));
        }

        drop(tx)
    });

    let mut f = rx.timeout(Duration::from_millis(10));
    let mut ok = 0;
    let mut err = 0;
    loop {
        let next = f.try_next().await;
        match next {
            Ok(None) => { break; }
            Ok(_) => { ok += 1; }
            Err(_) => { err += 1; }
        }
    }

    // Exactly 10 successes
    assert_eq!(ok, 10);
    // We should have way more errors than success (non-deterministic)
    assert!(err > ok * 5);

    Ok(())
}

#[runtime::test]
async fn stream_doesnt_timeout() -> TestResult {
    let (mut tx, rx) = unbounded::<io::Result<u8>>();

    // Produce a list of numbers that arrive safely within the timeout period
    thread::spawn(move || {
        for i in 0..10_u8 {
            tx.start_send(Ok(i)).unwrap();
            thread::sleep(Duration::from_millis(100));
        }

        drop(tx)
    });

    let mut f = rx.timeout(Duration::from_millis(200));
    let mut count = 0;
    loop {
        let next = f.try_next().await;
        if let Ok(None) = next {
            break;
        }
        // All of these items should be non-error
        next.unwrap();
        count += 1;
    }

    assert_eq!(count, 10);

    Ok(())
}

#[runtime::test]
async fn stream_error() -> TestResult {
    let mut error_stream =
        futures::stream::repeat(Err::<(), _>(())).timeout(Duration::from_millis(100));

    let res = error_stream.try_next().await;
    assert!(res.unwrap_err().is_inner());
    Ok(())
}
