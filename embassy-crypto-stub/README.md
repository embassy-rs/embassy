# embassy-crypto-stub

Stub drivers for [`embassy-crypto`](https://crates.io/crates/embassy-crypto).

This crate registers a driver for every operation, exactly like
[`embassy-crypto-rustcrypto`](../embassy-crypto-rustcrypto), but every
implementation panics with `unreachable!()`. It is a placeholder for builds
where no real crypto backend (hardware, hand-written ASM, RustCrypto) is
available, so that code linking against `embassy-crypto` still compiles and
links. Any actual crypto operation traps at runtime.

Each driver is behind one Cargo feature named `embassy-crypto-<operation>`
(`embassy-crypto-sha256`, `embassy-crypto-aes128-gcm`, `embassy-crypto-p256-ecdsa`, ...).
The features are empty and exist only so dependents can name them. The `all`
feature enables every driver.

```toml
[dependencies]
embassy-crypto = "0.1"
embassy-crypto-stub = { version = "0.1", features = ["embassy-crypto-sha256", "embassy-crypto-aes128-gcm"] }
```

To make sure Rust links this crate, add a dummy `use` like this:

```rust
use embassy_crypto_stub as _;
```
