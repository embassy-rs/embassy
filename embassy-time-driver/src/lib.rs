#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! ## Implementing a driver
//!
//! - Define a struct `MyDriver`
//! - Implement [`Driver`] for it
//! - Register it as the global driver with [`time_driver_impl`](crate::time_driver_impl).
//!
//! If your driver has a single set tick rate, enable the corresponding [`tick-hz-*`](crate#tick-rate) feature,
//! which will prevent users from needing to configure it themselves (or selecting an incorrect configuration).
//!
//! If your driver supports a small number of set tick rates, expose your own cargo features and have each one
//! enable the corresponding `embassy-time-driver/tick-*`.
//!
//! Otherwise, don’t enable any `tick-hz-*` feature to let the user configure the tick rate themselves by
//! enabling a feature on `embassy-time`.
//!
//! # Linkage details
//!
//! Instead of the usual "trait + generic params" approach, calls from embassy to the driver are done via `extern` functions.
//!
//! `embassy` internally defines the driver functions as `extern "Rust" { fn _embassy_time_now() -> u64; }` and calls them.
//! The driver crate defines the functions as `#[no_mangle] fn _embassy_time_now() -> u64`. The linker will resolve the
//! calls from the `embassy` crate to call into the driver crate.
//!
//! If there is none or multiple drivers in the crate tree, linking will fail.
//!
//! This method has a few key advantages for something as foundational as timekeeping:
//!
//! - The time driver is available everywhere easily, without having to thread the implementation
//!   through generic parameters. This is especially helpful for libraries.
//! - It means comparing `Instant`s will always make sense: if there were multiple drivers
//!   active, one could compare an `Instant` from driver A to an `Instant` from driver B, which
//!   would yield incorrect results.
//!
//! # Example
//!
//! ```
//! use embassy_time_driver::{Driver, AlarmHandle};
//!
//! struct MyDriver{} // not public!
//!
//! impl Driver for MyDriver {
//!     fn now(&self) -> u64 {
//!         todo!()
//!     }
//!     unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
//!         todo!()
//!     }
//!     fn set_alarm_callback(&self, alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
//!         todo!()
//!     }
//!     fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool {
//!         todo!()
//!     }
//! }
//!
//! embassy_time_driver::time_driver_impl!(static DRIVER: MyDriver = MyDriver{});
//! ```

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

mod tick;

/// Ticks per second of the global timebase.
///
/// This value is specified by the [`tick-*` Cargo features](crate#tick-rate)
pub const TICK_HZ: u64 = tick::TICK_HZ;

/// Alarm handle, assigned by the driver.
#[derive(Clone, Copy)]
pub struct AlarmHandle {
    id: u8,
}

impl AlarmHandle {
    /// Create an AlarmHandle
    ///
    /// Safety: May only be called by the current global Driver impl.
    /// The impl is allowed to rely on the fact that all `AlarmHandle` instances
    /// are created by itself in unsafe code (e.g. indexing operations)
    pub unsafe fn new(id: u8) -> Self {
        Self { id }
    }

    /// Get the ID of the AlarmHandle.
    pub fn id(&self) -> u8 {
        self.id
    }
}

/// Time driver
pub trait Driver: Send + Sync + 'static {
    /// Return the current timestamp in ticks.
    ///
    /// Implementations MUST ensure that:
    /// - This is guaranteed to be monotonic, i.e. a call to now() will always return
    ///   a greater or equal value than earler calls. Time can't "roll backwards".
    /// - It "never" overflows. It must not overflow in a sufficiently long time frame, say
    ///   in 10_000 years (Human civilization is likely to already have self-destructed
    ///   10_000 years from now.). This means if your hardware only has 16bit/32bit timers
    ///   you MUST extend them to 64-bit, for example by counting overflows in software,
    ///   or chaining multiple timers together.
    fn now(&self) -> u64;

    /// Try allocating an alarm handle. Returns None if no alarms left.
    /// Initially the alarm has no callback set, and a null `ctx` pointer.
    ///
    /// # Safety
    /// It is UB to make the alarm fire before setting a callback.
    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle>;

    /// Sets the callback function to be called when the alarm triggers.
    /// The callback may be called from any context (interrupt or thread mode).
    fn set_alarm_callback(&self, alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ());

    /// Sets an alarm at the given timestamp. When the current timestamp reaches the alarm
    /// timestamp, the provided callback function will be called.
    ///
    /// The `Driver` implementation should guarantee that the alarm callback is never called synchronously from `set_alarm`.
    /// Rather - if `timestamp` is already in the past - `false` should be returned and alarm should not be set,
    /// or alternatively, the driver should return `true` and arrange to call the alarm callback as soon as possible, but not synchronously.
    /// There is a rare third possibility that the alarm was barely in the future, and by the time it was enabled, it had slipped into the
    /// past.  This is can be detected by double-checking that the alarm is still in the future after enabling it; if it isn't, `false`
    /// should also be returned to indicate that the callback may have been called already by the alarm, but it is not guaranteed, so the
    /// caller should also call the callback, just like in the more common `false` case. (Note: This requires idempotency of the callback.)
    ///
    /// When callback is called, it is guaranteed that now() will return a value greater or equal than timestamp.
    ///
    /// Only one alarm can be active at a time for each AlarmHandle. This overwrites any previously-set alarm if any.
    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool;
}

extern "Rust" {
    fn _embassy_time_now() -> u64;
    fn _embassy_time_allocate_alarm() -> Option<AlarmHandle>;
    fn _embassy_time_set_alarm_callback(alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ());
    fn _embassy_time_set_alarm(alarm: AlarmHandle, timestamp: u64) -> bool;
}

/// See [`Driver::now`]
pub fn now() -> u64 {
    unsafe { _embassy_time_now() }
}

/// See [`Driver::allocate_alarm`]
///
/// Safety: it is UB to make the alarm fire before setting a callback.
pub unsafe fn allocate_alarm() -> Option<AlarmHandle> {
    _embassy_time_allocate_alarm()
}

/// See [`Driver::set_alarm_callback`]
pub fn set_alarm_callback(alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
    unsafe { _embassy_time_set_alarm_callback(alarm, callback, ctx) }
}

/// See [`Driver::set_alarm`]
pub fn set_alarm(alarm: AlarmHandle, timestamp: u64) -> bool {
    unsafe { _embassy_time_set_alarm(alarm, timestamp) }
}

/// Set the time Driver implementation.
///
/// See the module documentation for an example.
#[macro_export]
macro_rules! time_driver_impl {
    (static $name:ident: $t: ty = $val:expr) => {
        static $name: $t = $val;

        #[no_mangle]
        fn _embassy_time_now() -> u64 {
            <$t as $crate::Driver>::now(&$name)
        }

        #[no_mangle]
        unsafe fn _embassy_time_allocate_alarm() -> Option<$crate::AlarmHandle> {
            <$t as $crate::Driver>::allocate_alarm(&$name)
        }

        #[no_mangle]
        fn _embassy_time_set_alarm_callback(alarm: $crate::AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
            <$t as $crate::Driver>::set_alarm_callback(&$name, alarm, callback, ctx)
        }

        #[no_mangle]
        fn _embassy_time_set_alarm(alarm: $crate::AlarmHandle, timestamp: u64) -> bool {
            <$t as $crate::Driver>::set_alarm(&$name, alarm, timestamp)
        }
    };
}
