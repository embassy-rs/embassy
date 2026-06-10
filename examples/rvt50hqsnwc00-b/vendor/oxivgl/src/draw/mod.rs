// SPDX-License-Identifier: MIT OR Apache-2.0
//! Draw task wrappers for `LV_EVENT_DRAW_TASK_ADDED` handlers.
//!
//! All types in this module are **callback-scoped**: valid only during the
//! `DRAW_TASK_ADDED` event callback. Storing them beyond that scope is
//! undefined behaviour (LVGL frees the draw task after the callback returns).
//!
//! See `docs/spec-memory-lifetime.md` §2 for the lifetime table.

pub mod area;
pub mod dsc;
pub mod layer;
pub mod task;

pub use area::*;
pub use dsc::*;
pub use layer::*;
pub use task::*;
