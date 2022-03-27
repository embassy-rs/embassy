#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

#[cfg(feature = "unstable-pac")]
pub use rp2040_pac2 as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use rp2040_pac2 as pac;

pub use embassy::util::Unborrow;
pub use embassy_hal_common::unborrow;

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod interrupt;
pub use embassy_macros::interrupt;

pub mod dma;
pub mod gpio;
pub mod spi;
pub mod timer;
pub mod uart;

mod clocks;
mod reset;

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
}

#[link_section = ".boot2"]
#[used]
static BOOT2: [u8; 256] = *include_bytes!("boot2.bin");

pub mod config {
    #[non_exhaustive]
    pub struct Config {}

    impl Default for Config {
        fn default() -> Self {
            Self {}
        }
    }
}

pub fn init(_config: config::Config) -> Peripherals {
    // Do this first, so that it panics if user is calling `init` a second time
    // before doing anything important.
    let peripherals = Peripherals::take();

    unsafe {
        clocks::init();
        timer::init();
    }

    peripherals
}
