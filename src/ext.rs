//! Extension traits for the standard `Stream` and `Future` traits.

use std::time::{Duration, Instant};
use std::io;
use std::pin::{Pin, Unpin};
use std::ops::Try;

use futures::prelude::*;
use futures::{Poll, task};

use crate::Delay;

/// An extension trait for futures which provides convenient accessors for
/// timing out execution and such.
pub trait FutureExt: Future + Sized {

    /// Creates a new future which will take at most `dur` time to resolve from
    /// the point at which this method is called.
    ///
    /// This combinator creates a new future which wraps the receiving future
    /// in a timeout. The future returned will resolve in at most `dur` time
    /// specified (relative to when this function is called).
    ///
    /// If the future completes before `dur` elapses then the future will
    /// resolve with that item. Otherwise the future will resolve to an error
    /// once `dur` has elapsed.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(futures_api)]
    /// extern crate futures;
    /// extern crate futures_timer;
    ///
    /// use std::time::Duration;
    /// use futures::prelude::*;
    /// use futures::executor::block_on;
    /// use futures_timer::FutureExt;
    ///
    /// # fn long_future() -> impl futures::Future<Output=Result<(), std::io::Error>> {
    /// #   futures::future::ok(())
    /// # }
    /// #
    /// fn main() {
    ///     let future = long_future();
    ///     let timed_out = future.timeout(Duration::from_secs(1));
    ///
    ///     match block_on(timed_out) {
    ///         Ok(item) => println!("got {:?} within enough time!", item),
    ///         Err(_) => println!("took too long to produce the item"),
    ///     }
    /// }
    /// ```
    fn timeout(self, dur: Duration) -> Timeout<Self>
    {
        Timeout {
            timeout: Delay::new(dur),
            future: self,
        }
    }

    /// Creates a new future which will resolve no later than `at` specified.
    ///
    /// This method is otherwise equivalent to the `timeout` method except that
    /// it tweaks the moment at when the timeout elapsed to being specified with
    /// an absolute value rather than a relative one. For more documentation see
    /// the `timeout` method.
    fn timeout_at(self, at: Instant) -> Timeout<Self>
    {
        Timeout {
            timeout: Delay::new_at(at),
            future: self,
        }
    }
}

impl<F: Future> FutureExt for F {}

/// Future returned by the `FutureExt::timeout` method.
pub struct Timeout<F> {
    timeout: Delay,
    future: F,
}

impl<F> Future for Timeout<F>
    where F: Future + Unpin,
          F::Output: Try,
          <F::Output as Try>::Error: From<io::Error>,
          Poll<F::Output>:
              Try<Ok = Poll<<F::Output as Try>::Ok>, Error = <F::Output as Try>::Error>,
{
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, lw: &task::LocalWaker) -> Poll<Self::Output> {
        let future = Pin::new(&mut self.future);
        match Future::poll(future, &lw) {
            Poll::Pending => {}
            other => return other,
        }

        let timeout = Pin::new(&mut self.timeout);
        if Future::poll(timeout, lw).is_ready() {
            Try::from_error(io::Error::new(io::ErrorKind::TimedOut, "future timed out").into())
        } else {
            Poll::Pending
        }
    }
}

/// An extension trait for streams which provides convenient accessors for
/// timing out execution and such.
pub trait StreamExt: Stream + Sized {

    /// Creates a new stream which will take at most `dur` time to yield each
    /// item of the stream.
    ///
    /// This combinator creates a new stream which wraps the receiving stream
    /// in a timeout-per-item. The stream returned will resolve in at most
    /// `dur` time for each item yielded from the stream. The first item's timer
    /// starts when this method is called.
    ///
    /// If a stream's item completes before `dur` elapses then the timer will be
    /// reset for the next item. If the timeout elapses, however, then an error
    /// will be yielded on the stream and the timer will be reset.
    fn timeout(self, dur: Duration) -> TimeoutStream<Self>
    {
        TimeoutStream {
            timeout: Delay::new(dur),
            dur,
            stream: self,
        }
    }
}

impl<S: Stream> StreamExt for S {}

/// Stream returned by the `StreamExt::timeout` method.
pub struct TimeoutStream<S> {
    timeout: Delay,
    dur: Duration,
    stream: S,
}

impl<S> Stream for TimeoutStream<S>
    where S: Stream + Unpin,
          S::Item: Try,
          <S::Item as Try>::Error: From<io::Error>,
          Poll<Option<S::Item>>:
            Try<Ok = Poll<Option<<S::Item as Try>::Ok>>, Error = <S::Item as Try>::Error>,
{
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, lw: &task::LocalWaker) -> Poll<Option<Self::Item>> {
        let dur = self.dur.clone();
        let mut stream = Pin::new(&mut self.stream);
        match Stream::poll_next(stream.as_mut(), lw) {
            Poll::Pending => {}
            other => {
                self.timeout.reset(dur);
                return other
            }
        }

        let mut timeout = Pin::new(&mut self.timeout);
        if Future::poll(timeout.as_mut(), lw).is_ready() {
            timeout.reset(dur);
            Try::from_error(io::Error::new(io::ErrorKind::TimedOut, "stream item timed out").into())
        } else {
            Poll::Pending
        }
    }
}
