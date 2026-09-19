//! Advanced Encryption Standard (AES) hardware accelerator.
//!
//! This module drives the on-chip AES block. The hardware revisions below are
//! supported, selected automatically from the target chip:
//!
//! - **`aes_v1`** (STM32L0, L1, L4, F423) — blocking driver, plus an
//!   interrupt-driven async API where the engine has its own interrupt (L4).
//!   This revision only does ECB, CBC and CTR with 128-bit keys.
//! - **`aes_v2`** (STM32G0, G4, L5, U0, WL) — blocking driver, plus an
//!   interrupt-driven async API where the engine has its own interrupt (L5, WL).
//!   Elsewhere the engine shares its interrupt line with another peripheral
//!   (e.g. RNG) or has no dedicated line at all, so only the blocking API
//!   is offered.
//! - **`aes_f7`** (STM32F72x, F73x) — same register map as `aes_v2` minus the
//!   `NPBLB` field, so a final partial payload block cannot be masked out of
//!   the authentication: GCM (both directions) and CCM decryption return
//!   [`Error::ConfigError`] when the payload is not a multiple of 16 bytes.
//! - **`aes_v3a`** (STM32U5) and **`aes_v3b`** (STM32H5, WBA) — blocking driver
//!   plus an interrupt/DMA-backed async API. The two revisions have the same
//!   register map.
//!
//! All revisions expose the same cipher types and the same
//! [`start`](Aes::start) / [`aad_blocking`](Aes::aad_blocking) /
//! [`payload_blocking`](Aes::payload_blocking) / [`finish_blocking`](Aes::finish_blocking)
//! blocking flow, so blocking code is portable across them.
//!
//! # Supported cipher modes
//!
//! | Mode | Padding  | Auth | Use case                                |
//! |------|----------|------|-----------------------------------------|
//! | ECB  | Required | No   | Keys only (not recommended for data)    |
//! | CBC  | Required | No   | File/disk encryption                    |
//! | CTR  | No       | No   | Streaming data, random access           |
//! | GCM  | No       | Yes  | **Recommended** — modern applications   |
//! | GMAC | No       | Yes  | Authentication without encryption       |
//! | CCM  | No       | Yes  | Resource‑constrained devices            |
//!
//! GCM, GMAC and CCM are not available on `aes_v1`.
//!
//! # Key sizes
//!
//! - 128-bit (16 bytes) and 256-bit (32 bytes); only 128-bit on `aes_v1`.
//! - 192-bit keys are **not** supported by this hardware.
//!
//! # IV / nonce requirements
//!
//! - **CBC**: random, unique per message.
//! - **CTR**: must never repeat with the same key.
//! - **GCM/GMAC**: 96-bit (12 bytes), unique per message. IV reuse is
//!   catastrophic.

// The cipher types and the GCM/CCM state machine are shared across revisions;
// only a handful of register primitives differ (see `common`). The register-
// level driver shell is selected per hardware version, following the `adc`
// pattern.
mod common;
pub use common::*;

#[cfg_attr(any(aes_v1, aes_v2, aes_f7), path = "v2.rs")]
#[cfg_attr(any(aes_v3a, aes_v3b), path = "v3.rs")]
mod _version;

pub use _version::*;

#[cfg(any(
    feature = "embassy-crypto-aes128-ecb",
    feature = "embassy-crypto-aes128-cbc",
    feature = "embassy-crypto-aes128-ctr",
    feature = "embassy-crypto-aes128-gcm",
    feature = "embassy-crypto-aes128-ccm",
    feature = "embassy-crypto-aes256-ecb",
    feature = "embassy-crypto-aes256-cbc",
    feature = "embassy-crypto-aes256-ctr",
    feature = "embassy-crypto-aes256-gcm",
    feature = "embassy-crypto-aes256-ccm",
))]
mod driver;
