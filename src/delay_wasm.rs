//! A version of `Delay` that works on wasm.

use gloo_timers::future::TimeoutFuture;
use std::{time::Duration, pin::Pin, task::{Context, Poll}, future::Future};

#[derive(Debug)]
pub struct Delay(TimeoutFuture);

impl Delay {
	#[inline]
	pub fn new(dur: Duration) -> Delay {
		Self(TimeoutFuture::new(dur.as_millis() as u32))
	}
}

impl Future for Delay {
	type Output = ();

	fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		Pin::new(*self.0).poll(cx)
	}
}
