//! Advanced Encryption Standard (AES) hardware accelerator.
//!
//! This module drives the on-chip AES block. Two hardware revisions are
//! supported, selected automatically from the target chip:
//!
//! - **`aes_v2`** (STM32G0, G4, L5, U0, WL) — blocking driver. On these parts
//!   the AES engine either shares its interrupt line with another peripheral
//!   (e.g. RNG) or has no dedicated line at all, so only a polling/blocking API
//!   is offered.
//! - **`aes_v3b`** (STM32H5, WBA) — blocking driver plus an interrupt/DMA-backed
//!   async API.
//!
//! Both revisions expose the same cipher types and the same
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
//! # Key sizes
//!
//! - 128-bit (16 bytes) and 256-bit (32 bytes).
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

#[cfg_attr(aes_v2, path = "v2.rs")]
#[cfg_attr(aes_v3b, path = "v3b.rs")]
mod _version;

pub use _version::*;
