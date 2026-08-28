//! Module to suspend/resume peripherals

use core::ops::{Deref, DerefMut};
use core::ptr;

trait_set::trait_set! {
    /// Peripheral that can be suspended
    #[allow(private_bounds)]
    pub trait SuspendablePeripheral = SealedSuspendablePeripheral;
}

pub(crate) trait SealedSuspendablePeripheral {
    type InternalState;

    #[allow(dead_code)]
    fn suspend(self) -> Self::InternalState;
    #[allow(dead_code)]
    fn resume(state: Self::InternalState) -> Self;
}

/// A suspended peripheral
pub struct SuspendedPeripheral<T: SuspendablePeripheral> {
    state: T::InternalState,
}

impl<T: SuspendablePeripheral> SuspendedPeripheral<T> {
    /// Suspend a peripheral
    pub fn from(peripheral: T) -> Self {
        Self {
            state: T::suspend(peripheral),
        }
    }

    /// Resume a peripheral
    pub fn resume(self) -> T {
        T::resume(self.state)
    }
}

enum State<T: SuspendablePeripheral> {
    Suspended(SuspendedPeripheral<T>),
    Resumed(T),
}

impl<T: SuspendablePeripheral> State<T> {
    pub fn resume(&mut self) -> &mut T {
        if let State::Suspended(peripheral) = &self {
            unsafe {
                let state = State::Resumed(ptr::read(peripheral).resume());

                ptr::write(self, state);
            }
        }

        if let State::Resumed(peripheral) = self {
            peripheral
        } else {
            unreachable!()
        }
    }

    pub fn suspend(&mut self) {
        if let State::Resumed(peripheral) = &self {
            unsafe {
                let state = State::Suspended(SuspendedPeripheral::from(ptr::read(peripheral)));

                ptr::write(self, state);
            }
        }
    }
}

/// A mutex-like object to resume a peripheral
pub struct ResumablePeripheral<T: SuspendablePeripheral> {
    state: State<T>,
}

impl<T: SuspendablePeripheral> ResumablePeripheral<T> {
    /// Create the object. Will suspend the peripheral as soon as it is passed.
    pub fn new(peripheral: T) -> Self {
        Self {
            state: State::Suspended(SuspendedPeripheral::from(peripheral)),
        }
    }

    /// Create the object from internal state
    #[allow(dead_code)]
    pub(crate) const fn new_suspended(state: T::InternalState) -> Self {
        Self {
            state: State::Suspended(SuspendedPeripheral { state }),
        }
    }

    /// Suspend the peripheral, if it is resumed
    pub fn suspend(&mut self) {
        self.state.suspend()
    }

    /// Resume the peripheral and get a mutable reference to it
    pub fn resume(&mut self) -> &mut T {
        self.state.resume()
    }

    /// Get a guard that will put the peripheral back to sleep once it is dropped
    pub fn borrow(&mut self) -> ResumablePeripheralGuard<'_, T> {
        self.state.resume();

        ResumablePeripheralGuard { state: &mut self.state }
    }
}

/// A mutex-like object guard, that when held, activates the peripheral
pub struct ResumablePeripheralGuard<'a, T: SuspendablePeripheral> {
    state: &'a mut State<T>,
}

impl<'a, T: SuspendablePeripheral> Drop for ResumablePeripheralGuard<'a, T> {
    fn drop(&mut self) {
        self.state.suspend();
    }
}

impl<'a, T: SuspendablePeripheral> Deref for ResumablePeripheralGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        if let State::Resumed(peripheral) = &self.state {
            peripheral
        } else {
            unreachable!()
        }
    }
}

impl<'a, T: SuspendablePeripheral> DerefMut for ResumablePeripheralGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        if let State::Resumed(peripheral) = &mut self.state {
            peripheral
        } else {
            unreachable!()
        }
    }
}
