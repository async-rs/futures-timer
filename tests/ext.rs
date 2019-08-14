use std::error::Error;
use std::io;
use std::task::Poll;
use std::thread;
use std::time::Duration;

use futures::channel::mpsc::*;
use futures::future::poll_fn;
use futures::TryStreamExt as TryStreamExt03;
use futures_timer::*;

type TestResult = io::Result<()>;

#[runtime::test]
async fn future_timeout() -> TestResult {
    // Never completes
    let long_future = poll_fn::<TestResult, _>(|_| Poll::Pending);

    let res = long_future.timeout(Duration::from_millis(100)).await;
    assert_eq!("future timed out", res.unwrap_err().description());
    Ok(())
}

#[runtime::test]
async fn future_doesnt_timeout() -> TestResult {
    // Never completes
    let short_future = futures::future::ready::<TestResult>(Ok(()));
    short_future.timeout(Duration::from_millis(100)).await?;
    Ok(())
}

#[runtime::test]
async fn stream() -> TestResult {
    let dur = Duration::from_millis(10);
    Delay::new(dur).await?;
    Delay::new(dur).await?;
    Ok(())
}

#[runtime::test]
async fn stream_timeout() -> TestResult {
    let (mut tx, rx) = unbounded::<io::Result<u8>>();

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
            Ok(None) => {
                break;
            }
            Ok(_) => {
                ok += 1;
            }
            Err(_) => {
                err += 1;
            }
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
