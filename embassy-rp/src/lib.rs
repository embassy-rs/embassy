#![no_std]
#![cfg_attr(feature = "nightly", feature(async_fn_in_trait, impl_trait_projections))]
#![cfg_attr(feature = "nightly", allow(incomplete_features))]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

#[cfg(feature = "critical-section-impl")]
mod critical_section_impl;

mod intrinsics;

pub mod adc;
pub mod clocks;
pub mod dma;
pub mod flash;
mod float;
pub mod gpio;
pub mod i2c;
pub mod interrupt;
pub mod multicore;
pub mod pwm;
mod reset;
pub mod rom_data;
pub mod rtc;
pub mod spi;
#[cfg(feature = "time-driver")]
pub mod timer;
pub mod uart;
#[cfg(feature = "nightly")]
pub mod usb;
pub mod watchdog;

// PIO
// TODO: move `pio_instr_util` and `relocate` to inside `pio`
pub mod pio;
pub mod pio_instr_util;
pub mod relocate;

// Reexports
pub use embassy_cortex_m::executor;
pub use embassy_cortex_m::interrupt::_export::interrupt;
pub use embassy_hal_common::{into_ref, Peripheral, PeripheralRef};
#[cfg(feature = "unstable-pac")]
pub use rp_pac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use rp_pac as pac;

embassy_hal_common::peripherals! {
    PIN_0,
    PIN_1,
    PIN_2,
    PIN_3,
    PIN_4,
    PIN_5,
    PIN_6,
    PIN_7,
    PIN_8,
    PIN_9,
    PIN_10,
    PIN_11,
    PIN_12,
    PIN_13,
    PIN_14,
    PIN_15,
    PIN_16,
    PIN_17,
    PIN_18,
    PIN_19,
    PIN_20,
    PIN_21,
    PIN_22,
    PIN_23,
    PIN_24,
    PIN_25,
    PIN_26,
    PIN_27,
    PIN_28,
    PIN_29,
    PIN_QSPI_SCLK,
    PIN_QSPI_SS,
    PIN_QSPI_SD0,
    PIN_QSPI_SD1,
    PIN_QSPI_SD2,
    PIN_QSPI_SD3,

    UART0,
    UART1,

    SPI0,
    SPI1,

    I2C0,
    I2C1,

    DMA_CH0,
    DMA_CH1,
    DMA_CH2,
    DMA_CH3,
    DMA_CH4,
    DMA_CH5,
    DMA_CH6,
    DMA_CH7,
    DMA_CH8,
    DMA_CH9,
    DMA_CH10,
    DMA_CH11,

    PWM_CH0,
    PWM_CH1,
    PWM_CH2,
    PWM_CH3,
    PWM_CH4,
    PWM_CH5,
    PWM_CH6,
    PWM_CH7,

    USB,

    RTC,

    FLASH,

    ADC,

    CORE1,

    PIO0,
    PIO1,

    WATCHDOG,
}

#[link_section = ".boot2"]
#[used]
static BOOT2: [u8; 256] = *include_bytes!("boot2.bin");

pub mod config {
    use crate::clocks::ClockConfig;

    #[non_exhaustive]
    pub struct Config {
        pub clocks: ClockConfig,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                clocks: ClockConfig::crystal(12_000_000),
            }
        }
    }

    impl Config {
        pub fn new(clocks: ClockConfig) -> Self {
            Self { clocks }
        }
    }
}

pub fn init(config: config::Config) -> Peripherals {
    // Do this first, so that it panics if user is calling `init` a second time
    // before doing anything important.
    let peripherals = Peripherals::take();

    unsafe {
        clocks::init(config.clocks);
        #[cfg(feature = "time-driver")]
        timer::init();
        dma::init();
        pio::init();
        gpio::init();
    }

    peripherals
}

/// Extension trait for PAC regs, adding atomic xor/bitset/bitclear writes.
trait RegExt<T: Copy> {
    unsafe fn write_xor<R>(&self, f: impl FnOnce(&mut T) -> R) -> R;
    unsafe fn write_set<R>(&self, f: impl FnOnce(&mut T) -> R) -> R;
    unsafe fn write_clear<R>(&self, f: impl FnOnce(&mut T) -> R) -> R;
}

impl<T: Default + Copy, A: pac::common::Write> RegExt<T> for pac::common::Reg<T, A> {
    unsafe fn write_xor<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut val = Default::default();
        let res = f(&mut val);
        let ptr = (self.ptr() as *mut u8).add(0x1000) as *mut T;
        ptr.write_volatile(val);
        res
    }

    unsafe fn write_set<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut val = Default::default();
        let res = f(&mut val);
        let ptr = (self.ptr() as *mut u8).add(0x2000) as *mut T;
        ptr.write_volatile(val);
        res
    }

    unsafe fn write_clear<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut val = Default::default();
        let res = f(&mut val);
        let ptr = (self.ptr() as *mut u8).add(0x3000) as *mut T;
        ptr.write_volatile(val);
        res
    }
}
