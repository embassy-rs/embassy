//! NeoChrom (GPU2D) support for STM32 MCUs via NemaGFX.
//!
//! This crate wraps the [`stm32-bindings`](https://github.com/embassy-rs/stm32-bindings)
//! NemaGFX FFI with a small safe-ish driver surface. Platform register access and
//! memory allocation are provided by [`nema-gfx-hal`](https://github.com/embassy-rs/stm32-bindings/tree/main/nema-gfx-hal)
//! until `embassy-stm32`'s GPU2D driver grows HAL hooks for N6/H7RS.

#![no_std]
#![warn(missing_docs)]
#![allow(unsafe_op_in_unsafe_fn)]

pub use nema_gfx_hal;

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
#[cfg(feature = "embedded-graphics")]
pub use target::NeoChromTarget;
