#![macro_use]

use core::marker::PhantomData;
use embassy::interrupt::Interrupt;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

use crate::dma::NoDma;
use crate::gpio::sealed::AFType;
use crate::pac::usart::{regs, vals};
use crate::peripherals;
use crate::rcc::RccPeripheral;

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
    inner: T,
    phantom: PhantomData<&'d mut T>,
    tx_dma: TxDma,
    rx_dma: RxDma,
}

impl<'d, T: Instance, TxDma, RxDma> Uart<'d, T, TxDma, RxDma> {
    pub fn new(
        inner: impl Unborrow<Target = T> + 'd,
        rx: impl Unborrow<Target = impl RxPin<T>> + 'd,
        tx: impl Unborrow<Target = impl TxPin<T>> + 'd,
        tx_dma: impl Unborrow<Target = TxDma> + 'd,
        rx_dma: impl Unborrow<Target = RxDma> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(inner, rx, tx, tx_dma, rx_dma);

        T::enable();
        let pclk_freq = T::frequency();

        // TODO: better calculation, including error checking and OVER8 if possible.
        let div = (pclk_freq.0 + (config.baudrate / 2)) / config.baudrate;

        let r = inner.regs();

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
                w.set_m0(vals::M0::BIT8);
                w.set_pce(config.parity != Parity::ParityNone);
                w.set_ps(match config.parity {
                    Parity::ParityOdd => vals::Ps::ODD,
                    Parity::ParityEven => vals::Ps::EVEN,
                    _ => vals::Ps::EVEN,
                });
            });
        }

        Self {
            inner,
            phantom: PhantomData,
            tx_dma,
            rx_dma,
        }
    }

    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error>
    where
        TxDma: crate::usart::TxDma<T>,
    {
        let ch = &mut self.tx_dma;
        let request = ch.request();
        unsafe {
            self.inner.regs().cr3().modify(|reg| {
                reg.set_dmat(true);
            });
        }
        let r = self.inner.regs();
        let dst = tdr(r);
        crate::dma::write(ch, request, buffer, dst).await;
        Ok(())
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error>
    where
        RxDma: crate::usart::RxDma<T>,
    {
        let ch = &mut self.rx_dma;
        let request = ch.request();
        unsafe {
            self.inner.regs().cr3().modify(|reg| {
                reg.set_dmar(true);
            });
        }
        let r = self.inner.regs();
        let src = rdr(r);
        crate::dma::read(ch, request, src, buffer).await;
        Ok(())
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        unsafe {
            let r = self.inner.regs();
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

    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        unsafe {
            let r = self.inner.regs();
            for &b in buffer {
                while !sr(r).read().txe() {}
                tdr(r).write_volatile(b);
            }
        }
        Ok(())
    }

    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        unsafe {
            let r = self.inner.regs();
            while !sr(r).read().tc() {}
        }
        Ok(())
    }
}

mod eh02 {
    use super::*;

    impl<'d, T: Instance, TxDma, RxDma> embedded_hal_02::serial::Read<u8>
        for Uart<'d, T, TxDma, RxDma>
    {
        type Error = Error;
        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            let r = self.inner.regs();
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

    impl<'d, T: Instance, TxDma, RxDma> embedded_hal_02::blocking::serial::Write<u8>
        for Uart<'d, T, TxDma, RxDma>
    {
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

    impl<'d, T: Instance, TxDma, RxDma> embedded_hal_1::serial::ErrorType
        for Uart<'d, T, TxDma, RxDma>
    {
        type Error = Error;
    }
}

#[cfg(all(feature = "unstable-traits", feature = "nightly"))]
mod eh1a {
    use super::*;
    use core::future::Future;

    impl<'d, T: Instance, TxDma, RxDma> embedded_hal_async::serial::Write for Uart<'d, T, TxDma, RxDma>
    where
        TxDma: crate::usart::TxDma<T>,
    {
        type WriteFuture<'a>
        where
            Self: 'a,
        = impl Future<Output = Result<(), Self::Error>> + 'a;

        fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
            self.write(buf)
        }

        type FlushFuture<'a>
        where
            Self: 'a,
        = impl Future<Output = Result<(), Self::Error>> + 'a;

        fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
            async move { Ok(()) }
        }
    }

    impl<'d, T: Instance, TxDma, RxDma> embedded_hal_async::serial::Read for Uart<'d, T, TxDma, RxDma>
    where
        RxDma: crate::usart::RxDma<T>,
    {
        type ReadFuture<'a>
        where
            Self: 'a,
        = impl Future<Output = Result<(), Self::Error>> + 'a;

        fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            self.read(buf)
        }
    }
}

pub use buffered::*;
mod buffered {
    use atomic_polyfill::{compiler_fence, Ordering};
    use core::pin::Pin;
    use core::task::Context;
    use core::task::Poll;
    use embassy::waitqueue::WakerRegistration;
    use embassy_hal_common::peripheral::{PeripheralMutex, PeripheralState, StateStorage};
    use embassy_hal_common::ring_buffer::RingBuffer;

    use super::*;

    pub struct State<'d, T: Instance>(StateStorage<StateInner<'d, T>>);
    impl<'d, T: Instance> State<'d, T> {
        pub fn new() -> Self {
            Self(StateStorage::new())
        }
    }

    struct StateInner<'d, T: Instance> {
        uart: Uart<'d, T, NoDma, NoDma>,
        phantom: PhantomData<&'d mut T>,

        rx_waker: WakerRegistration,
        rx: RingBuffer<'d>,

        tx_waker: WakerRegistration,
        tx: RingBuffer<'d>,
    }

    unsafe impl<'d, T: Instance> Send for StateInner<'d, T> {}
    unsafe impl<'d, T: Instance> Sync for StateInner<'d, T> {}

    pub struct BufferedUart<'d, T: Instance> {
        inner: PeripheralMutex<'d, StateInner<'d, T>>,
    }

    impl<'d, T: Instance> Unpin for BufferedUart<'d, T> {}

    impl<'d, T: Instance> BufferedUart<'d, T> {
        pub unsafe fn new(
            state: &'d mut State<'d, T>,
            uart: Uart<'d, T, NoDma, NoDma>,
            irq: impl Unborrow<Target = T::Interrupt> + 'd,
            tx_buffer: &'d mut [u8],
            rx_buffer: &'d mut [u8],
        ) -> BufferedUart<'d, T> {
            unborrow!(irq);

            let r = uart.inner.regs();
            r.cr1().modify(|w| {
                w.set_rxneie(true);
                w.set_idleie(true);
            });

            Self {
                inner: PeripheralMutex::new_unchecked(irq, &mut state.0, move || StateInner {
                    uart,
                    phantom: PhantomData,
                    tx: RingBuffer::new(tx_buffer),
                    tx_waker: WakerRegistration::new(),

                    rx: RingBuffer::new(rx_buffer),
                    rx_waker: WakerRegistration::new(),
                }),
            }
        }
    }

    impl<'d, T: Instance> StateInner<'d, T>
    where
        Self: 'd,
    {
        fn on_rx(&mut self) {
            let r = self.uart.inner.regs();
            unsafe {
                let sr = sr(r).read();
                // TODO: do we want to handle interrupts the same way on v1 hardware?
                if sr.pe() {
                    clear_interrupt_flag(r, InterruptFlag::PE);
                    trace!("Parity error");
                } else if sr.fe() {
                    clear_interrupt_flag(r, InterruptFlag::FE);
                    trace!("Framing error");
                } else if sr.ne() {
                    clear_interrupt_flag(r, InterruptFlag::NE);
                    trace!("Noise error");
                } else if sr.ore() {
                    clear_interrupt_flag(r, InterruptFlag::ORE);
                    trace!("Overrun error");
                } else if sr.rxne() {
                    let buf = self.rx.push_buf();
                    if buf.is_empty() {
                        self.rx_waker.wake();
                    } else {
                        buf[0] = rdr(r).read_volatile();
                        self.rx.push(1);
                    }
                } else if sr.idle() {
                    clear_interrupt_flag(r, InterruptFlag::IDLE);
                    self.rx_waker.wake();
                };
            }
        }

        fn on_tx(&mut self) {
            let r = self.uart.inner.regs();
            unsafe {
                if sr(r).read().txe() {
                    let buf = self.tx.pop_buf();
                    if !buf.is_empty() {
                        r.cr1().modify(|w| {
                            w.set_txeie(true);
                        });
                        tdr(r).write_volatile(buf[0].into());
                        self.tx.pop(1);
                        self.tx_waker.wake();
                    } else {
                        // Disable interrupt until we have something to transmit again
                        r.cr1().modify(|w| {
                            w.set_txeie(false);
                        });
                    }
                }
            }
        }
    }

    impl<'d, T: Instance> PeripheralState for StateInner<'d, T>
    where
        Self: 'd,
    {
        type Interrupt = T::Interrupt;
        fn on_interrupt(&mut self) {
            self.on_rx();
            self.on_tx();
        }
    }

    impl<'d, T: Instance> embassy::io::AsyncBufRead for BufferedUart<'d, T> {
        fn poll_fill_buf(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<&[u8], embassy::io::Error>> {
            self.inner.with(|state| {
                compiler_fence(Ordering::SeqCst);

                // We have data ready in buffer? Return it.
                let buf = state.rx.pop_buf();
                if !buf.is_empty() {
                    let buf: &[u8] = buf;
                    // Safety: buffer lives as long as uart
                    let buf: &[u8] = unsafe { core::mem::transmute(buf) };
                    return Poll::Ready(Ok(buf));
                }

                state.rx_waker.register(cx.waker());
                Poll::<Result<&[u8], embassy::io::Error>>::Pending
            })
        }
        fn consume(mut self: Pin<&mut Self>, amt: usize) {
            let signal = self.inner.with(|state| {
                let full = state.rx.is_full();
                state.rx.pop(amt);
                full
            });
            if signal {
                self.inner.pend();
            }
        }
    }

    impl<'d, T: Instance> embassy::io::AsyncWrite for BufferedUart<'d, T> {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, embassy::io::Error>> {
            let (poll, empty) = self.inner.with(|state| {
                let empty = state.tx.is_empty();
                let tx_buf = state.tx.push_buf();
                if tx_buf.is_empty() {
                    state.tx_waker.register(cx.waker());
                    return (Poll::Pending, empty);
                }

                let n = core::cmp::min(tx_buf.len(), buf.len());
                tx_buf[..n].copy_from_slice(&buf[..n]);
                state.tx.push(n);

                (Poll::Ready(Ok(n)), empty)
            });
            if empty {
                self.inner.pend();
            }
            poll
        }

        fn poll_flush(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), embassy::io::Error>> {
            self.inner.with(|state| {
                if !state.tx.is_empty() {
                    state.tx_waker.register(cx.waker());
                    return Poll::Pending;
                }

                Poll::Ready(Ok(()))
            })
        }
    }
}

#[cfg(usart_v1)]
fn tdr(r: crate::pac::usart::Usart) -> *mut u8 {
    r.dr().ptr() as _
}

#[cfg(usart_v1)]
fn rdr(r: crate::pac::usart::Usart) -> *mut u8 {
    r.dr().ptr() as _
}

enum InterruptFlag {
    PE,
    FE,
    NE,
    ORE,
    IDLE,
}

#[cfg(usart_v1)]
fn sr(r: crate::pac::usart::Usart) -> crate::pac::common::Reg<regs::Sr, crate::pac::common::RW> {
    r.sr()
}

#[cfg(usart_v1)]
unsafe fn clear_interrupt_flag(r: crate::pac::usart::Usart, _flag: InterruptFlag) {
    // This bit is set by hardware when noise is detected on a received frame. It is cleared by a
    // software sequence (an read to the USART_SR register followed by a read to the
    // USART_DR register).

    // this is the same as what st's HAL does on v1 hardware
    r.sr().read();
    r.dr().read();
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
#[inline]
unsafe fn clear_interrupt_flag(r: crate::pac::usart::Usart, flag: InterruptFlag) {
    // v2 has a separate register for clearing flags (nice)
    match flag {
        InterruptFlag::PE => r.icr().write(|w| {
            w.set_pe(true);
        }),
        InterruptFlag::FE => r.icr().write(|w| {
            w.set_fe(true);
        }),
        InterruptFlag::NE => r.icr().write(|w| {
            w.set_ne(true);
        }),
        InterruptFlag::ORE => r.icr().write(|w| {
            w.set_ore(true);
        }),
        InterruptFlag::IDLE => r.icr().write(|w| {
            w.set_idle(true);
        }),
    }
}

pub(crate) mod sealed {
    pub trait Instance {
        fn regs(&self) -> crate::pac::usart::Usart;
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

crate::pac::interrupts!(
    ($inst:ident, usart, $block:ident, $signal_name:ident, $irq:ident) => {
        impl sealed::Instance for peripherals::$inst {
            fn regs(&self) -> crate::pac::usart::Usart {
                crate::pac::$inst
            }
        }

        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::$irq;
        }
    };
);
