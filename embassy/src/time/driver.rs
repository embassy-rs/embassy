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
pub trait Driver {
    /// Return the current timestamp in ticks.
    /// This is guaranteed to be monotonic, i.e. a call to now() will always return
    /// a greater or equal value than earler calls.
    fn now() -> u64;

    /// Try allocating an alarm handle. Returns None if no alarms left.
    /// Initially the alarm has no callback set, and a null `ctx` pointer.
    ///
    /// # Safety
    /// It is UB to make the alarm fire before setting a callback.
    unsafe fn allocate_alarm() -> Option<AlarmHandle>;

    /// Sets the callback function to be called when the alarm triggers.
    /// The callback may be called from any context (interrupt or thread mode).
    fn set_alarm_callback(alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ());

    /// Sets an alarm at the given timestamp. When the current timestamp reaches that
    /// timestamp, the provided callback funcion will be called.
    ///
    /// When callback is called, it is guaranteed that now() will return a value greater or equal than timestamp.
    ///
    /// Only one alarm can be active at a time. This overwrites any previously-set alarm if any.
    fn set_alarm(alarm: AlarmHandle, timestamp: u64);
}

extern "Rust" {
    fn _embassy_time_now() -> u64;
    fn _embassy_time_allocate_alarm() -> Option<AlarmHandle>;
    fn _embassy_time_set_alarm_callback(alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ());
    fn _embassy_time_set_alarm(alarm: AlarmHandle, timestamp: u64);
}

pub(crate) fn now() -> u64 {
    unsafe { _embassy_time_now() }
}
/// Safety: it is UB to make the alarm fire before setting a callback.
pub(crate) unsafe fn allocate_alarm() -> Option<AlarmHandle> {
    _embassy_time_allocate_alarm()
}
pub(crate) fn set_alarm_callback(alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
    unsafe { _embassy_time_set_alarm_callback(alarm, callback, ctx) }
}
pub(crate) fn set_alarm(alarm: AlarmHandle, timestamp: u64) {
    unsafe { _embassy_time_set_alarm(alarm, timestamp) }
}

/// Set the time Driver implementation.
///
/// # Example
///
/// ```
/// struct MyDriver;
/// embassy::time_driver_impl!(MyDriver);
///
/// unsafe impl embassy::time::driver::Driver for MyDriver {
///     fn now() -> u64 {
///         todo!()
///     }
///     unsafe fn allocate_alarm() -> Option<AlarmHandle> {
///         todo!()
///     }
///     fn set_alarm_callback(alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
///         todo!()
///     }
///     fn set_alarm(alarm: AlarmHandle, timestamp: u64) {
///         todo!()
///     }
/// }
///
#[macro_export]
macro_rules! time_driver_impl {
    ($t: ty) => {
        #[no_mangle]
        fn _embassy_time_now() -> u64 {
            <$t as $crate::time::driver::Driver>::now()
        }
        #[no_mangle]
        unsafe fn _embassy_time_allocate_alarm() -> Option<AlarmHandle> {
            <$t as $crate::time::driver::Driver>::allocate_alarm()
        }
        #[no_mangle]
        fn _embassy_time_set_alarm_callback(
            alarm: AlarmHandle,
            callback: fn(*mut ()),
            ctx: *mut (),
        ) {
            <$t as $crate::time::driver::Driver>::set_alarm_callback(alarm, callback, ctx)
        }
        #[no_mangle]
        fn _embassy_time_set_alarm(alarm: AlarmHandle, timestamp: u64) {
            <$t as $crate::time::driver::Driver>::set_alarm(alarm, timestamp)
        }
    };
}
