#![no_std]

//! See the manifest for why this crate exists. The `embassy-crypto`
//! dispatch references the registered driver symbols, which pulls the
//! implementation out of `mcu-crypto-asm` at link time; the `use` below
//! only makes that dependency explicit.
use mcu_crypto_asm as _;
