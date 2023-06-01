//! Interrupt definitions and macros to bind them.
pub use cortex_m::interrupt::{CriticalSection, Mutex};
use embassy_cortex_m::interrupt::_export::declare;
pub use embassy_cortex_m::interrupt::{Binding, Handler, Interrupt, Priority};

use crate::pac::Interrupt as InterruptEnum;
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
declare!(SWI_IRQ_0);
declare!(SWI_IRQ_1);
declare!(SWI_IRQ_2);
declare!(SWI_IRQ_3);
declare!(SWI_IRQ_4);
declare!(SWI_IRQ_5);

/// Macro to bind interrupts to handlers.
///
/// This defines the right interrupt handlers, and creates a unit struct (like `struct Irqs;`)
/// and implements the right [`Binding`]s for it. You can pass this struct to drivers to
/// prove at compile-time that the right interrupts have been bound.
// developer note: this macro can't be in `embassy-cortex-m` due to the use of `$crate`.
#[macro_export]
macro_rules! bind_interrupts {
    ($vis:vis struct $name:ident { $($irq:ident => $($handler:ty),*;)* }) => {
        $vis struct $name;

        $(
            #[allow(non_snake_case)]
            #[no_mangle]
            unsafe extern "C" fn $irq() {
                $(
                    <$handler as $crate::interrupt::Handler<$crate::interrupt::$irq>>::on_interrupt();
                )*
            }

            $(
                unsafe impl $crate::interrupt::Binding<$crate::interrupt::$irq, $handler> for $name {}
            )*
        )*
    };
}
