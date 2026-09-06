# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

### Changed
- The crate no longer depends on the RustCrypto crates and no longer implements their traits. Every type
  has inherent methods instead (`Sha256::new`/`update`/`finalize`/`digest`, `Aes128Gcm::encrypt`/`decrypt`, ...).
- The driver traits moved here from `embassy-crypto-driver`, at `embassy_crypto::driver`. The `driver-*`
  features are gone: this crate contains no driver implementations. Software drivers are provided by
  `embassy-crypto-rustcrypto`, hardware drivers by the HALs, each behind one `embassy-crypto-<operation>` feature.
- Elliptic curves have three drivers each: arithmetic (`P256Arith`), ECDH (`P256Ecdh`) and ECDSA (`P256Ecdsa`),
  replacing the scalar-mul/invert/lincomb and `P256Ec` traits. The public API is split the same way, in
  the `p256`, `p384` and `x25519` modules.
- The generic `ec` module (RustCrypto `CurveArithmetic` over a driver) is gone.
- `CryptoError` is renamed `Error`. `Rng::rng_fill` is renamed `fill_bytes`.
- Driver contexts are `Send + Sync + Clone`.
