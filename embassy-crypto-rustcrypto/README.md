# embassy-crypto-rustcrypto

Software drivers for [`embassy-crypto`](https://crates.io/crates/embassy-crypto),
built on the [RustCrypto](https://github.com/RustCrypto) crates.

Each driver is behind one Cargo feature named `embassy-crypto-<operation>`
(`embassy-crypto-sha256`, `embassy-crypto-aes128-gcm`, `embassy-crypto-p256-ecdsa`, ...).
Enabling it registers this crate's implementation as the global driver for that
operation, and pulls in only the RustCrypto crates it needs. The `all` feature
enables every driver.

```toml
[dependencies]
embassy-crypto = "0.1"
embassy-crypto-rustcrypto = { version = "0.1", features = ["embassy-crypto-sha256", "embassy-crypto-aes128-gcm"] }
```

To make sure Rust links this crate, add a dummy `use` like this:

```rust
use embassy_crypto_rustcrypto as _;
```

