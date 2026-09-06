# embassy-crypto

Cryptography for embedded systems: one API, pluggable software and hardware drivers.

This crate provides the API an application uses (`Sha256`, `HmacSha256`,
`Aes128Gcm`, `p256::SecretKey`, ...) and the driver interface that serves
it (`embassy_crypto::driver`). It contains no implementations: every operation
is dispatched at link time to the *driver* registered for it by some crate in
the dependency tree. Drivers are registered one operation at a time, so an
application can take SHA-256 from its HAL's hardware accelerator and AES-GCM
from a software implementation.

- `embassy-crypto-rustcrypto` provides software drivers built on the
  RustCrypto crates.
- HALs such as `embassy-stm32` provide drivers for their hardware accelerators.

# Selecting drivers

Each driver crate exposes one Cargo feature per operation, named
`embassy-crypto-<operation>`. Enable, for each operation the application uses,
exactly one such feature across all driver crates:

```toml
[dependencies]
embassy-crypto = "0.1"
# Hardware SHA-256 and HMAC-SHA-256 from the STM32 HASH peripheral...
embassy-stm32 = { version = "0.6", features = ["stm32h563zi", "embassy-crypto-sha256", "embassy-crypto-hmac-sha256"] }
# ...and everything else in software.
embassy-crypto-rustcrypto = { version = "0.1", features = ["embassy-crypto-aes128-gcm", "embassy-crypto-p256-ecdh", "embassy-crypto-p256-ecdsa"] }
```

If the application uses an operation no crate provides a driver for, or two
crates provide one for, linking fails, with an undefined or duplicate
`_embassy_crypto_<operation>_*` symbol. The symbol names the operation and
therefore the feature to enable or disable. Note that Rust only links crates
the code names: a driver crate the application never otherwise mentions needs
a `use embassy_crypto_rustcrypto as _;` (a HAL is always named already).

Applications should not hard-code driver features themselves when they can
avoid it: a library crate using `embassy-crypto` only depends on
`embassy-crypto`, and leaves the choice of drivers to the final application.

# Operations

| Category   | Types |
|------------|-------|
| Digests    | `Md5`, `Sha1`, `Sha224`, `Sha256`, `Sha384`, `Sha512`, `Sha512_224`, `Sha512_256` |
| HMAC       | `HmacSha1`, `HmacSha224`, `HmacSha256`, `HmacSha384`, `HmacSha512`, `HmacSha512_224`, `HmacSha512_256` |
| AES        | `Aes128`, `Aes256` (block cipher); `Aes128CbcEncrypt`/`Aes128CbcDecrypt`, `Aes128Ctr`, `Aes128Gcm`, `Aes128Ccm`, `Aes128Cmac` and the AES-256 equivalents |
| P-256      | `p256::{Scalar, Point}` (arithmetic), `p256::{SecretKey, PublicKey, SharedSecret}` (ECDH), `p256::{SigningKey, VerifyingKey, Signature}` (ECDSA) |
| P-384      | `p384::*`, same as P-256 |
| X25519     | `x25519::{SecretKey, PublicKey, SharedSecret}` |

# Usage

```rust,ignore
use embassy_crypto::{Sha256, HmacSha256, Aes128Gcm};

let digest = Sha256::digest(b"hello world");

let mut mac = HmacSha256::new(b"key");
mac.update(b"hello world");
let tag = mac.finalize();

let cipher = Aes128Gcm::new(&key);
let tag = cipher.encrypt(&nonce, b"aad", &mut buffer)?;
cipher.decrypt(&nonce, b"aad", &mut buffer, &tag)?;
```

Operations that need randomness (key generation, ECDSA signing) take an
`embassy_crypto::Rng`, implemented by the HALs' random number generators.

# Writing a driver

See the `driver` module. A driver implements the trait of an operation and
registers itself with the matching `*_impl!` macro, behind a Cargo feature
named `embassy-crypto-<operation>`:

```rust,ignore
#[cfg(feature = "embassy-crypto-sha256")]
mod sha256 {
    struct Driver;

    impl embassy_crypto::driver::Sha256 for Driver {
        type Context = MyHashState;
        fn init() -> MyHashState { .. }
        fn update(ctx: &mut MyHashState, data: &[u8]) { .. }
        fn finalize(ctx: MyHashState, out: &mut [u8; 32]) { .. }
    }

    embassy_crypto::sha256_impl!(Driver);
}
```

Hardware drivers whose state does not fit the default opaque context size of an
operation enable the corresponding `large-<operation>` feature of this crate.

# Not yet covered

- ChaCha20-Poly1305
- Ed25519
- SHA-3 and SHAKE
- AES key wrap
- ML-KEM and ML-DSA
- Asynchronous (DMA-driven) operation: all drivers are blocking.
