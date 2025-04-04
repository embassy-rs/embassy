#![cfg_attr(not(test), no_std)]
#![allow(async_fn_in_trait)]
#![cfg_attr(
    docsrs,
    doc = "<div style='padding:30px;background:#810;color:#fff;text-align:center;'><p>You might want to <a href='https://docs.embassy.dev/embassy-stm32'>browse the `embassy-stm32` documentation on the Embassy website</a> instead.</p><p>The documentation here on `docs.rs` is built for a single chip only (stm32h7, stm32h7rs55 in particular), while on the Embassy website you can pick your exact chip from the top menu. Available peripherals and their APIs change depending on the chip.</p></div>\n\n"
)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

// This must go FIRST so that all the other modules see its macros.
mod fmt;
include!(concat!(env!("OUT_DIR"), "/_macros.rs"));

// Utilities
mod macros;
pub mod time;
/// Operating modes for peripherals.
pub mod mode {
    trait SealedMode {}

    /// Operating mode for a peripheral.
    #[allow(private_bounds)]
    pub trait Mode: SealedMode {}

    macro_rules! impl_mode {
        ($name:ident) => {
            impl SealedMode for $name {}
            impl Mode for $name {}
        };
    }

    /// Blocking mode.
    pub struct Blocking;
    /// Async mode.
    pub struct Async;

    impl_mode!(Blocking);
    impl_mode!(Async);
}

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
// FIXME: Cordic driver cause stm32u5a5zj crash
#[cfg(all(cordic, not(any(stm32u5a5, stm32u5a9))))]
pub mod cordic;
#[cfg(crc)]
pub mod crc;
#[cfg(cryp)]
pub mod cryp;
#[cfg(dac)]
pub mod dac;
#[cfg(dcmi)]
pub mod dcmi;
#[cfg(dsihost)]
pub mod dsihost;
#[cfg(dts)]
pub mod dts;
#[cfg(eth)]
pub mod eth;
#[cfg(feature = "exti")]
pub mod exti;
pub mod flash;
#[cfg(fmc)]
pub mod fmc;
#[cfg(hash)]
pub mod hash;
#[cfg(hrtim)]
pub mod hrtim;
#[cfg(hsem)]
pub mod hsem;
#[cfg(hspi)]
pub mod hspi;
#[cfg(i2c)]
pub mod i2c;
#[cfg(any(all(spi_v1, rcc_f4), spi_v3))]
pub mod i2s;
#[cfg(stm32wb)]
pub mod ipcc;
#[cfg(feature = "low-power")]
pub mod low_power;
#[cfg(lptim)]
pub mod lptim;
#[cfg(ltdc)]
pub mod ltdc;
#[cfg(opamp)]
pub mod opamp;
#[cfg(octospi)]
pub mod ospi;
#[cfg(quadspi)]
pub mod qspi;
#[cfg(rng)]
pub mod rng;
#[cfg(all(rtc, not(rtc_v1)))]
pub mod rtc;
#[cfg(sai)]
pub mod sai;
#[cfg(sdmmc)]
pub mod sdmmc;
#[cfg(spdifrx)]
pub mod spdifrx;
#[cfg(spi)]
pub mod spi;
#[cfg(tsc)]
pub mod tsc;
#[cfg(ucpd)]
pub mod ucpd;
#[cfg(uid)]
pub mod uid;
#[cfg(usart)]
pub mod usart;
#[cfg(any(usb, otg))]
pub mod usb;
#[cfg(iwdg)]
pub mod wdg;
#[cfg(xspi)]
pub mod xspi;

// This must go last, so that it sees all the impl_foo! macros defined earlier.
pub(crate) mod _generated {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]
    #![allow(missing_docs)]

    include!(concat!(env!("OUT_DIR"), "/_generated.rs"));
}

pub use crate::_generated::interrupt;

/// Macro to bind interrupts to handlers.
///
/// This defines the right interrupt handlers, and creates a unit struct (like `struct Irqs;`)
/// and implements the right [`Binding`]s for it. You can pass this struct to drivers to
/// prove at compile-time that the right interrupts have been bound.
///
/// Example of how to bind one interrupt:
///
/// ```rust,ignore
/// use embassy_stm32::{bind_interrupts, usb, peripherals};
///
/// bind_interrupts!(struct Irqs {
///     OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
/// });
/// ```
///
/// Example of how to bind multiple interrupts, and multiple handlers to each interrupt, in a single macro invocation:
///
/// ```rust,ignore
/// use embassy_stm32::{bind_interrupts, i2c, peripherals};
///
/// bind_interrupts!(struct Irqs {
///     I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
///     I2C2_3 => i2c::EventInterruptHandler<peripherals::I2C2>, i2c::ErrorInterruptHandler<peripherals::I2C2>,
///         i2c::EventInterruptHandler<peripherals::I2C3>, i2c::ErrorInterruptHandler<peripherals::I2C3>;
/// });
/// ```

// developer note: this macro can't be in `embassy-hal-internal` due to the use of `$crate`.
#[macro_export]
macro_rules! bind_interrupts {
    ($vis:vis struct $name:ident {
        $(
            $(#[cfg($cond_irq:meta)])?
            $irq:ident => $(
                $(#[cfg($cond_handler:meta)])?
                $handler:ty
            ),*;
        )*
    }) => {
        #[derive(Copy, Clone)]
        $vis struct $name;

        $(
            #[allow(non_snake_case)]
            #[no_mangle]
            $(#[cfg($cond_irq)])?
            unsafe extern "C" fn $irq() {
                $(
                    $(#[cfg($cond_handler)])?
                    <$handler as $crate::interrupt::typelevel::Handler<$crate::interrupt::typelevel::$irq>>::on_interrupt();

                )*
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

// Reexports
pub use _generated::{peripherals, Peripherals};
pub use embassy_hal_internal::{Peri, PeripheralType};
#[cfg(feature = "unstable-pac")]
pub use stm32_metapac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use stm32_metapac as pac;

use crate::interrupt::Priority;
#[cfg(feature = "rt")]
pub use crate::pac::NVIC_PRIO_BITS;

/// `embassy-stm32` global configuration.
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    /// RCC config.
    pub rcc: rcc::Config,

    /// Enable debug during sleep and stop.
    ///
    /// May increase power consumption. Defaults to true.
    #[cfg(dbgmcu)]
    pub enable_debug_during_sleep: bool,

    /// On low-power boards (eg. `stm32l4`, `stm32l5` and `stm32u5`),
    /// some GPIO pins are powered by an auxiliary, independent power supply (`VDDIO2`),
    /// which needs to be enabled before these pins can be used.
    ///
    /// May increase power consumption. Defaults to true.
    #[cfg(any(stm32l4, stm32l5, stm32u5))]
    pub enable_independent_io_supply: bool,

    /// On the U5 series all analog peripherals are powered by a separate supply.
    #[cfg(stm32u5)]
    pub enable_independent_analog_supply: bool,

    /// BDMA interrupt priority.
    ///
    /// Defaults to P0 (highest).
    #[cfg(bdma)]
    pub bdma_interrupt_priority: Priority,

    /// DMA interrupt priority.
    ///
    /// Defaults to P0 (highest).
    #[cfg(dma)]
    pub dma_interrupt_priority: Priority,

    /// GPDMA interrupt priority.
    ///
    /// Defaults to P0 (highest).
    #[cfg(gpdma)]
    pub gpdma_interrupt_priority: Priority,

    /// Enables UCPD1 dead battery functionality.
    ///
    /// Defaults to false (disabled).
    #[cfg(peri_ucpd1)]
    pub enable_ucpd1_dead_battery: bool,

    /// Enables UCPD2 dead battery functionality.
    ///
    /// Defaults to false (disabled).
    #[cfg(peri_ucpd2)]
    pub enable_ucpd2_dead_battery: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rcc: Default::default(),
            #[cfg(dbgmcu)]
            enable_debug_during_sleep: true,
            #[cfg(any(stm32l4, stm32l5, stm32u5))]
            enable_independent_io_supply: true,
            #[cfg(stm32u5)]
            enable_independent_analog_supply: true,
            #[cfg(bdma)]
            bdma_interrupt_priority: Priority::P0,
            #[cfg(dma)]
            dma_interrupt_priority: Priority::P0,
            #[cfg(gpdma)]
            gpdma_interrupt_priority: Priority::P0,
            #[cfg(peri_ucpd1)]
            enable_ucpd1_dead_battery: false,
            #[cfg(peri_ucpd2)]
            enable_ucpd2_dead_battery: false,
        }
    }
}

/// Initialize the `embassy-stm32` HAL with the provided configuration.
///
/// This returns the peripheral singletons that can be used for creating drivers.
///
/// This should only be called once at startup, otherwise it panics.
#[cfg(not(feature = "_dual-core"))]
pub fn init(config: Config) -> Peripherals {
    init_hw(config)
}

#[cfg(feature = "_dual-core")]
mod dual_core {
    use core::cell::UnsafeCell;
    use core::mem::MaybeUninit;
    use core::sync::atomic::{AtomicUsize, Ordering};

    use rcc::Clocks;

    use super::*;

    /// Object containing data that embassy needs to share between cores.
    ///
    /// It cannot be initialized by the user. The intended use is:
    ///
    /// ```
    /// use core::mem::MaybeUninit;
    /// use embassy_stm32::{init_secondary, SharedData};
    ///
    /// #[link_section = ".ram_d3"]
    /// static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();
    ///
    /// init_secondary(&SHARED_DATA);
    /// ```
    ///
    /// This static must be placed in the same position for both cores. How and where this is done is left to the user.
    #[repr(C)]
    pub struct SharedData {
        init_flag: AtomicUsize,
        clocks: UnsafeCell<MaybeUninit<Clocks>>,
        config: UnsafeCell<MaybeUninit<SharedConfig>>,
    }

    unsafe impl Sync for SharedData {}

    const INIT_DONE_FLAG: usize = 0xca11ab1e;

    /// Initialize the `embassy-stm32` HAL with the provided configuration.
    /// This function does the actual initialization of the hardware, in contrast to [init_secondary] or [try_init_secondary].
    /// Any core can do the init, but it's important only one core does it.
    ///
    /// This returns the peripheral singletons that can be used for creating drivers.
    ///
    /// This should only be called once at startup, otherwise it panics.
    ///
    /// The `shared_data` is used to coordinate the init with the second core. Read the [SharedData] docs
    /// for more information on its requirements.
    pub fn init_primary(config: Config, shared_data: &'static MaybeUninit<SharedData>) -> Peripherals {
        let shared_data = unsafe { shared_data.assume_init_ref() };

        // Write the flag as soon as possible. Reading this flag uninitialized in the `init_secondary`
        // is maybe unsound? Unclear. If it is indeed unsound, writing it sooner doesn't fix it all,
        // but improves the odds of it going right
        shared_data.init_flag.store(0, Ordering::SeqCst);

        rcc::set_freqs_ptr(shared_data.clocks.get());
        let p = init_hw(config);

        unsafe { *shared_data.config.get() }.write(config.into());

        shared_data.init_flag.store(INIT_DONE_FLAG, Ordering::SeqCst);

        p
    }

    /// Try to initialize the `embassy-stm32` HAL based on the init done by the other core using [init_primary].
    ///
    /// This returns the peripheral singletons that can be used for creating drivers if the other core is done with its init.
    /// If the other core is not done yet, this will return `None`.
    ///
    /// This should only be called once at startup, otherwise it may panic.
    ///
    /// The `shared_data` is used to coordinate the init with the second core. Read the [SharedData] docs
    /// for more information on its requirements.
    pub fn try_init_secondary(shared_data: &'static MaybeUninit<SharedData>) -> Option<Peripherals> {
        let shared_data = unsafe { shared_data.assume_init_ref() };

        if shared_data.init_flag.load(Ordering::SeqCst) != INIT_DONE_FLAG {
            return None;
        }

        // Separate load and store to support the CM0 of the STM32WL
        shared_data.init_flag.store(0, Ordering::SeqCst);

        Some(init_secondary_hw(shared_data))
    }

    /// Initialize the `embassy-stm32` HAL based on the init done by the other core using [init_primary].
    ///
    /// This returns the peripheral singletons that can be used for creating drivers when the other core is done with its init.
    /// If the other core is not done yet, this will spinloop wait on it.
    ///
    /// This should only be called once at startup, otherwise it may panic.
    ///
    /// The `shared_data` is used to coordinate the init with the second core. Read the [SharedData] docs
    /// for more information on its requirements.
    pub fn init_secondary(shared_data: &'static MaybeUninit<SharedData>) -> Peripherals {
        loop {
            if let Some(p) = try_init_secondary(shared_data) {
                return p;
            }
        }
    }

    fn init_secondary_hw(shared_data: &'static SharedData) -> Peripherals {
        rcc::set_freqs_ptr(shared_data.clocks.get());

        let config = unsafe { (*shared_data.config.get()).assume_init() };

        // We use different timers on the different cores, so we have to still initialize one here
        critical_section::with(|cs| {
            unsafe {
                dma::init(
                    cs,
                    #[cfg(bdma)]
                    config.bdma_interrupt_priority,
                    #[cfg(dma)]
                    config.dma_interrupt_priority,
                    #[cfg(gpdma)]
                    config.gpdma_interrupt_priority,
                )
            }

            #[cfg(feature = "_time-driver")]
            // must be after rcc init
            time_driver::init(cs);
        });

        Peripherals::take()
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct SharedConfig {
        #[cfg(bdma)]
        bdma_interrupt_priority: Priority,
        #[cfg(dma)]
        dma_interrupt_priority: Priority,
        #[cfg(gpdma)]
        gpdma_interrupt_priority: Priority,
    }

    impl From<Config> for SharedConfig {
        fn from(value: Config) -> Self {
            let Config {
                #[cfg(bdma)]
                bdma_interrupt_priority,
                #[cfg(dma)]
                dma_interrupt_priority,
                #[cfg(gpdma)]
                gpdma_interrupt_priority,
                ..
            } = value;

            SharedConfig {
                #[cfg(bdma)]
                bdma_interrupt_priority,
                #[cfg(dma)]
                dma_interrupt_priority,
                #[cfg(gpdma)]
                gpdma_interrupt_priority,
            }
        }
    }
}

#[cfg(feature = "_dual-core")]
pub use dual_core::*;

fn init_hw(config: Config) -> Peripherals {
    critical_section::with(|cs| {
        let p = Peripherals::take_with_cs(cs);

        #[cfg(dbgmcu)]
        crate::pac::DBGMCU.cr().modify(|cr| {
            #[cfg(dbgmcu_h5)]
            {
                cr.set_stop(config.enable_debug_during_sleep);
                cr.set_standby(config.enable_debug_during_sleep);
            }
            #[cfg(any(dbgmcu_f0, dbgmcu_c0, dbgmcu_g0, dbgmcu_u0, dbgmcu_u5, dbgmcu_wba, dbgmcu_l5))]
            {
                cr.set_dbg_stop(config.enable_debug_during_sleep);
                cr.set_dbg_standby(config.enable_debug_during_sleep);
            }
            #[cfg(any(
                dbgmcu_f1, dbgmcu_f2, dbgmcu_f3, dbgmcu_f4, dbgmcu_f7, dbgmcu_g4, dbgmcu_f7, dbgmcu_l0, dbgmcu_l1,
                dbgmcu_l4, dbgmcu_wb, dbgmcu_wl
            ))]
            {
                cr.set_dbg_sleep(config.enable_debug_during_sleep);
                cr.set_dbg_stop(config.enable_debug_during_sleep);
                cr.set_dbg_standby(config.enable_debug_during_sleep);
            }
            #[cfg(dbgmcu_h7)]
            {
                cr.set_d1dbgcken(config.enable_debug_during_sleep);
                cr.set_d3dbgcken(config.enable_debug_during_sleep);
                cr.set_dbgsleep_d1(config.enable_debug_during_sleep);
                cr.set_dbgstby_d1(config.enable_debug_during_sleep);
                cr.set_dbgstop_d1(config.enable_debug_during_sleep);
            }
        });

        #[cfg(not(any(stm32f1, stm32wb, stm32wl)))]
        rcc::enable_and_reset_with_cs::<peripherals::SYSCFG>(cs);
        #[cfg(not(any(stm32h5, stm32h7, stm32h7rs, stm32wb, stm32wl)))]
        rcc::enable_and_reset_with_cs::<peripherals::PWR>(cs);
        #[cfg(not(any(stm32f2, stm32f4, stm32f7, stm32l0, stm32h5, stm32h7, stm32h7rs)))]
        rcc::enable_and_reset_with_cs::<peripherals::FLASH>(cs);

        // Enable the VDDIO2 power supply on chips that have it.
        // Note that this requires the PWR peripheral to be enabled first.
        #[cfg(any(stm32l4, stm32l5))]
        {
            crate::pac::PWR.cr2().modify(|w| {
                // The official documentation states that we should ideally enable VDDIO2
                // through the PVME2 bit, but it looks like this isn't required,
                // and CubeMX itself skips this step.
                w.set_iosv(config.enable_independent_io_supply);
            });
        }
        #[cfg(stm32u5)]
        {
            crate::pac::PWR.svmcr().modify(|w| {
                w.set_io2sv(config.enable_independent_io_supply);
            });
            if config.enable_independent_analog_supply {
                crate::pac::PWR.svmcr().modify(|w| {
                    w.set_avm1en(true);
                });
                while !crate::pac::PWR.svmsr().read().vdda1rdy() {}
                crate::pac::PWR.svmcr().modify(|w| {
                    w.set_asv(true);
                });
            } else {
                crate::pac::PWR.svmcr().modify(|w| {
                    w.set_avm1en(false);
                    w.set_avm2en(false);
                });
            }
        }

        // dead battery functionality is still present on these
        // chips despite them not having UCPD- disable it
        #[cfg(any(stm32g070, stm32g0b0))]
        {
            crate::pac::SYSCFG.cfgr1().modify(|w| {
                w.set_ucpd1_strobe(true);
                w.set_ucpd2_strobe(true);
            });
        }

        unsafe {
            #[cfg(ucpd)]
            ucpd::init(
                cs,
                #[cfg(peri_ucpd1)]
                config.enable_ucpd1_dead_battery,
                #[cfg(peri_ucpd2)]
                config.enable_ucpd2_dead_battery,
            );

            #[cfg(feature = "_split-pins-enabled")]
            crate::pac::SYSCFG.pmcr().modify(|pmcr| {
                #[cfg(feature = "split-pa0")]
                pmcr.set_pa0so(true);
                #[cfg(feature = "split-pa1")]
                pmcr.set_pa1so(true);
                #[cfg(feature = "split-pc2")]
                pmcr.set_pc2so(true);
                #[cfg(feature = "split-pc3")]
                pmcr.set_pc3so(true);
            });

            gpio::init(cs);
            dma::init(
                cs,
                #[cfg(bdma)]
                config.bdma_interrupt_priority,
                #[cfg(dma)]
                config.dma_interrupt_priority,
                #[cfg(gpdma)]
                config.gpdma_interrupt_priority,
            );
            #[cfg(feature = "exti")]
            exti::init(cs);

            rcc::init(config.rcc);

            // must be after rcc init
            #[cfg(feature = "_time-driver")]
            time_driver::init(cs);

            #[cfg(feature = "low-power")]
            {
                crate::rcc::REFCOUNT_STOP2 = 0;
                crate::rcc::REFCOUNT_STOP1 = 0;
            }
        }

        p
    })
}
