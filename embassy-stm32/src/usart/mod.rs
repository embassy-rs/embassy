#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_cortex_m::interrupt::InterruptExt;
use embassy_futures::select::{select, Either};
use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};

use crate::dma::NoDma;
use crate::gpio::sealed::AFType;
#[cfg(any(lpuart_v1, lpuart_v2))]
use crate::pac::lpuart::{regs, vals, Lpuart as Regs};
#[cfg(not(any(lpuart_v1, lpuart_v2)))]
use crate::pac::usart::{regs, vals, Usart as Regs};
use crate::time::Hertz;
use crate::{peripherals, Peripheral};

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
    /// if true, on read-like method, if there is a latent error pending,
    /// read will abort, the error reported and cleared
    /// if false, the error is ignored and cleared
    pub detect_previous_overrun: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::STOP1,
            parity: Parity::ParityNone,
            // historical behavior
            detect_previous_overrun: false,
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
    /// Buffer too large for DMA
    BufferTooLong,
}

enum ReadCompletionEvent {
    // DMA Read transfer completed first
    DmaCompleted,
    // Idle line detected first
    Idle,
}

pub struct Uart<'d, T: BasicInstance, TxDma = NoDma, RxDma = NoDma> {
    tx: UartTx<'d, T, TxDma>,
    rx: UartRx<'d, T, RxDma>,
}

pub struct UartTx<'d, T: BasicInstance, TxDma = NoDma> {
    phantom: PhantomData<&'d mut T>,
    tx_dma: PeripheralRef<'d, TxDma>,
}

pub struct UartRx<'d, T: BasicInstance, RxDma = NoDma> {
    _peri: PeripheralRef<'d, T>,
    rx_dma: PeripheralRef<'d, RxDma>,
    detect_previous_overrun: bool,
}

impl<'d, T: BasicInstance, TxDma> UartTx<'d, T, TxDma> {
    /// usefull if you only want Uart Tx. It saves 1 pin and consumes a little less power
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        tx_dma: impl Peripheral<P = TxDma> + 'd,
        config: Config,
    ) -> Self {
        T::enable();
        T::reset();

        Self::new_inner(peri, tx, tx_dma, config)
    }

    pub fn new_with_cts(
        peri: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        tx_dma: impl Peripheral<P = TxDma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(cts);

        T::enable();
        T::reset();

        unsafe {
            cts.set_as_af(cts.af_num(), AFType::Input);
            T::regs().cr3().write(|w| {
                w.set_ctse(true);
            });
        }
        Self::new_inner(peri, tx, tx_dma, config)
    }

    fn new_inner(
        _peri: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        tx_dma: impl Peripheral<P = TxDma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(_peri, tx, tx_dma);

        let r = T::regs();

        unsafe {
            tx.set_as_af(tx.af_num(), AFType::OutputPushPull);
        }

        configure(r, &config, T::frequency(), T::MULTIPLIER, false, true);

        // create state once!
        let _s = T::state();

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

impl<'d, T: BasicInstance, RxDma> UartRx<'d, T, RxDma> {
    /// usefull if you only want Uart Rx. It saves 1 pin and consumes a little less power
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rx_dma: impl Peripheral<P = RxDma> + 'd,
        config: Config,
    ) -> Self {
        T::enable();
        T::reset();

        Self::new_inner(peri, irq, rx, rx_dma, config)
    }

    pub fn new_with_rts(
        peri: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        rx_dma: impl Peripheral<P = RxDma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(rts);

        T::enable();
        T::reset();

        unsafe {
            rts.set_as_af(rts.af_num(), AFType::OutputPushPull);
            T::regs().cr3().write(|w| {
                w.set_rtse(true);
            });
        }

        Self::new_inner(peri, irq, rx, rx_dma, config)
    }

    fn new_inner(
        peri: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rx_dma: impl Peripheral<P = RxDma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, irq, rx, rx_dma);

        let r = T::regs();

        unsafe {
            rx.set_as_af(rx.af_num(), AFType::Input);
        }

        configure(r, &config, T::frequency(), T::MULTIPLIER, true, false);

        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        // create state once!
        let _s = T::state();

        Self {
            _peri: peri,
            rx_dma,
            detect_previous_overrun: config.detect_previous_overrun,
        }
    }

    fn on_interrupt(_: *mut ()) {
        let r = T::regs();
        let s = T::state();

        let (sr, cr1, cr3) = unsafe { (sr(r).read(), r.cr1().read(), r.cr3().read()) };

        let mut wake = false;
        let has_errors = (sr.pe() && cr1.peie()) || ((sr.fe() || sr.ne() || sr.ore()) && cr3.eie());
        if has_errors {
            // clear all interrupts and DMA Rx Request
            unsafe {
                r.cr1().modify(|w| {
                    // disable RXNE interrupt
                    w.set_rxneie(false);
                    // disable parity interrupt
                    w.set_peie(false);
                    // disable idle line interrupt
                    w.set_idleie(false);
                });
                r.cr3().modify(|w| {
                    // disable Error Interrupt: (Frame error, Noise error, Overrun error)
                    w.set_eie(false);
                    // disable DMA Rx Request
                    w.set_dmar(false);
                });
            }

            wake = true;
        } else {
            if cr1.idleie() && sr.idle() {
                // IDLE detected: no more data will come
                unsafe {
                    r.cr1().modify(|w| {
                        // disable idle line detection
                        w.set_idleie(false);
                    });

                    r.cr3().modify(|w| {
                        // disable DMA Rx Request
                        w.set_dmar(false);
                    });
                }

                wake = true;
            }

            if cr1.rxneie() {
                // We cannot check the RXNE flag as it is auto-cleared by the DMA controller

                // It is up to the listener to determine if this in fact was a RX event and disable the RXNE detection

                wake = true;
            }
        }

        if wake {
            compiler_fence(Ordering::SeqCst);

            s.rx_waker.wake();
        }
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error>
    where
        RxDma: crate::usart::RxDma<T>,
    {
        self.inner_read(buffer, false).await?;

        Ok(())
    }

    pub fn nb_read(&mut self) -> Result<u8, nb::Error<Error>> {
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

    pub async fn read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize, Error>
    where
        RxDma: crate::usart::RxDma<T>,
    {
        self.inner_read(buffer, true).await
    }

    async fn inner_read_run(
        &mut self,
        buffer: &mut [u8],
        enable_idle_line_detection: bool,
    ) -> Result<ReadCompletionEvent, Error>
    where
        RxDma: crate::usart::RxDma<T>,
    {
        let r = T::regs();

        // make sure USART state is restored to neutral state when this future is dropped
        let on_drop = OnDrop::new(move || {
            // defmt::trace!("Clear all USART interrupts and DMA Read Request");
            // clear all interrupts and DMA Rx Request
            // SAFETY: only clears Rx related flags
            unsafe {
                r.cr1().modify(|w| {
                    // disable RXNE interrupt
                    w.set_rxneie(false);
                    // disable parity interrupt
                    w.set_peie(false);
                    // disable idle line interrupt
                    w.set_idleie(false);
                });
                r.cr3().modify(|w| {
                    // disable Error Interrupt: (Frame error, Noise error, Overrun error)
                    w.set_eie(false);
                    // disable DMA Rx Request
                    w.set_dmar(false);
                });
            }
        });

        let ch = &mut self.rx_dma;
        let request = ch.request();

        // Start USART DMA
        // will not do anything yet because DMAR is not yet set
        // future which will complete when DMA Read request completes
        let transfer = crate::dma::read(ch, request, rdr(T::regs()), buffer);

        // SAFETY: The only way we might have a problem is using split rx and tx
        // here we only modify or read Rx related flags, interrupts and DMA channel
        unsafe {
            // clear ORE flag just before enabling DMA Rx Request: can be mandatory for the second transfer
            if !self.detect_previous_overrun {
                let sr = sr(r).read();
                // This read also clears the error and idle interrupt flags on v1.
                rdr(r).read_volatile();
                clear_interrupt_flags(r, sr);
            }

            r.cr1().modify(|w| {
                // disable RXNE interrupt
                w.set_rxneie(false);
                // enable parity interrupt if not ParityNone
                w.set_peie(w.pce());
            });

            r.cr3().modify(|w| {
                // enable Error Interrupt: (Frame error, Noise error, Overrun error)
                w.set_eie(true);
                // enable DMA Rx Request
                w.set_dmar(true);
            });

            compiler_fence(Ordering::SeqCst);

            // In case of errors already pending when reception started, interrupts may have already been raised
            // and lead to reception abortion (Overrun error for instance). In such a case, all interrupts
            // have been disabled in interrupt handler and DMA Rx Request has been disabled.

            let cr3 = r.cr3().read();

            if !cr3.dmar() {
                // something went wrong
                // because the only way to get this flag cleared is to have an interrupt

                // DMA will be stopped when transfer is dropped

                let sr = sr(r).read();
                // This read also clears the error and idle interrupt flags on v1.
                rdr(r).read_volatile();
                clear_interrupt_flags(r, sr);

                if sr.pe() {
                    return Err(Error::Parity);
                }
                if sr.fe() {
                    return Err(Error::Framing);
                }
                if sr.ne() {
                    return Err(Error::Noise);
                }
                if sr.ore() {
                    return Err(Error::Overrun);
                }

                unreachable!();
            }

            if !enable_idle_line_detection {
                transfer.await;

                return Ok(ReadCompletionEvent::DmaCompleted);
            }

            // clear idle flag
            let sr = sr(r).read();
            // This read also clears the error and idle interrupt flags on v1.
            rdr(r).read_volatile();
            clear_interrupt_flags(r, sr);

            // enable idle interrupt
            r.cr1().modify(|w| {
                w.set_idleie(true);
            });
        }

        compiler_fence(Ordering::SeqCst);

        // future which completes when idle line is detected
        let idle = poll_fn(move |cx| {
            let s = T::state();

            s.rx_waker.register(cx.waker());

            // SAFETY: read only and we only use Rx related flags
            let sr = unsafe { sr(r).read() };

            // SAFETY: only clears Rx related flags
            unsafe {
                // This read also clears the error and idle interrupt flags on v1.
                rdr(r).read_volatile();
                clear_interrupt_flags(r, sr);
            }

            compiler_fence(Ordering::SeqCst);

            let has_errors = sr.pe() || sr.fe() || sr.ne() || sr.ore();

            if has_errors {
                // all Rx interrupts and Rx DMA Request have already been cleared in interrupt handler

                if sr.pe() {
                    return Poll::Ready(Err(Error::Parity));
                }
                if sr.fe() {
                    return Poll::Ready(Err(Error::Framing));
                }
                if sr.ne() {
                    return Poll::Ready(Err(Error::Noise));
                }
                if sr.ore() {
                    return Poll::Ready(Err(Error::Overrun));
                }
            }

            if sr.idle() {
                // Idle line detected
                return Poll::Ready(Ok(()));
            }

            Poll::Pending
        });

        // wait for the first of DMA request or idle line detected to completes
        // select consumes its arguments
        // when transfer is dropped, it will stop the DMA request
        let r = match select(transfer, idle).await {
            // DMA transfer completed first
            Either::First(()) => Ok(ReadCompletionEvent::DmaCompleted),

            // Idle line detected first
            Either::Second(Ok(())) => Ok(ReadCompletionEvent::Idle),

            // error occurred
            Either::Second(Err(e)) => Err(e),
        };

        drop(on_drop);

        r
    }

    async fn inner_read(&mut self, buffer: &mut [u8], enable_idle_line_detection: bool) -> Result<usize, Error>
    where
        RxDma: crate::usart::RxDma<T>,
    {
        if buffer.is_empty() {
            return Ok(0);
        } else if buffer.len() > 0xFFFF {
            return Err(Error::BufferTooLong);
        }

        let buffer_len = buffer.len();

        // wait for DMA to complete or IDLE line detection if requested
        let res = self.inner_read_run(buffer, enable_idle_line_detection).await;

        let ch = &mut self.rx_dma;

        match res {
            Ok(ReadCompletionEvent::DmaCompleted) => Ok(buffer_len),
            Ok(ReadCompletionEvent::Idle) => {
                let n = buffer_len - (ch.remaining_transfers() as usize);
                Ok(n)
            }
            Err(e) => Err(e),
        }
    }
}

impl<'d, T: BasicInstance, TxDma, RxDma> Uart<'d, T, TxDma, RxDma> {
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        tx_dma: impl Peripheral<P = TxDma> + 'd,
        rx_dma: impl Peripheral<P = RxDma> + 'd,
        config: Config,
    ) -> Self {
        T::enable();
        T::reset();

        Self::new_inner(peri, rx, tx, irq, tx_dma, rx_dma, config)
    }

    pub fn new_with_rtscts(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        tx_dma: impl Peripheral<P = TxDma> + 'd,
        rx_dma: impl Peripheral<P = RxDma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(cts, rts);

        T::enable();
        T::reset();

        unsafe {
            rts.set_as_af(rts.af_num(), AFType::OutputPushPull);
            cts.set_as_af(cts.af_num(), AFType::Input);
            T::regs().cr3().write(|w| {
                w.set_rtse(true);
                w.set_ctse(true);
            });
        }
        Self::new_inner(peri, rx, tx, irq, tx_dma, rx_dma, config)
    }

    #[cfg(not(usart_v1))]
    pub fn new_with_de(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        de: impl Peripheral<P = impl DePin<T>> + 'd,
        tx_dma: impl Peripheral<P = TxDma> + 'd,
        rx_dma: impl Peripheral<P = RxDma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(de);

        T::enable();
        T::reset();

        unsafe {
            de.set_as_af(de.af_num(), AFType::OutputPushPull);
            T::regs().cr3().write(|w| {
                w.set_dem(true);
            });
        }
        Self::new_inner(peri, rx, tx, irq, tx_dma, rx_dma, config)
    }

    fn new_inner(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        tx_dma: impl Peripheral<P = TxDma> + 'd,
        rx_dma: impl Peripheral<P = RxDma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, rx, tx, irq, tx_dma, rx_dma);

        let r = T::regs();

        unsafe {
            rx.set_as_af(rx.af_num(), AFType::Input);
            tx.set_as_af(tx.af_num(), AFType::OutputPushPull);
        }

        configure(r, &config, T::frequency(), T::MULTIPLIER, true, true);

        irq.set_handler(UartRx::<T, RxDma>::on_interrupt);
        irq.unpend();
        irq.enable();

        // create state once!
        let _s = T::state();

        Self {
            tx: UartTx {
                tx_dma,
                phantom: PhantomData,
            },
            rx: UartRx {
                _peri: peri,
                rx_dma,
                detect_previous_overrun: config.detect_previous_overrun,
            },
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

    pub fn nb_read(&mut self) -> Result<u8, nb::Error<Error>> {
        self.rx.nb_read()
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    pub async fn read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize, Error>
    where
        RxDma: crate::usart::RxDma<T>,
    {
        self.rx.read_until_idle(buffer).await
    }

    /// Split the Uart into a transmitter and receiver, which is
    /// particuarly useful when having two tasks correlating to
    /// transmitting and receiving.
    pub fn split(self) -> (UartTx<'d, T, TxDma>, UartRx<'d, T, RxDma>) {
        (self.tx, self.rx)
    }
}

fn configure(r: Regs, config: &Config, pclk_freq: Hertz, multiplier: u32, enable_rx: bool, enable_tx: bool) {
    if !enable_rx && !enable_tx {
        panic!("USART: At least one of RX or TX should be enabled");
    }

    // TODO: better calculation, including error checking and OVER8 if possible.
    let div = (pclk_freq.0 + (config.baudrate / 2)) / config.baudrate * multiplier;

    unsafe {
        r.brr().write_value(regs::Brr(div));
        r.cr2().write(|w| {
            w.set_stop(match config.stop_bits {
                StopBits::STOP0P5 => vals::Stop::STOP0P5,
                StopBits::STOP1 => vals::Stop::STOP1,
                StopBits::STOP1P5 => vals::Stop::STOP1P5,
                StopBits::STOP2 => vals::Stop::STOP2,
            });
        });
        r.cr1().write(|w| {
            // enable uart
            w.set_ue(true);
            // enable transceiver
            w.set_te(enable_tx);
            // enable receiver
            w.set_re(enable_rx);
            // configure word size
            w.set_m0(if config.parity != Parity::ParityNone {
                vals::M0::BIT9
            } else {
                vals::M0::BIT8
            });
            // configure parity
            w.set_pce(config.parity != Parity::ParityNone);
            w.set_ps(match config.parity {
                Parity::ParityOdd => vals::Ps::ODD,
                Parity::ParityEven => vals::Ps::EVEN,
                _ => vals::Ps::EVEN,
            });
        });
    }
}

mod eh02 {
    use super::*;

    impl<'d, T: BasicInstance, RxDma> embedded_hal_02::serial::Read<u8> for UartRx<'d, T, RxDma> {
        type Error = Error;
        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            self.nb_read()
        }
    }

    impl<'d, T: BasicInstance, TxDma> embedded_hal_02::blocking::serial::Write<u8> for UartTx<'d, T, TxDma> {
        type Error = Error;
        fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }
        fn bflush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }

    impl<'d, T: BasicInstance, TxDma, RxDma> embedded_hal_02::serial::Read<u8> for Uart<'d, T, TxDma, RxDma> {
        type Error = Error;
        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            self.nb_read()
        }
    }

    impl<'d, T: BasicInstance, TxDma, RxDma> embedded_hal_02::blocking::serial::Write<u8> for Uart<'d, T, TxDma, RxDma> {
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
                Self::BufferTooLong => embedded_hal_1::serial::ErrorKind::Other,
            }
        }
    }

    impl<'d, T: BasicInstance, TxDma, RxDma> embedded_hal_1::serial::ErrorType for Uart<'d, T, TxDma, RxDma> {
        type Error = Error;
    }

    impl<'d, T: BasicInstance, TxDma> embedded_hal_1::serial::ErrorType for UartTx<'d, T, TxDma> {
        type Error = Error;
    }

    impl<'d, T: BasicInstance, RxDma> embedded_hal_1::serial::ErrorType for UartRx<'d, T, RxDma> {
        type Error = Error;
    }

    impl<'d, T: BasicInstance, RxDma> embedded_hal_nb::serial::Read for UartRx<'d, T, RxDma> {
        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            self.nb_read()
        }
    }

    impl<'d, T: BasicInstance, TxDma> embedded_hal_1::serial::Write for UartTx<'d, T, TxDma> {
        fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }

    impl<'d, T: BasicInstance, TxDma> embedded_hal_nb::serial::Write for UartTx<'d, T, TxDma> {
        fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
            self.blocking_write(&[char]).map_err(nb::Error::Other)
        }

        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            self.blocking_flush().map_err(nb::Error::Other)
        }
    }

    impl<'d, T: BasicInstance, TxDma, RxDma> embedded_hal_nb::serial::Read for Uart<'d, T, TxDma, RxDma> {
        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            self.nb_read()
        }
    }

    impl<'d, T: BasicInstance, TxDma, RxDma> embedded_hal_1::serial::Write for Uart<'d, T, TxDma, RxDma> {
        fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }

    impl<'d, T: BasicInstance, TxDma, RxDma> embedded_hal_nb::serial::Write for Uart<'d, T, TxDma, RxDma> {
        fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
            self.blocking_write(&[char]).map_err(nb::Error::Other)
        }

        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            self.blocking_flush().map_err(nb::Error::Other)
        }
    }
}

#[cfg(all(feature = "unstable-traits", feature = "nightly"))]
mod eio {
    use embedded_io::asynch::Write;
    use embedded_io::Io;

    use super::*;

    impl<T, TxDma, RxDma> Io for Uart<'_, T, TxDma, RxDma>
    where
        T: BasicInstance,
    {
        type Error = Error;
    }

    impl<T, TxDma, RxDma> Write for Uart<'_, T, TxDma, RxDma>
    where
        T: BasicInstance,
        TxDma: super::TxDma<T>,
    {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.write(buf).await?;
            Ok(buf.len())
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }

    impl<T, TxDma> Io for UartTx<'_, T, TxDma>
    where
        T: BasicInstance,
    {
        type Error = Error;
    }

    impl<T, TxDma> Write for UartTx<'_, T, TxDma>
    where
        T: BasicInstance,
        TxDma: super::TxDma<T>,
    {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.write(buf).await?;
            Ok(buf.len())
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }
}

#[cfg(all(
    feature = "unstable-traits",
    feature = "nightly",
    feature = "_todo_embedded_hal_serial"
))]
mod eha {
    use core::future::Future;

    use super::*;

    impl<'d, T: BasicInstance, TxDma> embedded_hal_async::serial::Write for UartTx<'d, T, TxDma>
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

    impl<'d, T: BasicInstance, RxDma> embedded_hal_async::serial::Read for UartRx<'d, T, RxDma>
    where
        RxDma: crate::usart::RxDma<T>,
    {
        type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            self.read(buf)
        }
    }

    impl<'d, T: BasicInstance, TxDma, RxDma> embedded_hal_async::serial::Write for Uart<'d, T, TxDma, RxDma>
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

    impl<'d, T: BasicInstance, TxDma, RxDma> embedded_hal_async::serial::Read for Uart<'d, T, TxDma, RxDma>
    where
        RxDma: crate::usart::RxDma<T>,
    {
        type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            self.read(buf)
        }
    }
}

#[cfg(feature = "nightly")]
pub use buffered::*;
#[cfg(feature = "nightly")]
mod buffered;
mod dma_ringbuffer;
mod rx_ringbuffered;
pub use rx_ringbuffered::RingBufferedUartRx;

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
unsafe fn clear_interrupt_flags(_r: Regs, _sr: regs::Sr) {
    // On v1 the flags are cleared implicitly by reads and writes to DR.
}

#[cfg(usart_v2)]
fn tdr(r: Regs) -> *mut u8 {
    r.tdr().ptr() as _
}

#[cfg(usart_v2)]
fn rdr(r: Regs) -> *mut u8 {
    r.rdr().ptr() as _
}

#[cfg(usart_v2)]
fn sr(r: Regs) -> crate::pac::common::Reg<regs::Isr, crate::pac::common::R> {
    r.isr()
}

#[cfg(usart_v2)]
#[allow(unused)]
unsafe fn clear_interrupt_flags(r: Regs, sr: regs::Isr) {
    r.icr().write(|w| *w = regs::Icr(sr.0));
}

pub(crate) mod sealed {
    use embassy_sync::waitqueue::AtomicWaker;

    use super::*;

    pub struct State {
        pub rx_waker: AtomicWaker,
        pub tx_waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                rx_waker: AtomicWaker::new(),
                tx_waker: AtomicWaker::new(),
            }
        }
    }

    pub trait BasicInstance: crate::rcc::RccPeripheral {
        const MULTIPLIER: u32;
        type Interrupt: crate::interrupt::Interrupt;

        fn regs() -> Regs;
        fn state() -> &'static State;
    }

    pub trait FullInstance: BasicInstance {
        fn regs_uart() -> crate::pac::usart::Usart;
    }
}

pub trait BasicInstance: Peripheral<P = Self> + sealed::BasicInstance + 'static + Send {}

pub trait FullInstance: sealed::FullInstance {}

pin_trait!(RxPin, BasicInstance);
pin_trait!(TxPin, BasicInstance);
pin_trait!(CtsPin, BasicInstance);
pin_trait!(RtsPin, BasicInstance);
pin_trait!(CkPin, BasicInstance);
pin_trait!(DePin, BasicInstance);

dma_trait!(TxDma, BasicInstance);
dma_trait!(RxDma, BasicInstance);

macro_rules! impl_lpuart {
    ($inst:ident, $irq:ident, $mul:expr) => {
        impl sealed::BasicInstance for crate::peripherals::$inst {
            const MULTIPLIER: u32 = $mul;
            type Interrupt = crate::interrupt::$irq;

            fn regs() -> Regs {
                Regs(crate::pac::$inst.0)
            }

            fn state() -> &'static crate::usart::sealed::State {
                static STATE: crate::usart::sealed::State = crate::usart::sealed::State::new();
                &STATE
            }
        }

        impl BasicInstance for peripherals::$inst {}
    };
}

foreach_interrupt!(
    ($inst:ident, lpuart, $block:ident, $signal_name:ident, $irq:ident) => {
        impl_lpuart!($inst, $irq, 256);
    };

    ($inst:ident, usart, $block:ident, $signal_name:ident, $irq:ident) => {
        impl_lpuart!($inst, $irq, 1);

        impl sealed::FullInstance for peripherals::$inst {

            fn regs_uart() -> crate::pac::usart::Usart {
                crate::pac::$inst
            }
        }

        impl FullInstance for peripherals::$inst {
        }
    };
);
