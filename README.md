# one-shot-mutex

[![Crates.io](https://img.shields.io/crates/v/one-shot-mutex)](https://crates.io/crates/one-shot-mutex)
[![docs.rs](https://img.shields.io/docsrs/one-shot-mutex)](https://docs.rs/one-shot-mutex)
[![CI](https://github.com/mkroening/one-shot-mutex/actions/workflows/ci.yml/badge.svg)](https://github.com/mkroening/one-shot-mutex/actions/workflows/ci.yml)

A one-shot mutex that panics instead of (dead)locking on contention.

```rust
use one_shot_mutex::OneShotMutex;

static X: OneShotMutex<i32> = OneShotMutex::new(42);

let x = X.lock();

// This panics instead of deadlocking.
// let x2 = X.lock();

// Once we unlock the mutex, we can lock it again.
drop(x);
let x = X.lock();
```

For API documentation, see the [docs].

[docs]: https://docs.rs/one-shot-mutex

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
