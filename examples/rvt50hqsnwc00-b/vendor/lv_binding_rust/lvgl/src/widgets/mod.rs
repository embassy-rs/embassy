//! Widget-specific features
//!
//! Widgets represent individual elements on the screen. Each widget has
//! associated information, namely its parent widget and its styling data. A
//! widget with no parent will have a screen as its parent. Style data is
//! inherited from parent objects by default.

mod arc;
mod bar;
mod keyboard;
mod label;
mod meter;
mod slider;
mod table;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

use crate::NativeObject;
pub use arc::*;
pub use bar::*;
pub use keyboard::*;
pub use label::*;
pub use meter::*;
pub use slider::*;
pub use table::*;
