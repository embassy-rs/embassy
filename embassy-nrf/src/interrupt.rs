//! Interrupt management
//!
//! This module implements an API for managing interrupts compatible with
//! nrf_softdevice::interrupt. Intended for switching between the two at compile-time.

use core::sync::atomic::{compiler_fence, AtomicBool, Ordering};

use crate::pac::{NVIC, NVIC_PRIO_BITS};

// Re-exports
pub use crate::pac::Interrupt;
pub use crate::pac::Interrupt::*; // needed for cortex-m-rt #[interrupt]
pub use bare_metal::{CriticalSection, Mutex};

#[derive(defmt::Format, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum Priority {
    Level0 = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
    Level5 = 5,
    Level6 = 6,
    Level7 = 7,
}

impl Priority {
    #[inline]
    fn to_nvic(self) -> u8 {
        (self as u8) << (8 - NVIC_PRIO_BITS)
    }

    #[inline]
    fn from_nvic(priority: u8) -> Self {
        match priority >> (8 - NVIC_PRIO_BITS) {
            0 => Self::Level0,
            1 => Self::Level1,
            2 => Self::Level2,
            3 => Self::Level3,
            4 => Self::Level4,
            5 => Self::Level5,
            6 => Self::Level6,
            7 => Self::Level7,
            _ => unreachable!(),
        }
    }
}

#[inline]
pub fn free<F, R>(f: F) -> R
where
    F: FnOnce(&CriticalSection) -> R,
{
    unsafe {
        // TODO: assert that we're in privileged level
        // Needed because disabling irqs in non-privileged level is a noop, which would break safety.

        let primask: u32;
        asm!("mrs {}, PRIMASK", out(reg) primask);

        asm!("cpsid i");

        // Prevent compiler from reordering operations inside/outside the critical section.
        compiler_fence(Ordering::SeqCst);

        let r = f(&CriticalSection::new());

        compiler_fence(Ordering::SeqCst);

        if primask & 1 == 0 {
            asm!("cpsie i");
        }

        r
    }
}

#[inline]
pub fn enable(irq: Interrupt) {
    unsafe {
        NVIC::unmask(irq);
    }
}

#[inline]
pub fn disable(irq: Interrupt) {
    NVIC::mask(irq);
}

#[inline]
pub fn is_active(irq: Interrupt) -> bool {
    NVIC::is_active(irq)
}

#[inline]
pub fn is_enabled(irq: Interrupt) -> bool {
    NVIC::is_enabled(irq)
}

#[inline]
pub fn is_pending(irq: Interrupt) -> bool {
    NVIC::is_pending(irq)
}

#[inline]
pub fn pend(irq: Interrupt) {
    NVIC::pend(irq)
}

#[inline]
pub fn unpend(irq: Interrupt) {
    NVIC::unpend(irq)
}

#[inline]
pub fn get_priority(irq: Interrupt) -> Priority {
    Priority::from_nvic(NVIC::get_priority(irq))
}

#[inline]
pub fn set_priority(irq: Interrupt, prio: Priority) {
    unsafe {
        cortex_m::peripheral::Peripherals::steal()
            .NVIC
            .set_priority(irq, prio.to_nvic())
    }
}
