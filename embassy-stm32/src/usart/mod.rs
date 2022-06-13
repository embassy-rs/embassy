#![macro_use]

use core::marker::PhantomData;

use embassy_hal_common::unborrow;

use crate::dma::NoDma;
use crate::gpio::sealed::AFType;
use crate::interrupt::Interrupt;
use crate::pac::usart::{regs, vals};
use crate::rcc::RccPeripheral;
use crate::{peripherals, Unborrow};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DataBits {
    DataBits8,
    DataBits9,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Parity {
    ParityNone,
    ParityEven,
    ParityOdd,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StopBits {
    #[doc = "1 stop bit"]
    STOP1,
    #[doc = "0.5 stop bits"]
    STOP0P5,
    #[doc = "2 stop bits"]
    STOP2,
    #[doc = "1.5 stop bits"]
    STOP1P5,
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Config {
    pub baudrate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::STOP1,
            parity: Parity::ParityNone,
        }
    }
}

/// Serial error
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// RX buffer overrun
    Overrun,
    /// Parity check error
    Parity,
}

pub struct Uart<'d, T: Instance, TxDma = NoDma, RxDma = NoDma> {
    phantom: PhantomData<&'d mut T>,
    tx: UartTx<'d, T, TxDma>,
    rx: UartRx<'d, T, RxDma>,
}

pub struct UartTx<'d, T: Instance, TxDma = NoDma> {
    phantom: PhantomData<&'d mut T>,
    tx_dma: TxDma,
}

pub struct UartRx<'d, T: Instance, RxDma = NoDma> {
    phantom: PhantomData<&'d mut T>,
    rx_dma: RxDma,
}

impl<'d, T: Instance, TxDma> UartTx<'d, T, TxDma> {
    fn new(tx_dma: TxDma) -> Self {
        Self {
            tx_dma,
            phantom: PhantomData,
        }
    }

    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error>
    where
        TxDma: crate::usart::TxDma<T>,
    {
        let ch = &mut self.tx_dma;
        let request = ch.request();
        unsafe {
            T::regs().cr3().modify(|reg| {
                reg.set_dmat(true);
            });
        }
        // If we don't assign future to a variable, the data register pointer
        // is held across an await and makes the future non-Send.
        let transfer = crate::dma::write(ch, request, buffer, tdr(T::regs()));
        transfer.await;
        Ok(())
    }

    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        unsafe {
            let r = T::regs();
            for &b in buffer {
                while !sr(r).read().txe() {}
                tdr(r).write_volatile(b);
            }
        }
        Ok(())
    }

    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        unsafe {
            let r = T::regs();
            while !sr(r).read().tc() {}
        }
        Ok(())
    }
}

impl<'d, T: Instance, RxDma> UartRx<'d, T, RxDma> {
    fn new(rx_dma: RxDma) -> Self {
        Self {
            rx_dma,
            phantom: PhantomData,
        }
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error>
    where
        RxDma: crate::usart::RxDma<T>,
    {
        let ch = &mut self.rx_dma;
        let request = ch.request();
        unsafe {
            T::regs().cr3().modify(|reg| {
                reg.set_dmar(true);
            });
        }
        // If we don't assign future to a variable, the data register pointer
        // is held across an await and makes the future non-Send.
        let transfer = crate::dma::read(ch, request, rdr(T::regs()), buffer);
        transfer.await;
        Ok(())
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        unsafe {
            let r = T::regs();
            for b in buffer {
                loop {
                    let sr = sr(r).read();
                    if sr.pe() {
                        rdr(r).read_volatile();
                        return Err(Error::Parity);
                    } else if sr.fe() {
                        rdr(r).read_volatile();
                        return Err(Error::Framing);
                    } else if sr.ne() {
                        rdr(r).read_volatile();
                        return Err(Error::Noise);
                    } else if sr.ore() {
                        rdr(r).read_volatile();
                        return Err(Error::Overrun);
                    } else if sr.rxne() {
                        break;
                    }
                }
                *b = rdr(r).read_volatile();
            }
        }
        Ok(())
    }
}

impl<'d, T: Instance, TxDma, RxDma> Uart<'d, T, TxDma, RxDma> {
    pub fn new(
        _inner: impl Unborrow<Target = T> + 'd,
        rx: impl Unborrow<Target = impl RxPin<T>> + 'd,
        tx: impl Unborrow<Target = impl TxPin<T>> + 'd,
        tx_dma: impl Unborrow<Target = TxDma> + 'd,
        rx_dma: impl Unborrow<Target = RxDma> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(_inner, rx, tx, tx_dma, rx_dma);

        T::enable();
        T::reset();
        let pclk_freq = T::frequency();

        // TODO: better calculation, including error checking and OVER8 if possible.
        let div = (pclk_freq.0 + (config.baudrate / 2)) / config.baudrate;

        let r = T::regs();

        unsafe {
            rx.set_as_af(rx.af_num(), AFType::Input);
            tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

            r.cr2().write(|_w| {});
            r.cr3().write(|_w| {});
            r.brr().write_value(regs::Brr(div));
            r.cr1().write(|w| {
                w.set_ue(true);
                w.set_te(true);
                w.set_re(true);
                w.set_m0(if config.parity != Parity::ParityNone {
                    vals::M0::BIT9
                } else {
                    vals::M0::BIT8
                });
                w.set_pce(config.parity != Parity::ParityNone);
                w.set_ps(match config.parity {
                    Parity::ParityOdd => vals::Ps::ODD,
                    Parity::ParityEven => vals::Ps::EVEN,
                    _ => vals::Ps::EVEN,
                });
            });
        }

        Self {
            phantom: PhantomData,
            tx: UartTx::new(tx_dma),
            rx: UartRx::new(rx_dma),
        }
    }

    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error>
    where
        TxDma: crate::usart::TxDma<T>,
    {
        self.tx.write(buffer).await
    }

    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error>
    where
        RxDma: crate::usart::RxDma<T>,
    {
        self.rx.read(buffer).await
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    /// Split the Uart into a transmitter and receiver, which is
    /// particuarly useful when having two tasks correlating to
    /// transmitting and receiving.
    pub fn split(self) -> (UartTx<'d, T, TxDma>, UartRx<'d, T, RxDma>) {
        (self.tx, self.rx)
    }
}

mod eh02 {
    use super::*;

    impl<'d, T: Instance, RxDma> embedded_hal_02::serial::Read<u8> for UartRx<'d, T, RxDma> {
        type Error = Error;
        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            let r = T::regs();
            unsafe {
                let sr = sr(r).read();
                if sr.pe() {
                    rdr(r).read_volatile();
                    Err(nb::Error::Other(Error::Parity))
                } else if sr.fe() {
                    rdr(r).read_volatile();
                    Err(nb::Error::Other(Error::Framing))
                } else if sr.ne() {
                    rdr(r).read_volatile();
                    Err(nb::Error::Other(Error::Noise))
                } else if sr.ore() {
                    rdr(r).read_volatile();
                    Err(nb::Error::Other(Error::Overrun))
                } else if sr.rxne() {
                    Ok(rdr(r).read_volatile())
                } else {
                    Err(nb::Error::WouldBlock)
                }
            }
        }
    }

    impl<'d, T: Instance, TxDma> embedded_hal_02::blocking::serial::Write<u8> for UartTx<'d, T, TxDma> {
        type Error = Error;
        fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }
        fn bflush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }

    impl<'d, T: Instance, TxDma, RxDma> embedded_hal_02::serial::Read<u8> for Uart<'d, T, TxDma, RxDma> {
        type Error = Error;
        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            embedded_hal_02::serial::Read::read(&mut self.rx)
        }
    }

    impl<'d, T: Instance, TxDma, RxDma> embedded_hal_02::blocking::serial::Write<u8> for Uart<'d, T, TxDma, RxDma> {
        type Error = Error;
        fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }
        fn bflush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl embedded_hal_1::serial::Error for Error {
        fn kind(&self) -> embedded_hal_1::serial::ErrorKind {
            match *self {
                Self::Framing => embedded_hal_1::serial::ErrorKind::FrameFormat,
                Self::Noise => embedded_hal_1::serial::ErrorKind::Noise,
                Self::Overrun => embedded_hal_1::serial::ErrorKind::Overrun,
                Self::Parity => embedded_hal_1::serial::ErrorKind::Parity,
            }
        }
    }

    impl<'d, T: Instance, TxDma, RxDma> embedded_hal_1::serial::ErrorType for Uart<'d, T, TxDma, RxDma> {
        type Error = Error;
    }

    impl<'d, T: Instance, TxDma> embedded_hal_1::serial::ErrorType for UartTx<'d, T, TxDma> {
        type Error = Error;
    }

    impl<'d, T: Instance, RxDma> embedded_hal_1::serial::ErrorType for UartRx<'d, T, RxDma> {
        type Error = Error;
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(feature = "unstable-traits", feature = "nightly", feature = "_todo_embedded_hal_serial"))] {
        use core::future::Future;

        impl<'d, T: Instance, TxDma> embedded_hal_async::serial::Write for UartTx<'d, T, TxDma>
        where
            TxDma: crate::usart::TxDma<T>,
        {
            type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
                self.write(buf)
            }

            type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
                async move { Ok(()) }
            }
        }

        impl<'d, T: Instance, RxDma> embedded_hal_async::serial::Read for UartRx<'d, T, RxDma>
        where
            RxDma: crate::usart::RxDma<T>,
        {
            type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
                self.read(buf)
            }
        }

        impl<'d, T: Instance, TxDma, RxDma> embedded_hal_async::serial::Write for Uart<'d, T, TxDma, RxDma>
        where
            TxDma: crate::usart::TxDma<T>,
        {
            type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
                self.write(buf)
            }

            type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
                async move { Ok(()) }
            }
        }

        impl<'d, T: Instance, TxDma, RxDma> embedded_hal_async::serial::Read for Uart<'d, T, TxDma, RxDma>
        where
            RxDma: crate::usart::RxDma<T>,
        {
            type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
                self.read(buf)
            }
        }
    }
}

#[cfg(feature = "nightly")]
pub use buffered::*;
#[cfg(feature = "nightly")]
mod buffered;

#[cfg(usart_v1)]
fn tdr(r: crate::pac::usart::Usart) -> *mut u8 {
    r.dr().ptr() as _
}

#[cfg(usart_v1)]
fn rdr(r: crate::pac::usart::Usart) -> *mut u8 {
    r.dr().ptr() as _
}

#[cfg(usart_v1)]
fn sr(r: crate::pac::usart::Usart) -> crate::pac::common::Reg<regs::Sr, crate::pac::common::RW> {
    r.sr()
}

#[cfg(usart_v1)]
#[allow(unused)]
unsafe fn clear_interrupt_flags(_r: crate::pac::usart::Usart, _sr: regs::Sr) {
    // On v1 the flags are cleared implicitly by reads and writes to DR.
}

#[cfg(usart_v2)]
fn tdr(r: crate::pac::usart::Usart) -> *mut u8 {
    r.tdr().ptr() as _
}

#[cfg(usart_v2)]
fn rdr(r: crate::pac::usart::Usart) -> *mut u8 {
    r.rdr().ptr() as _
}

#[cfg(usart_v2)]
fn sr(r: crate::pac::usart::Usart) -> crate::pac::common::Reg<regs::Ixr, crate::pac::common::R> {
    r.isr()
}

#[cfg(usart_v2)]
#[allow(unused)]
unsafe fn clear_interrupt_flags(r: crate::pac::usart::Usart, sr: regs::Ixr) {
    r.icr().write(|w| *w = sr);
}

pub(crate) mod sealed {
    pub trait Instance {
        fn regs() -> crate::pac::usart::Usart;
    }
}

pub trait Instance: sealed::Instance + RccPeripheral {
    type Interrupt: Interrupt;
}

pin_trait!(RxPin, Instance);
pin_trait!(TxPin, Instance);
pin_trait!(CtsPin, Instance);
pin_trait!(RtsPin, Instance);
pin_trait!(CkPin, Instance);

dma_trait!(TxDma, Instance);
dma_trait!(RxDma, Instance);

foreach_interrupt!(
    ($inst:ident, usart, $block:ident, $signal_name:ident, $irq:ident) => {
        impl sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::usart::Usart {
                crate::pac::$inst
            }
        }

        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::$irq;
        }
    };
);
