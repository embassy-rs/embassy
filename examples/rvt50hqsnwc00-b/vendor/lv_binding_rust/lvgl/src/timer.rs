//! Rust timer handling logic
//!
//! LVGL allows for an external timer function to be used. This feature enables
//! a generic `LvClock` interface that can be used to build Rust-native timers.
//!
//! # Building
//!
//! Set `LV_TICK_CUSTOM` to `1` and `LV_TICK_CUSTOM_INCLUDE` to `<rs_timer.h>`
//! in `lv_conf.h`, and enable the `rust_timer` feature on the `lvgl` crate to
//! enable this functionality.
//!
//! # Usage
//!
//! Implement the `lvgl::timer::LvClock` trait on a type and initialize it on
//! the first frame. The `since_init()` function should return a `Duration`
//! representing time elapsed since the beginning of the first frame of the
//! program.
//!
//! ```no_run
//! use lvgl::timer::LvClock;
//!
//! struct Clock {
//!     start: Instant,
//! }
//!
//! impl Default for Clock {
//!     fn default() -> Self {
//!         Self {
//!             start: Instant::now(),
//!         }
//!     }
//! }
//!
//! impl LvClock for Clock {
//!     fn since_init(&self) -> Duration {
//!         Instant::now().duration_since(self.start)
//!     }
//! }
//!
//! fn main() {
//!     // Initialize displays, etc.
//!     let clock = Clock::default();
//!     loop {
//!         // Looping UI logic
//!         lvgl::timer::update_clock(&clock).unwrap();
//!     }
//! }
//! ```
//!
//! For a full example implementation similar to the above, see the
//! `rust_timer` example. When running, make sure to modify the config in
//! `examples/include/lv_conf.h` (or your own) as above first.

use core::num::TryFromIntError;
use core::time::Duration;

static mut RET_VAL: u32 = 0;

/// An LVGL-compatible clock
pub trait LvClock {
    /// Returns the time since the clock was first initialized
    fn since_init(&self) -> Duration;
}

/// Synchronize the clock with LVGL. FIXME: When to call
pub fn update_clock(clock: &impl LvClock) -> Result<(), TryFromIntError> {
    unsafe { RET_VAL = clock.since_init().as_millis().try_into()? }
    Ok(())
}

#[no_mangle]
unsafe extern "C" fn rs_lv_timer() -> u32 {
    RET_VAL
}
