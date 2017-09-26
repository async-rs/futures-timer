//! Support for creating futures that represent timeouts.
//!
//! This module contains the `Timeout` type which is a future that will resolve
//! at a particular point in the future.

use std::io;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::time::{Duration, Instant};

use futures::{Future, Poll, Async};
use futures::task::AtomicTask;

use arc_list::Node;
use global;
use {TimerHandle, ScheduledTimer};

/// A future representing the notification that a timeout has occurred.
///
/// Timeouts are created through the `Timeout::new` or
/// `Timeout::new_at` methods indicating when a timeout should fire at.
/// Note that timeouts are not intended for high resolution timers, but rather
/// they will likely fire some granularity after the exact instant that they're
/// otherwise indicated to fire at.
pub struct Timeout {
    state: Option<Arc<Node<ScheduledTimer>>>,
    when: Instant,
}

impl Timeout {
    /// Creates a new timeout which will fire at `dur` time into the future.
    ///
    /// This function will return a future that will resolve to the actual
    /// timeout object. The timeout object itself is then a future which will be
    /// set to fire at the specified point in the future.
    pub fn new(dur: Duration) -> Timeout {
        let when = Instant::now() + dur;
        match global::timer() {
            Some(h) => Timeout::new_handle(when, h),
            None => Timeout { state: None, when: when },
        }
    }

    /// Creates a new timeout which will fire at the time specified by `at`.
    ///
    /// This function will return a future that will resolve to the actual
    /// timeout object. The timeout object itself is then a future which will be
    /// set to fire at the specified point in the future.
    pub fn new_at(at: Instant) -> Timeout {
        match global::timer() {
            Some(h) => Timeout::new_handle(at, h),
            None => Timeout { state: None, when: at },
        }
    }

    /// Creates a new timeout which will fire at the time specified by `at`.
    ///
    /// This function will return a future that will resolve to the actual
    /// timeout object. The timeout object itself is then a future which will be
    /// set to fire at the specified point in the future.
    pub fn new_handle(at: Instant, handle: TimerHandle) -> Timeout {
        let inner = match handle.inner.upgrade() {
            Some(i) => i,
            None => return Timeout { state: None, when: at },
        };
        let state = Arc::new(Node::new(ScheduledTimer {
            at: Mutex::new(Some(at)),
            state: AtomicUsize::new(0),
            task: AtomicTask::new(),
            inner: handle.inner,
            slot: Mutex::new(None),
        }));
        inner.list.push(&state);
        inner.task.notify();
        Timeout {
            state: Some(state),
            when: at,
        }
    }

    /// Resets this timeout to an new timeout which will fire at the time
    /// specified by `dur`.
    ///
    /// This is equivalent to calling `reset_at` with `Instant::now() + dur`
    pub fn reset(&mut self, dur: Duration) {
        self.reset_at(Instant::now() + dur)
    }

    /// Resets this timeout to an new timeout which will fire at the time
    /// specified by `at`.
    ///
    /// This method is usable even of this instance of `Timeout` has "already
    /// fired". That is, if this future has resovled, calling this method means
    /// that the future will still re-resolve at the specified instant.
    ///
    /// If `at` is in the past then this future will immediately be resolved
    /// (when `poll` is called).
    ///
    /// Note that if any task is currently blocked on this future then that task
    /// will be dropped. It is required to call `poll` again after this method
    /// has been called to ensure tha ta task is blocked on this future.
    pub fn reset_at(&mut self, at: Instant) {
        self.when = at;
        let state = match self.state {
            Some(ref state) => state,
            None => return,
        };
        if let Some(timeouts) = state.inner.upgrade() {
            let mut bits = state.state.load(SeqCst);
            loop {
                let new = bits.wrapping_add(2) & !1;
                match state.state.compare_exchange(bits, new, SeqCst, SeqCst) {
                    Ok(_) => break,
                    Err(s) => bits = s,
                }
            }
            *state.at.lock().unwrap() = Some(at);
            timeouts.list.push(state);
            timeouts.task.notify();
        }
    }

    pub fn fires_at(&self) -> Instant {
        self.when
    }
}

impl Future for Timeout {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        let state = match self.state {
            Some(ref state) => state,
            None => return Err(io::Error::new(io::ErrorKind::Other,
                                              "failed to create timer")),
        };
        if state.state.load(SeqCst) & 1 != 0 {
            return Ok(Async::Ready(()))
        }

        state.task.register();

        if state.inner.upgrade().is_none() {
            return Err(io::Error::new(io::ErrorKind::Other,
                                      "timer has gone away"))
        }

        // Need to check after we register as well
        if state.state.load(SeqCst) & 1 != 0 {
            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }
}

impl Drop for Timeout {
    fn drop(&mut self) {
        let state = match self.state {
            Some(ref s) => s,
            None => return,
        };
        if let Some(timeouts) = state.inner.upgrade() {
            *state.at.lock().unwrap() = None;
            timeouts.list.push(state);
            timeouts.task.notify();
        }
    }
}
