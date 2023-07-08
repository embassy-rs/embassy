#![no_std]
#![cfg_attr(feature = "nightly", feature(async_fn_in_trait, impl_trait_projections))]

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
pub use embassy_hal_common::{into_ref, Peripheral, PeripheralRef};
#[cfg(feature = "unstable-pac")]
pub use rp_pac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use rp_pac as pac;

#[cfg(feature = "rt")]
pub use crate::pac::NVIC_PRIO_BITS;

embassy_hal_common::interrupt_mod!(
    TIMER_IRQ_0,
    TIMER_IRQ_1,
    TIMER_IRQ_2,
    TIMER_IRQ_3,
    PWM_IRQ_WRAP,
    USBCTRL_IRQ,
    XIP_IRQ,
    PIO0_IRQ_0,
    PIO0_IRQ_1,
    PIO1_IRQ_0,
    PIO1_IRQ_1,
    DMA_IRQ_0,
    DMA_IRQ_1,
    IO_IRQ_BANK0,
    IO_IRQ_QSPI,
    SIO_IRQ_PROC0,
    SIO_IRQ_PROC1,
    CLOCKS_IRQ,
    SPI0_IRQ,
    SPI1_IRQ,
    UART0_IRQ,
    UART1_IRQ,
    ADC_IRQ_FIFO,
    I2C0_IRQ,
    I2C1_IRQ,
    RTC_IRQ,
    SWI_IRQ_0,
    SWI_IRQ_1,
    SWI_IRQ_2,
    SWI_IRQ_3,
    SWI_IRQ_4,
    SWI_IRQ_5,
);

/// Macro to bind interrupts to handlers.
///
/// This defines the right interrupt handlers, and creates a unit struct (like `struct Irqs;`)
/// and implements the right [`Binding`]s for it. You can pass this struct to drivers to
/// prove at compile-time that the right interrupts have been bound.
// developer note: this macro can't be in `embassy-hal-common` due to the use of `$crate`.
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

macro_rules! select_bootloader {
    ( $( $feature:literal => $loader:ident, )+ default => $default:ident ) => {
        $(
            #[cfg(feature = $feature)]
            #[link_section = ".boot2"]
            #[used]
            static BOOT2: [u8; 256] = rp2040_boot2::$loader;
        )*

        #[cfg(not(any( $( feature = $feature),* )))]
        #[link_section = ".boot2"]
        #[used]
        static BOOT2: [u8; 256] = rp2040_boot2::$default;
    }
}

select_bootloader! {
    "boot2-at25sf128a" => BOOT_LOADER_AT25SF128A,
    "boot2-gd25q64cs" => BOOT_LOADER_GD25Q64CS,
    "boot2-generic-03h" => BOOT_LOADER_GENERIC_03H,
    "boot2-is25lp080" => BOOT_LOADER_IS25LP080,
    "boot2-ram-memcpy" => BOOT_LOADER_RAM_MEMCPY,
    "boot2-w25q080" => BOOT_LOADER_W25Q080,
    "boot2-w25x10cl" => BOOT_LOADER_W25X10CL,
    default => BOOT_LOADER_W25Q080
}

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
        gpio::init();
    }

    peripherals
}

/// Extension trait for PAC regs, adding atomic xor/bitset/bitclear writes.
trait RegExt<T: Copy> {
    fn write_xor<R>(&self, f: impl FnOnce(&mut T) -> R) -> R;
    fn write_set<R>(&self, f: impl FnOnce(&mut T) -> R) -> R;
    fn write_clear<R>(&self, f: impl FnOnce(&mut T) -> R) -> R;
}

impl<T: Default + Copy, A: pac::common::Write> RegExt<T> for pac::common::Reg<T, A> {
    fn write_xor<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut val = Default::default();
        let res = f(&mut val);
        unsafe {
            let ptr = (self.as_ptr() as *mut u8).add(0x1000) as *mut T;
            ptr.write_volatile(val);
        }
        res
    }

    fn write_set<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut val = Default::default();
        let res = f(&mut val);
        unsafe {
            let ptr = (self.as_ptr() as *mut u8).add(0x2000) as *mut T;
            ptr.write_volatile(val);
        }
        res
    }

    fn write_clear<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut val = Default::default();
        let res = f(&mut val);
        unsafe {
            let ptr = (self.as_ptr() as *mut u8).add(0x3000) as *mut T;
            ptr.write_volatile(val);
        }
        res
    }
}
