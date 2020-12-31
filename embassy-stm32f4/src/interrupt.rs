//! Interrupt management
//!
//! This module implements an API for managing interrupts compatible with
//! nrf_softdevice::interrupt. Intended for switching between the two at compile-time.

use core::sync::atomic::{compiler_fence, Ordering};

use crate::pac::NVIC_PRIO_BITS;

// Re-exports
pub use crate::pac::Interrupt;
pub use crate::pac::Interrupt::*; // needed for cortex-m-rt #[interrupt]
pub use cortex_m::interrupt::{CriticalSection, Mutex};
pub use embassy::interrupt::{declare, take, OwnedInterrupt};

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

#[cfg(feature = "stm32f405")]
mod irqs {
    use super::*;
    declare!(WWDG);
    declare!(PVD);
    declare!(TAMP_STAMP);
    declare!(RTC_WKUP);
    //    declare!(FLASH);
    declare!(RCC);
    declare!(EXTI0);
    declare!(EXTI1);
    declare!(EXTI2);
    declare!(EXTI3);
    declare!(EXTI4);
    declare!(DMA1_STREAM0);
    declare!(DMA1_STREAM1);
    declare!(DMA1_STREAM2);
    declare!(DMA1_STREAM3);
    declare!(DMA1_STREAM4);
    declare!(DMA1_STREAM5);
    declare!(DMA1_STREAM6);
    declare!(ADC);
    declare!(CAN1_TX);
    declare!(CAN1_RX0);
    declare!(CAN1_RX1);
    declare!(CAN1_SCE);
    declare!(EXTI9_5);
    declare!(TIM1_BRK_TIM9);
    declare!(TIM1_UP_TIM10);
    declare!(TIM1_TRG_COM_TIM11);
    declare!(TIM1_CC);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM4);
    declare!(I2C1_EV);
    declare!(I2C1_ER);
    declare!(I2C2_EV);
    declare!(I2C2_ER);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(USART2);
    declare!(USART3);
    declare!(EXTI15_10);
    declare!(RTC_ALARM);
    declare!(OTG_FS_WKUP);
    declare!(TIM8_BRK_TIM12);
    declare!(TIM8_UP_TIM13);
    declare!(TIM8_TRG_COM_TIM14);
    declare!(TIM8_CC);
    declare!(DMA1_STREAM7);
    //    declare!(FMC);
    declare!(SDIO);
    declare!(TIM5);
    declare!(SPI3);
    declare!(UART4);
    declare!(UART5);
    declare!(TIM6_DAC);
    declare!(TIM7);
    declare!(DMA2_STREAM0);
    declare!(DMA2_STREAM1);
    declare!(DMA2_STREAM2);
    declare!(DMA2_STREAM3);
    declare!(DMA2_STREAM4);
    declare!(ETH);
    declare!(ETH_WKUP);
    declare!(CAN2_TX);
    declare!(CAN2_RX0);
    declare!(CAN2_RX1);
    declare!(CAN2_SCE);
    declare!(OTG_FS);
    declare!(DMA2_STREAM5);
    declare!(DMA2_STREAM6);
    declare!(DMA2_STREAM7);
    declare!(USART6);
    declare!(I2C3_EV);
    declare!(I2C3_ER);
    declare!(OTG_HS_EP1_OUT);
    declare!(OTG_HS_EP1_IN);
    declare!(OTG_HS_WKUP);
    declare!(OTG_HS);
    declare!(DCMI);
    declare!(CRYP);
    declare!(HASH_RNG);
    declare!(FPU);
    //    declare!(UART7);
    //    declare!(UART8);
    //    declare!(SPI4);
    //    declare!(SPI5);
    //    declare!(SPI6);
    //    declare!(SAI1);
    declare!(LCD_TFT);
    declare!(LCD_TFT_1);
    //    declare!(DMA2D);
}

pub use irqs::*;
