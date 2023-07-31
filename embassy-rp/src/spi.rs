//! Serial Peripheral Interface
use core::marker::PhantomData;

use embassy_embedded_hal::SetConfig;
use embassy_futures::join::join;
use embassy_hal_internal::{into_ref, PeripheralRef};
pub use embedded_hal_02::spi::{Phase, Polarity};

use crate::dma::{AnyChannel, Channel};
use crate::gpio::sealed::Pin as _;
use crate::gpio::{AnyPin, Pin as GpioPin};
use crate::{pac, peripherals, Peripheral};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    // No errors for now
}

#[non_exhaustive]
#[derive(Clone)]
pub struct Config {
    pub frequency: u32,
    pub phase: Phase,
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

pub struct Spi<'d, T: Instance, M: Mode> {
    inner: PeripheralRef<'d, T>,
    tx_dma: Option<PeripheralRef<'d, AnyChannel>>,
    rx_dma: Option<PeripheralRef<'d, AnyChannel>>,
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
        inner: impl Peripheral<P = T> + 'd,
        clk: Option<PeripheralRef<'d, AnyPin>>,
        mosi: Option<PeripheralRef<'d, AnyPin>>,
        miso: Option<PeripheralRef<'d, AnyPin>>,
        cs: Option<PeripheralRef<'d, AnyPin>>,
        tx_dma: Option<PeripheralRef<'d, AnyChannel>>,
        rx_dma: Option<PeripheralRef<'d, AnyChannel>>,
        config: Config,
    ) -> Self {
        into_ref!(inner);

        let p = inner.regs();
        let (presc, postdiv) = calc_prescs(config.frequency);

        p.cpsr().write(|w| w.set_cpsdvsr(presc));
        p.cr0().write(|w| {
            w.set_dss(0b0111); // 8bit
            w.set_spo(config.polarity == Polarity::IdleHigh);
            w.set_sph(config.phase == Phase::CaptureOnSecondTransition);
            w.set_scr(postdiv);
        });

        // Always enable DREQ signals -- harmless if DMA is not listening
        p.dmacr().write(|reg| {
            reg.set_rxdmae(true);
            reg.set_txdmae(true);
        });

        // finally, enable.
        p.cr1().write(|w| w.set_sse(true));

        if let Some(pin) = &clk {
            pin.gpio().ctrl().write(|w| w.set_funcsel(1));
        }
        if let Some(pin) = &mosi {
            pin.gpio().ctrl().write(|w| w.set_funcsel(1));
        }
        if let Some(pin) = &miso {
            pin.gpio().ctrl().write(|w| w.set_funcsel(1));
        }
        if let Some(pin) = &cs {
            pin.gpio().ctrl().write(|w| w.set_funcsel(1));
        }
        Self {
            inner,
            tx_dma,
            rx_dma,
            phantom: PhantomData,
        }
    }

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

    pub fn flush(&mut self) -> Result<(), Error> {
        let p = self.inner.regs();
        while p.sr().read().bsy() {}
        Ok(())
    }

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
}

impl<'d, T: Instance> Spi<'d, T, Blocking> {
    pub fn new_blocking(
        inner: impl Peripheral<P = T> + 'd,
        clk: impl Peripheral<P = impl ClkPin<T> + 'd> + 'd,
        mosi: impl Peripheral<P = impl MosiPin<T> + 'd> + 'd,
        miso: impl Peripheral<P = impl MisoPin<T> + 'd> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(clk, mosi, miso);
        Self::new_inner(
            inner,
            Some(clk.map_into()),
            Some(mosi.map_into()),
            Some(miso.map_into()),
            None,
            None,
            None,
            config,
        )
    }

    pub fn new_blocking_txonly(
        inner: impl Peripheral<P = T> + 'd,
        clk: impl Peripheral<P = impl ClkPin<T> + 'd> + 'd,
        mosi: impl Peripheral<P = impl MosiPin<T> + 'd> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(clk, mosi);
        Self::new_inner(
            inner,
            Some(clk.map_into()),
            Some(mosi.map_into()),
            None,
            None,
            None,
            None,
            config,
        )
    }

    pub fn new_blocking_rxonly(
        inner: impl Peripheral<P = T> + 'd,
        clk: impl Peripheral<P = impl ClkPin<T> + 'd> + 'd,
        miso: impl Peripheral<P = impl MisoPin<T> + 'd> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(clk, miso);
        Self::new_inner(
            inner,
            Some(clk.map_into()),
            None,
            Some(miso.map_into()),
            None,
            None,
            None,
            config,
        )
    }
}

impl<'d, T: Instance> Spi<'d, T, Async> {
    pub fn new(
        inner: impl Peripheral<P = T> + 'd,
        clk: impl Peripheral<P = impl ClkPin<T> + 'd> + 'd,
        mosi: impl Peripheral<P = impl MosiPin<T> + 'd> + 'd,
        miso: impl Peripheral<P = impl MisoPin<T> + 'd> + 'd,
        tx_dma: impl Peripheral<P = impl Channel> + 'd,
        rx_dma: impl Peripheral<P = impl Channel> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(tx_dma, rx_dma, clk, mosi, miso);
        Self::new_inner(
            inner,
            Some(clk.map_into()),
            Some(mosi.map_into()),
            Some(miso.map_into()),
            None,
            Some(tx_dma.map_into()),
            Some(rx_dma.map_into()),
            config,
        )
    }

    pub fn new_txonly(
        inner: impl Peripheral<P = T> + 'd,
        clk: impl Peripheral<P = impl ClkPin<T> + 'd> + 'd,
        mosi: impl Peripheral<P = impl MosiPin<T> + 'd> + 'd,
        tx_dma: impl Peripheral<P = impl Channel> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(tx_dma, clk, mosi);
        Self::new_inner(
            inner,
            Some(clk.map_into()),
            Some(mosi.map_into()),
            None,
            None,
            Some(tx_dma.map_into()),
            None,
            config,
        )
    }

    pub fn new_rxonly(
        inner: impl Peripheral<P = T> + 'd,
        clk: impl Peripheral<P = impl ClkPin<T> + 'd> + 'd,
        miso: impl Peripheral<P = impl MisoPin<T> + 'd> + 'd,
        rx_dma: impl Peripheral<P = impl Channel> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(rx_dma, clk, miso);
        Self::new_inner(
            inner,
            Some(clk.map_into()),
            None,
            Some(miso.map_into()),
            None,
            None,
            Some(rx_dma.map_into()),
            config,
        )
    }

    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let tx_ch = self.tx_dma.as_mut().unwrap();
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

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        // Start RX first. Transfer starts when TX starts, if RX
        // is not started yet we might lose bytes.
        let rx_ch = self.rx_dma.as_mut().unwrap();
        let rx_transfer = unsafe {
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::read(rx_ch, self.inner.regs().dr().as_ptr() as *const _, buffer, T::RX_DREQ)
        };

        let tx_ch = self.tx_dma.as_mut().unwrap();
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

    pub async fn transfer(&mut self, rx_buffer: &mut [u8], tx_buffer: &[u8]) -> Result<(), Error> {
        self.transfer_inner(rx_buffer, tx_buffer).await
    }

    pub async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Error> {
        self.transfer_inner(words, words).await
    }

    async fn transfer_inner(&mut self, rx_ptr: *mut [u8], tx_ptr: *const [u8]) -> Result<(), Error> {
        let (_, tx_len) = crate::dma::slice_ptr_parts(tx_ptr);
        let (_, rx_len) = crate::dma::slice_ptr_parts_mut(rx_ptr);

        // Start RX first. Transfer starts when TX starts, if RX
        // is not started yet we might lose bytes.
        let rx_ch = self.rx_dma.as_mut().unwrap();
        let rx_transfer = unsafe {
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::read(rx_ch, self.inner.regs().dr().as_ptr() as *const _, rx_ptr, T::RX_DREQ)
        };

        let mut tx_ch = self.tx_dma.as_mut().unwrap();
        // If we don't assign future to a variable, the data register pointer
        // is held across an await and makes the future non-Send.
        let tx_transfer = async {
            let p = self.inner.regs();
            unsafe {
                crate::dma::write(&mut tx_ch, tx_ptr, p.dr().as_ptr() as *mut _, T::TX_DREQ).await;

                if rx_len > tx_len {
                    let write_bytes_len = rx_len - tx_len;
                    // write dummy data
                    // this will disable incrementation of the buffers
                    crate::dma::write_repeated(tx_ch, p.dr().as_ptr() as *mut u8, write_bytes_len, T::TX_DREQ).await
                }
            }
        };
        join(tx_transfer, rx_transfer).await;

        // if tx > rx we should clear any overflow of the FIFO SPI buffer
        if tx_len > rx_len {
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

mod sealed {
    use super::*;

    pub trait Mode {}

    pub trait Instance {
        const TX_DREQ: u8;
        const RX_DREQ: u8;

        fn regs(&self) -> pac::spi::Spi;
    }
}

pub trait Mode: sealed::Mode {}
pub trait Instance: sealed::Instance {}

macro_rules! impl_instance {
    ($type:ident, $irq:ident, $tx_dreq:expr, $rx_dreq:expr) => {
        impl sealed::Instance for peripherals::$type {
            const TX_DREQ: u8 = $tx_dreq;
            const RX_DREQ: u8 = $rx_dreq;

            fn regs(&self) -> pac::spi::Spi {
                pac::$type
            }
        }
        impl Instance for peripherals::$type {}
    };
}

impl_instance!(SPI0, Spi0, 16, 17);
impl_instance!(SPI1, Spi1, 18, 19);

pub trait ClkPin<T: Instance>: GpioPin {}
pub trait CsPin<T: Instance>: GpioPin {}
pub trait MosiPin<T: Instance>: GpioPin {}
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

macro_rules! impl_mode {
    ($name:ident) => {
        impl sealed::Mode for $name {}
        impl Mode for $name {}
    };
}

pub struct Blocking;
pub struct Async;

impl_mode!(Blocking);
impl_mode!(Async);

// ====================

mod eh02 {
    use super::*;

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
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

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
}

#[cfg(all(feature = "unstable-traits", feature = "nightly"))]
mod eha {
    use super::*;

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
}

impl<'d, T: Instance, M: Mode> SetConfig for Spi<'d, T, M> {
    type Config = Config;
    fn set_config(&mut self, config: &Self::Config) {
        let p = self.inner.regs();
        let (presc, postdiv) = calc_prescs(config.frequency);
        p.cpsr().write(|w| w.set_cpsdvsr(presc));
        p.cr0().write(|w| {
            w.set_dss(0b0111); // 8bit
            w.set_spo(config.polarity == Polarity::IdleHigh);
            w.set_sph(config.phase == Phase::CaptureOnSecondTransition);
            w.set_scr(postdiv);
        });
    }
}
