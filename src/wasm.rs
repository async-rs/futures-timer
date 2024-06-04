//! A version of `Delay` that works on wasm.

use gloo_timers::future::TimeoutFuture;
use send_wrapper::SendWrapper;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

/// A version of `Delay` that works on wasm.
#[derive(Debug)]
pub struct Delay(Option<SendWrapper<TimeoutFuture>>);

impl Delay {
    /// Creates a new future which will fire at `dur` time into the future.
    #[inline]
    pub fn new(dur: Duration) -> Delay {
        Self(Some(SendWrapper::new(TimeoutFuture::new(
            dur.as_millis() as u32
        ))))
    }

    /// Resets the timeout.
    #[inline]
    pub fn reset(&mut self, dur: Duration) {
        *self = Delay::new(dur);
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.0.as_mut() {
            Some(delay) => match Pin::new(&mut **delay).poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(()) => {
                    self.0.take();
                    Poll::Ready(())
                }
            },
            None => Poll::Ready(()),
        }
    }
}
