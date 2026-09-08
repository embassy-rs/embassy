# embassy-crypto-rand

Random number generator driver for [`embassy-crypto`](https://crates.io/crates/embassy-crypto),
backed by the operating system's secure random source through
[`rand::rngs::OsRng`](https://docs.rs/rand/latest/rand/rngs/struct.OsRng.html).

Depending on this crate registers `OsRng` as the global
[`embassy_crypto::driver::Rng`](https://docs.embassy.dev/embassy-crypto/git/default/driver/trait.Rng.html).

```toml
[dependencies]
embassy-crypto = "0.1"
embassy-crypto-rand = "0.1"
```

To make sure Rust links this crate, add a dummy `use` like this:

```rust
use embassy_crypto_rand as _;
```
