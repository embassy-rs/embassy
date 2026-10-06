//! NeoChrom (GPU2D) support for STM32 MCUs via NemaGFX.
//!
//! This crate wraps the [`stm32-bindings`](https://github.com/embassy-rs/stm32-bindings)
//! NemaGFX FFI with a small safe-ish driver surface. Platform register access and
//! memory allocation are provided by the in-tree [`hal`] module until
//! `embassy-stm32`'s GPU2D driver grows HAL hooks for N6/H7RS.

#![no_std]
#![warn(missing_docs)]
#![allow(unsafe_op_in_unsafe_fn)]

pub mod hal;
/// Platform HAL glue, re-exported under its former crate name.
pub use crate::hal as nema_gfx_hal;

/// C ABI hooks the prebuilt NemaGFX library calls into.
mod platform;

/// Bump allocator for the NemaGFX command buffers (bare-metal only).
#[cfg(target_os = "none")]
mod bump_alloc;

/// Do-nothing GPU2D HAL used for CI/link tests.
#[cfg(feature = "stub-gpu2d")]
mod gpu2d_stub;

pub(crate) mod fmt;

mod coherency;
mod color;
mod command;
mod driver;
mod error;
mod framebuffer;
#[cfg(not(feature = "stub-gpu2d"))]
mod gpu2d_bridge;

pub mod ffi;
#[cfg(feature = "embedded-graphics")]
pub mod target;

pub use color::{ColorFormat, Rgba8888};
pub use driver::{BlendMode, NeoChrom, TextureFilter, TextureWrap};
pub use error::{Error, InitError};
pub use framebuffer::{ExternalFrameBuffer, FrameBuffer, GpuSurface};
#[cfg(not(feature = "stub-gpu2d"))]
pub use gpu2d_bridge::take_hardware_error;
#[cfg(feature = "embedded-graphics")]
pub use target::NeoChromTarget;
