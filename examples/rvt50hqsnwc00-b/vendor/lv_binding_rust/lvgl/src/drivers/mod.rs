//! Logic for interoperability with the [`lv_drivers`] project.
//!
//! This dummy module hosts the documentation for using the reexported macros,
//! which are namespaced under LVGL. The appropriate drivers must be enabled in
//! `lv_drv_conf.h`, or compilation will error.
//!
//! The `sdl` example shows how to use both input and display drivers to port
//! the `button_click` example.
//!
//! # Building
//! To compile in support for drivers, ensure the `drivers` feature is enabled
//! (this feature is enabled by default). Also ensure that the C configuration
//! for the drivers is located at the same path as the configuration for LVGL
//! itsel (i.e. the directory pointed to by `DEP_LV_CONFIG_PATH` contains both
//! `lv_conf.h` and `lv_drv_conf.h`).
//!
//! Depending on desired drivers, certain environment variables need to be set.
//! `LVGL_INCLUDE` lists directories to be searched for headers during
//! compilation, and `LVGL_LINK` lists libraries that will be linked in. By
//! default, `LVGL_INCLUDE` is set to `/usr/include,/usr/local/include` and
//! `LVGL_LINK` is set to `SDL2`. Automatically detecting necessary libraries
//! based on enabled features is on the roadmap but not yet implemented.
//!
//! # Display drivers
//!
//! A display driver can be instantiated based on a buffer and output
//! dimensions. The initialization macro will also perform any logic
//! necessary to initialize its respective display, and register it with LVGL.
//!
//! Note: Some drivers, such as the GTK driver, are broken in upstream
//! [`lv_drivers`]. Check open issues upstream if you encounter errors.
//! ```no_run
//! use lvgl::DrawBuffer;
//! use lvgl::lv_drv_disp_sdl;
//!
//! const HOR_RES: u32 = 240;
//! const VER_RES: u32 = 240;
//!
//! let buffer = DrawBuffer::<{ (HOR_RES * VER_RES / 2) as usize }>::default();
//! let display = lv_drv_disp_sdl!(buffer, HOR_RES, VER_RES).unwrap();
//! // ...
//! ```
//!
//! # Input drivers
//!
//! Input drivers can immediately be instantiated with no parameters. Similar
//! to display drivers, initialization logic and registration with LVGL is
//! performed automatically on macro invocation. Setting a custom `feedback_cb`
//! is not (yet) supported, but can be done unsafely with raw C FFI calls.
//!
//! Note: The `InputDevice` trait must be in scope at macro invocation.
//!
//! ```ignore
//! use lvgl::input_device::InputDriver;
//! use lvgl::lv_drv_input_pointer_sdl;
//!
//! fn main() {
//!     // IMPORTANT: Initialize a display driver first!
//!     // ...
//!     let _input = lv_drv_input_pointer_sdl!(display);
//!     // ...
//! }
//! ```
//!
//! [`lv_drivers`]: https://github.com/lvgl/lv_drivers

mod lv_drv_display;
mod lv_drv_input;
