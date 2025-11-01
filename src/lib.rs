#![no_std]

pub mod clocks; // still provide clock helpers
#[cfg(feature = "gpio")]
pub mod gpio;
pub mod pins; // pin mux helpers
pub mod reset; // reset control helpers

pub mod config;
pub mod interrupt;
pub mod ostimer;
pub mod uart;
pub mod lpuart;
pub mod rtc;
pub mod adc;

embassy_hal_internal::peripherals!(
    #[cfg(feature = "lpuart2")]
    LPUART2,
    #[cfg(feature = "ostimer0")]
    OSTIMER0,
    #[cfg(feature = "gpio")]
    GPIO,
    #[cfg(feature = "rtc0")]
    RTC0,
    #[cfg(feature = "adc1")]
    ADC1,
);

/// Get access to the PAC Peripherals for low-level register access.
/// This is a lazy-initialized singleton that can be called after init().
#[allow(static_mut_refs)]
pub fn pac() -> &'static pac::Peripherals {
    // SAFETY: We only call this after init(), and the PAC is a singleton.
    // The embassy peripheral tokens ensure we don't have multiple mutable accesses.
    unsafe {
        static mut PAC_INSTANCE: Option<pac::Peripherals> = None;
        if PAC_INSTANCE.is_none() {
            PAC_INSTANCE = Some(pac::Peripherals::steal());
        }
        PAC_INSTANCE.as_ref().unwrap()
    }
}

pub use cortex_m_rt;
pub use mcxa276_pac as pac;
// Use cortex-m-rt's #[interrupt] attribute directly; PAC does not re-export it.

// Re-export interrupt traits and types
pub use interrupt::InterruptExt;
#[cfg(feature = "ostimer0")]
pub use ostimer::Ostimer0 as Ostimer0Token;
#[cfg(feature = "lpuart2")]
pub use uart::Lpuart2 as Uart2Token;
#[cfg(feature = "rtc0")]
pub use rtc::Rtc0 as Rtc0Token;
#[cfg(feature = "adc1")]
pub use adc::Adc1 as Adc1Token;
#[cfg(feature = "gpio")]
pub use gpio::{pins::*, AnyPin, Flex, Gpio as GpioToken, Input, Level, Output};

/// Initialize HAL with configuration (mirrors embassy-imxrt style). Minimal: just take peripherals.
/// Also applies configurable NVIC priority for the OSTIMER OS_EVENT interrupt (no enabling).
#[allow(unused_variables)]
pub fn init(cfg: crate::config::Config) -> Peripherals {
    let peripherals = Peripherals::take();
    #[cfg(feature = "ostimer0")]
    {
        // Apply user-configured priority early; enabling is left to examples/apps
        crate::interrupt::OS_EVENT.set_priority(cfg.time_interrupt_priority);
    }
    #[cfg(feature = "rtc0")]
    {
        // Apply user-configured priority early; enabling is left to examples/apps
        crate::interrupt::RTC.set_priority(cfg.rtc_interrupt_priority);
    }
    #[cfg(feature = "adc1")]
    {
        // Apply user-configured priority early; enabling is left to examples/apps
        crate::interrupt::ADC1.set_priority(cfg.adc_interrupt_priority);
    }
    peripherals
}

/// Optional hook called by cortex-m-rt before RAM init.
/// We proactively mask and clear all NVIC IRQs to avoid wedges from stale state
/// left by soft resets/debug sessions.
///
/// NOTE: Manual VTOR setup is required for RAM execution. The cortex-m-rt 'set-vtor'
/// feature is incompatible with our setup because it expects __vector_table to be
/// defined differently than how our RAM-based linker script arranges it.
#[no_mangle]
pub unsafe extern "C" fn __pre_init() {
    // Set the VTOR to point to the interrupt vector table in RAM
    // This is required since code runs from RAM on this MCU
    crate::interrupt::vtor_set_ram_vector_base(0x2000_0000 as *const u32);

    // Mask and clear pending for all NVIC lines (0..127) to avoid stale state across runs.
    let nvic = &*cortex_m::peripheral::NVIC::PTR;
    for i in 0..4 {
        // 4 words x 32 = 128 IRQs
        nvic.icer[i].write(0xFFFF_FFFF);
        nvic.icpr[i].write(0xFFFF_FFFF);
    }
    // Do NOT touch peripheral registers here: clocks may be off and accesses can fault.
    crate::interrupt::clear_default_handler_snapshot();
}

/// Internal helper to dispatch a type-level interrupt handler.
#[inline(always)]
#[doc(hidden)]
pub unsafe fn __handle_interrupt<T, H>()
where
    T: crate::interrupt::typelevel::Interrupt,
    H: crate::interrupt::typelevel::Handler<T>,
{
    H::on_interrupt();
}

/// Macro to bind interrupts to handlers, similar to embassy-imxrt.
///
/// Example:
/// - Bind OS_EVENT to the OSTIMER time-driver handler
///   bind_interrupts!(struct Irqs { OS_EVENT => crate::ostimer::time_driver::OsEventHandler; });
#[macro_export]
macro_rules! bind_interrupts {
    ($(#[$attr:meta])* $vis:vis struct $name:ident {
        $(
            $(#[cfg($cond_irq:meta)])?
            $irq:ident => $(
                $(#[cfg($cond_handler:meta)])?
                $handler:ty
            ),*;
        )*
    }) => {
        #[derive(Copy, Clone)]
        $(#[$attr])*
        $vis struct $name;

        $(
            #[allow(non_snake_case)]
            #[no_mangle]
            $(#[cfg($cond_irq)])?
            unsafe extern "C" fn $irq() {
                unsafe {
                    $(
                        $(#[cfg($cond_handler)])?
                        <$handler as $crate::interrupt::typelevel::Handler<$crate::interrupt::typelevel::$irq>>::on_interrupt();
                    )*
                }
            }

            $(#[cfg($cond_irq)])?
            $crate::bind_interrupts!(@inner
                $(
                    $(#[cfg($cond_handler)])?
                    unsafe impl $crate::interrupt::typelevel::Binding<$crate::interrupt::typelevel::$irq, $handler> for $name {}
                )*
            );
        )*
    };
    (@inner $($t:tt)*) => {
        $($t)*
    }
}
