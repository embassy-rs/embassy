//! Serial Peripheral Interface
use core::marker::PhantomData;

use embassy_embedded_hal::SetConfig;
use embassy_futures::join::join;
use embassy_hal_internal::{Peri, PeripheralType};
pub use embedded_hal_02::spi::{Phase, Polarity};

use crate::dma::{AnyChannel, Channel};
use crate::gpio::{AnyPin, Pin as GpioPin, SealedPin as _};
use crate::{pac, peripherals};

/// SPI errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    // No errors for now
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: 1_000_000,
            phase: Phase::CaptureOnFirstTransition,
            polarity: Polarity::IdleLow,
        }
    }
}

/// SPI driver.
pub struct Spi<'d, T: Instance, M: Mode> {
    inner: Peri<'d, T>,
    tx_dma: Option<Peri<'d, AnyChannel>>,
    rx_dma: Option<Peri<'d, AnyChannel>>,
    phantom: PhantomData<(&'d mut T, M)>,
}

fn div_roundup(a: u32, b: u32) -> u32 {
    (a + b - 1) / b
}

fn calc_prescs(freq: u32) -> (u8, u8) {
    let clk_peri = crate::clocks::clk_peri_freq();

    // final SPI frequency: spi_freq = clk_peri / presc / postdiv
    // presc must be in 2..=254, and must be even
    // postdiv must be in 1..=256

    // divide extra by 2, so we get rid of the "presc must be even" requirement
    let ratio = div_roundup(clk_peri, freq * 2);
    if ratio > 127 * 256 {
        panic!("Requested too low SPI frequency");
    }

    let presc = div_roundup(ratio, 256);
    let postdiv = if presc == 1 { ratio } else { div_roundup(ratio, presc) };

    ((presc * 2) as u8, (postdiv - 1) as u8)
}

impl<'d, T: Instance, M: Mode> Spi<'d, T, M> {
    fn new_inner(
        inner: Peri<'d, T>,
        clk: Option<Peri<'d, AnyPin>>,
        mosi: Option<Peri<'d, AnyPin>>,
        miso: Option<Peri<'d, AnyPin>>,
        cs: Option<Peri<'d, AnyPin>>,
        tx_dma: Option<Peri<'d, AnyChannel>>,
        rx_dma: Option<Peri<'d, AnyChannel>>,
        config: Config,
    ) -> Self {
        Self::apply_config(&inner, &config);

        let p = inner.regs();

        // Always enable DREQ signals -- harmless if DMA is not listening
        p.dmacr().write(|reg| {
            reg.set_rxdmae(true);
            reg.set_txdmae(true);
        });

        // finally, enable.
        p.cr1().write(|w| w.set_sse(true));

        if let Some(pin) = &clk {
            pin.gpio().ctrl().write(|w| w.set_funcsel(1));
            pin.pad_ctrl().write(|w| {
                #[cfg(feature = "_rp235x")]
                w.set_iso(false);
                w.set_schmitt(true);
                w.set_slewfast(false);
                w.set_ie(true);
                w.set_od(false);
                w.set_pue(false);
                w.set_pde(false);
            });
        }
        if let Some(pin) = &mosi {
            pin.gpio().ctrl().write(|w| w.set_funcsel(1));
            pin.pad_ctrl().write(|w| {
                #[cfg(feature = "_rp235x")]
                w.set_iso(false);
                w.set_schmitt(true);
                w.set_slewfast(false);
                w.set_ie(true);
                w.set_od(false);
                w.set_pue(false);
                w.set_pde(false);
            });
        }
        if let Some(pin) = &miso {
            pin.gpio().ctrl().write(|w| w.set_funcsel(1));
            pin.pad_ctrl().write(|w| {
                #[cfg(feature = "_rp235x")]
                w.set_iso(false);
                w.set_schmitt(true);
                w.set_slewfast(false);
                w.set_ie(true);
                w.set_od(false);
                w.set_pue(false);
                w.set_pde(false);
            });
        }
        if let Some(pin) = &cs {
            pin.gpio().ctrl().write(|w| w.set_funcsel(1));
            pin.pad_ctrl().write(|w| {
                #[cfg(feature = "_rp235x")]
                w.set_iso(false);
                w.set_schmitt(true);
                w.set_slewfast(false);
                w.set_ie(true);
                w.set_od(false);
                w.set_pue(false);
                w.set_pde(false);
            });
        }
        Self {
            inner,
            tx_dma,
            rx_dma,
            phantom: PhantomData,
        }
    }

    /// Private function to apply SPI configuration (phase, polarity, frequency) settings.
    ///
    /// Driver should be disabled before making changes and reenabled after the modifications
    /// are applied.
    fn apply_config(inner: &Peri<'d, T>, config: &Config) {
        let p = inner.regs();
        let (presc, postdiv) = calc_prescs(config.frequency);

        p.cpsr().write(|w| w.set_cpsdvsr(presc));
        p.cr0().write(|w| {
            w.set_dss(0b0111); // 8bit
            w.set_spo(config.polarity == Polarity::IdleHigh);
            w.set_sph(config.phase == Phase::CaptureOnSecondTransition);
            w.set_scr(postdiv);
        });
    }

    /// Write data to SPI blocking execution until done.
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), Error> {
        let p = self.inner.regs();
        for &b in data {
            while !p.sr().read().tnf() {}
            p.dr().write(|w| w.set_data(b as _));
            while !p.sr().read().rne() {}
            let _ = p.dr().read();
        }
        self.flush()?;
        Ok(())
    }

    /// Transfer data in place to SPI blocking execution until done.
    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        let p = self.inner.regs();
        for b in data {
            while !p.sr().read().tnf() {}
            p.dr().write(|w| w.set_data(*b as _));
            while !p.sr().read().rne() {}
            *b = p.dr().read().data() as u8;
        }
        self.flush()?;
        Ok(())
    }

    /// Read data from SPI blocking execution until done.
    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        let p = self.inner.regs();
        for b in data {
            while !p.sr().read().tnf() {}
            p.dr().write(|w| w.set_data(0));
            while !p.sr().read().rne() {}
            *b = p.dr().read().data() as u8;
        }
        self.flush()?;
        Ok(())
    }

    /// Transfer data to SPI blocking execution until done.
    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        let p = self.inner.regs();
        let len = read.len().max(write.len());
        for i in 0..len {
            let wb = write.get(i).copied().unwrap_or(0);
            while !p.sr().read().tnf() {}
            p.dr().write(|w| w.set_data(wb as _));
            while !p.sr().read().rne() {}
            let rb = p.dr().read().data() as u8;
            if let Some(r) = read.get_mut(i) {
                *r = rb;
            }
        }
        self.flush()?;
        Ok(())
    }

    /// Block execution until SPI is done.
    pub fn flush(&mut self) -> Result<(), Error> {
        let p = self.inner.regs();
        while p.sr().read().bsy() {}
        Ok(())
    }

    /// Set SPI frequency.
    pub fn set_frequency(&mut self, freq: u32) {
        let (presc, postdiv) = calc_prescs(freq);
        let p = self.inner.regs();
        // disable
        p.cr1().write(|w| w.set_sse(false));

        // change stuff
        p.cpsr().write(|w| w.set_cpsdvsr(presc));
        p.cr0().modify(|w| {
            w.set_scr(postdiv);
        });

        // enable
        p.cr1().write(|w| w.set_sse(true));
    }

    /// Set SPI config.
    pub fn set_config(&mut self, config: &Config) {
        let p = self.inner.regs();

        // disable
        p.cr1().write(|w| w.set_sse(false));

        // change stuff
        Self::apply_config(&self.inner, config);

        // enable
        p.cr1().write(|w| w.set_sse(true));
    }
}

impl<'d, T: Instance> Spi<'d, T, Blocking> {
    /// Create an SPI driver in blocking mode.
    pub fn new_blocking(
        inner: Peri<'d, T>,
        clk: Peri<'d, impl ClkPin<T> + 'd>,
        mosi: Peri<'d, impl MosiPin<T> + 'd>,
        miso: Peri<'d, impl MisoPin<T> + 'd>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            inner,
            Some(clk.into()),
            Some(mosi.into()),
            Some(miso.into()),
            None,
            None,
            None,
            config,
        )
    }

    /// Create an SPI driver in blocking mode supporting writes only.
    pub fn new_blocking_txonly(
        inner: Peri<'d, T>,
        clk: Peri<'d, impl ClkPin<T> + 'd>,
        mosi: Peri<'d, impl MosiPin<T> + 'd>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            inner,
            Some(clk.into()),
            Some(mosi.into()),
            None,
            None,
            None,
            None,
            config,
        )
    }

    /// Create an SPI driver in blocking mode supporting reads only.
    pub fn new_blocking_rxonly(
        inner: Peri<'d, T>,
        clk: Peri<'d, impl ClkPin<T> + 'd>,
        miso: Peri<'d, impl MisoPin<T> + 'd>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            inner,
            Some(clk.into()),
            None,
            Some(miso.into()),
            None,
            None,
            None,
            config,
        )
    }
}

impl<'d, T: Instance> Spi<'d, T, Async> {
    /// Create an SPI driver in async mode supporting DMA operations.
    pub fn new(
        inner: Peri<'d, T>,
        clk: Peri<'d, impl ClkPin<T> + 'd>,
        mosi: Peri<'d, impl MosiPin<T> + 'd>,
        miso: Peri<'d, impl MisoPin<T> + 'd>,
        tx_dma: Peri<'d, impl Channel>,
        rx_dma: Peri<'d, impl Channel>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            inner,
            Some(clk.into()),
            Some(mosi.into()),
            Some(miso.into()),
            None,
            Some(tx_dma.into()),
            Some(rx_dma.into()),
            config,
        )
    }

    /// Create an SPI driver in async mode supporting DMA write operations only.
    pub fn new_txonly(
        inner: Peri<'d, T>,
        clk: Peri<'d, impl ClkPin<T> + 'd>,
        mosi: Peri<'d, impl MosiPin<T> + 'd>,
        tx_dma: Peri<'d, impl Channel>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            inner,
            Some(clk.into()),
            Some(mosi.into()),
            None,
            None,
            Some(tx_dma.into()),
            None,
            config,
        )
    }

    /// Create an SPI driver in async mode supporting DMA read operations only.
    pub fn new_rxonly(
        inner: Peri<'d, T>,
        clk: Peri<'d, impl ClkPin<T> + 'd>,
        miso: Peri<'d, impl MisoPin<T> + 'd>,
        tx_dma: Peri<'d, impl Channel>,
        rx_dma: Peri<'d, impl Channel>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            inner,
            Some(clk.into()),
            None,
            Some(miso.into()),
            None,
            Some(tx_dma.into()),
            Some(rx_dma.into()),
            config,
        )
    }

    /// Write data to SPI using DMA.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let tx_ch = self.tx_dma.as_mut().unwrap().reborrow();
        let tx_transfer = unsafe {
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::write(tx_ch, buffer, self.inner.regs().dr().as_ptr() as *mut _, T::TX_DREQ)
        };
        tx_transfer.await;

        let p = self.inner.regs();
        while p.sr().read().bsy() {}

        // clear RX FIFO contents to prevent stale reads
        while p.sr().read().rne() {
            let _: u16 = p.dr().read().data();
        }
        // clear RX overrun interrupt
        p.icr().write(|w| w.set_roric(true));

        Ok(())
    }

    /// Read data from SPI using DMA.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        // Start RX first. Transfer starts when TX starts, if RX
        // is not started yet we might lose bytes.
        let rx_ch = self.rx_dma.as_mut().unwrap().reborrow();
        let rx_transfer = unsafe {
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::read(rx_ch, self.inner.regs().dr().as_ptr() as *const _, buffer, T::RX_DREQ)
        };

        let tx_ch = self.tx_dma.as_mut().unwrap().reborrow();
        let tx_transfer = unsafe {
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::write_repeated(
                tx_ch,
                self.inner.regs().dr().as_ptr() as *mut u8,
                buffer.len(),
                T::TX_DREQ,
            )
        };
        join(tx_transfer, rx_transfer).await;
        Ok(())
    }

    /// Transfer data to SPI using DMA.
    pub async fn transfer(&mut self, rx_buffer: &mut [u8], tx_buffer: &[u8]) -> Result<(), Error> {
        self.transfer_inner(rx_buffer, tx_buffer).await
    }

    /// Transfer data in place to SPI using DMA.
    pub async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Error> {
        self.transfer_inner(words, words).await
    }

    async fn transfer_inner(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(), Error> {
        // Start RX first. Transfer starts when TX starts, if RX
        // is not started yet we might lose bytes.
        let rx_ch = self.rx_dma.as_mut().unwrap().reborrow();
        let rx_transfer = unsafe {
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::read(rx_ch, self.inner.regs().dr().as_ptr() as *const _, rx, T::RX_DREQ)
        };

        let mut tx_ch = self.tx_dma.as_mut().unwrap().reborrow();
        // If we don't assign future to a variable, the data register pointer
        // is held across an await and makes the future non-Send.
        let tx_transfer = async {
            let p = self.inner.regs();
            unsafe {
                crate::dma::write(tx_ch.reborrow(), tx, p.dr().as_ptr() as *mut _, T::TX_DREQ).await;

                if rx.len() > tx.len() {
                    let write_bytes_len = rx.len() - tx.len();
                    // write dummy data
                    // this will disable incrementation of the buffers
                    crate::dma::write_repeated(tx_ch, p.dr().as_ptr() as *mut u8, write_bytes_len, T::TX_DREQ).await
                }
            }
        };
        join(tx_transfer, rx_transfer).await;

        // if tx > rx we should clear any overflow of the FIFO SPI buffer
        if tx.len() > rx.len() {
            let p = self.inner.regs();
            while p.sr().read().bsy() {}

            // clear RX FIFO contents to prevent stale reads
            while p.sr().read().rne() {
                let _: u16 = p.dr().read().data();
            }
            // clear RX overrun interrupt
            p.icr().write(|w| w.set_roric(true));
        }

        Ok(())
    }
}

trait SealedMode {}

trait SealedInstance {
    const TX_DREQ: pac::dma::vals::TreqSel;
    const RX_DREQ: pac::dma::vals::TreqSel;

    fn regs(&self) -> pac::spi::Spi;
}

/// Mode.
#[allow(private_bounds)]
pub trait Mode: SealedMode {}

/// SPI instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {}

macro_rules! impl_instance {
    ($type:ident, $irq:ident, $tx_dreq:expr, $rx_dreq:expr) => {
        impl SealedInstance for peripherals::$type {
            const TX_DREQ: pac::dma::vals::TreqSel = $tx_dreq;
            const RX_DREQ: pac::dma::vals::TreqSel = $rx_dreq;

            fn regs(&self) -> pac::spi::Spi {
                pac::$type
            }
        }
        impl Instance for peripherals::$type {}
    };
}

impl_instance!(
    SPI0,
    Spi0,
    pac::dma::vals::TreqSel::SPI0_TX,
    pac::dma::vals::TreqSel::SPI0_RX
);
impl_instance!(
    SPI1,
    Spi1,
    pac::dma::vals::TreqSel::SPI1_TX,
    pac::dma::vals::TreqSel::SPI1_RX
);

/// CLK pin.
pub trait ClkPin<T: Instance>: GpioPin {}
/// CS pin.
pub trait CsPin<T: Instance>: GpioPin {}
/// MOSI pin.
pub trait MosiPin<T: Instance>: GpioPin {}
/// MISO pin.
pub trait MisoPin<T: Instance>: GpioPin {}

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, $function:ident) => {
        impl $function<peripherals::$instance> for peripherals::$pin {}
    };
}

impl_pin!(PIN_0, SPI0, MisoPin);
impl_pin!(PIN_1, SPI0, CsPin);
impl_pin!(PIN_2, SPI0, ClkPin);
impl_pin!(PIN_3, SPI0, MosiPin);
impl_pin!(PIN_4, SPI0, MisoPin);
impl_pin!(PIN_5, SPI0, CsPin);
impl_pin!(PIN_6, SPI0, ClkPin);
impl_pin!(PIN_7, SPI0, MosiPin);
impl_pin!(PIN_8, SPI1, MisoPin);
impl_pin!(PIN_9, SPI1, CsPin);
impl_pin!(PIN_10, SPI1, ClkPin);
impl_pin!(PIN_11, SPI1, MosiPin);
impl_pin!(PIN_12, SPI1, MisoPin);
impl_pin!(PIN_13, SPI1, CsPin);
impl_pin!(PIN_14, SPI1, ClkPin);
impl_pin!(PIN_15, SPI1, MosiPin);
impl_pin!(PIN_16, SPI0, MisoPin);
impl_pin!(PIN_17, SPI0, CsPin);
impl_pin!(PIN_18, SPI0, ClkPin);
impl_pin!(PIN_19, SPI0, MosiPin);
impl_pin!(PIN_20, SPI0, MisoPin);
impl_pin!(PIN_21, SPI0, CsPin);
impl_pin!(PIN_22, SPI0, ClkPin);
impl_pin!(PIN_23, SPI0, MosiPin);
impl_pin!(PIN_24, SPI1, MisoPin);
impl_pin!(PIN_25, SPI1, CsPin);
impl_pin!(PIN_26, SPI1, ClkPin);
impl_pin!(PIN_27, SPI1, MosiPin);
impl_pin!(PIN_28, SPI1, MisoPin);
impl_pin!(PIN_29, SPI1, CsPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_30, SPI1, ClkPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_31, SPI1, MosiPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_32, SPI0, MisoPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_33, SPI0, CsPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_34, SPI0, ClkPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_35, SPI0, MosiPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_36, SPI0, MisoPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_37, SPI0, CsPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_38, SPI0, ClkPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_39, SPI0, MosiPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_40, SPI1, MisoPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_41, SPI1, CsPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_42, SPI1, ClkPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_43, SPI1, MosiPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_44, SPI1, MisoPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_45, SPI1, CsPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_46, SPI1, ClkPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_47, SPI1, MosiPin);

macro_rules! impl_mode {
    ($name:ident) => {
        impl SealedMode for $name {}
        impl Mode for $name {}
    };
}

/// Blocking mode.
pub struct Blocking;
/// Async mode.
pub struct Async;

impl_mode!(Blocking);
impl_mode!(Async);

// ====================

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::spi::Transfer<u8> for Spi<'d, T, M> {
    type Error = Error;
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.blocking_transfer_in_place(words)?;
        Ok(words)
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::spi::Write<u8> for Spi<'d, T, M> {
    type Error = Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(words)
    }
}

impl embedded_hal_1::spi::Error for Error {
    fn kind(&self) -> embedded_hal_1::spi::ErrorKind {
        match *self {}
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_1::spi::ErrorType for Spi<'d, T, M> {
    type Error = Error;
}

impl<'d, T: Instance, M: Mode> embedded_hal_1::spi::SpiBus<u8> for Spi<'d, T, M> {
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_transfer(words, &[])
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(words)
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.blocking_transfer(read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_transfer_in_place(words)
    }
}

impl<'d, T: Instance> embedded_hal_async::spi::SpiBus<u8> for Spi<'d, T, Async> {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.write(words).await
    }

    async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.read(words).await
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.transfer(read, write).await
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.transfer_in_place(words).await
    }
}

impl<'d, T: Instance, M: Mode> SetConfig for Spi<'d, T, M> {
    type Config = Config;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), ()> {
        self.set_config(config);

        Ok(())
    }
}
