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
    feature = "_nrf51",
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
    feature = "nrf54l15-app-s",
    feature = "nrf54l15-app-ns",
    feature = "nrf9160-s",
    feature = "nrf9160-ns",
    feature = "nrf9120-s",
    feature = "nrf9120-ns",
    feature = "nrf9151-s",
    feature = "nrf9151-ns",
    feature = "nrf9161-s",
    feature = "nrf9161-ns",
)))]
compile_error!(
    "No chip feature activated. You must activate exactly one of the following features:
    nrf51,
    nrf52805,
    nrf52810,
    nrf52811,
    nrf52820,
    nrf52832,
    nrf52833,
    nrf52840,
    nrf5340-app-s,
    nrf5340-app-ns,
    nrf5340-net,
    nrf54l15-app-s,
    nrf54l15-app-ns,
    nrf9160-s,
    nrf9160-ns,
    nrf9120-s,
    nrf9120-ns,
    nrf9151-s,
    nrf9151-ns,
    nrf9161-s,
    nrf9161-ns,
    "
);

#[cfg(all(feature = "reset-pin-as-gpio", not(feature = "_nrf52")))]
compile_error!("feature `reset-pin-as-gpio` is only valid for nRF52 series chips.");

#[cfg(all(feature = "nfc-pins-as-gpio", not(any(feature = "_nrf52", feature = "_nrf5340-app"))))]
compile_error!("feature `nfc-pins-as-gpio` is only valid for nRF52, or nRF53's application core.");

#[cfg(all(feature = "lfxo-pins-as-gpio", not(feature = "_nrf5340")))]
compile_error!("feature `lfxo-pins-as-gpio` is only valid for nRF53 series chips.");

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;
pub(crate) mod util;

#[cfg(feature = "_time-driver")]
mod time_driver;

#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(not(feature = "_nrf51"))]
pub mod buffered_uarte;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(not(feature = "_nrf51"))]
pub mod egu;
pub mod gpio;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(feature = "gpiote")]
pub mod gpiote;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840"))]
pub mod i2s;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(any(
    feature = "nrf52832",
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "_nrf5340-app"
))]
pub mod nfct;
#[cfg(not(feature = "_nrf54l"))] // TODO
pub mod nvmc;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(any(
    feature = "nrf52810",
    feature = "nrf52811",
    feature = "nrf52832",
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "_nrf5340-app",
    feature = "_nrf91",
))]
pub mod pdm;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(any(feature = "nrf52840", feature = "nrf9160-s", feature = "nrf9160-ns"))]
pub mod power;
#[cfg(not(feature = "_nrf54l"))] // TODO
pub mod ppi;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(not(any(
    feature = "_nrf51",
    feature = "nrf52805",
    feature = "nrf52820",
    feature = "_nrf5340-net"
)))]
pub mod pwm;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(not(any(feature = "_nrf51", feature = "_nrf91", feature = "_nrf5340-net")))]
pub mod qdec;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(any(feature = "nrf52840", feature = "_nrf5340-app"))]
pub mod qspi;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(not(any(feature = "_nrf91", feature = "_nrf5340-app")))]
pub mod radio;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(feature = "_nrf5340")]
pub mod reset;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(not(any(feature = "_nrf5340-app", feature = "_nrf91")))]
pub mod rng;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(not(any(feature = "_nrf51", feature = "nrf52820", feature = "_nrf5340-net")))]
pub mod saadc;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(not(feature = "_nrf51"))]
pub mod spim;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(not(feature = "_nrf51"))]
pub mod spis;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(not(any(feature = "_nrf5340", feature = "_nrf91")))]
pub mod temp;
#[cfg(not(feature = "_nrf54l"))] // TODO
pub mod timer;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(not(feature = "_nrf51"))]
pub mod twim;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(not(feature = "_nrf51"))]
pub mod twis;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(not(feature = "_nrf51"))]
pub mod uarte;
#[cfg(not(feature = "_nrf54l"))] // TODO
#[cfg(any(
    feature = "_nrf5340-app",
    feature = "nrf52820",
    feature = "nrf52833",
    feature = "nrf52840"
))]
pub mod usb;
#[cfg(not(feature = "_nrf54l"))] // TODO
pub mod wdt;

// This mod MUST go last, so that it sees all the `impl_foo!` macros
#[cfg_attr(feature = "_nrf51", path = "chips/nrf51.rs")]
#[cfg_attr(feature = "nrf52805", path = "chips/nrf52805.rs")]
#[cfg_attr(feature = "nrf52810", path = "chips/nrf52810.rs")]
#[cfg_attr(feature = "nrf52811", path = "chips/nrf52811.rs")]
#[cfg_attr(feature = "nrf52820", path = "chips/nrf52820.rs")]
#[cfg_attr(feature = "nrf52832", path = "chips/nrf52832.rs")]
#[cfg_attr(feature = "nrf52833", path = "chips/nrf52833.rs")]
#[cfg_attr(feature = "nrf52840", path = "chips/nrf52840.rs")]
#[cfg_attr(feature = "_nrf5340-app", path = "chips/nrf5340_app.rs")]
#[cfg_attr(feature = "_nrf5340-net", path = "chips/nrf5340_net.rs")]
#[cfg_attr(feature = "_nrf54l15-app", path = "chips/nrf54l15_app.rs")]
#[cfg_attr(feature = "_nrf9160", path = "chips/nrf9160.rs")]
#[cfg_attr(feature = "_nrf9120", path = "chips/nrf9120.rs")]
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
///     TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
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

#[cfg(feature = "unstable-pac")]
pub use chip::pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use chip::pac;
pub use chip::{peripherals, Peripherals, EASY_DMA_SIZE};
pub use embassy_hal_internal::{Peri, PeripheralType};

pub use crate::chip::interrupt;
#[cfg(feature = "rt")]
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
        #[cfg(not(feature = "_nrf91"))]
        Synthesized,
        /// External source from xtal.
        #[cfg(not(feature = "lfxo-pins-as-gpio"))]
        ExternalXtal,
        /// External source from xtal with low swing applied.
        #[cfg(not(any(feature = "lfxo-pins-as-gpio", feature = "_nrf91", feature = "_nrf54l")))]
        ExternalLowSwing,
        /// External source from xtal with full swing applied.
        #[cfg(not(any(feature = "lfxo-pins-as-gpio", feature = "_nrf91", feature = "_nrf54l")))]
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
    #[cfg(not(any(feature = "_nrf5340", feature = "_nrf91")))]
    pub struct DcdcConfig {
        /// Config for the first stage DCDC (VDDH -> VDD), if disabled LDO will be used.
        #[cfg(feature = "nrf52840")]
        pub reg0: bool,
        /// Configure the voltage of the first stage DCDC. It is stored in non-volatile memory (UICR.REGOUT0 register); pass None to not touch it.
        #[cfg(any(feature = "nrf52840", feature = "nrf52833"))]
        pub reg0_voltage: Option<Reg0Voltage>,
        /// Config for the second stage DCDC (VDD -> DEC4), if disabled LDO will be used.
        pub reg1: bool,
    }

    ///  Output voltage setting for REG0 regulator stage.
    #[cfg(any(feature = "nrf52840", feature = "nrf52833"))]
    pub enum Reg0Voltage {
        /// 1.8 V
        _1V8 = 0,
        /// 2.1 V
        _2V1 = 1,
        /// 2.4 V
        _2V4 = 2,
        /// 2.7 V
        _2V7 = 3,
        /// 3.0 V
        _3V0 = 4,
        /// 3.3 V
        _3v3 = 5,
        //ERASED = 7, means 1.8V
    }

    /// Settings for enabling the built in DCDC converters.
    #[cfg(feature = "_nrf5340-app")]
    pub struct DcdcConfig {
        /// Config for the high voltage stage, if disabled LDO will be used.
        pub regh: bool,
        /// Configure the voltage of the high voltage stage. It is stored in non-volatile memory (UICR.VREGHVOUT register); pass None to not touch it.
        #[cfg(feature = "nrf5340-app-s")]
        pub regh_voltage: Option<ReghVoltage>,
        /// Config for the main rail, if disabled LDO will be used.
        pub regmain: bool,
        /// Config for the radio rail, if disabled LDO will be used.
        pub regradio: bool,
    }

    ///  Output voltage setting for VREGH regulator stage.
    #[cfg(feature = "nrf5340-app-s")]
    pub enum ReghVoltage {
        /// 1.8 V
        _1V8 = 0,
        /// 2.1 V
        _2V1 = 1,
        /// 2.4 V
        _2V4 = 2,
        /// 2.7 V
        _2V7 = 3,
        /// 3.0 V
        _3V0 = 4,
        /// 3.3 V
        _3v3 = 5,
        //ERASED = 7, means 1.8V
    }

    /// Settings for enabling the built in DCDC converter.
    #[cfg(feature = "_nrf91")]
    pub struct DcdcConfig {
        /// Config for the main rail, if disabled LDO will be used.
        pub regmain: bool,
    }

    /// Settings for the internal capacitors.
    #[cfg(feature = "nrf5340-app-s")]
    pub struct InternalCapacitors {
        /// Config for the internal capacitors on pins XC1 and XC2.
        pub hfxo: Option<HfxoCapacitance>,
        /// Config for the internal capacitors between pins XL1 and XL2.
        pub lfxo: Option<LfxoCapacitance>,
    }

    /// Internal capacitance value for the HFXO.
    #[cfg(feature = "nrf5340-app-s")]
    #[derive(Copy, Clone)]
    pub enum HfxoCapacitance {
        /// 7.0 pF
        _7_0pF,
        /// 7.5 pF
        _7_5pF,
        /// 8.0 pF
        _8_0pF,
        /// 8.5 pF
        _8_5pF,
        /// 9.0 pF
        _9_0pF,
        /// 9.5 pF
        _9_5pF,
        /// 10.0 pF
        _10_0pF,
        /// 10.5 pF
        _10_5pF,
        /// 11.0 pF
        _11_0pF,
        /// 11.5 pF
        _11_5pF,
        /// 12.0 pF
        _12_0pF,
        /// 12.5 pF
        _12_5pF,
        /// 13.0 pF
        _13_0pF,
        /// 13.5 pF
        _13_5pF,
        /// 14.0 pF
        _14_0pF,
        /// 14.5 pF
        _14_5pF,
        /// 15.0 pF
        _15_0pF,
        /// 15.5 pF
        _15_5pF,
        /// 16.0 pF
        _16_0pF,
        /// 16.5 pF
        _16_5pF,
        /// 17.0 pF
        _17_0pF,
        /// 17.5 pF
        _17_5pF,
        /// 18.0 pF
        _18_0pF,
        /// 18.5 pF
        _18_5pF,
        /// 19.0 pF
        _19_0pF,
        /// 19.5 pF
        _19_5pF,
        /// 20.0 pF
        _20_0pF,
    }

    #[cfg(feature = "nrf5340-app-s")]
    impl HfxoCapacitance {
        /// The capacitance value times two.
        pub(crate) const fn value2(self) -> i32 {
            match self {
                HfxoCapacitance::_7_0pF => 14,
                HfxoCapacitance::_7_5pF => 15,
                HfxoCapacitance::_8_0pF => 16,
                HfxoCapacitance::_8_5pF => 17,
                HfxoCapacitance::_9_0pF => 18,
                HfxoCapacitance::_9_5pF => 19,
                HfxoCapacitance::_10_0pF => 20,
                HfxoCapacitance::_10_5pF => 21,
                HfxoCapacitance::_11_0pF => 22,
                HfxoCapacitance::_11_5pF => 23,
                HfxoCapacitance::_12_0pF => 24,
                HfxoCapacitance::_12_5pF => 25,
                HfxoCapacitance::_13_0pF => 26,
                HfxoCapacitance::_13_5pF => 27,
                HfxoCapacitance::_14_0pF => 28,
                HfxoCapacitance::_14_5pF => 29,
                HfxoCapacitance::_15_0pF => 30,
                HfxoCapacitance::_15_5pF => 31,
                HfxoCapacitance::_16_0pF => 32,
                HfxoCapacitance::_16_5pF => 33,
                HfxoCapacitance::_17_0pF => 34,
                HfxoCapacitance::_17_5pF => 35,
                HfxoCapacitance::_18_0pF => 36,
                HfxoCapacitance::_18_5pF => 37,
                HfxoCapacitance::_19_0pF => 38,
                HfxoCapacitance::_19_5pF => 39,
                HfxoCapacitance::_20_0pF => 40,
            }
        }
    }

    /// Internal capacitance value for the LFXO.
    #[cfg(feature = "nrf5340-app-s")]
    pub enum LfxoCapacitance {
        /// 6 pF
        _6pF = 1,
        /// 7 pF
        _7pF = 2,
        /// 9 pF
        _9pF = 3,
    }

    #[cfg(feature = "nrf5340-app-s")]
    impl From<LfxoCapacitance> for super::pac::oscillators::vals::Intcap {
        fn from(t: LfxoCapacitance) -> Self {
            match t {
                LfxoCapacitance::_6pF => Self::C6PF,
                LfxoCapacitance::_7pF => Self::C7PF,
                LfxoCapacitance::_9pF => Self::C9PF,
            }
        }
    }

    /// Configuration for peripherals. Default configuration should work on any nRF chip.
    #[non_exhaustive]
    pub struct Config {
        /// High frequency clock source.
        pub hfclk_source: HfclkSource,
        /// Low frequency clock source.
        pub lfclk_source: LfclkSource,
        #[cfg(feature = "nrf5340-app-s")]
        /// Internal capacitor configuration, for use with the `ExternalXtal` clock source. See
        /// nrf5340-PS §4.12.
        pub internal_capacitors: InternalCapacitors,
        #[cfg(not(any(feature = "_nrf5340-net", feature = "_nrf54l")))]
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
                #[cfg(feature = "nrf5340-app-s")]
                internal_capacitors: InternalCapacitors { hfxo: None, lfxo: None },
                #[cfg(not(any(feature = "_nrf5340", feature = "_nrf91", feature = "_nrf54l")))]
                dcdc: DcdcConfig {
                    #[cfg(feature = "nrf52840")]
                    reg0: false,
                    #[cfg(any(feature = "nrf52840", feature = "nrf52833"))]
                    reg0_voltage: None,
                    reg1: false,
                },
                #[cfg(feature = "_nrf5340-app")]
                dcdc: DcdcConfig {
                    regh: false,
                    #[cfg(feature = "nrf5340-app-s")]
                    regh_voltage: None,
                    regmain: false,
                    regradio: false,
                },
                #[cfg(feature = "_nrf91")]
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

#[cfg(feature = "_nrf91")]
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
    pub const UICR_VREGHVOUT: *mut u32 = 0x00FF8010 as *mut u32;
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
    pub const UICR_REGOUT0: *mut u32 = 0x10001304 as *mut u32;
    pub const APPROTECT_ENABLED: u32 = 0x0000_0000;
    pub const APPROTECT_DISABLED: u32 = 0x0000_005a;
}

#[cfg(not(any(feature = "_nrf51", feature = "_nrf54l")))]
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

#[cfg(not(any(feature = "_nrf51", feature = "_nrf54l")))]
unsafe fn uicr_write(address: *mut u32, value: u32) -> WriteResult {
    uicr_write_masked(address, value, 0xFFFF_FFFF)
}

#[cfg(not(any(feature = "_nrf51", feature = "_nrf54l")))]
unsafe fn uicr_write_masked(address: *mut u32, value: u32, mask: u32) -> WriteResult {
    let curr_val = address.read_volatile();
    if curr_val & mask == value & mask {
        return WriteResult::Noop;
    }

    // We can only change `1` bits to `0` bits.
    if curr_val & value & mask != value & mask {
        return WriteResult::Failed;
    }

    let nvmc = pac::NVMC;
    nvmc.config().write(|w| w.set_wen(pac::nvmc::vals::Wen::WEN));
    while !nvmc.ready().read().ready() {}
    address.write_volatile(value | !mask);
    while !nvmc.ready().read().ready() {}
    nvmc.config().write(|_| {});
    while !nvmc.ready().read().ready() {}

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
    #[cfg(not(feature = "_nrf51"))]
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
                let p = pac::CTRLAP;

                let res = uicr_write(consts::UICR_APPROTECT, consts::APPROTECT_DISABLED);
                needs_reset |= res == WriteResult::Written;
                p.approtect().disable().write_value(consts::APPROTECT_DISABLED);

                #[cfg(feature = "_nrf5340-app")]
                {
                    let res = uicr_write(consts::UICR_SECUREAPPROTECT, consts::APPROTECT_DISABLED);
                    needs_reset |= res == WriteResult::Written;
                    p.secureapprotect().disable().write_value(consts::APPROTECT_DISABLED);
                }
            }

            // TAMPC is only accessible for secure code
            #[cfg(all(feature = "_nrf54l", feature = "_s"))]
            {
                use crate::pac::tampc::vals;

                // UICR cannot be written here, because it can only be written once after an erase all.
                // The erase all value means that debug access is allowed if permitted by the firmware.

                // Write to TAMPC to permit debug access
                //
                // See https://docs.nordicsemi.com/bundle/ps_nrf54L15/page/debug.html#ariaid-title6

                let p = pac::TAMPC;

                // Unlock dbgen
                p.protect().domain(0).dbgen().ctrl().write(|w| {
                    w.set_key(vals::DomainDbgenCtrlKey::KEY);
                    w.set_writeprotection(vals::DomainDbgenCtrlWriteprotection::CLEAR);
                });
                p.protect().domain(0).dbgen().ctrl().write(|w| {
                    w.set_key(vals::DomainDbgenCtrlKey::KEY);
                    w.set_value(vals::DomainDbgenCtrlValue::HIGH);
                });

                // Unlock niden
                p.protect().domain(0).niden().ctrl().write(|w| {
                    w.set_key(vals::NidenCtrlKey::KEY);
                    w.set_writeprotection(vals::NidenCtrlWriteprotection::CLEAR);
                });
                p.protect().domain(0).niden().ctrl().write(|w| {
                    w.set_key(vals::NidenCtrlKey::KEY);
                    w.set_value(vals::NidenCtrlValue::HIGH);
                });

                p.protect().domain(0).spiden().ctrl().write(|w| {
                    w.set_key(vals::SpidenCtrlKey::KEY);
                    w.set_writeprotection(vals::SpidenCtrlWriteprotection::CLEAR);
                });
                p.protect().domain(0).spiden().ctrl().write(|w| {
                    w.set_key(vals::SpidenCtrlKey::KEY);
                    w.set_value(vals::SpidenCtrlValue::HIGH);
                });

                p.protect().domain(0).spniden().ctrl().write(|w| {
                    w.set_key(vals::SpnidenCtrlKey::KEY);
                    w.set_writeprotection(vals::SpnidenCtrlWriteprotection::CLEAR);
                });
                p.protect().domain(0).spniden().ctrl().write(|w| {
                    w.set_key(vals::SpnidenCtrlKey::KEY);
                    w.set_value(vals::SpnidenCtrlValue::HIGH);
                });
            }

            // nothing to do on the nrf9160, debug is allowed by default.
        }
        config::Debug::Disallowed => {
            // TODO: Handle nRF54L
            //       By default, debug access is not allowed if the firmware doesn't allow it.
            //       Code could be added here to disable debug access in UICR as well.
            #[cfg(not(feature = "_nrf54l"))]
            unsafe {
                // UICR.APPROTECT = Enabled
                let res = uicr_write(consts::UICR_APPROTECT, consts::APPROTECT_ENABLED);
                needs_reset |= res == WriteResult::Written;
                #[cfg(any(feature = "_nrf5340-app", feature = "_nrf91"))]
                {
                    let res = uicr_write(consts::UICR_SECUREAPPROTECT, consts::APPROTECT_ENABLED);
                    needs_reset |= res == WriteResult::Written;
                }
            }
        }
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

    #[cfg(any(feature = "nrf52840", feature = "nrf52833"))]
    unsafe {
        if let Some(value) = config.dcdc.reg0_voltage {
            let value = value as u32;
            let res = uicr_write_masked(consts::UICR_REGOUT0, value, 0b00000000_00000000_00000000_00000111);
            needs_reset |= res == WriteResult::Written;
            if res == WriteResult::Failed {
                warn!(
                    "Failed to set regulator voltage, as UICR is already programmed to some other setting, and can't be changed without erasing it.\n\
                    To fix this, erase UICR manually, for example using `probe-rs erase` or `nrfjprog --eraseuicr`."
                );
            }
        }
    }

    #[cfg(feature = "nrf5340-app-s")]
    unsafe {
        if let Some(value) = config.dcdc.regh_voltage {
            let value = value as u32;
            let res = uicr_write_masked(consts::UICR_VREGHVOUT, value, 0b00000000_00000000_00000000_00000111);
            needs_reset |= res == WriteResult::Written;
            if res == WriteResult::Failed {
                warn!(
                    "Failed to set regulator voltage, as UICR is already programmed to some other setting, and can't be changed without erasing it.\n\
                    To fix this, erase UICR manually, for example using `probe-rs erase` or `nrfjprog --eraseuicr`."
                );
            }
        }
    }

    if needs_reset {
        cortex_m::peripheral::SCB::sys_reset();
    }

    // Configure internal capacitors
    #[cfg(feature = "nrf5340-app-s")]
    {
        if let Some(cap) = config.internal_capacitors.hfxo {
            let mut slope = pac::FICR.xosc32mtrim().read().slope() as i32;
            let offset = pac::FICR.xosc32mtrim().read().offset() as i32;
            // slope is a signed 5-bit integer
            if slope >= 16 {
                slope -= 32;
            }
            let capvalue = (((slope + 56) * (cap.value2() - 14)) + ((offset - 8) << 4) + 32) >> 6;
            pac::OSCILLATORS.xosc32mcaps().write(|w| {
                w.set_capvalue(capvalue as u8);
                w.set_enable(true);
            });
        }
        if let Some(cap) = config.internal_capacitors.lfxo {
            pac::OSCILLATORS.xosc32ki().intcap().write(|w| w.set_intcap(cap.into()));
        }
    }

    let r = pac::CLOCK;

    // Start HFCLK.
    match config.hfclk_source {
        config::HfclkSource::Internal => {}
        config::HfclkSource::ExternalXtal => {
            #[cfg(feature = "_nrf54l")]
            {
                r.events_xostarted().write_value(0);
                r.tasks_xostart().write_value(1);
                while r.events_xostarted().read() == 0 {}
            }

            #[cfg(not(feature = "_nrf54l"))]
            {
                // Datasheet says this is likely to take 0.36ms
                r.events_hfclkstarted().write_value(0);
                r.tasks_hfclkstart().write_value(1);
                while r.events_hfclkstarted().read() == 0 {}
            }
        }
    }

    // Workaround for anomaly 140
    #[cfg(feature = "nrf5340-app-s")]
    if unsafe { (0x50032420 as *mut u32).read_volatile() } & 0x80000000 != 0 {
        r.events_lfclkstarted().write_value(0);
        r.lfclksrc()
            .write(|w| w.set_src(nrf_pac::clock::vals::Lfclksrc::LFSYNT));
        r.tasks_lfclkstart().write_value(1);
        while r.events_lfclkstarted().read() == 0 {}
        r.events_lfclkstarted().write_value(0);
        r.tasks_lfclkstop().write_value(1);
        r.lfclksrc().write(|w| w.set_src(nrf_pac::clock::vals::Lfclksrc::LFRC));
    }

    // Configure LFCLK.
    #[cfg(not(any(feature = "_nrf51", feature = "_nrf5340", feature = "_nrf91", feature = "_nrf54l")))]
    match config.lfclk_source {
        config::LfclkSource::InternalRC => r.lfclksrc().write(|w| w.set_src(pac::clock::vals::Lfclksrc::RC)),
        config::LfclkSource::Synthesized => r.lfclksrc().write(|w| w.set_src(pac::clock::vals::Lfclksrc::SYNTH)),
        config::LfclkSource::ExternalXtal => r.lfclksrc().write(|w| w.set_src(pac::clock::vals::Lfclksrc::XTAL)),
        config::LfclkSource::ExternalLowSwing => r.lfclksrc().write(|w| {
            w.set_src(pac::clock::vals::Lfclksrc::XTAL);
            w.set_external(true);
            w.set_bypass(false);
        }),
        config::LfclkSource::ExternalFullSwing => r.lfclksrc().write(|w| {
            w.set_src(pac::clock::vals::Lfclksrc::XTAL);
            w.set_external(true);
            w.set_bypass(true);
        }),
    }
    #[cfg(feature = "_nrf5340")]
    {
        #[allow(unused_mut)]
        let mut lfxo = false;
        match config.lfclk_source {
            config::LfclkSource::InternalRC => r.lfclksrc().write(|w| w.set_src(pac::clock::vals::Lfclksrc::LFRC)),
            config::LfclkSource::Synthesized => r.lfclksrc().write(|w| w.set_src(pac::clock::vals::Lfclksrc::LFSYNT)),
            #[cfg(not(feature = "lfxo-pins-as-gpio"))]
            config::LfclkSource::ExternalXtal => lfxo = true,
            #[cfg(not(feature = "lfxo-pins-as-gpio"))]
            config::LfclkSource::ExternalLowSwing => lfxo = true,
            #[cfg(not(feature = "lfxo-pins-as-gpio"))]
            config::LfclkSource::ExternalFullSwing => {
                #[cfg(feature = "_nrf5340-app")]
                pac::OSCILLATORS.xosc32ki().bypass().write(|w| w.set_bypass(true));
                lfxo = true;
            }
        }
        if lfxo {
            if cfg!(feature = "_s") {
                // MCUSEL is only accessible from secure code.
                let p0 = pac::P0;
                p0.pin_cnf(0)
                    .write(|w| w.set_mcusel(pac::gpio::vals::Mcusel::PERIPHERAL));
                p0.pin_cnf(1)
                    .write(|w| w.set_mcusel(pac::gpio::vals::Mcusel::PERIPHERAL));
            }
            r.lfclksrc().write(|w| w.set_src(pac::clock::vals::Lfclksrc::LFXO));
        }
    }
    #[cfg(feature = "_nrf91")]
    match config.lfclk_source {
        config::LfclkSource::InternalRC => r.lfclksrc().write(|w| w.set_src(pac::clock::vals::Lfclksrc::LFRC)),
        config::LfclkSource::ExternalXtal => r.lfclksrc().write(|w| w.set_src(pac::clock::vals::Lfclksrc::LFXO)),
    }
    #[cfg(feature = "_nrf54l")]
    match config.lfclk_source {
        config::LfclkSource::InternalRC => r.lfclk().src().write(|w| w.set_src(pac::clock::vals::Lfclksrc::LFRC)),
        config::LfclkSource::Synthesized => r.lfclk().src().write(|w| w.set_src(pac::clock::vals::Lfclksrc::LFSYNT)),
        config::LfclkSource::ExternalXtal => r.lfclk().src().write(|w| w.set_src(pac::clock::vals::Lfclksrc::LFXO)),
    }

    // Start LFCLK.
    // Datasheet says this could take 100us from synth source
    // 600us from rc source, 0.25s from an external source.
    r.events_lfclkstarted().write_value(0);
    r.tasks_lfclkstart().write_value(1);
    while r.events_lfclkstarted().read() == 0 {}

    #[cfg(not(any(feature = "_nrf5340", feature = "_nrf91", feature = "_nrf54l")))]
    {
        // Setup DCDCs.
        #[cfg(feature = "nrf52840")]
        if config.dcdc.reg0 {
            pac::POWER.dcdcen0().write(|w| w.set_dcdcen(true));
        }
        if config.dcdc.reg1 {
            pac::POWER.dcdcen().write(|w| w.set_dcdcen(true));
        }
    }
    #[cfg(feature = "_nrf91")]
    {
        // Setup DCDC.
        if config.dcdc.regmain {
            pac::REGULATORS.dcdcen().write(|w| w.set_dcdcen(true));
        }
    }
    #[cfg(feature = "_nrf5340-app")]
    {
        // Setup DCDC.
        let reg = pac::REGULATORS;
        if config.dcdc.regh {
            reg.vregh().dcdcen().write(|w| w.set_dcdcen(true));
        }
        if config.dcdc.regmain {
            reg.vregmain().dcdcen().write(|w| w.set_dcdcen(true));
        }
        if config.dcdc.regradio {
            reg.vregradio().dcdcen().write(|w| w.set_dcdcen(true));
        }
    }

    // Init GPIOTE
    #[cfg(not(feature = "_nrf54l"))] // TODO
    #[cfg(feature = "gpiote")]
    gpiote::init(config.gpiote_interrupt_priority);

    // init RTC time driver
    #[cfg(feature = "_time-driver")]
    time_driver::init(config.time_interrupt_priority);

    // Disable UARTE (enabled by default for some reason)
    #[cfg(feature = "_nrf91")]
    {
        use pac::uarte::vals::Enable;
        pac::UARTE0.enable().write(|w| w.set_enable(Enable::DISABLED));
        pac::UARTE1.enable().write(|w| w.set_enable(Enable::DISABLED));
    }

    peripherals
}
