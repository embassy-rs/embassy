//! TIMER instance traits by counter width. `build.rs` implements them for each TIMER.

use embassy_hal_internal::PeripheralType;

pub(crate) trait SealedInstance {}

/// A TIMER instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + crate::cmu::CmuPeripheral + PeripheralType + 'static {
    /// Interrupt for this instance.
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

/// A TIMER with a 16-bit counter (`Timer` block).
pub trait Instance16bit: Instance {
    /// Returns this instance's registers.
    fn regs_16bit() -> crate::pac::timer::Timer;
}

/// A TIMER with a 32-bit counter (`Timer32` block).
pub trait Instance32bit: Instance {
    /// Returns this instance's registers.
    fn regs_32bit() -> crate::pac::timer::Timer32;
}
