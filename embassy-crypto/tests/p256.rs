//! `embassy_crypto::p256` with the `driver-p256-scalar-mul` feature: the
//! wrappers short-circuit to the `p256` crate and never touch the unitrait.
#![cfg(all(feature = "p256", feature = "p256-ecdsa", feature = "driver-p256-scalar-mul"))]

#[path = "common/p256.rs"]
mod suite;
