//! Input driver logic and handling
//!
//! LVGL supports 4 types of input device. The current status as to support in
//! this library is:
//! - Pointer: Fully supported
//! - Keyboard: Unsupported
//! - Button: Unsupported
//! - Encoder: Unsupported
//!
//! The general order of operations when creating an input device is
//! initializing an instance of the desired device, setting a callback function
//! (and any other parameters and functions, depending on type), and
//! registering the device to LVGL. Inputs are sent to LVGL via a buffer. An
//! example making use of the `embedded_graphics` crate could be:
//! ```ignore
//! use lvgl::input_device::InputDriver;
//! use lvgl::input_device::pointer::{Pointer, PointerInputData};
//! use embedded_graphics::prelude::*;
//!
//! fn main() {
//!     // IMPORTANT: Initialize a display driver first!
//!     // ...
//!     // Define the initial state of your input//! 
//!     let mut latest_touch_status = PointerInputData::Touch(Point::new(0, 0)).released().once();
//!     // Register a new input device that's capable of reading the current state of the input
//!     let pointer = Pointer::register(|| latest_touch_status, &display).unwrap();
//!     // ...
//! }
//! ```
//! For a full example, see the `button_click` example.

mod generic;
pub use generic::*;

pub mod encoder;
pub mod pointer;
