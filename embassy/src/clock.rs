/// Monotonic clock with support for setting an alarm.
///
/// The clock uses a "tick" time unit, whose length is an implementation-dependent constant.
pub trait Monotonic {
    /// Returns the current timestamp in ticks.
    /// This is guaranteed to be monotonic, i.e. a call to now() will always return
    /// a greater or equal value than earler calls.
    fn now(&self) -> u64;

    /// Sets an alarm at the given timestamp. When the clock reaches that
    /// timestamp, the provided callback funcion will be called.
    ///
    /// When callback is called, it is guaranteed that now() will return a value greater or equal than timestamp.
    ///
    /// Only one alarm can be active at a time. This overwrites any previously-set alarm if any.
    fn set_alarm(&self, timestamp: u64, callback: fn());

    /// Clears the previously-set alarm.
    /// If no alarm was set, this is a noop.
    fn clear_alarm(&self);
}
