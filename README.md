# futures-timer

[![Build Status](https://travis-ci.org/alexcrichton/futures-timer.svg?branch=master)](https://travis-ci.org/alexcrichton/futures-timer)

[Documentation](https://docs.rs/futures-timer)

A library for working with timers, timeouts, and intervals with the `futures`
crate.

```toml
# Cargo.toml
[dependencies]
futures-timer = "0.1"
```

An example of using a `Delay` is:

```rust
extern crate futures;
extern crate futures_timer;

use std::time::Duration;

use futures::prelude::*;
use futures_timer::Delay;

fn main() {
    Delay::new(Duration::from_secs(3))
      .map(|()| println!("printed after three seconds"))
      .wait()
      .unwrap();
}
```

And using an `Interval`:

```rust
extern crate futures;
extern crate futures_timer;

use std::time::Duration;

use futures::prelude::*;
use futures_timer::Interval;

fn main() {
    Interval::new(Duration::from_secs(4))
      .take(4)
      .for_each(|()| Ok(println!("printed after three seconds")))
      .wait()
      .unwrap();
}
```

Or timing out a future

```rust
extern crate futures_timer;

use std::time::Duration;

use futures_timer::FutureExt;

fn main() {
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
