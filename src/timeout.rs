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

        // If we fail to actually push our node then we've become an inert
        // timer, meaning that we'll want to immediately return an error from
        // `poll`.
        if inner.list.push(&state).is_err() {
            return Timeout { state: None, when: at }
        }

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
        if self._reset(at).is_err() {
            self.state = None
        }
    }

    fn _reset(&mut self, at: Instant) -> Result<(), ()> {
        let state = match self.state {
            Some(ref state) => state,
            None => return Err(()),
        };
        if let Some(timeouts) = state.inner.upgrade() {
            let mut bits = state.state.load(SeqCst);
            loop {
                // If we've been invalidated, cancel this reset
                if bits & 0b10 != 0 {
                    return Err(())
                }
                let new = bits.wrapping_add(0b100) & !0b11;
                match state.state.compare_exchange(bits, new, SeqCst, SeqCst) {
                    Ok(_) => break,
                    Err(s) => bits = s,
                }
            }
            *state.at.lock().unwrap() = Some(at);
            // If we fail to push our node then we've become an inert timer, so
            // we'll want to clear our `state` field accordingly
            timeouts.list.push(state)?;
            timeouts.task.notify();
        }

        Ok(())
    }
}

pub fn fires_at(timeout: &Timeout) -> Instant {
    timeout.when
}

impl Future for Timeout {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        let state = match self.state {
            Some(ref state) => state,
            None => return Err(io::Error::new(io::ErrorKind::Other,
                                              "timer has gone away")),
        };
        if state.state.load(SeqCst) & 1 != 0 {
            return Ok(Async::Ready(()))
        }

        state.task.register();

        // Now that we've registered, do the full check of our own internal
        // state. If we've fired the first bit is set, and if we've been
        // invalidated the second bit is set.
        match state.state.load(SeqCst) {
            n if n & 0b01 != 0 => Ok(Async::Ready(())),
            n if n & 0b10 != 0 => Err(io::Error::new(io::ErrorKind::Other,
                                                     "timer has gone away")),
            _ => Ok(Async::NotReady),
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
            if timeouts.list.push(state).is_ok() {
                timeouts.task.notify();
            }
        }
    }
}
