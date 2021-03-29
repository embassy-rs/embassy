//! Interrupt management
//!
//! This module implements an API for managing interrupts compatible with
//! nrf_softdevice::interrupt. Intended for switching between the two at compile-time.

use core::sync::atomic::{compiler_fence, Ordering};

use crate::pac::NVIC_PRIO_BITS;

// Re-exports
pub use cortex_m::interrupt::{CriticalSection, Mutex};
pub use embassy::interrupt::{declare, take, Interrupt};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

impl From<u8> for Priority {
    fn from(priority: u8) -> Self {
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

impl From<Priority> for u8 {
    fn from(p: Priority) -> Self {
        (p as u8) << (8 - NVIC_PRIO_BITS)
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

mod irqs {
    use super::*;

    declare!(TIMER_IRQ_0);
    declare!(TIMER_IRQ_1);
    declare!(TIMER_IRQ_2);
    declare!(TIMER_IRQ_3);
    declare!(PWM_IRQ_WRAP);
    declare!(USBCTRL_IRQ);
    declare!(XIP_IRQ);
    declare!(PIO0_IRQ_0);
    declare!(PIO0_IRQ_1);
    declare!(PIO1_IRQ_0);
    declare!(PIO1_IRQ_1);
    declare!(DMA_IRQ_0);
    declare!(DMA_IRQ_1);
    declare!(IO_IRQ_BANK0);
    declare!(IO_IRQ_QSPI);
    declare!(SIO_IRQ_PROC0);
    declare!(SIO_IRQ_PROC1);
    declare!(CLOCKS_IRQ);
    declare!(SPI0_IRQ);
    declare!(SPI1_IRQ);
    declare!(UART0_IRQ);
    declare!(UART1_IRQ);
    declare!(ADC_IRQ_FIFO);
    declare!(I2C0_IRQ);
    declare!(I2C1_IRQ);
    declare!(RTC_IRQ);
}

pub use irqs::*;
