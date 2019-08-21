# futures-timer

[![Build Status](https://api.travis-ci.com/rustasync/futures-timer.svg?branch=master)](https://travis-ci.com/rustasync/futures-timer)

[Documentation](https://docs.rs/futures-timer)

A library for working with timers, timeouts, and intervals with the `futures`
crate.

```toml
# Cargo.toml
[dependencies]
futures-timer = "0.3"
```

An example of using a `Delay` is:

```rust
use std::time::Duration;

use futures::prelude::*;
use futures_timer::Delay;

async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    Delay::new(Duration::from_secs(3))
        .map(|()| println!("printed after three seconds"))
        .await?;
}
```

And using an `Interval`:

```rust
use std::time::Duration;

use futures::prelude::*;
use futures_timer::Interval;

#[runtime::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    Interval::new(Duration::from_secs(4))
        .take(4)
        .for_each(|()| Ok(println!("printed after three seconds")))
        .await?;
}
```

Or timing out a future

```rust
use std::time::Duration;

use futures_timer::FutureExt;

async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // create a future that will take at most 3 seconds to resolve
    let future = long_running_future()
      .timeout(Duration::from_secs(3));
}
```

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
