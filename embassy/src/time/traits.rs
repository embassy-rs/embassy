/// Monotonic clock
pub trait Clock {
    /// Return the current timestamp in ticks.
    /// This is guaranteed to be monotonic, i.e. a call to now() will always return
    /// a greater or equal value than earler calls.
    fn now(&self) -> u64;
}

impl<T: Clock + ?Sized> Clock for &T {
    fn now(&self) -> u64 {
        T::now(self)
    }
}

/// Trait to register a callback at a given timestamp.
pub trait Alarm {
    /// Sets the callback function to be called when the alarm triggers.
    /// The callback may be called from any context (interrupt or thread mode).
    fn set_callback(&self, callback: fn());

    /// Sets an alarm at the given timestamp. When the clock reaches that
    /// timestamp, the provided callback funcion will be called.
    ///
    /// When callback is called, it is guaranteed that now() will return a value greater or equal than timestamp.
    ///
    /// Only one alarm can be active at a time. This overwrites any previously-set alarm if any.
    fn set(&self, timestamp: u64);

    /// Clears the previously-set alarm.
    /// If no alarm was set, this is a noop.
    fn clear(&self);
}

impl<T: Alarm + ?Sized> Alarm for &T {
    fn set_callback(&self, callback: fn()) {
        T::set_callback(self, callback);
    }
    fn set(&self, timestamp: u64) {
        T::set(self, timestamp);
    }
    fn clear(&self) {
        T::clear(self)
    }
}
