//! I2C driver
#![macro_use]

use embassy_hal_internal::{Peri, PeripheralType};

use crate::gpio::{AnyPin, SealedPin};
use crate::pac::flexcomm::Flexcomm as FlexcommReg;
use crate::pac::i2c::I2c as I2cReg;
use crate::pac::iocon::vals::PioFunc;
use crate::{Async, Blocking, Mode};

/// I2C error.
/// Copied from: https://github.com/embassy-rs/embassy/blob/main/embassy-stm32/src/i2c/mod.rs#L34
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Bus error
    Bus,
    /// Arbitration lost
    Arbitration,
    /// ACK not received (either to the address or to a data byte)
    Nack,
    /// Timeout
    Timeout,
    /// CRC error
    Crc,
    /// Overrun error
    Overrun,
    /// Zero-length transfers are not allowed.
    ZeroLengthTransfer,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::Bus => "Bus Error",
            Self::Arbitration => "Arbitration Lost",
            Self::Nack => "ACK Not Received",
            Self::Timeout => "Request Timed Out",
            Self::Crc => "CRC Mismatch",
            Self::Overrun => "Buffer Overrun",
            Self::ZeroLengthTransfer => "Zero-Length Transfers are not allowed",
        };

        write!(f, "{}", message)
    }
}

impl core::error::Error for Error {}

/// I2C configuration
pub struct Config {
    pub frequency: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self { frequency: 100_000 } // 100 kHz
    }
}

pub(crate) struct Info {
    pub(crate) i2c_reg: I2cReg,
    pub(crate) fc_reg: FlexcommReg,
}

pub(crate) trait SealedInstance {
    fn info() -> &'static Info;
    fn instance_number() -> usize;
}

pub(crate) trait Instance: SealedInstance + PeripheralType {}

pub struct I2c<'d, M: Mode> {
    info: &'static Info,
}

impl<'d, M: Mode> I2c<'d, M> {
    fn new_inner<T: Instance>(
        scl: (Peri<'d, AnyPin>, PioFunc),
        sda: (Peri<'d, AnyPin>, PioFunc),
        config: Config,
    ) -> Self {
        Self::init::<T>(scl, sda, config);
        Self {
            info: T::info(),
        }

    }

    fn init<T: Instance>(scl: (Peri<'_, AnyPin>, PioFunc), sda: (Peri<'_, AnyPin>, PioFunc), config: Config) {
        Self::configure_flexcomm(T::info().fc_reg, T::instance_number());
        Self::configure_clock::<T>(&config);
        Self::configure_pins(scl, sda);
        
        let i2c = T::info().i2c_reg;
        // Enable master mode
        i2c.cfg().modify(|w| w.set_msten(true));
    }

    /// Configure felxcomm for I2C
    /// Copied from usart/lpc55.rs and adapted for I2C
    fn configure_flexcomm(flexcomm_reg: FlexcommReg, instance_number: usize) {
        critical_section::with(|_cs| {
            if !(SYSCON.ahbclkctrl0().read().iocon()) {
                SYSCON.ahbclkctrl0().modify(|w| w.set_iocon(true));
            }
        });
        critical_section::with(|_cs| {
            if !(SYSCON.ahbclkctrl1().read().fc(instance_number)) {
                SYSCON.ahbclkctrl1().modify(|w| w.set_fc(instance_number, true));
            }
        });
        SYSCON
            .presetctrl1()
            .modify(|w| w.set_fc_rst(instance_number, syscon::vals::FcRst::Asserted));
        SYSCON
            .presetctrl1()
            .modify(|w| w.set_fc_rst(instance_number, syscon::vals::FcRst::Released));
        flexcomm_register.pselid().modify(|w| {
            w.set_persel(flexcomm::vals::Persel::I2c);
            // This will lock the peripheral PERSEL and will not allow any changes until the board is reset.
            w.set_lock(true);
        });
    }

    fn configure_clock<T: Instance>(config: &Config) {
        // TODO
    }

    fn configure_pins(scl: (Peri<'_, AnyPin>, PioFunc), sda: (Peri<'_, AnyPin>, PioFunc)) {
        for (pin, func) in [scl, sda] {
            pin.pio().modify(|w| {
                w.set_func(func);
                w.set_mode(iocon::vals::PioMode::Inactive);
                w.set_slew(iocon::vals::PioSlew::Standard);
                w.set_invert(false);
                w.set_digimode(iocon::vals::PioDigimode::Digital);
                // OpenDrain = the line relies on PullUp resitors and should only be pulled low
                w.set_od(iocon::vals::PioOd::OpenDrain); 
            });
        }
    }
}