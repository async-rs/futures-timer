use std::time::Duration;
use std::thread;

use futures::channel::mpsc::*;
use futures_timer::*;
use futures::StreamExt as StreamExt03;

#[runtime::test]
async fn future_timeout() {
    let long_future = futures::future::pending::<()>();
    assert!(long_future.timeout(Duration::from_millis(100)).await.is_err());
}

#[runtime::test]
async fn future_doesnt_timeout() {
    let short_future = futures::future::ready(());
    assert!(!short_future.timeout(Duration::from_millis(100)).await.is_err());
}

#[runtime::test]
async fn stream_timeout() {
    let (mut tx, rx) = unbounded::<u8>();

    thread::spawn(move || {
        for i in 0..10_u8 {
            tx.start_send(i).unwrap();
            thread::sleep(Duration::from_millis(100));
        }

        drop(tx)
    });

    let mut f = rx.timeout(Duration::from_millis(10));
    let mut ok = 0;
    let mut err = 0;
    loop {
        let next = f.next().await;
        match next {
            None => { break; }
            Some(Ok(_)) => { ok += 1; }
            Some(Err(_)) => { err += 1; }
        }
    }

    // Exactly 10 successes
    assert_eq!(ok, 10);
    // We should have way more errors than success (non-deterministic)
    assert!(err > ok * 5);
}

#[runtime::test]
async fn stream_doesnt_timeout() {
    let (mut tx, rx) = unbounded::<u8>();

    // Produce a list of numbers that arrive safely within the timeout period
    thread::spawn(move || {
        for i in 0..10_u8 {
            tx.start_send(i).unwrap();
            thread::sleep(Duration::from_millis(100));
        }

        drop(tx)
    });

    let mut f = rx.timeout(Duration::from_millis(200));
    let mut count = 0;
    loop {
        let next = f.next().await;
        if next.is_none() {
            break;
        }
        // All of these items should be non-error
        next.unwrap().unwrap();
        count += 1;
    }

    assert_eq!(count, 10);
}
