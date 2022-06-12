//! Interrupt management
//!
//! This module implements an API for managing interrupts compatible with
//! nrf_softdevice::interrupt. Intended for switching between the two at compile-time.

// Re-exports
pub use embassy_cortex_m::interrupt::*;

use crate::pac::Interrupt as InterruptEnum;
use embassy_macros::cortex_m_interrupt_declare as declare;
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
