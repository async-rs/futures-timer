use std::error::Error;
use std::pin::Pin;
use std::time::Duration;

use futures::FutureExt;
use futures_timer::Delay;

cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen"))] {
        use wasm_bindgen_test::wasm_bindgen_test as async_test;
        use web_time::Instant;
        wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
    } else {
        use std::time::Instant;
        use async_std::test as async_test;
    }
}

#[async_test]
async fn works() {
    let i = Instant::now();
    let dur = Duration::from_millis(100);
    let _d = Delay::new(dur).await;
    assert!(i.elapsed() > dur);
}

#[async_test]
async fn reset() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let i = Instant::now();
    let dur = Duration::from_millis(100);
    let mut d = Delay::new(dur);

    // Allow us to re-use a future
    Pin::new(&mut d).await;

    assert!(i.elapsed() > dur);

    let i = Instant::now();
    d.reset(dur);
    d.await;
    assert!(i.elapsed() > dur);
    Ok(())
}

#[async_test]
async fn use_after_ready() {
    let dur = Duration::from_millis(100);
    let mut d = Delay::new(dur);

    Pin::new(&mut d).await;

    // Use after ready should return immediately if `Delay::reset`
    // was not called.
    Pin::new(&mut d).now_or_never().unwrap();
}
