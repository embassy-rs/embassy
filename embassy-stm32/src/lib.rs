#![cfg_attr(not(test), no_std)]
#![cfg_attr(feature = "nightly", feature(async_fn_in_trait, impl_trait_projections))]

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

// This must go FIRST so that all the other modules see its macros.
mod fmt;
include!(concat!(env!("OUT_DIR"), "/_macros.rs"));

// Utilities
pub mod time;
mod traits;

// Always-present hardware
pub mod dma;
pub mod gpio;
pub mod rcc;
#[cfg(feature = "_time-driver")]
mod time_driver;
pub mod timer;

// Sometimes-present hardware

#[cfg(adc)]
pub mod adc;
#[cfg(can)]
pub mod can;
#[cfg(crc)]
pub mod crc;
#[cfg(dac)]
pub mod dac;
#[cfg(dcmi)]
pub mod dcmi;
#[cfg(eth)]
pub mod eth;
#[cfg(feature = "exti")]
pub mod exti;
pub mod flash;
#[cfg(fmc)]
pub mod fmc;
#[cfg(hrtim)]
pub mod hrtim;
#[cfg(i2c)]
pub mod i2c;
#[cfg(all(spi_v1, rcc_f4))]
pub mod i2s;
#[cfg(stm32wb)]
pub mod ipcc;
#[cfg(feature = "low-power")]
pub mod low_power;
#[cfg(quadspi)]
pub mod qspi;
#[cfg(rng)]
pub mod rng;
#[cfg(all(rtc, not(rtc_v1)))]
pub mod rtc;
#[cfg(sdmmc)]
pub mod sdmmc;
#[cfg(spi)]
pub mod spi;
#[cfg(usart)]
pub mod usart;
#[cfg(usb)]
pub mod usb;
#[cfg(otg)]
pub mod usb_otg;
#[cfg(iwdg)]
pub mod wdg;

// This must go last, so that it sees all the impl_foo! macros defined earlier.
pub(crate) mod _generated {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/_generated.rs"));
}

pub use crate::_generated::interrupt;

/// Macro to bind interrupts to handlers.
///
/// This defines the right interrupt handlers, and creates a unit struct (like `struct Irqs;`)
/// and implements the right [`Binding`]s for it. You can pass this struct to drivers to
/// prove at compile-time that the right interrupts have been bound.
// developer note: this macro can't be in `embassy-hal-internal` due to the use of `$crate`.
#[macro_export]
macro_rules! bind_interrupts {
    ($vis:vis struct $name:ident { $($irq:ident => $($handler:ty),*;)* }) => {
        $vis struct $name;

        $(
            #[allow(non_snake_case)]
            #[no_mangle]
            unsafe extern "C" fn $irq() {
                $(
                    <$handler as $crate::interrupt::typelevel::Handler<$crate::interrupt::typelevel::$irq>>::on_interrupt();
                )*
            }

            $(
                unsafe impl $crate::interrupt::typelevel::Binding<$crate::interrupt::typelevel::$irq, $handler> for $name {}
            )*
        )*
    };
}

// Reexports
pub use _generated::{peripherals, Peripherals};
pub use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
#[cfg(feature = "unstable-pac")]
pub use stm32_metapac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use stm32_metapac as pac;

use crate::interrupt::Priority;
#[cfg(feature = "rt")]
pub use crate::pac::NVIC_PRIO_BITS;

#[non_exhaustive]
pub struct Config {
    pub rcc: rcc::Config,
    #[cfg(dbgmcu)]
    pub enable_debug_during_sleep: bool,
    #[cfg(bdma)]
    pub bdma_interrupt_priority: Priority,
    #[cfg(dma)]
    pub dma_interrupt_priority: Priority,
    #[cfg(gpdma)]
    pub gpdma_interrupt_priority: Priority,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rcc: Default::default(),
            #[cfg(dbgmcu)]
            enable_debug_during_sleep: true,
            #[cfg(bdma)]
            bdma_interrupt_priority: Priority::P0,
            #[cfg(dma)]
            dma_interrupt_priority: Priority::P0,
            #[cfg(gpdma)]
            gpdma_interrupt_priority: Priority::P0,
        }
    }
}

/// Initialize embassy.
pub fn init(config: Config) -> Peripherals {
    let p = Peripherals::take();

    #[cfg(dbgmcu)]
    if config.enable_debug_during_sleep {
        crate::pac::DBGMCU.cr().modify(|cr| {
            #[cfg(any(dbgmcu_f0, dbgmcu_c0, dbgmcu_g0, dbgmcu_u5))]
            {
                cr.set_dbg_stop(true);
                cr.set_dbg_standby(true);
            }
            #[cfg(any(
                dbgmcu_f1, dbgmcu_f2, dbgmcu_f3, dbgmcu_f4, dbgmcu_f7, dbgmcu_g4, dbgmcu_f7, dbgmcu_l0, dbgmcu_l1,
                dbgmcu_l4, dbgmcu_wb, dbgmcu_wl
            ))]
            {
                cr.set_dbg_sleep(true);
                cr.set_dbg_stop(true);
                cr.set_dbg_standby(true);
            }
            #[cfg(dbgmcu_h7)]
            {
                cr.set_d1dbgcken(true);
                cr.set_d3dbgcken(true);
                cr.set_dbgsleep_d1(true);
                cr.set_dbgstby_d1(true);
                cr.set_dbgstop_d1(true);
            }
        });
    }

    unsafe {
        gpio::init();
        dma::init(
            #[cfg(bdma)]
            config.bdma_interrupt_priority,
            #[cfg(dma)]
            config.dma_interrupt_priority,
            #[cfg(gpdma)]
            config.gpdma_interrupt_priority,
        );
        #[cfg(feature = "exti")]
        exti::init();

        rcc::init(config.rcc);

        // must be after rcc init
        #[cfg(feature = "_time-driver")]
        time_driver::init();

        #[cfg(feature = "low-power")]
        while !crate::rcc::low_power_ready() {
            crate::rcc::clock_refcount_sub();
        }
    }

    p
}
