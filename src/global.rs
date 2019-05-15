use std::io;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::Context;
use std::task::{RawWaker, Waker, RawWakerVTable};
use std::thread;
use std::time::Instant;
use std::future::Future;
use std::mem::{self, ManuallyDrop};

use crate::{Timer, TimerHandle};

pub struct HelperThread {
    thread: Option<thread::JoinHandle<()>>,
    timer: TimerHandle,
    done: Arc<AtomicBool>,
}

impl HelperThread {
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

    pub fn handle(&self) -> TimerHandle {
        self.timer.clone()
    }

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
    let me = Arc::into_raw(Arc::new(ThreadUnpark {
        thread: thread::current(),
    }));

    unsafe fn raw_clone(ptr: *const ()) -> RawWaker {
        let me = ManuallyDrop::new(Arc::from_raw(ptr as *const ThreadUnpark));
        mem::forget(me.clone());
        RawWaker::new(ptr, &VTABLE)
    }
    unsafe fn raw_wake(ptr: *const ()) {
        let me = Arc::from_raw(ptr as *const ThreadUnpark);
        me.thread.unpark()
    }
    unsafe fn raw_wake_by_ref(ptr: *const ()) {
        let me = ManuallyDrop::new(Arc::from_raw(ptr as *const ThreadUnpark));
        me.thread.unpark()
    }
    unsafe fn raw_drop(ptr: *const ()) {
        Arc::from_raw(ptr as *const ThreadUnpark);
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(raw_clone, raw_wake, raw_wake_by_ref, raw_drop);
    let waker = unsafe { Waker::from_raw(RawWaker::new(me as *const (), &VTABLE)) };
    let mut cx = Context::from_waker(&waker);

    while !done.load(Ordering::SeqCst) {
        drop(Pin::new(&mut timer).poll(&mut cx));
        Pin::new(&mut timer).get_mut().advance();
        match Pin::new(&mut timer).get_mut().next_event() {
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

// impl Notify for ThreadUnpark {
//     fn notify(&self, _unpark_id: usize) {
//         self.thread.unpark()
//     }
// }
