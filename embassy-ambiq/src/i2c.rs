//! I2C driver

use embassy_hal_internal::Peri;

use crate::gpio::SealedConfigure;
use crate::iom::{Instance, Iom};
use crate::pac;

// I2C Pin Marker Traits
#[allow(private_bounds)]
pub trait SdaPin<T: Instance>: crate::gpio::Pin + SealedConfigure {
    const FNCSEL: u8;
}
#[allow(private_bounds)]
pub trait SclPin<T: Instance>: crate::gpio::Pin + SealedConfigure {
    const FNCSEL: u8;
}

macro_rules! impl_i2c_pin {
    ($pin:ident, $iom:ident, $fncsel:expr, $trait:ident) => {
        impl $trait<crate::peripherals::$iom> for crate::peripherals::$pin {
            const FNCSEL: u8 = $fncsel;
        }
    };
}

// IOM0
impl_i2c_pin!(P5, IOM0, 0, SclPin);
impl_i2c_pin!(P6, IOM0, 0, SdaPin);

// IOM1
impl_i2c_pin!(P8, IOM1, 0, SclPin);
impl_i2c_pin!(P9, IOM1, 0, SdaPin);

// IOM2
impl_i2c_pin!(P27, IOM2, 4, SclPin);
impl_i2c_pin!(P25, IOM2, 4, SdaPin);

// IOM3
impl_i2c_pin!(P42, IOM3, 4, SclPin);
impl_i2c_pin!(P43, IOM3, 4, SdaPin);

// IOM4
impl_i2c_pin!(P39, IOM4, 4, SclPin);
impl_i2c_pin!(P40, IOM4, 4, SdaPin);

// IOM5
impl_i2c_pin!(P48, IOM5, 4, SclPin);
impl_i2c_pin!(P49, IOM5, 4, SdaPin);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Frequency {
    Freq100kHz,
    Freq400kHz,
    Freq1MHz,
}

/// Configuration for the I2C driver
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Config {
    /// I2C operating frequency
    pub frequency: Frequency,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::Freq100kHz,
        }
    }
}

pub struct I2c<'d, T: Instance> {
    _iom: Iom<'d, T>,
}

async fn wait_complete<T: Instance>() -> Result<(), Error> {
    use pac::shared::Cmdstat;
    let regs = T::regs();
    let state = T::state();

    core::future::poll_fn(|cx| {
        state.waker.register(cx.waker());
        let s = regs.cmdstat().read().cmdstat();
        match s {
            Cmdstat::Idle | Cmdstat::Err => {
                regs.intclr().write(|w| w.set_cmdcmp(true));
                core::task::Poll::Ready(())
            }
            _ => {
                regs.inten().modify(|w| w.set_cmdcmp(true));
                core::task::Poll::Pending
            }
        }
    })
    .await;

    let intstat = regs.intstat().read();

    // Clear errors so they don't persist to the next transaction
    regs.intclr().write(|w| {
        w.set_nak(true);
        w.set_fovfl(true);
        w.set_fundfl(true);
        w.set_iacc(true);
        w.set_icmd(true);
    });

    if intstat.nak() {
        return Err(Error::NoAcknowledge);
    }
    if intstat.fovfl() {
        return Err(Error::Overflow);
    }
    if intstat.fundfl() {
        return Err(Error::Underflow);
    }
    if intstat.iacc() {
        return Err(Error::IllegalAccess);
    }
    if intstat.icmd() {
        return Err(Error::IllegalCommand);
    }

    Ok(())
}

impl<'d, T: Instance> I2c<'d, T> {
    pub fn new<Scl: SclPin<T>, Sda: SdaPin<T>>(
        peri: Peri<'d, T>,
        scl: Peri<'d, Scl>,
        sda: Peri<'d, Sda>,
        _config: Config,
    ) -> Self {
        // Configure pins for I2C (pullups + input enable + fncsel)
        scl.configure_i2c_pad(Scl::FNCSEL);
        sda.configure_i2c_pad(Sda::FNCSEL);

        let iom = Iom::new(peri);
        let regs = T::regs();

        // Disable IOM first
        regs.submodctrl().write(|w| {
            w.set_smod0en(false);
            w.set_smod1en(false);
        });

        // Set frequency (Simplified)
        // HFRC 48MHz / 8 = 6MHz source. The IOM divider produces an SCL period of
        // 2*(TOTPER+1) source clocks, so TOTPER+1 = 30 -> 6MHz/60 = 100kHz.
        // LOWPER+1 = 15 -> ~50% duty.
        regs.clkcfg().write(|w| {
            w.set_ioclken(true);
            w.set_diven(true);
            w.set_fsel(pac::shared::Fsel::HfrcDiv8); // 48MHz / 8 = 6MHz
            w.set_totper(29); // SCL = 6MHz / (2*30) = 100kHz
            w.set_lowper(14); // ~50% duty
        });

        // Enable IOM (I2C Master is smod1)
        regs.submodctrl().modify(|w| w.set_smod1en(true));

        Self { _iom: iom }
    }
}

impl<'d, T: Instance> embedded_hal_async::i2c::I2c for I2c<'d, T> {
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        let regs = T::regs();
        regs.devcfg().write(|w| w.set_devaddr(address as u16));

        let mut offset = 0;
        while offset < read.len() {
            let chunk_size = core::cmp::min(read.len() - offset, 32);
            let is_last = offset + chunk_size == read.len();

            regs.cmd().write(|w| {
                w.set_cmd(pac::iom0::vals::Cmd::Read);
                w.set_tsize(chunk_size as u16);
                w.set_cont(!is_last);
            });

            wait_complete::<T>().await?;

            for chunk_words in read[offset..offset + chunk_size].chunks_mut(4) {
                let word = regs.fifopop().read();
                for (i, b) in chunk_words.iter_mut().enumerate() {
                    *b = (word >> (i * 8)) as u8;
                }
            }

            offset += chunk_size;
        }

        Ok(())
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        let regs = T::regs();
        regs.devcfg().write(|w| w.set_devaddr(address as u16));

        if write.is_empty() {
            // Handle empty write (e.g. for bus scanning)
            regs.cmd().write(|w| {
                w.set_cmd(pac::iom0::vals::Cmd::Write);
                w.set_tsize(0);
                w.set_cont(false);
            });
            wait_complete::<T>().await?;
            return Ok(());
        }

        let mut offset = 0;
        while offset < write.len() {
            let chunk_size = core::cmp::min(write.len() - offset, 32);
            let chunk = &write[offset..offset + chunk_size];

            for chunk_words in chunk.chunks(4) {
                let mut word: u32 = 0;
                for (i, &b) in chunk_words.iter().enumerate() {
                    word |= (b as u32) << (i * 8);
                }
                regs.fifopush().write_value(word);
            }

            let is_last = offset + chunk_size == write.len();

            regs.cmd().write(|w| {
                w.set_cmd(pac::iom0::vals::Cmd::Write);
                w.set_tsize(chunk_size as u16);
                w.set_cont(!is_last);
            });

            wait_complete::<T>().await?;

            offset += chunk_size;
        }

        Ok(())
    }

    async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        let regs = T::regs();
        regs.devcfg().write(|w| w.set_devaddr(address as u16));

        // Write Phase (cont = true so we issue a repeated START before the read)
        let mut offset = 0;
        while offset < write.len() {
            let chunk_size = core::cmp::min(write.len() - offset, 32);
            let chunk = &write[offset..offset + chunk_size];

            for chunk_words in chunk.chunks(4) {
                let mut word: u32 = 0;
                for (i, &b) in chunk_words.iter().enumerate() {
                    word |= (b as u32) << (i * 8);
                }
                regs.fifopush().write_value(word);
            }

            regs.cmd().write(|w| {
                w.set_cmd(pac::iom0::vals::Cmd::Write);
                w.set_tsize(chunk_size as u16);
                w.set_cont(true);
            });

            wait_complete::<T>().await?;

            offset += chunk_size;
        }

        // Read Phase
        offset = 0;
        while offset < read.len() {
            let chunk_size = core::cmp::min(read.len() - offset, 32);
            let is_last = offset + chunk_size == read.len();

            regs.cmd().write(|w| {
                w.set_cmd(pac::iom0::vals::Cmd::Read);
                w.set_tsize(chunk_size as u16);
                w.set_cont(!is_last);
            });

            wait_complete::<T>().await?;

            for chunk_words in read[offset..offset + chunk_size].chunks_mut(4) {
                let word = regs.fifopop().read();
                for (i, b) in chunk_words.iter_mut().enumerate() {
                    *b = (word >> (i * 8)) as u8;
                }
            }

            offset += chunk_size;
        }

        Ok(())
    }

    async fn transaction(
        &mut self,
        _address: u8,
        _operations: &mut [embedded_hal_async::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        // Not implemented for now
        unimplemented!()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    NoAcknowledge,
    Overflow,
    Underflow,
    IllegalAccess,
    IllegalCommand,
    ArbitrationLoss,
    BusError,
}

impl embedded_hal_async::i2c::Error for Error {
    fn kind(&self) -> embedded_hal_async::i2c::ErrorKind {
        use embedded_hal_async::i2c::ErrorKind;
        match self {
            Error::NoAcknowledge => ErrorKind::NoAcknowledge(embedded_hal_async::i2c::NoAcknowledgeSource::Unknown),
            Error::Overflow | Error::Underflow | Error::IllegalAccess => ErrorKind::Overrun,
            Error::ArbitrationLoss => ErrorKind::ArbitrationLoss,
            Error::BusError | Error::IllegalCommand => ErrorKind::Other,
        }
    }
}

impl<'d, T: Instance> embedded_hal_async::i2c::ErrorType for I2c<'d, T> {
    type Error = Error;
}
