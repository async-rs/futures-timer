use std::io;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use futures::Async;
use futures::sync::oneshot;
use futures::executor::{spawn, Notify};

use {TimerHandle, Timer};

struct HelperThread {
    thread: Option<thread::JoinHandle<()>>,
    tx: Option<oneshot::Sender<()>>,
    timer: TimerHandle,
}

statik!(static DEFAULT: Option<HelperThread> = HelperThread::new().ok());

pub fn timer() -> Option<TimerHandle> {
    DEFAULT.with(|h| h.as_ref().map(|c| c.timer.clone())).and_then(|x| x)
}

#[allow(dead_code)]
pub fn shutdown() {
    DEFAULT.drop();
}

impl HelperThread {
    fn new() -> io::Result<HelperThread> {
        let (tx, rx) = oneshot::channel();
        let timer = Timer::new();
        let timer_handle = timer.handle();
        let thread = thread::spawn(move || run(timer, rx));

        Ok(HelperThread {
            thread: Some(thread),
            tx: Some(tx),
            timer: timer_handle,
        })
    }
}

impl Drop for HelperThread {
    fn drop(&mut self) {
        drop(self.tx.take());
        drop(self.thread.take().unwrap().join());
    }
}

fn run(timer: Timer, shutdown: oneshot::Receiver<()>) {
    let mut shutdown = spawn(shutdown);
    let mut timer = spawn(timer);
    let me = Arc::new(ThreadUnpark {
        thread: thread::current(),
    });
    loop {
        match shutdown.poll_future_notify(&me, 0) {
            Ok(Async::Ready(_)) | Err(_) => break,
            Ok(Async::NotReady) => {}
        }
        drop(timer.poll_future_notify(&me, 0));
        timer.get_mut().advance();

        match timer.get_ref().next_event() {
            // Ok, block for the specified time
            Some(when) => {
                let now = Instant::now();
                if now < when {
                    thread::park_timeout(when - now)
                } else {
                    // .. continue...
                }
            }

            // Just wait for one of our futures to wake up
            None => thread::park(),
        }
    }
}

struct ThreadUnpark {
    thread: thread::Thread,
}

impl Notify for ThreadUnpark {
    fn notify(&self, _unpark_id: usize) {
        self.thread.unpark()
    }
}
