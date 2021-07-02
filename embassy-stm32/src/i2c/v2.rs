use core::cmp;
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_extras::unborrow;
use embedded_hal::blocking::i2c::Read;
use embedded_hal::blocking::i2c::Write;
use embedded_hal::blocking::i2c::WriteRead;

use crate::i2c::{Error, Instance, SclPin, SdaPin};
use crate::pac::gpio::vals::{Afr, Moder, Ot};
use crate::pac::gpio::Gpio;
use crate::pac::i2c;
use crate::time::Hertz;

pub struct I2c<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> I2c<'d, T> {
    pub fn new<F>(
        _peri: impl Unborrow<Target = T> + 'd,
        scl: impl Unborrow<Target = impl SclPin<T>>,
        sda: impl Unborrow<Target = impl SdaPin<T>>,
        freq: F,
    ) -> Self
    where
        F: Into<Hertz>,
    {
        unborrow!(scl, sda);

        T::enable();

        unsafe {
            Self::configure_pin(scl.block(), scl.pin() as _, scl.af_num());
            Self::configure_pin(sda.block(), sda.pin() as _, sda.af_num());
        }

        unsafe {
            T::regs().cr1().modify(|reg| {
                reg.set_pe(false);
                reg.set_anfoff(false);
            });
        }

        let timings = Timings::new(T::frequency(), freq.into());

        unsafe {
            T::regs().timingr().write(|reg| {
                reg.set_presc(timings.prescale);
                reg.set_scll(timings.scll);
                reg.set_sclh(timings.sclh);
                reg.set_sdadel(timings.sdadel);
                reg.set_scldel(timings.scldel);
            });
        }

        unsafe {
            T::regs().cr1().modify(|reg| {
                reg.set_pe(true);
            });
        }

        Self {
            phantom: PhantomData,
        }
    }

    unsafe fn configure_pin(block: Gpio, pin: usize, af_num: u8) {
        let (afr, n_af) = if pin < 8 { (0, pin) } else { (1, pin - 8) };
        block.moder().modify(|w| w.set_moder(pin, Moder::ALTERNATE));
        block.afr(afr).modify(|w| w.set_afr(n_af, Afr(af_num)));
        block.otyper().modify(|w| w.set_ot(pin, Ot::OPENDRAIN));
        //block
        //.ospeedr()
        //.modify(|w| w.set_ospeedr(pin, crate::pac::gpio::vals::Ospeedr::VERYHIGHSPEED));
    }

    fn master_stop(&mut self) {
        unsafe {
            T::regs().cr2().write(|w| w.set_stop(i2c::vals::Stop::STOP));
        }
    }

    fn master_read(&mut self, address: u8, length: usize, stop: Stop) {
        assert!(length < 256 && length > 0);

        // Wait for any previous address sequence to end
        // automatically. This could be up to 50% of a bus
        // cycle (ie. up to 0.5/freq)
        while unsafe { T::regs().cr2().read().start() == i2c::vals::Start::START } {}

        // Set START and prepare to receive bytes into
        // `buffer`. The START bit can be set even if the bus
        // is BUSY or I2C is in slave mode.

        unsafe {
            T::regs().cr2().modify(|w| {
                w.set_sadd((address << 1 | 0) as u16);
                w.set_rd_wrn(i2c::vals::RdWrn::READ);
                w.set_nbytes(length as u8);
                w.set_start(i2c::vals::Start::START);
                w.set_autoend(stop.autoend());
            });
        }
    }

    fn master_write(&mut self, address: u8, length: usize, stop: Stop) {
        assert!(length < 256 && length > 0);

        // Wait for any previous address sequence to end
        // automatically. This could be up to 50% of a bus
        // cycle (ie. up to 0.5/freq)
        while unsafe { T::regs().cr2().read().start() == i2c::vals::Start::START } {}

        // Set START and prepare to send `bytes`. The
        // START bit can be set even if the bus is BUSY or
        // I2C is in slave mode.
        unsafe {
            T::regs().cr2().modify(|w| {
                w.set_sadd((address << 1 | 0) as u16);
                w.set_add10(i2c::vals::Add::BIT7);
                w.set_rd_wrn(i2c::vals::RdWrn::WRITE);
                w.set_nbytes(length as u8);
                w.set_start(i2c::vals::Start::START);
                w.set_autoend(stop.autoend());
            });
        }
    }

    fn master_re_start(&mut self, address: u8, length: usize, stop: Stop) {
        assert!(length < 256 && length > 0);

        unsafe {
            T::regs().cr2().modify(|w| {
                w.set_sadd((address << 1 | 1) as u16);
                w.set_add10(i2c::vals::Add::BIT7);
                w.set_rd_wrn(i2c::vals::RdWrn::READ);
                w.set_nbytes(length as u8);
                w.set_start(i2c::vals::Start::START);
                w.set_autoend(stop.autoend());
            });
        }
    }

    fn flush_txdr(&self) {
        //if $i2c.isr.read().txis().bit_is_set() {
        //$i2c.txdr.write(|w| w.txdata().bits(0));
        //}

        unsafe {
            if T::regs().isr().read().txis() {
                T::regs().txdr().write(|w| w.set_txdata(0));
            }
            if T::regs().isr().read().txe() {
                T::regs().isr().modify(|w| w.set_txe(true))
            }
        }

        // If TXDR is not flagged as empty, write 1 to flush it
        //if $i2c.isr.read().txe().is_not_empty() {
        //$i2c.isr.write(|w| w.txe().set_bit());
        //}
    }

    fn wait_txe(&self) -> Result<(), Error> {
        loop {
            unsafe {
                let isr = T::regs().isr().read();
                if isr.txe() {
                    return Ok(());
                } else if isr.berr() {
                    T::regs().icr().write(|reg| reg.set_berrcf(true));
                    return Err(Error::Bus);
                } else if isr.arlo() {
                    T::regs().icr().write(|reg| reg.set_arlocf(true));
                    return Err(Error::Arbitration);
                } else if isr.nackf() {
                    T::regs().icr().write(|reg| reg.set_nackcf(true));
                    self.flush_txdr();
                    return Err(Error::Nack);
                }
            }
        }
    }

    fn wait_rxne(&self) -> Result<(), Error> {
        loop {
            unsafe {
                let isr = T::regs().isr().read();
                if isr.rxne() {
                    return Ok(());
                } else if isr.berr() {
                    T::regs().icr().write(|reg| reg.set_berrcf(true));
                    return Err(Error::Bus);
                } else if isr.arlo() {
                    T::regs().icr().write(|reg| reg.set_arlocf(true));
                    return Err(Error::Arbitration);
                } else if isr.nackf() {
                    T::regs().icr().write(|reg| reg.set_nackcf(true));
                    self.flush_txdr();
                    return Err(Error::Nack);
                }
            }
        }
    }

    fn wait_tc(&self) -> Result<(), Error> {
        loop {
            unsafe {
                let isr = T::regs().isr().read();
                if isr.tc() {
                    return Ok(());
                } else if isr.berr() {
                    T::regs().icr().write(|reg| reg.set_berrcf(true));
                    return Err(Error::Bus);
                } else if isr.arlo() {
                    T::regs().icr().write(|reg| reg.set_arlocf(true));
                    return Err(Error::Arbitration);
                } else if isr.nackf() {
                    T::regs().icr().write(|reg| reg.set_nackcf(true));
                    self.flush_txdr();
                    return Err(Error::Nack);
                }
            }
        }
    }
}

impl<'d, T: Instance> Read for I2c<'d, T> {
    type Error = Error;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        assert!(buffer.len() < 256 && buffer.len() > 0);

        self.master_read(address, buffer.len(), Stop::Automatic);

        for byte in buffer {
            // Wait until we have received something
            self.wait_rxne()?;

            //*byte = self.i2c.rxdr.read().rxdata().bits();
            unsafe {
                *byte = T::regs().rxdr().read().rxdata();
            }
        }

        // automatic STOP
        Ok(())
    }
}

impl<'d, T: Instance> Write for I2c<'d, T> {
    type Error = Error;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        // TODO support transfers of more than 255 bytes
        assert!(bytes.len() < 256 && bytes.len() > 0);

        // I2C start
        //
        // ST SAD+W
        self.master_write(address, bytes.len(), Stop::Software);

        for byte in bytes {
            // Wait until we are allowed to send data
            // (START has been ACKed or last byte when
            // through)
            self.wait_txe()?;

            // Put byte on the wire
            //self.i2c.txdr.write(|w| w.txdata().bits(*byte));
            unsafe {
                T::regs().txdr().write(|w| w.set_txdata(*byte));
            }
        }

        // Wait until the write finishes
        self.wait_tc()?;

        // Stop
        self.master_stop();

        Ok(())
    }
}

impl<'d, T: Instance> WriteRead for I2c<'d, T> {
    type Error = Error;

    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        // TODO support transfers of more than 255 bytes
        assert!(bytes.len() < 256 && bytes.len() > 0);
        assert!(buffer.len() < 256 && buffer.len() > 0);

        // I2C start
        //
        // ST SAD+W
        self.master_write(address, bytes.len(), Stop::Software);

        for byte in bytes {
            // Wait until we are allowed to send data
            // (START has been ACKed or last byte went through)
            self.wait_txe()?;

            // Put byte on the wire
            //self.i2c.txdr.write(|w| w.txdata().bits(*byte));
            unsafe {
                T::regs().txdr().write(|w| w.set_txdata(*byte));
            }
        }

        // Wait until the write finishes before beginning to read.
        self.wait_tc()?;

        // I2C re-start
        //
        // SR  SAD+R
        self.master_re_start(address, buffer.len(), Stop::Automatic);

        for byte in buffer {
            // Wait until we have received something
            self.wait_rxne()?;

            //*byte = self.i2c.rxdr.read().rxdata().bits();
            unsafe {
                *byte = T::regs().rxdr().read().rxdata();
            }
        }

        // automatic STOP

        Ok(())
    }
}

/// I2C Stop Configuration
///
/// Peripheral options for generating the STOP condition
#[derive(Copy, Clone, PartialEq)]
pub enum Stop {
    /// Software end mode: Must write register to generate STOP condition
    Software,
    /// Automatic end mode: A STOP condition is automatically generated once the
    /// configured number of bytes have been transferred
    Automatic,
}

impl Stop {
    fn autoend(&self) -> i2c::vals::Autoend {
        match self {
            Stop::Software => i2c::vals::Autoend::SOFTWARE,
            Stop::Automatic => i2c::vals::Autoend::AUTOMATIC,
        }
    }
}

struct Timings {
    prescale: u8,
    scll: u8,
    sclh: u8,
    sdadel: u8,
    scldel: u8,
}

impl Timings {
    fn new(i2cclk: Hertz, freq: Hertz) -> Self {
        let i2cclk = i2cclk.0;
        let freq = freq.0;
        // Refer to RM0433 Rev 7 Figure 539 for setup and hold timing:
        //
        // t_I2CCLK = 1 / PCLK1
        // t_PRESC  = (PRESC + 1) * t_I2CCLK
        // t_SCLL   = (SCLL + 1) * t_PRESC
        // t_SCLH   = (SCLH + 1) * t_PRESC
        //
        // t_SYNC1 + t_SYNC2 > 4 * t_I2CCLK
        // t_SCL ~= t_SYNC1 + t_SYNC2 + t_SCLL + t_SCLH
        let ratio = i2cclk / freq;

        // For the standard-mode configuration method, we must have a ratio of 4
        // or higher
        assert!(
            ratio >= 4,
            "The I2C PCLK must be at least 4 times the bus frequency!"
        );

        let (presc_reg, scll, sclh, sdadel, scldel) = if freq > 100_000 {
            // Fast-mode (Fm) or Fast-mode Plus (Fm+)
            // here we pick SCLL + 1 = 2 * (SCLH + 1)

            // Prescaler, 384 ticks for sclh/scll. Round up then subtract 1
            let presc_reg = ((ratio - 1) / 384) as u8;
            // ratio < 1200 by pclk 120MHz max., therefore presc < 16

            // Actual precale value selected
            let presc = (presc_reg + 1) as u32;

            let sclh = ((ratio / presc) - 3) / 3;
            let scll = (2 * (sclh + 1)) - 1;

            let (sdadel, scldel) = if freq > 400_000 {
                // Fast-mode Plus (Fm+)
                assert!(i2cclk >= 17_000_000); // See table in datsheet

                let sdadel = i2cclk / 8_000_000 / presc;
                let scldel = i2cclk / 4_000_000 / presc - 1;

                (sdadel, scldel)
            } else {
                // Fast-mode (Fm)
                assert!(i2cclk >= 8_000_000); // See table in datsheet

                let sdadel = i2cclk / 4_000_000 / presc;
                let scldel = i2cclk / 2_000_000 / presc - 1;

                (sdadel, scldel)
            };

            (
                presc_reg,
                scll as u8,
                sclh as u8,
                sdadel as u8,
                scldel as u8,
            )
        } else {
            // Standard-mode (Sm)
            // here we pick SCLL = SCLH
            assert!(i2cclk >= 2_000_000); // See table in datsheet

            // Prescaler, 512 ticks for sclh/scll. Round up then
            // subtract 1
            let presc = (ratio - 1) / 512;
            let presc_reg = cmp::min(presc, 15) as u8;

            // Actual prescale value selected
            let presc = (presc_reg + 1) as u32;

            let sclh = ((ratio / presc) - 2) / 2;
            let scll = sclh;

            // Speed check
            assert!(
                sclh < 256,
                "The I2C PCLK is too fast for this bus frequency!"
            );

            let sdadel = i2cclk / 2_000_000 / presc;
            let scldel = i2cclk / 500_000 / presc - 1;

            (
                presc_reg,
                scll as u8,
                sclh as u8,
                sdadel as u8,
                scldel as u8,
            )
        };

        // Sanity check
        assert!(presc_reg < 16);

        // Keep values within reasonable limits for fast per_ck
        let sdadel = cmp::max(sdadel, 2);
        let scldel = cmp::max(scldel, 4);

        //(presc_reg, scll, sclh, sdadel, scldel)
        Self {
            prescale: presc_reg,
            scll,
            sclh,
            sdadel,
            scldel,
        }
    }
}
