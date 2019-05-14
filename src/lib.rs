//! A general purpose crate for working with timeouts and delays with futures.
//!
//! This crate is intended to provide general purpose timeouts and interval
//! streams for working with `futures`. The implementation may not be optimized
//! for your particular use case, though, so be sure to read up on the details
//! if you're concerned about that!
//!
//! Basic usage of this crate is relatively simple:
//!
//! ```
//! # extern crate futures;
//! # extern crate futures_timer;
//! # fn main() {
//! use std::time::Duration;
//! use futures_timer::Delay;
//! use futures::prelude::*;
//!
//! let dur = Duration::from_secs(3);
//! let fires_in_three_seconds = Delay::new(dur)
//!     .map(|()| println!("prints three seconds later"));
//! // spawn or use the future above
//! # }
//! ```
//!
//! In addition to a one-shot future you can also create a stream of delayed
//! notifications with the `Interval` type:
//!
//! ```
//! # extern crate futures;
//! # extern crate futures_timer;
//! # fn main() {
//! use std::time::Duration;
//! use futures_timer::Interval;
//! use futures::prelude::*;
//!
//! let dur = Duration::from_secs(4);
//! let stream = Interval::new(dur)
//!     .map(|()| println!("prints every four seconds"));
//! // spawn or use the stream
//! # }
//! ```
//!
//! And you're off to the races! Check out the API documentation for more
//! details about the various methods on `Delay` and `Interval`.
//!
//! # Implementation details
//!
//! The `Delay` and `Interval` types are powered by an associated `Timer`. By
//! default constructors like `Delay::new` and `Interval::new` use a global
//! instance of `Timer` to power their usage. This global `Timer` is spawned
//! onto a helper thread which continuously runs in the background sending out
//! timer notifications.
//!
//! If needed, however, a `Timer` can be constructed manually and the
//! `Delay::new_handle`-style methods can be used to create delays/intervals
//! associated with a specific instance of `Timer`. Each `Timer` has a
//! `TimerHandle` type which is used to associate new objects to it.
//!
//! Note that there's also a `TimerHandle::set_fallback` method which will
//! globally configure the fallback timer handle as well if you'd like to run
//! your own timer.
//!
//! Finally, the implementation of `Timer` itself is currently a binary heap.
//! Timer insertion is O(log n) where n is the number of active timers, and so
//! is firing a timer (which invovles removing from the heap).

#![deny(missing_docs)]

extern crate futures;

use std::cmp::Ordering;
use std::mem;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Arc, Mutex, Weak};
use std::time::Instant;

use futures::task::AtomicTask;
use futures::{Async, Future, Poll};

use arc_list::{ArcList, Node};
use heap::{Heap, Slot};

mod arc_list;
pub mod ext;
mod global;
mod heap;
pub use ext::{FutureExt, StreamExt};

/// A "timer heap" used to power separately owned instances of `Delay` and
/// `Interval`.
///
/// This timer is implemented as a priority queued-based heap. Each `Timer`
/// contains a few primary methods which which to drive it:
///
/// * `next_wake` indicates how long the ambient system needs to sleep until it
///   invokes further processing on a `Timer`
/// * `advance_to` is what actually fires timers on the `Timer`, and should be
///   called essentially every iteration of the event loop, or when the time
///   specified by `next_wake` has elapsed.
/// * The `Future` implementation for `Timer` is used to process incoming timer
///   updates and requests. This is used to schedule new timeouts, update
///   existing ones, or delete existing timeouts. The `Future` implementation
///   will never resolve, but it'll schedule notifications of when to wake up
///   and process more messages.
///
/// Note that if you're using this crate you probably don't need to use a
/// `Timer` as there is a global one already available for you run on a helper
/// thread. If this isn't desirable, though, then the
/// `TimerHandle::set_fallback` method can be used instead!
pub struct Timer {
    inner: Arc<Inner>,
    timer_heap: Heap<HeapTimer>,
}

/// A handle to a `Timer` which is used to create instances of a `Delay`.
#[derive(Clone)]
pub struct TimerHandle {
    inner: Weak<Inner>,
}

mod delay;
mod interval;
pub use self::delay::Delay;
pub use self::interval::Interval;

struct Inner {
    /// List of updates the `Timer` needs to process
    list: ArcList<ScheduledTimer>,

    /// The blocked `Timer` task to receive notifications to the `list` above.
    task: AtomicTask,
}

/// Shared state between the `Timer` and a `Delay`.
struct ScheduledTimer {
    task: AtomicTask,

    // The lowest bit here is whether the timer has fired or not, the second
    // lowest bit is whether the timer has been invalidated, and all the other
    // bits are the "generation" of the timer which is reset during the `reset`
    // function. Only timers for a matching generation are fired.
    state: AtomicUsize,

    inner: Weak<Inner>,
    at: Mutex<Option<Instant>>,

    // TODO: this is only accessed by the timer thread, should have a more
    // lightweight protection than a `Mutex`
    slot: Mutex<Option<Slot>>,
}

/// Entries in the timer heap, sorted by the instant they're firing at and then
/// also containing some payload data.
struct HeapTimer {
    at: Instant,
    gen: usize,
    node: Arc<Node<ScheduledTimer>>,
}

impl Timer {
    /// Creates a new timer heap ready to create new timers.
    pub fn new() -> Timer {
        Timer {
            inner: Arc::new(Inner {
                list: ArcList::new(),
                task: AtomicTask::new(),
            }),
            timer_heap: Heap::new(),
        }
    }

    /// Returns a handle to this timer heap, used to create new timeouts.
    pub fn handle(&self) -> TimerHandle {
        TimerHandle {
            inner: Arc::downgrade(&self.inner),
        }
    }

    /// Returns the time at which this timer next needs to be invoked with
    /// `advance_to`.
    ///
    /// Event loops or threads typically want to sleep until the specified
    /// instant.
    pub fn next_event(&self) -> Option<Instant> {
        self.timer_heap.peek().map(|t| t.at)
    }

    /// Proces any timers which are supposed to fire at or before the current
    /// instant.
    ///
    /// This method is equivalent to `self.advance_to(Instant::now())`.
    pub fn advance(&mut self) {
        self.advance_to(Instant::now())
    }

    /// Proces any timers which are supposed to fire before `now` specified.
    ///
    /// This method should be called on `Timer` periodically to advance the
    /// internal state and process any pending timers which need to fire.
    pub fn advance_to(&mut self, now: Instant) {
        loop {
            match self.timer_heap.peek() {
                Some(head) if head.at <= now => {}
                Some(_) => break,
                None => break,
            };

            // Flag the timer as fired and then notify its task, if any, that's
            // blocked.
            let heap_timer = self.timer_heap.pop().unwrap();
            *heap_timer.node.slot.lock().unwrap() = None;
            let bits = heap_timer.gen << 2;
            match heap_timer
                .node
                .state
                .compare_exchange(bits, bits | 0b01, SeqCst, SeqCst)
            {
                Ok(_) => heap_timer.node.task.notify(),
                Err(_b) => {}
            }
        }
    }

    /// Either updates the timer at slot `idx` to fire at `at`, or adds a new
    /// timer at `idx` and sets it to fire at `at`.
    fn update_or_add(&mut self, at: Instant, node: Arc<Node<ScheduledTimer>>) {
        // TODO: avoid remove + push and instead just do one sift of the heap?
        // In theory we could update it in place and then do the percolation
        // as necessary
        let gen = node.state.load(SeqCst) >> 2;
        let mut slot = node.slot.lock().unwrap();
        if let Some(heap_slot) = slot.take() {
            self.timer_heap.remove(heap_slot);
        }
        *slot = Some(self.timer_heap.push(HeapTimer {
            at: at,
            gen: gen,
            node: node.clone(),
        }));
    }

    fn remove(&mut self, node: Arc<Node<ScheduledTimer>>) {
        // If this `idx` is still around and it's still got a registered timer,
        // then we jettison it form the timer heap.
        let mut slot = node.slot.lock().unwrap();
        let heap_slot = match slot.take() {
            Some(slot) => slot,
            None => return,
        };
        self.timer_heap.remove(heap_slot);
    }

    fn invalidate(&mut self, node: Arc<Node<ScheduledTimer>>) {
        node.state.fetch_or(0b10, SeqCst);
        node.task.notify();
    }
}

impl Future for Timer {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
        self.inner.task.register();
        let mut list = self.inner.list.take();
        while let Some(node) = list.pop() {
            let at = *node.at.lock().unwrap();
            match at {
                Some(at) => self.update_or_add(at, node),
                None => self.remove(node),
            }
        }
        Ok(Poll::Pending)
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        // Seal off our list to prevent any more updates from getting pushed on.
        // Any timer which sees an error from the push will immediately become
        // inert.
        let mut list = self.inner.list.take_and_seal();

        // Now that we'll never receive another timer, drain the list of all
        // updates and also drain our heap of all active timers, invalidating
        // everything.
        while let Some(t) = list.pop() {
            self.invalidate(t);
        }
        while let Some(t) = self.timer_heap.pop() {
            self.invalidate(t.node);
        }
    }
}

impl PartialEq for HeapTimer {
    fn eq(&self, other: &HeapTimer) -> bool {
        self.at == other.at
    }
}

impl Eq for HeapTimer {}

impl PartialOrd for HeapTimer {
    fn partial_cmp(&self, other: &HeapTimer) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapTimer {
    fn cmp(&self, other: &HeapTimer) -> Ordering {
        self.at.cmp(&other.at)
    }
}

static HANDLE_FALLBACK: AtomicUsize = AtomicUsize::new(0);

/// Error returned from `TimerHandle::set_fallback`.
#[derive(Clone, Debug)]
pub struct SetDefaultError(());

impl TimerHandle {
    /// Configures this timer handle to be the one returned by
    /// `TimerHandle::default`.
    ///
    /// By default a global thread is initialized on the first call to
    /// `TimerHandle::default`. This first call can happen transitively through
    /// `Delay::new`. If, however, that hasn't happened yet then the global
    /// default timer handle can be configured through this method.
    ///
    /// This method can be used to prevent the global helper thread from
    /// spawning. If this method is successful then the global helper thread
    /// will never get spun up.
    ///
    /// On success this timer handle will have installed itself globally to be
    /// used as the return value for `TimerHandle::default` unless otherwise
    /// specified.
    ///
    /// # Errors
    ///
    /// If another thread has already called `set_as_global_fallback` or this
    /// thread otherwise loses a race to call this method then it will fail
    /// returning an error. Once a call to `set_as_global_fallback` is
    /// successful then no future calls may succeed.
    pub fn set_as_global_fallback(self) -> Result<(), SetDefaultError> {
        unsafe {
            let val = self.into_usize();
            match HANDLE_FALLBACK.compare_exchange(0, val, SeqCst, SeqCst) {
                Ok(_) => Ok(()),
                Err(_) => {
                    drop(TimerHandle::from_usize(val));
                    Err(SetDefaultError(()))
                }
            }
        }
    }

    fn into_usize(self) -> usize {
        unsafe { mem::transmute::<Weak<Inner>, usize>(self.inner) }
    }

    unsafe fn from_usize(val: usize) -> TimerHandle {
        let inner = mem::transmute::<usize, Weak<Inner>>(val);;
        TimerHandle { inner }
    }
}

impl Default for TimerHandle {
    fn default() -> TimerHandle {
        let mut fallback = HANDLE_FALLBACK.load(SeqCst);

        // If the fallback hasn't been previously initialized then let's spin
        // up a helper thread and try to initialize with that. If we can't
        // actually create a helper thread then we'll just return a "defunkt"
        // handle which will return errors when timer objects are attempted to
        // be associated.
        if fallback == 0 {
            let helper = match global::HelperThread::new() {
                Ok(helper) => helper,
                Err(_) => return TimerHandle { inner: Weak::new() },
            };

            // If we successfully set ourselves as the actual fallback then we
            // want to `forget` the helper thread to ensure that it persists
            // globally. If we fail to set ourselves as the fallback that means
            // that someone was racing with this call to
            // `TimerHandle::default`.  They ended up winning so we'll destroy
            // our helper thread (which shuts down the thread) and reload the
            // fallback.
            if helper.handle().set_as_global_fallback().is_ok() {
                let ret = helper.handle();
                helper.forget();
                return ret;
            }
            fallback = HANDLE_FALLBACK.load(SeqCst);
        }

        // At this point our fallback handle global was configured so we use
        // its value to reify a handle, clone it, and then forget our reified
        // handle as we don't actually have an owning reference to it.
        assert!(fallback != 0);
        unsafe {
            let handle = TimerHandle::from_usize(fallback);
            let ret = handle.clone();
            drop(handle.into_usize());
            return ret;
        }
    }
}
