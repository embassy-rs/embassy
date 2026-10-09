//! Cryptographic accelerator: CryptoCell (nRF52840, nRF5340, nRF91) or CRACEN (nRF54L).
//!
//! The accelerator has three independent engines, each owned through its own peripheral:
//!
//! - The **symmetric engines** (AES, hash, ChaCha20). They share one DMA, so only one of
//!   them runs at a time. Owned through `CRYPTO_SYMMETRIC` and the
//!   [`symmetric::Symmetric`] driver.
//! - The **public key accelerator**. Owned through `CRYPTO_PKA` and the [`pka::Pka`] driver.
//! - The **true random number generator**. Owned through `CRYPTO_RNG` and the [`rng::Rng`]
//!   driver.
//!
//! The accelerator is powered while any of these drivers exists. It is powered down when the
//! last one is dropped.
//!
//! # `embassy-crypto`
//!
//! The `embassy-crypto-*` features register the accelerator as `embassy-crypto` drivers.
//!
//! Each of these features takes over the corresponding peripheral, and its singleton
//! disappears from [`Peripherals`](crate::Peripherals). For each engine you have to choose
//! between the direct API in this module and `embassy-crypto`. You cannot use both.

#[cfg(feature = "_cracen")]
pub(crate) mod cracen;
#[cfg(feature = "_cryptocell")]
pub(crate) mod cryptocell;
pub mod pka;
pub mod rng;
pub mod symmetric;

#[cfg(feature = "_cracen")]
pub(crate) use cracen::{ActivationHandle, activate};
#[cfg(feature = "_cryptocell")]
pub(crate) use cryptocell::{ActivationHandle, activate};
