use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Instant;

use futures::prelude::*;
use futures::executor::{SpawnError, Executor};

use {TimerHandle, Timer};

/// A thread which drives execution of a `Timer`.
///
/// Note that if you're using this crate you probably don't need to use this
/// type, as there is a global `HelperThread` started in the background for you
/// by default. This type is available in case you need more close control over
/// when/how threads are spawned.
pub struct HelperThread {
    thread: Option<thread::JoinHandle<()>>,
    timer: TimerHandle,
    done: Arc<AtomicBool>,
}

impl HelperThread {
    /// Creates a new HelperThread, and associated Timer whose execution it drives.
    pub fn new() -> io::Result<HelperThread> {
        let timer = Timer::new();
        let timer_handle = timer.handle();
        let done = Arc::new(AtomicBool::new(false));
        let done2 = done.clone();
        let thread = thread::Builder::new().spawn(move || run(timer, done2))?;

        Ok(HelperThread {
            thread: Some(thread),
            done,
            timer: timer_handle,
        })
    }

    /// Returns a handle to the Timer drive by this thread.
    pub fn handle(&self) -> TimerHandle {
        self.timer.clone()
    }

    /// Ensures that the thread persists even if the `HelperThread` goes out of scope.
    pub fn forget(mut self) {
        self.thread.take();
    }
}

impl Drop for HelperThread {
    fn drop(&mut self) {
        let thread = match self.thread.take() {
            Some(thread) => thread,
            None => return,
        };
        self.done.store(true, Ordering::SeqCst);
        thread.thread().unpark();
        drop(thread.join());
    }
}

fn run(mut timer: Timer, done: Arc<AtomicBool>) {
	let me = Arc::new(ThreadUnpark {
		thread: thread::current(),
	});
    let mut local_map = task::LocalMap::new();
    let waker = task::Waker::from(me);
    let mut exec = NonFunctionalExecutor;
    let mut cx = task::Context::new(&mut local_map, &waker, &mut exec);
    while !done.load(Ordering::SeqCst) {
        drop(timer.poll(&mut cx));
        timer.advance();
        match timer.next_event() {
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

impl task::Wake for ThreadUnpark {
    fn wake(arc_self: &Arc<Self>) {
        arc_self.thread.unpark()
    }
}

struct NonFunctionalExecutor;

impl Executor for NonFunctionalExecutor {
    fn spawn(&mut self, _: Box<Future<Item = (), Error = Never> + 'static + Send>) -> Result<(), SpawnError> {
        Err(SpawnError::shutdown())
    }
}
