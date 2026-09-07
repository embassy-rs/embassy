# embassy-crypto-tests

Known-answer test suites for `embassy-crypto` drivers.

Every suite drives the public `embassy-crypto` API, so it tests whichever
driver the binary links: the same call runs the RustCrypto software drivers in
a host test and a hardware accelerator in a hardware-in-the-loop test.

```rust
use embassy_crypto_rustcrypto as _; // or a HAL that registers drivers

embassy_crypto_tests::sha256().unwrap();
embassy_crypto_tests::aes128_gcm().unwrap();
```

## Vectors

- [Wycheproof](https://github.com/C2SP/wycheproof) for HMAC, AES-CBC, GCM,
  CCM, CMAC, ECDH, ECDSA verification and X25519. `build.rs` clones the
  repository at a pinned commit into `OUT_DIR`; set `WYCHEPROOF_DIR` to an
  existing checkout to build offline.
- Generated at build time with the RustCrypto crates for what Wycheproof has
  no file for: plain digests, AES-ECB, AES-CTR, curve arithmetic and X25519
  key generation.

The tables are compact byte slices into one blob, so a binary only links the
suites it calls. The largest, P-384 ECDH, is about 170 KiB; every other suite
is under 70 KiB.
