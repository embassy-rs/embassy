#![macro_use]

/// Amount of bits of a timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TimerBits {
    /// 16 bits.
    Bits16,
    /// 32 bits.
    Bits32,
}

#[allow(private_bounds)]
pub trait Timer: SealedTimer + 'static {
    /// Amount of bits this timer has.
    const BITS: TimerBits;
}

pub(crate) trait SealedTimer {
    /// Registers for this timer.
    ///
    /// This is a raw pointer to the register block. The actual register block layout varies depending on the
    /// timer type.
    fn regs() -> *mut ();

    /// Enable the interrupt corresponding to this timer.
    unsafe fn enable_interrupt();
}

macro_rules! impl_timer {
    ($name: ident, $bits: ident) => {
        impl crate::timer::SealedTimer for crate::peripherals::$name {
            fn regs() -> *mut () {
                crate::pac::$name.as_ptr()
            }

            unsafe fn enable_interrupt() {
                use embassy_hal_internal::interrupt::InterruptExt;
                crate::interrupt::$name.unpend();
                crate::interrupt::$name.enable();
            }
        }

        impl crate::timer::Timer for crate::peripherals::$name {
            const BITS: crate::timer::TimerBits = crate::timer::TimerBits::$bits;
        }
    };
}
