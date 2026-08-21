#![no_std]
#![doc = include_str!("../README.md")]
#![allow(non_snake_case)]
#![cfg_attr(target_arch = "avr", feature(abi_avr_interrupt))]

#[cfg(all(feature = "clock-16mhz", feature = "clock-8mhz"))]
compile_error!("enable only one CPU clock feature: `clock-16mhz` or `clock-8mhz`");
#[cfg(not(any(feature = "clock-16mhz", feature = "clock-8mhz")))]
compile_error!("enable one CPU clock feature: `clock-16mhz` or `clock-8mhz`");

pub mod adc;
pub mod delay;
pub mod gpio;
pub mod i2c;
pub mod onewire;
pub mod pwm;
pub mod spi;
pub mod usart;

#[cfg(any(feature = "time-driver-16mhz", feature = "time-driver-8mhz"))]
mod time_driver;

/// Low-level peripheral access crate for advanced use.
///
/// This API is not covered by this crate's semantic-versioning guarantees.
#[cfg(feature = "unstable-pac")]
pub use avr_device::atmega328p as pac;

/// All peripheral ownership tokens exposed by this HAL.
///
/// There can be only one safe instance. Obtain it with [`init`] or
/// [`try_init`].
pub struct Peripherals {
    pub ADC: adc::AdcPeripheral,
    pub TWI: i2c::TwiPeripheral,
    pub SPI: spi::SpiPeripheral,
    pub USART0: usart::Usart0Peripheral,
    pub TIMER0: pwm::Timer0,
    pub TIMER2: pwm::Timer2,
    pub PB0: gpio::PB0,
    pub PB1: gpio::PB1,
    pub PB2: gpio::PB2,
    pub PB3: gpio::PB3,
    pub PB4: gpio::PB4,
    pub PB5: gpio::PB5,
    pub PB6: gpio::PB6,
    pub PB7: gpio::PB7,
    pub PC0: gpio::PC0,
    pub PC1: gpio::PC1,
    pub PC2: gpio::PC2,
    pub PC3: gpio::PC3,
    pub PC4: gpio::PC4,
    pub PC5: gpio::PC5,
    pub PC6: gpio::PC6,
    pub PD0: gpio::PD0,
    pub PD1: gpio::PD1,
    pub PD2: gpio::PD2,
    pub PD3: gpio::PD3,
    pub PD4: gpio::PD4,
    pub PD5: gpio::PD5,
    pub PD6: gpio::PD6,
    pub PD7: gpio::PD7,
}

impl Peripherals {
    fn new() -> Self {
        Self {
            ADC: adc::AdcPeripheral::new(),
            TWI: i2c::TwiPeripheral::new(),
            SPI: spi::SpiPeripheral::new(),
            USART0: usart::Usart0Peripheral::new(),
            TIMER0: pwm::Timer0::new(),
            TIMER2: pwm::Timer2::new(),
            PB0: gpio::PB0::new(),
            PB1: gpio::PB1::new(),
            PB2: gpio::PB2::new(),
            PB3: gpio::PB3::new(),
            PB4: gpio::PB4::new(),
            PB5: gpio::PB5::new(),
            PB6: gpio::PB6::new(),
            PB7: gpio::PB7::new(),
            PC0: gpio::PC0::new(),
            PC1: gpio::PC1::new(),
            PC2: gpio::PC2::new(),
            PC3: gpio::PC3::new(),
            PC4: gpio::PC4::new(),
            PC5: gpio::PC5::new(),
            PC6: gpio::PC6::new(),
            PD0: gpio::PD0::new(),
            PD1: gpio::PD1::new(),
            PD2: gpio::PD2::new(),
            PD3: gpio::PD3::new(),
            PD4: gpio::PD4::new(),
            PD5: gpio::PD5::new(),
            PD6: gpio::PD6::new(),
            PD7: gpio::PD7::new(),
        }
    }
}

/// CPU clock frequency selected at compile time.
#[cfg(feature = "clock-16mhz")]
pub const CPU_HZ: u32 = 16_000_000;

/// CPU clock frequency selected at compile time.
#[cfg(feature = "clock-8mhz")]
pub const CPU_HZ: u32 = 8_000_000;

/// Initializes the HAL and returns its peripheral ownership tokens.
///
/// # Panics
///
/// Panics if the HAL or the underlying PAC has already been taken.
pub fn init() -> Peripherals {
    try_init().expect("embassy-atmega328p is already initialized")
}

/// Attempts to initialize the HAL.
///
/// Returns `None` if the HAL or the underlying PAC has already been taken.
pub fn try_init() -> Option<Peripherals> {
    avr_device::atmega328p::Peripherals::take().map(|_| {
        #[cfg(any(feature = "time-driver-16mhz", feature = "time-driver-8mhz"))]
        time_driver::init();

        Peripherals::new()
    })
}
