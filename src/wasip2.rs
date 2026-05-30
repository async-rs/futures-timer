//! A version of `Delay` that works on `wasm32-wasip2`.
//!
//! The default `native` backend relies on a background timer thread, which the
//! WASI Preview 2 component model cannot spawn (it is single-threaded). This
//! backend instead arms a `wasi:clocks/monotonic-clock` timer and awaits the
//! resulting pollable through the [`wstd`] reactor, so it integrates with any
//! executor built on `wstd::runtime` (e.g. `#[wstd::main]` / `wstd::runtime::block_on`).

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use wstd::runtime::{AsyncPollable, WaitFor};

/// A future which will fire at `dur` time into the future, backed by
/// `wasi:clocks/monotonic-clock` and driven by the `wstd` reactor.
pub struct Delay {
    wait_for: WaitFor,
}

// SAFETY: `wasm32-wasip2` is single-threaded; the underlying WASI pollable
// handle is a plain integer that is trivially safe to move across the
// non-existent thread boundary. `Delay` must be `Send + Sync` to match the
// other backends' API.
unsafe impl Send for Delay {}
unsafe impl Sync for Delay {}

impl Delay {
    /// Creates a new future which will fire at `dur` time into the future.
    ///
    /// Must be polled from within a `wstd::runtime` executor context.
    pub fn new(dur: Duration) -> Delay {
        Delay {
            wait_for: arm(dur),
        }
    }

    /// Resets the timeout to fire `dur` time into the future.
    pub fn reset(&mut self, dur: Duration) {
        self.wait_for = arm(dur);
    }
}

/// Arm a monotonic-clock timer for `dur` and wrap it as an awaitable pollable.
fn arm(dur: Duration) -> WaitFor {
    let ns = dur.as_nanos().min(u64::MAX as u128) as u64;
    let pollable = wasip2::clocks::monotonic_clock::subscribe_duration(ns);
    AsyncPollable::new(pollable).wait_for()
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // `WaitFor` is `Unpin`, so projecting through `get_mut` is sound.
        Pin::new(&mut self.get_mut().wait_for).poll(cx)
    }
}

impl fmt::Debug for Delay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Delay").finish()
    }
}
