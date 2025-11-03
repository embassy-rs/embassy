#![macro_use]

use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
pub use embedded_hal_02::spi::{Phase, Polarity};

use crate::gpio::{AnyPin, SealedPin};
use crate::pac::flexcomm::Flexcomm as FlexcommReg;
use crate::pac::iocon::vals::PioFunc;
use crate::pac::spi::Spi as SpiReg;
use crate::pac::*;
use crate::{Blocking, Mode};

/// SPI errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Triggered when the FIFO (or shift-register) is overflowed.
    Overrun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConstructorError {
    /// Neither MOSI nor MISO pin is provided.
    NoTransferringPinsError,
    /// Frequency is too low or too high.
    IncompatibleFrequencyError,
}

/// SPI configuration.
#[non_exhaustive]
#[derive(Clone)]
pub struct Config {
    /// Frequency.
    pub frequency: u32,
    /// Phase.
    pub phase: Phase,
    /// Polarity.
    pub polarity: Polarity,
    /// Binary representaion of sent data.
    pub data_format: DataFormat,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: 1_000_000,
            phase: Phase::CaptureOnFirstTransition,
            polarity: Polarity::IdleLow,
            data_format: DataFormat::MsbFirst,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DataFormat {
    LsbFirst,
    MsbFirst,
}

/// SPI driver.
pub struct Spi<'d, M: Mode> {
    info: &'static Info,
    phantom: PhantomData<(&'d (), M)>,
}

impl<'d, M: Mode> Spi<'d, M> {
    fn new_inner<T: Instance>(
        mosi: Option<Peri<'d, AnyPin>>,
        mosi_func: Option<PioFunc>,
        miso: Option<Peri<'d, AnyPin>>,
        miso_func: Option<PioFunc>,
        sck: Peri<'d, AnyPin>,
        sck_func: PioFunc,
        config: Config,
    ) -> Result<Self, ConstructorError> {
        let mut is_transfer_pin_present = false;
        let mut mosi_tuple = None;
        let mut miso_tuple = None;
        if let (Some(mosi), Some(mosi_func)) = (mosi, mosi_func) {
            is_transfer_pin_present = true;
            mosi_tuple = Some((mosi, mosi_func));
        }
        if let (Some(miso), Some(miso_func)) = (miso, miso_func) {
            is_transfer_pin_present = true;
            miso_tuple = Some((miso, miso_func));
        }
        if !is_transfer_pin_present {
            return Err(ConstructorError::NoTransferringPinsError);
        }
        Self::init::<T>(mosi_tuple, miso_tuple, (sck, sck_func), config);
        Ok(Self {
            info: T::info(),
            phantom: PhantomData,
        })
    }

    fn init<T: Instance>(
        mosi: Option<(Peri<'_, AnyPin>, PioFunc)>,
        miso: Option<(Peri<'_, AnyPin>, PioFunc)>,
        sck: (Peri<'_, AnyPin>, PioFunc),
        config: Config,
    ) {
        Self::configure_flexcomm(T::info().fc_reg, T::instance_number());
        Self::configure_clock(T::info(), T::instance_number(), &config);
        Self::configure_pins(mosi, miso, sck);
        Self::configure_spi(T::info(), &config);
        T::info().spi_reg.cfg().modify(|w| w.set_enable(true));
    }

    fn configure_flexcomm(flexcomm_register: crate::pac::flexcomm::Flexcomm, instance_number: usize) {
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
            .modify(|w| w.set_fc_rst(instance_number, syscon::vals::FcRst::ASSERTED));
        SYSCON
            .presetctrl1()
            .modify(|w| w.set_fc_rst(instance_number, syscon::vals::FcRst::RELEASED));
        flexcomm_register.pselid().modify(|w| {
            w.set_persel(flexcomm::vals::Persel::SPI);
            // This will lock the peripheral PERSEL and will not allow any changes until the board is reset.
            w.set_lock(true);
        });
    }

    fn configure_clock(registers: &'static Info, instance_number: usize, config: &Config) {
        // Adaptive clock choice based on desired frequency
        // To get the desired frequency, it is necessary to choose the clock bigger than the desired value so that it can be 'chiseled'
        // There are two types of dividers: integer divider (rate divider value)
        // and fractional divider (fractional rate divider).

        // Minimum and maximum values were computed taking these formulas into account:
        // For minimum value, MULT = 255, DIVVAL = 65536
        // For maximum value, MULT = 0, DIVVAL = 0
        // Flexcomm Interface function clock = (clock selected via FCCLKSEL) / (1 + MULT / DIV)
        // Final frequency = FCLK / (DIVVAL + 1)

        // 96 MHz is chosen because the frequency range used by SPI can be obtained using simple divisions.
        // Minimum value is around 732 Hz (96 MHz / ((1 + 256/256) * 65536) = 96 MHz / 131_072 = 732.42 Hz)

        SYSCON
            .fcclksel(instance_number)
            .modify(|w| w.set_sel(syscon::vals::FcclkselSel::ENUM_0X3));
        let source_clock = 96_000_000;

        // Parameter calculation for clock division
        let div_val = (source_clock / config.frequency).min(0xFFFF);
        let raw_clock = 96_000_000 / div_val;
        let mult_val = ((raw_clock * 256 / config.frequency) - 256).min(255);

        // FCLK =  (clock selected via FCCLKSEL) / (1+ MULT / DIV)
        // Remark: To use the fractional baud rate generator, 0xFF must be written to the DIV value
        // to yield a denominator value of 256. All other values are not supported
        SYSCON.flexfrgctrl(instance_number).modify(|w| {
            w.set_div(0xFF);
            w.set_mult(mult_val as u8);
        });

        // Final frequency = FCLK / (DIVVAL + 1)
        registers.spi_reg.div().modify(|w| w.set_divval(div_val as u16));
    }

    fn configure_pins(
        mosi: Option<(Peri<'_, AnyPin>, PioFunc)>,
        miso: Option<(Peri<'_, AnyPin>, PioFunc)>,
        sck: (Peri<'_, AnyPin>, PioFunc),
    ) {
        if let Some((mosi_pin, func)) = mosi {
            mosi_pin.pio().modify(|w| {
                w.set_func(func);
                w.set_mode(iocon::vals::PioMode::INACTIVE);
                w.set_slew(iocon::vals::PioSlew::STANDARD);
                w.set_invert(false);
                w.set_digimode(iocon::vals::PioDigimode::DIGITAL);
                w.set_od(iocon::vals::PioOd::NORMAL);
            });
        }

        if let Some((miso_pin, func)) = miso {
            miso_pin.pio().modify(|w| {
                w.set_func(func);
                w.set_mode(iocon::vals::PioMode::INACTIVE);
                w.set_slew(iocon::vals::PioSlew::STANDARD);
                w.set_invert(false);
                w.set_digimode(iocon::vals::PioDigimode::DIGITAL);
                w.set_od(iocon::vals::PioOd::NORMAL);
            });
        };

        sck.0.pio().modify(|w| {
            w.set_func(sck.1);
            w.set_mode(iocon::vals::PioMode::INACTIVE);
            w.set_slew(iocon::vals::PioSlew::STANDARD);
            w.set_invert(false);
            w.set_digimode(iocon::vals::PioDigimode::DIGITAL);
            w.set_od(iocon::vals::PioOd::NORMAL);
        });
    }

    fn configure_spi(info: &'static Info, config: &Config) {
        let registers = info.spi_reg;

        registers.fifocfg().modify(|w| {
            w.set_enablerx(false);
            w.set_enabletx(false);
        });

        registers.cfg().modify(|w| {
            w.set_enable(false);
            w.set_master(spi::vals::Master::MASTER_MODE);
            w.set_loop_(false);
        });

        // Configurations based on the config written by a user.
        registers.cfg().modify(|w| {
            w.set_cpha(match config.phase {
                Phase::CaptureOnFirstTransition => spi::vals::Cpha::CAPTURE,
                Phase::CaptureOnSecondTransition => spi::vals::Cpha::CHANGE,
            });
            w.set_cpol(match config.polarity {
                Polarity::IdleLow => spi::vals::Cpol::LOW,
                Polarity::IdleHigh => spi::vals::Cpol::HIGH,
            });
            w.set_lsbf(match config.data_format {
                DataFormat::LsbFirst => spi::vals::Lsbf::REVERSE,
                DataFormat::MsbFirst => spi::vals::Lsbf::STANDARD,
            });
        });

        registers.fifocfg().modify(|w| {
            // DMA is going to be disabled until the async version is implemented.
            w.set_dmatx(false);
            w.set_dmarx(false);
            w.set_enabletx(true);
            w.set_enablerx(true);
            w.set_emptytx(true);
            w.set_emptyrx(true);
        });
    }

    /// Private function to apply SPI configuration (phase, polarity, frequency) settings.
    ///
    /// Driver should be disabled before making changes and re-enabled after the modifications
    /// are applied.

    /// Write data to SPI blocking execution until done.
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), Error> {
        let spi_reg = self.info.spi_reg;
        for d in data {
            while !spi_reg.fifostat().read().txnotfull() {}
            if spi_reg.fifostat().read().txerr() {
                return Err(Error::Overrun);
            }
            spi_reg.fifowr().write(|w| {
                w.set_rxignore(spi::vals::Rxignore::IGNORE);
                w.set_txdata(*d as u16); // Data to be transferred
                w.set_len(7);
            });
        }
        self.flush()?;

        Ok(())
    }

    /// Transfer data in place to SPI blocking execution until done.
    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        let spi_reg = self.info.spi_reg;
        for d in data {
            while !spi_reg.fifostat().read().txnotfull() {}
            if spi_reg.fifostat().read().txerr() {
                return Err(Error::Overrun);
            }
            spi_reg.fifowr().write(|w| {
                w.set_rxignore(spi::vals::Rxignore::READ);
                w.set_txdata(*d as u16); // Data to be transferred
                w.set_len(7);
            });
            while !spi_reg.fifostat().read().rxnotempty() {}
            if spi_reg.fifostat().read().rxerr() {
                return Err(Error::Overrun);
            }
            *d = spi_reg.fiford().read().rxdata() as u8;
        }
        self.flush()?;
        Ok(())
    }

    /// Read data from SPI blocking execution until done.
    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        let spi_reg = self.info.spi_reg;
        for d in data {
            while !spi_reg.fifostat().read().txnotfull() {}
            spi_reg.fifowr().write(|w| {
                w.set_rxignore(spi::vals::Rxignore::READ);
                w.set_txdata(0u16); // Data to be transferred
                w.set_len(7);
            });
            while !spi_reg.fifostat().read().rxnotempty() {}
            if spi_reg.fifostat().read().rxerr() {
                return Err(Error::Overrun);
            }
            *d = spi_reg.fiford().read().rxdata() as u8;
        }
        Ok(())
    }

    /// Transfer data to SPI blocking execution until done.
    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        let spi_reg = self.info.spi_reg;
        let len = read.len().max(write.len());
        for i in 0..len {
            let wb = write.get(i).copied().unwrap_or(0);
            while !spi_reg.fifostat().read().txnotfull() {}
            if spi_reg.fifostat().read().txerr() {
                return Err(Error::Overrun);
            }
            spi_reg.fifowr().write(|w| {
                w.set_rxignore(spi::vals::Rxignore::READ);
                w.set_txdata(wb as u16); // Data to be transferred
                w.set_len(7);
            });
            while !spi_reg.fifostat().read().rxnotempty() {}
            if spi_reg.fifostat().read().rxerr() {
                return Err(Error::Overrun);
            }
            let rb = spi_reg.fiford().read().rxdata() as u8;
            if let Some(r) = read.get_mut(i) {
                *r = rb;
            }
        }
        self.flush()?;
        Ok(())
    }

    /// Block execution until SPI is done.
    pub fn flush(&mut self) -> Result<(), Error> {
        while !self.info.spi_reg.fifostat().read().txempty() {}
        Ok(())
    }

    // TODO: write a function that updates SPI configuration (phase, polarity, frequency) settings
}

impl<'d> Spi<'d, Blocking> {
    /// Create an SPI driver in blocking mode.
    pub fn new_blocking<T: Instance>(
        _inner: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T> + 'd>,
        mosi: Peri<'d, impl MosiPin<T> + 'd>,
        miso: Peri<'d, impl MisoPin<T> + 'd>,
        config: Config,
    ) -> Result<Self, ConstructorError> {
        let mosi_func = mosi.pin_func();
        let miso_func = miso.pin_func();
        let sck_func = sck.pin_func();
        Self::new_inner::<T>(
            Some(mosi.into()),
            Some(mosi_func),
            Some(miso.into()),
            Some(miso_func),
            sck.into(),
            sck_func,
            config,
        )
    }

    /// Create an SPI driver in blocking mode supporting writes only.
    pub fn new_blocking_txonly<T: Instance>(
        _inner: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T> + 'd>,
        mosi: Peri<'d, impl MosiPin<T> + 'd>,
        config: Config,
    ) -> Result<Self, ConstructorError> {
        let mosi_func = mosi.pin_func();
        let sck_func = sck.pin_func();
        Self::new_inner::<T>(
            Some(mosi.into()),
            Some(mosi_func),
            None,
            None,
            sck.into(),
            sck_func,
            config,
        )
    }

    /// Create an SPI driver in blocking mode supporting reads only.
    pub fn new_blocking_rxonly<T: Instance>(
        _inner: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T> + 'd>,
        miso: Peri<'d, impl MisoPin<T> + 'd>,
        config: Config,
    ) -> Result<Self, ConstructorError> {
        let miso_func = miso.pin_func();
        let sck_func = sck.pin_func();
        Self::new_inner::<T>(
            None,
            None,
            Some(miso.into()),
            Some(miso_func),
            sck.into(),
            sck_func,
            config,
        )
    }
}

impl<'d> embedded_hal_02::blocking::spi::Transfer<u8> for Spi<'d, Blocking> {
    type Error = Error;
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.blocking_transfer_in_place(words)?;
        Ok(words)
    }
}

impl<'d> embedded_hal_02::blocking::spi::Write<u8> for Spi<'d, Blocking> {
    type Error = Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(words)
    }
}

pub(crate) struct Info {
    pub(crate) spi_reg: SpiReg,
    pub(crate) fc_reg: FlexcommReg,
}

pub(crate) trait SealedInstance {
    fn info() -> &'static Info;
    fn instance_number() -> usize;
}

/// SPI instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {}

macro_rules! impl_spi_instance {
    ($inst:ident, $fc:ident, $fc_num:expr) => {
        impl $crate::spi::SealedInstance for $crate::peripherals::$inst {
            fn info() -> &'static crate::spi::Info {
                static INFO: $crate::spi::Info = $crate::spi::Info {
                    spi_reg: $crate::pac::$inst,
                    fc_reg: $crate::pac::$fc,
                };
                &INFO
            }

            #[inline]
            fn instance_number() -> usize {
                $fc_num
            }
        }
        impl $crate::spi::Instance for $crate::peripherals::$inst {}
    };
}

#[cfg(has_spi_sck_pins)]
pub(crate) trait SealedSckPin<T: Instance>: crate::gpio::Pin {
    fn pin_func(&self) -> PioFunc;
}

#[cfg(has_spi_mosi_pins)]
pub(crate) trait SealedMosiPin<T: Instance>: crate::gpio::Pin {
    fn pin_func(&self) -> PioFunc;
}

#[cfg(has_spi_miso_pins)]
pub(crate) trait SealedMisoPin<T: Instance>: crate::gpio::Pin {
    fn pin_func(&self) -> PioFunc;
}

/// Trait for clock pins.
#[cfg(has_spi_sck_pins)]
#[allow(private_bounds)]
pub trait SckPin<T: Instance>: SealedSckPin<T> + crate::gpio::Pin {}
/// Trait for MOSI pins.
#[cfg(has_spi_mosi_pins)]
#[allow(private_bounds)]
pub trait MosiPin<T: Instance>: SealedMosiPin<T> + crate::gpio::Pin {}
/// Trait for MISO pins.
#[cfg(has_spi_miso_pins)]
#[allow(private_bounds)]
pub trait MisoPin<T: Instance>: SealedMisoPin<T> + crate::gpio::Pin {}

#[cfg(has_spi_miso_pins)]
macro_rules! impl_spi_miso_pin {
    ($pin:ident, $instance:ident, $func: ident) => {
        impl crate::spi::SealedMisoPin<crate::peripherals::$instance> for crate::peripherals::$pin {
            fn pin_func(&self) -> crate::pac::iocon::vals::PioFunc {
                use crate::pac::iocon::vals::PioFunc;
                PioFunc::$func
            }
        }

        impl crate::spi::MisoPin<crate::peripherals::$instance> for crate::peripherals::$pin {}
    };
}

#[cfg(has_spi_mosi_pins)]
macro_rules! impl_spi_mosi_pin {
    ($pin:ident, $instance:ident, $func: ident) => {
        impl crate::spi::SealedMosiPin<crate::peripherals::$instance> for crate::peripherals::$pin {
            fn pin_func(&self) -> crate::pac::iocon::vals::PioFunc {
                use crate::pac::iocon::vals::PioFunc;
                PioFunc::$func
            }
        }

        impl crate::spi::MosiPin<crate::peripherals::$instance> for crate::peripherals::$pin {}
    };
}

#[cfg(has_spi_sck_pins)]
macro_rules! impl_spi_sck_pin {
    ($pin:ident, $instance:ident, $func: ident) => {
        impl crate::spi::SealedSckPin<crate::peripherals::$instance> for crate::peripherals::$pin {
            fn pin_func(&self) -> crate::pac::iocon::vals::PioFunc {
                use crate::pac::iocon::vals::PioFunc;
                PioFunc::$func
            }
        }

        impl crate::spi::SckPin<crate::peripherals::$instance> for crate::peripherals::$pin {}
    };
}
