use std::error::Error;
use std::time::Duration;

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
async fn smoke() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let dur = Duration::from_millis(10);
    let start = Instant::now();
    Delay::new(dur).await;
    assert!(start.elapsed() >= (dur / 2));
    Ok(())
}

#[async_test]
async fn two() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let dur = Duration::from_millis(10);
    Delay::new(dur).await;
    Delay::new(dur).await;
    Ok(())
}
