#![no_std]
#![allow(async_fn_in_trait)]
#![cfg_attr(
    docsrs,
    doc = "<div style='padding:30px;background:#810;color:#fff;text-align:center;'><p>You might want to <a href='https://docs.embassy.dev/embassy-nrf'>browse the `embassy-nrf` documentation on the Embassy website</a> instead.</p><p>The documentation here on `docs.rs` is built for a single chip only (nRF52840 in particular), while on the Embassy website you can pick your exact chip from the top menu. Available peripherals and their APIs change depending on the chip.</p></div>\n\n"
)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

#[cfg(not(any(
    feature = "nrf51",
    feature = "nrf52805",
    feature = "nrf52810",
    feature = "nrf52811",
    feature = "nrf52820",
    feature = "nrf52832",
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "nrf5340-app-s",
    feature = "nrf5340-app-ns",
    feature = "nrf5340-net",
    feature = "nrf9160-s",
    feature = "nrf9160-ns",
)))]
compile_error!("No chip feature activated. You must activate exactly one of the following features: nrf52810, nrf52811, nrf52832, nrf52833, nrf52840");

#[cfg(all(feature = "reset-pin-as-gpio", not(feature = "_nrf52")))]
compile_error!("feature `reset-pin-as-gpio` is only valid for nRF52 series chips.");

#[cfg(all(feature = "nfc-pins-as-gpio", not(any(feature = "_nrf52", feature = "_nrf5340-app"))))]
compile_error!("feature `nfc-pins-as-gpio` is only valid for nRF52, or nRF53's application core.");

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;
pub(crate) mod util;

#[cfg(feature = "_time-driver")]
mod time_driver;

#[cfg(not(feature = "nrf51"))]
pub mod buffered_uarte;
pub mod gpio;
#[cfg(feature = "gpiote")]
pub mod gpiote;
#[cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840"))]
pub mod i2s;
pub mod nvmc;
#[cfg(any(
    feature = "nrf52810",
    feature = "nrf52811",
    feature = "nrf52832",
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "_nrf5340-app",
    feature = "_nrf9160"
))]
pub mod pdm;
pub mod ppi;
#[cfg(not(any(
    feature = "nrf51",
    feature = "nrf52805",
    feature = "nrf52820",
    feature = "_nrf5340-net"
)))]
pub mod pwm;
#[cfg(not(any(feature = "nrf51", feature = "_nrf9160", feature = "_nrf5340-net")))]
pub mod qdec;
#[cfg(any(feature = "nrf52840", feature = "_nrf5340-app"))]
pub mod qspi;
#[cfg(not(any(feature = "_nrf5340-app", feature = "_nrf9160")))]
pub mod rng;
#[cfg(not(any(feature = "nrf51", feature = "nrf52820", feature = "_nrf5340-net")))]
pub mod saadc;
#[cfg(not(feature = "nrf51"))]
pub mod spim;
#[cfg(not(feature = "nrf51"))]
pub mod spis;
#[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
pub mod temp;
pub mod timer;
#[cfg(not(feature = "nrf51"))]
pub mod twim;
#[cfg(not(feature = "nrf51"))]
pub mod twis;
#[cfg(not(feature = "nrf51"))]
pub mod uarte;
#[cfg(any(
    feature = "_nrf5340-app",
    feature = "nrf52820",
    feature = "nrf52833",
    feature = "nrf52840"
))]
pub mod usb;
#[cfg(not(feature = "_nrf5340"))]
pub mod wdt;

// This mod MUST go last, so that it sees all the `impl_foo!` macros
#[cfg_attr(feature = "nrf51", path = "chips/nrf51.rs")]
#[cfg_attr(feature = "nrf52805", path = "chips/nrf52805.rs")]
#[cfg_attr(feature = "nrf52810", path = "chips/nrf52810.rs")]
#[cfg_attr(feature = "nrf52811", path = "chips/nrf52811.rs")]
#[cfg_attr(feature = "nrf52820", path = "chips/nrf52820.rs")]
#[cfg_attr(feature = "nrf52832", path = "chips/nrf52832.rs")]
#[cfg_attr(feature = "nrf52833", path = "chips/nrf52833.rs")]
#[cfg_attr(feature = "nrf52840", path = "chips/nrf52840.rs")]
#[cfg_attr(feature = "_nrf5340-app", path = "chips/nrf5340_app.rs")]
#[cfg_attr(feature = "_nrf5340-net", path = "chips/nrf5340_net.rs")]
#[cfg_attr(feature = "_nrf9160", path = "chips/nrf9160.rs")]
mod chip;

/// Macro to bind interrupts to handlers.
///
/// This defines the right interrupt handlers, and creates a unit struct (like `struct Irqs;`)
/// and implements the right [`Binding`]s for it. You can pass this struct to drivers to
/// prove at compile-time that the right interrupts have been bound.
///
/// Example of how to bind one interrupt:
///
/// ```rust,ignore
/// use embassy_nrf::{bind_interrupts, spim, peripherals};
///
/// bind_interrupts!(struct Irqs {
///     SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
/// });
/// ```
///
/// Example of how to bind multiple interrupts in a single macro invocation:
///
/// ```rust,ignore
/// use embassy_nrf::{bind_interrupts, spim, twim, peripherals};
///
/// bind_interrupts!(struct Irqs {
///     SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
///     SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<peripherals::TWISPI0>;
/// });
/// ```

// developer note: this macro can't be in `embassy-hal-internal` due to the use of `$crate`.
#[macro_export]
macro_rules! bind_interrupts {
        ($vis:vis struct $name:ident { $($irq:ident => $($handler:ty),*;)* }) => {
            #[derive(Copy, Clone)]
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

#[cfg(feature = "unstable-pac")]
pub use chip::pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use chip::pac;
pub use chip::{peripherals, Peripherals, EASY_DMA_SIZE};
pub use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

pub use crate::chip::interrupt;
pub use crate::pac::NVIC_PRIO_BITS;

pub mod config {
    //! Configuration options used when initializing the HAL.

    /// High frequency clock source.
    pub enum HfclkSource {
        /// Internal source
        Internal,
        /// External source from xtal.
        ExternalXtal,
    }

    /// Low frequency clock source
    pub enum LfclkSource {
        /// Internal RC oscillator
        InternalRC,
        /// Synthesized from the high frequency clock source.
        #[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
        Synthesized,
        /// External source from xtal.
        ExternalXtal,
        /// External source from xtal with low swing applied.
        #[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
        ExternalLowSwing,
        /// External source from xtal with full swing applied.
        #[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
        ExternalFullSwing,
    }

    /// SWD access port protection setting.
    #[non_exhaustive]
    pub enum Debug {
        /// Debugging is allowed (APPROTECT is disabled). Default.
        Allowed,
        /// Debugging is not allowed (APPROTECT is enabled).
        Disallowed,
        /// APPROTECT is not configured (neither to enable it or disable it).
        /// This can be useful if you're already doing it by other means and
        /// you don't want embassy-nrf to touch UICR.
        NotConfigured,
    }

    /// Settings for enabling the built in DCDC converters.
    #[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
    pub struct DcdcConfig {
        /// Config for the first stage DCDC (VDDH -> VDD), if disabled LDO will be used.
        #[cfg(feature = "nrf52840")]
        pub reg0: bool,
        /// Config for the second stage DCDC (VDD -> DEC4), if disabled LDO will be used.
        pub reg1: bool,
    }

    /// Settings for enabling the built in DCDC converters.
    #[cfg(feature = "_nrf5340-app")]
    pub struct DcdcConfig {
        /// Config for the high voltage stage, if disabled LDO will be used.
        pub regh: bool,
        /// Config for the main rail, if disabled LDO will be used.
        pub regmain: bool,
        /// Config for the radio rail, if disabled LDO will be used.
        pub regradio: bool,
    }

    /// Settings for enabling the built in DCDC converter.
    #[cfg(feature = "_nrf9160")]
    pub struct DcdcConfig {
        /// Config for the main rail, if disabled LDO will be used.
        pub regmain: bool,
    }

    /// Configuration for peripherals. Default configuration should work on any nRF chip.
    #[non_exhaustive]
    pub struct Config {
        /// High frequency clock source.
        pub hfclk_source: HfclkSource,
        /// Low frequency clock source.
        pub lfclk_source: LfclkSource,
        #[cfg(not(feature = "_nrf5340-net"))]
        /// DCDC configuration.
        pub dcdc: DcdcConfig,
        /// GPIOTE interrupt priority. Should be lower priority than softdevice if used.
        #[cfg(feature = "gpiote")]
        pub gpiote_interrupt_priority: crate::interrupt::Priority,
        /// Time driver interrupt priority. Should be lower priority than softdevice if used.
        #[cfg(feature = "_time-driver")]
        pub time_interrupt_priority: crate::interrupt::Priority,
        /// Enable or disable the debug port.
        pub debug: Debug,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                // There are hobby nrf52 boards out there without external XTALs...
                // Default everything to internal so it Just Works. User can enable external
                // xtals if they know they have them.
                hfclk_source: HfclkSource::Internal,
                lfclk_source: LfclkSource::InternalRC,
                #[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
                dcdc: DcdcConfig {
                    #[cfg(feature = "nrf52840")]
                    reg0: false,
                    reg1: false,
                },
                #[cfg(feature = "_nrf5340-app")]
                dcdc: DcdcConfig {
                    regh: false,
                    regmain: false,
                    regradio: false,
                },
                #[cfg(feature = "_nrf9160")]
                dcdc: DcdcConfig { regmain: false },
                #[cfg(feature = "gpiote")]
                gpiote_interrupt_priority: crate::interrupt::Priority::P0,
                #[cfg(feature = "_time-driver")]
                time_interrupt_priority: crate::interrupt::Priority::P0,

                // In NS mode, default to NotConfigured, assuming the S firmware will do it.
                #[cfg(feature = "_ns")]
                debug: Debug::NotConfigured,
                #[cfg(not(feature = "_ns"))]
                debug: Debug::Allowed,
            }
        }
    }
}

#[cfg(feature = "_nrf9160")]
#[allow(unused)]
mod consts {
    pub const UICR_APPROTECT: *mut u32 = 0x00FF8000 as *mut u32;
    pub const UICR_SECUREAPPROTECT: *mut u32 = 0x00FF802C as *mut u32;
    pub const APPROTECT_ENABLED: u32 = 0x0000_0000;
}

#[cfg(feature = "_nrf5340-app")]
#[allow(unused)]
mod consts {
    pub const UICR_APPROTECT: *mut u32 = 0x00FF8000 as *mut u32;
    pub const UICR_SECUREAPPROTECT: *mut u32 = 0x00FF801C as *mut u32;
    pub const UICR_NFCPINS: *mut u32 = 0x00FF8028 as *mut u32;
    pub const APPROTECT_ENABLED: u32 = 0x0000_0000;
    pub const APPROTECT_DISABLED: u32 = 0x50FA50FA;
}

#[cfg(feature = "_nrf5340-net")]
#[allow(unused)]
mod consts {
    pub const UICR_APPROTECT: *mut u32 = 0x01FF8000 as *mut u32;
    pub const APPROTECT_ENABLED: u32 = 0x0000_0000;
    pub const APPROTECT_DISABLED: u32 = 0x50FA50FA;
}

#[cfg(feature = "_nrf52")]
#[allow(unused)]
mod consts {
    pub const UICR_PSELRESET1: *mut u32 = 0x10001200 as *mut u32;
    pub const UICR_PSELRESET2: *mut u32 = 0x10001204 as *mut u32;
    pub const UICR_NFCPINS: *mut u32 = 0x1000120C as *mut u32;
    pub const UICR_APPROTECT: *mut u32 = 0x10001208 as *mut u32;
    pub const APPROTECT_ENABLED: u32 = 0x0000_0000;
    pub const APPROTECT_DISABLED: u32 = 0x0000_005a;
}

#[cfg(not(feature = "nrf51"))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum WriteResult {
    /// Word was written successfully, needs reset.
    Written,
    /// Word was already set to the value we wanted to write, nothing was done.
    Noop,
    /// Word is already set to something else, we couldn't write the desired value.
    Failed,
}

#[cfg(not(feature = "nrf51"))]
unsafe fn uicr_write(address: *mut u32, value: u32) -> WriteResult {
    uicr_write_masked(address, value, 0xFFFF_FFFF)
}

#[cfg(not(feature = "nrf51"))]
unsafe fn uicr_write_masked(address: *mut u32, value: u32, mask: u32) -> WriteResult {
    let curr_val = address.read_volatile();
    if curr_val & mask == value & mask {
        return WriteResult::Noop;
    }

    // We can only change `1` bits to `0` bits.
    if curr_val & value & mask != value & mask {
        return WriteResult::Failed;
    }

    let nvmc = &*pac::NVMC::ptr();
    nvmc.config.write(|w| w.wen().wen());
    while nvmc.ready.read().ready().is_busy() {}
    address.write_volatile(value | !mask);
    while nvmc.ready.read().ready().is_busy() {}
    nvmc.config.reset();
    while nvmc.ready.read().ready().is_busy() {}

    WriteResult::Written
}

/// Initialize the `embassy-nrf` HAL with the provided configuration.
///
/// This returns the peripheral singletons that can be used for creating drivers.
///
/// This should only be called once at startup, otherwise it panics.
pub fn init(config: config::Config) -> Peripherals {
    // Do this first, so that it panics if user is calling `init` a second time
    // before doing anything important.
    let peripherals = Peripherals::take();

    #[allow(unused_mut)]
    let mut needs_reset = false;

    // Setup debug protection.
    #[cfg(not(feature = "nrf51"))]
    match config.debug {
        config::Debug::Allowed => {
            #[cfg(feature = "_nrf52")]
            unsafe {
                let variant = (0x1000_0104 as *mut u32).read_volatile();
                // Get the letter for the build code (b'A' .. b'F')
                let build_code = (variant >> 8) as u8;

                if build_code >= chip::APPROTECT_MIN_BUILD_CODE {
                    // Chips with a certain chip type-specific build code or higher have an
                    // improved APPROTECT ("hardware and software controlled access port protection")
                    // which needs explicit action by the firmware to keep it unlocked
                    // See https://devzone.nordicsemi.com/nordic/nordic-blog/b/blog/posts/working-with-the-nrf52-series-improved-approtect

                    // UICR.APPROTECT = SwDisabled
                    let res = uicr_write(consts::UICR_APPROTECT, consts::APPROTECT_DISABLED);
                    needs_reset |= res == WriteResult::Written;
                    // APPROTECT.DISABLE = SwDisabled
                    (0x4000_0558 as *mut u32).write_volatile(consts::APPROTECT_DISABLED);
                } else {
                    // nothing to do on older chips, debug is allowed by default.
                }
            }

            #[cfg(feature = "_nrf5340")]
            unsafe {
                let p = &*pac::CTRLAP::ptr();

                let res = uicr_write(consts::UICR_APPROTECT, consts::APPROTECT_DISABLED);
                needs_reset |= res == WriteResult::Written;
                p.approtect.disable.write(|w| w.bits(consts::APPROTECT_DISABLED));

                #[cfg(feature = "_nrf5340-app")]
                {
                    let res = uicr_write(consts::UICR_SECUREAPPROTECT, consts::APPROTECT_DISABLED);
                    needs_reset |= res == WriteResult::Written;
                    p.secureapprotect.disable.write(|w| w.bits(consts::APPROTECT_DISABLED));
                }
            }

            // nothing to do on the nrf9160, debug is allowed by default.
        }
        config::Debug::Disallowed => unsafe {
            // UICR.APPROTECT = Enabled
            let res = uicr_write(consts::UICR_APPROTECT, consts::APPROTECT_ENABLED);
            needs_reset |= res == WriteResult::Written;
            #[cfg(any(feature = "_nrf5340-app", feature = "_nrf9160"))]
            {
                let res = uicr_write(consts::UICR_SECUREAPPROTECT, consts::APPROTECT_ENABLED);
                needs_reset |= res == WriteResult::Written;
            }
        },
        config::Debug::NotConfigured => {}
    }

    #[cfg(feature = "_nrf52")]
    unsafe {
        let value = if cfg!(feature = "reset-pin-as-gpio") {
            !0
        } else {
            chip::RESET_PIN
        };
        let res1 = uicr_write(consts::UICR_PSELRESET1, value);
        let res2 = uicr_write(consts::UICR_PSELRESET2, value);
        needs_reset |= res1 == WriteResult::Written || res2 == WriteResult::Written;
        if res1 == WriteResult::Failed || res2 == WriteResult::Failed {
            #[cfg(not(feature = "reset-pin-as-gpio"))]
            warn!(
                "You have requested enabling chip reset functionality on the reset pin, by not enabling the Cargo feature `reset-pin-as-gpio`.\n\
                However, UICR is already programmed to some other setting, and can't be changed without erasing it.\n\
                To fix this, erase UICR manually, for example using `probe-rs erase` or `nrfjprog --eraseuicr`."
            );
            #[cfg(feature = "reset-pin-as-gpio")]
            warn!(
                "You have requested using the reset pin as GPIO, by enabling the Cargo feature `reset-pin-as-gpio`.\n\
                However, UICR is already programmed to some other setting, and can't be changed without erasing it.\n\
                To fix this, erase UICR manually, for example using `probe-rs erase` or `nrfjprog --eraseuicr`."
            );
        }
    }

    #[cfg(any(feature = "_nrf52", feature = "_nrf5340-app"))]
    unsafe {
        let value = if cfg!(feature = "nfc-pins-as-gpio") { 0 } else { 1 };
        let res = uicr_write_masked(consts::UICR_NFCPINS, value, 1);
        needs_reset |= res == WriteResult::Written;
        if res == WriteResult::Failed {
            // with nfc-pins-as-gpio, this can never fail because we're writing all zero bits.
            #[cfg(not(feature = "nfc-pins-as-gpio"))]
            warn!(
                "You have requested to use P0.09 and P0.10 pins for NFC, by not enabling the Cargo feature `nfc-pins-as-gpio`.\n\
                However, UICR is already programmed to some other setting, and can't be changed without erasing it.\n\
                To fix this, erase UICR manually, for example using `probe-rs erase` or `nrfjprog --eraseuicr`."
            );
        }
    }

    if needs_reset {
        cortex_m::peripheral::SCB::sys_reset();
    }

    let r = unsafe { &*pac::CLOCK::ptr() };

    // Start HFCLK.
    match config.hfclk_source {
        config::HfclkSource::Internal => {}
        config::HfclkSource::ExternalXtal => {
            // Datasheet says this is likely to take 0.36ms
            r.events_hfclkstarted.write(|w| unsafe { w.bits(0) });
            r.tasks_hfclkstart.write(|w| unsafe { w.bits(1) });
            while r.events_hfclkstarted.read().bits() == 0 {}
        }
    }

    // Configure LFCLK.
    #[cfg(not(any(feature = "nrf51", feature = "_nrf5340", feature = "_nrf9160")))]
    match config.lfclk_source {
        config::LfclkSource::InternalRC => r.lfclksrc.write(|w| w.src().rc()),
        config::LfclkSource::Synthesized => r.lfclksrc.write(|w| w.src().synth()),

        config::LfclkSource::ExternalXtal => r.lfclksrc.write(|w| w.src().xtal()),

        config::LfclkSource::ExternalLowSwing => r.lfclksrc.write(|w| {
            w.src().xtal();
            w.external().enabled();
            w.bypass().disabled();
            w
        }),
        config::LfclkSource::ExternalFullSwing => r.lfclksrc.write(|w| {
            w.src().xtal();
            w.external().enabled();
            w.bypass().enabled();
            w
        }),
    }
    #[cfg(feature = "_nrf9160")]
    match config.lfclk_source {
        config::LfclkSource::InternalRC => r.lfclksrc.write(|w| w.src().lfrc()),
        config::LfclkSource::ExternalXtal => r.lfclksrc.write(|w| w.src().lfxo()),
    }

    // Start LFCLK.
    // Datasheet says this could take 100us from synth source
    // 600us from rc source, 0.25s from an external source.
    r.events_lfclkstarted.write(|w| unsafe { w.bits(0) });
    r.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });
    while r.events_lfclkstarted.read().bits() == 0 {}

    #[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
    {
        // Setup DCDCs.
        let pwr = unsafe { &*pac::POWER::ptr() };
        #[cfg(feature = "nrf52840")]
        if config.dcdc.reg0 {
            pwr.dcdcen0.write(|w| w.dcdcen().set_bit());
        }
        if config.dcdc.reg1 {
            pwr.dcdcen.write(|w| w.dcdcen().set_bit());
        }
    }
    #[cfg(feature = "_nrf9160")]
    {
        // Setup DCDC.
        let reg = unsafe { &*pac::REGULATORS::ptr() };
        if config.dcdc.regmain {
            reg.dcdcen.write(|w| w.dcdcen().set_bit());
        }
    }
    #[cfg(feature = "_nrf5340-app")]
    {
        // Setup DCDC.
        let reg = unsafe { &*pac::REGULATORS::ptr() };
        if config.dcdc.regh {
            reg.vregh.dcdcen.write(|w| w.dcdcen().set_bit());
        }
        if config.dcdc.regmain {
            reg.vregmain.dcdcen.write(|w| w.dcdcen().set_bit());
        }
        if config.dcdc.regradio {
            reg.vregradio.dcdcen.write(|w| w.dcdcen().set_bit());
        }
    }

    // Init GPIOTE
    #[cfg(feature = "gpiote")]
    gpiote::init(config.gpiote_interrupt_priority);

    // init RTC time driver
    #[cfg(feature = "_time-driver")]
    time_driver::init(config.time_interrupt_priority);

    // Disable UARTE (enabled by default for some reason)
    #[cfg(feature = "_nrf9160")]
    unsafe {
        (*pac::UARTE0::ptr()).enable.write(|w| w.enable().disabled());
        (*pac::UARTE1::ptr()).enable.write(|w| w.enable().disabled());
    }

    peripherals
}
