# embassy-crypto-test

Known-answer test suites for `embassy-crypto` drivers.

Every suite drives the public `embassy-crypto` API, so it tests whichever
driver the binary links.

```rust
use embassy_crypto_rustcrypto as _; // or a HAL that registers drivers

embassy_crypto_test::sha256().unwrap();
embassy_crypto_test::aes128_gcm().unwrap();
```

## Vectors

- [Wycheproof](https://github.com/C2SP/wycheproof) for HMAC, AES-CBC, GCM,
  CCM, CMAC, ChaCha20-Poly1305, ECDH, ECDSA verification, X25519 and Ed25519 verification. `build.rs` clones the
  repository at a pinned commit into `OUT_DIR`. Set `WYCHEPROOF_DIR` to an
  existing checkout to build offline.
- Generated at build time with the RustCrypto crates: plain digests, AES-ECB, AES-CTR, ChaCha8/12/20, ChaCha8/12-Poly1305, CCM with a long AAD, curve arithmetic, X25519 key generation and Ed25519 signing.
