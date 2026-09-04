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
#[non_exhaustive]
pub enum Error {
    /// Invalid address provided
    Address,
    /// Bus error
    Bus,
    /// Arbitration lost
    Arbitration,
    /// ACK not received
    NackAddress,
    NackData,
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
#[non_exhaustive]
pub struct Config {
    pub frequency: u32,
}

impl Config {
    fn new(frequency: u32) -> Self {
        if frequency == 0 {
            panic!("0 frequency is not allowed!");
        } 

        Self { frequency }
    }
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
        SYSCON
            .fcclksel(T::instance_number())
            .modify(|w| w.set_sel(syscon::vals::FcclkselSel::Enum0x3)); // 96 MHz FRO
        SYSCON.flexfrgctrl(T::instance_number()).modify(|w| {
            w.set_div(0xFF); // denominator of the rate divider
            w.set_mult(0); // numerator of the rate divider
        });
        
        // Flexcomm interface clock rate
        let fclk: u32 = 96_000_000;
        
        // Quoting the datasheet: (LPC55S6xLPC55S2xLPC552x User manual, chapter 33.7.2.1)
        // Nominal SCL rate = Flexcomm Interface function clock rate / (SCL high time + SCL low time)
        // Remark: For 400 kHz clock rate, the clock frequency after the I2C divider (divval) must be <= 2 MHz.
        
        // The remark is the reason why we have to use 96 MHz clock, even though it may seem like an overkill
        // If go with 12 MHz instead, exactly 400 kHz rate forces an internal I2C clock (FCLK/(DIVVAL+1)) of 2.4 MHz, which is above the 2 MHz margin
        // 400 kHz requires (DIVVAL+1)*(high_mult+low_mult) = 12_000_000 / 400_000 = 30
        // The smallest valid divider that divides 30 evenly is (DIVVAL+1) = 5
        // 12_000_000 / 5 = 2.4 MHz => over 2 MHz
        
        // If we go with 96 MHz => 96_000_000 / 400_000 = 240, which divides evenly with (DIVVAL+1) = 60
        // 96_000_000 / 60 = 1.6 MHz => below 2 MHz

        // The frequency demanded by user
        let target = config.frequency;

        // To be replaced by best computed parameters
        let mut best_divval: u16 = 0;
        let mut best_mult: u8 = 2;
        let mut best_error = u32::MAX;

        // Formula from the datasheet: achieved rate = FCLK / ((DIVVAL+1) * (high_mult + low_mult))
        // In this driver we use 50/50 duty cycle, so high_mult = low_mult
        // MSTSCLLOW and MSTSCLHIGH registers only support values ranging from 2 to 9
        for mult in 2u32..=9 {
            let denom = 2 * mult; // = high_mult + low_mult
            
            // DIVVAL+1 = FCLK / (target * denom)
            let divisor = target * denom;
            let divval_plus_one = ((fclk + divisor / 2) / divisor).clamp(1, 0x1_0000); // keeping within u16 range
            let divval = (divval_plus_one - 1) as u16;

            let achieved = fclk / (divval_plus_one * denom);
            let error = achieved.abs_diff(target);
            let internal_clock = fclk / divval_plus_one;
            
            // Check if current and best results meet the <= 2 MHz requirement
            let meets_guideline = target < 400_000 || internal_clock <= 2_000_000;
            let best_meets_guideline = target < 400_000 || (fclk / (best_divval as u32 + 1)) <= 2_000_000;

            if error < best_error || (error == best_error && meets_guideline && !best_meets_guideline) {
                best_divval = divval;
                best_mult = mult as u8;
                best_error = error;
            }
        }

        let i2c = T::info().i2c_reg;
        i2c.clkdiv().modify(|w| w.set_divval(best_divval));
        i2c.msttime().modify(|w| {
            w.set_mstcllow(i2c::vals::Mstscllow::from_bits(best_mult - 2);
            w.set_mstclhigh(i2c::vals::Mstsclhigh::from_bits(best_mult - 2);
        });
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

    /// Initiate the transaction (master mode)
    fn start_transaction(&mut self, address: u8, read: bool) -> Result<(), Error> {
        if address >= 0x80 {
            return Err(Error::Address);
        }

        let i2c = self.info.i2c_reg;
        i2c.mstdat().write(|w| w.set_data((address << 1) | (read as u8)));
        i2c.mstctl().write(|w| w.set_mststart(i2c::vals::Mststart::Start));

        let mststate = self.wait_mststate()?;
        let expected = if read {
            i2c::vals::Mststate::ReceiveReady
        } else {
            i2c::vals::Mststate::TransmitReady
        };

        if mststate == expected {
            return Ok(());
        }

        let err = match state {
            i2c::vals::Mststate::NackAddress => Error::NackAddress,
            i2c::vals::Mststate::NackData => Error::NackData,
            _ => Error::Bus,
        };

        self.stop_transaction()?;
        Err(err)
    }

    /// Wait and get the MSTSTATE when ready
    fn wait_mststate(&mut self) -> Result<i2c::vals::Mststate, Error> {
        let i2c = self.info.i2c_reg;
        loop {
            let stat = i2c.stat().read();

            if stat.mstarbloss() == i2c::vals::Mstarbloss::ArbitrationLoss {
                // Write back a value with only this bit set
                i2c.stat()
                    .write(|w| w.set_mstarbloss(i2c::vals::Mstarbloss::ArbitrationLoss));
                return Err(Error::Arbitration);
            }
            if stat.mstststperr() {
                i2c.stat().write(|w| w.set_mstststperr(true));
                return Err(Error::Bus);
            }
            if stat.mstpending() == i2c::vals::Mstpending::Pending {
                return Ok(stat.mststate());
            }
        }
    }

    /// Stop and wait for the master to return to Idle
    fn stop_transaction(&mut self) -> Result<(), Error> {
        let i2c = self.info.i2c_reg;
        i2c.mstctl().write(|w| w.set_mststop(i2c::vals::Mststop::Stop));

        match self.wait_mststate()? {
            i2c::vals::Mststate::Idle => Ok(()),
            _ => Err(Error::Bus),
        }
    }
}

/// Blocking I2C implementation
impl<'d> I2c<'d, Blocking> {
    fn new_blocking(
        scl: (Peri<'d, AnyPin>, PioFunc),
        sda: (Peri<'d, AnyPin>, PioFunc),
        config: Config,
    ) -> Self {
        Self::new_inner(scl, sda, config)
    }

    fn blocking_read(&self, address: u8, buf: &[u8]) -> Result<(), Error> {
        self.start_transaction(address, true)?;
        // TODO
    }

    fn blocking_write(&self, address: u8, data: &[u8]) -> Result<(), Error> {
        self.start_transaction(address, false)?;

        if data.len() == 0 {
            return Err(Error::ZeroLengthTransfer);
        }
        // TODO
    }

    fn blocking_write_read(&self) {
        // TODO
    }
}