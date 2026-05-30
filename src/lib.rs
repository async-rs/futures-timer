//! A general purpose crate for working with timeouts and delays with futures.
//!
//! # Examples
//!
//! ```no_run
//! # #[async_std::main]
//! # async fn main() {
//! use std::time::Duration;
//! use futures_timer::Delay;
//!
//! let now = Delay::new(Duration::from_secs(3)).await;
//! println!("waited for 3 secs");
//! # }
//! ```

#![deny(missing_docs)]
#![warn(missing_debug_implementations)]

// On `wasm32-wasip2` the thread-based `native` backend cannot run (the WASI
// component model is single-threaded), so use a `wasi:clocks`-backed timer.
// `target_env = "p2"` selects WASI Preview 2 specifically, leaving `wasip1`
// (and any future preview) on the `native` backend.
#[cfg(all(target_arch = "wasm32", target_os = "wasi", target_env = "p2"))]
mod wasip2;
#[cfg(all(target_arch = "wasm32", target_os = "wasi", target_env = "p2"))]
pub use self::wasip2::Delay;

// Browser wasm with the `wasm-bindgen` feature: `setTimeout` via gloo-timers.
#[cfg(all(
    target_arch = "wasm32",
    feature = "wasm-bindgen",
    not(all(target_os = "wasi", target_env = "p2"))
))]
mod wasm;
#[cfg(all(
    target_arch = "wasm32",
    feature = "wasm-bindgen",
    not(all(target_os = "wasi", target_env = "p2"))
))]
pub use self::wasm::Delay;

// All other targets: the thread-backed timer wheel.
#[cfg(not(any(
    all(target_arch = "wasm32", target_os = "wasi", target_env = "p2"),
    all(target_arch = "wasm32", feature = "wasm-bindgen")
)))]
mod native;
#[cfg(not(any(
    all(target_arch = "wasm32", target_os = "wasi", target_env = "p2"),
    all(target_arch = "wasm32", feature = "wasm-bindgen")
)))]
pub use self::native::Delay;
