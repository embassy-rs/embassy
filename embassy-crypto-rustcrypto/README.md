# embassy-crypto-rustcrypto

Software drivers for [`embassy-crypto`](https://crates.io/crates/embassy-crypto),
built on the [RustCrypto](https://github.com/RustCrypto) crates.

Each driver is behind one Cargo feature named `embassy-crypto-<operation>`
(`embassy-crypto-sha256`, `embassy-crypto-aes128-gcm`, `embassy-crypto-p256-ecdsa`, ...).
Enabling it registers this crate's implementation as the global driver for that
operation, and pulls in only the RustCrypto crates it needs. The `all` feature
enables every driver.

Enable a driver here only for operations that no other crate (such as a HAL
with hardware accelerators) provides a driver for; registering two drivers for
one operation fails to link.

```toml
[dependencies]
embassy-crypto = "0.1"
embassy-crypto-rustcrypto = { version = "0.1", features = ["embassy-crypto-sha256", "embassy-crypto-aes128-gcm"] }
```

Nothing in this crate is meant to be used directly: the application uses the
`embassy-crypto` API. Rust does not link a crate that is never named, so the
application must mention this one once, typically next to its other link-only
dependencies:

```rust
use embassy_crypto_rustcrypto as _;
```

Forgetting this shows up as an undefined `_embassy_crypto_*` symbol at link time.
