//! A general purpose crate for working with timeouts and delays with futures.
//!
//! This crate is intended to provide general purpose timeouts and interval
//! streams for working with `futures`. The implementation may not be optimized
//! for your particular use case, though, so be sure to read up on the details
//! if you're concerned about that!
//!
//! Basic usage of this crate is relatively simple:
//!
//! ```no_run
//! # #[runtime::main]
//! # async fn main() {
//! use std::time::Duration;
//! use futures_timer::Delay;
//! use futures::prelude::*;
//!
//! let now = Delay::new(Duration::from_secs(3)).await;
//! println!("waited for 3 secs");
//! # }
//! ```
//!
//! And you're off to the races! Check out the API documentation for more
//! details about the various methods on `Delay`.
//!
//! # Implementation details
//!
//! The `Delay` type is powered by an associated `Timer`. By
//! default constructors like `Delay::new` use a global
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
#![warn(missing_debug_implementations)]

use arc_list::{ArcList, Node};
use heap::{Heap, Slot};
use heap_timer::HeapTimer;
use timer::{Timer, TimerHandle, ScheduledTimer};

mod heap_timer;
mod arc_list;
mod global;
mod heap;
mod timer;

mod delay;
pub use self::delay::Delay;
