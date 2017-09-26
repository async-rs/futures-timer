extern crate futures;

use std::cmp::Ordering;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Arc, Weak, Mutex};
use std::time::Instant;

use futures::task::AtomicTask;
use futures::{Future, Async, Poll};

use arc_list::{ArcList, Node};
use heap::{Heap, Slot};

#[macro_use]
mod statik;

mod arc_list;
mod global;
mod heap;

/// A "timer heap" used to power separately owned instances of `Timeout` and
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
pub struct Timer {
    inner: Arc<Inner>,
    timer_heap: Heap<HeapTimer>,
}

/// A handle to a `Timer` which is used to create instances of a `Timeout`.
#[derive(Clone)]
pub struct TimerHandle {
    inner: Weak<Inner>,
}

mod timeout;
mod interval;
pub use self::timeout::Timeout;
pub use self::interval::Interval;

struct Inner {
    /// List of updates the `Timer` needs to process
    list: ArcList<ScheduledTimer>,

    /// The blocked `Timer` task to receive notifications to the `list` above.
    task: AtomicTask,
}

/// Shared state between the `Timer` and a `Timeout`.
struct ScheduledTimer {
    task: AtomicTask,

    // The lowest bit here is whether the timer has fired or not, and all the
    // other bits are the "generation" of the timer which is reset during the
    // `reset` function. Only timers for a matching generation are fired.
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
        TimerHandle { inner: Arc::downgrade(&self.inner) }
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
            let bits = heap_timer.gen << 1;
            match heap_timer.node.state.compare_exchange(bits, bits | 1, SeqCst, SeqCst) {
                Ok(_) => heap_timer.node.task.notify(),
                Err(_b) => {}
            }
        }
    }

    /// Either updates the timer at slot `idx` to fire at `at`, or adds a new
    /// timer at `idx` and sets it to fire at `at`.
    fn update_or_add(&mut self,
                     at: Instant,
                     node: Arc<Node<ScheduledTimer>>) {
        // TODO: avoid remove + push and instead just do one sift of the heap?
        // In theory we could update it in place and then do the percolation
        // as necessary
        let gen = node.state.load(SeqCst) >> 1;
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
        Ok(Async::NotReady)
    }
}

// impl Drop for Timer {
//     fn drop(&mut self) {
//     }
// }

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
