#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{into_ref, PeripheralRef};
use futures::future::{select, Either};

use crate::dma::{NoDma, Transfer};
use crate::gpio::sealed::AFType;
use crate::interrupt::typelevel::Interrupt;
#[cfg(not(any(usart_v1, usart_v2)))]
#[allow(unused_imports)]
use crate::pac::usart::regs::Isr as Sr;
#[cfg(any(usart_v1, usart_v2))]
#[allow(unused_imports)]
use crate::pac::usart::regs::Sr;
#[cfg(not(any(usart_v1, usart_v2)))]
use crate::pac::usart::Lpuart as Regs;
#[cfg(any(usart_v1, usart_v2))]
use crate::pac::usart::Usart as Regs;
use crate::pac::usart::{regs, vals};
use crate::time::Hertz;
use crate::{interrupt, peripherals, Peripheral};

/// Interrupt handler.
pub struct InterruptHandler<T: BasicInstance> {
    _phantom: PhantomData<T>,
}

impl<T: BasicInstance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::regs();
        let s = T::state();

        let (sr, cr1, cr3) = (sr(r).read(), r.cr1().read(), r.cr3().read());

        let has_errors = (sr.pe() && cr1.peie()) || ((sr.fe() || sr.ne() || sr.ore()) && cr3.eie());
        if has_errors {
            // clear all interrupts and DMA Rx Request
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
        } else if cr1.idleie() && sr.idle() {
            // IDLE detected: no more data will come
            r.cr1().modify(|w| {
                // disable idle line detection
                w.set_idleie(false);
            });
        } else if cr1.rxneie() {
            // We cannot check the RXNE flag as it is auto-cleared by the DMA controller

            // It is up to the listener to determine if this in fact was a RX event and disable the RXNE detection
        } else {
            return;
        }

        compiler_fence(Ordering::SeqCst);
        s.rx_waker.wake();
    }
}

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

    /// Set this to true if the line is considered noise free.
    /// This will increase the receivers tolerance to clock deviations,
    /// but will effectively disable noise detection.
    #[cfg(not(usart_v1))]
    pub assume_noise_free: bool,

    /// Set this to true to swap the RX and TX pins.
    #[cfg(any(usart_v3, usart_v4))]
    pub swap_rx_tx: bool,
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
            #[cfg(not(usart_v1))]
            assume_noise_free: false,
            #[cfg(any(usart_v3, usart_v4))]
            swap_rx_tx: false,
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
    Idle(usize),
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
    #[cfg(any(usart_v1, usart_v2))]
    buffered_sr: stm32_metapac::usart::regs::Sr,
}

impl<'d, T: BasicInstance, TxDma> UartTx<'d, T, TxDma> {
    /// Useful if you only want Uart Tx. It saves 1 pin and consumes a little less power.
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

        cts.set_as_af(cts.af_num(), AFType::Input);
        T::regs().cr3().write(|w| {
            w.set_ctse(true);
        });
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

        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        configure(r, &config, T::frequency(), T::KIND, false, true);

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
        T::regs().cr3().modify(|reg| {
            reg.set_dmat(true);
        });
        // If we don't assign future to a variable, the data register pointer
        // is held across an await and makes the future non-Send.
        let transfer = unsafe { Transfer::new_write(ch, request, buffer, tdr(T::regs()), Default::default()) };
        transfer.await;
        Ok(())
    }

    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = T::regs();
        for &b in buffer {
            while !sr(r).read().txe() {}
            unsafe { tdr(r).write_volatile(b) };
        }
        Ok(())
    }

    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        let r = T::regs();
        while !sr(r).read().tc() {}
        Ok(())
    }
}

impl<'d, T: BasicInstance, RxDma> UartRx<'d, T, RxDma> {
    /// Useful if you only want Uart Rx. It saves 1 pin and consumes a little less power.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rx_dma: impl Peripheral<P = RxDma> + 'd,
        config: Config,
    ) -> Self {
        T::enable();
        T::reset();

        Self::new_inner(peri, rx, rx_dma, config)
    }

    pub fn new_with_rts(
        peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        rx_dma: impl Peripheral<P = RxDma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(rts);

        T::enable();
        T::reset();

        rts.set_as_af(rts.af_num(), AFType::OutputPushPull);
        T::regs().cr3().write(|w| {
            w.set_rtse(true);
        });

        Self::new_inner(peri, rx, rx_dma, config)
    }

    fn new_inner(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rx_dma: impl Peripheral<P = RxDma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, rx, rx_dma);

        let r = T::regs();

        rx.set_as_af(rx.af_num(), AFType::Input);

        configure(r, &config, T::frequency(), T::KIND, true, false);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        // create state once!
        let _s = T::state();

        Self {
            _peri: peri,
            rx_dma,
            detect_previous_overrun: config.detect_previous_overrun,
            #[cfg(any(usart_v1, usart_v2))]
            buffered_sr: stm32_metapac::usart::regs::Sr(0),
        }
    }

    #[cfg(any(usart_v1, usart_v2))]
    fn check_rx_flags(&mut self) -> Result<bool, Error> {
        let r = T::regs();
        loop {
            // Handle all buffered error flags.
            if self.buffered_sr.pe() {
                self.buffered_sr.set_pe(false);
                return Err(Error::Parity);
            } else if self.buffered_sr.fe() {
                self.buffered_sr.set_fe(false);
                return Err(Error::Framing);
            } else if self.buffered_sr.ne() {
                self.buffered_sr.set_ne(false);
                return Err(Error::Noise);
            } else if self.buffered_sr.ore() {
                self.buffered_sr.set_ore(false);
                return Err(Error::Overrun);
            } else if self.buffered_sr.rxne() {
                self.buffered_sr.set_rxne(false);
                return Ok(true);
            } else {
                // No error flags from previous iterations were set: Check the actual status register
                let sr = r.sr().read();
                if !sr.rxne() {
                    return Ok(false);
                }

                // Buffer the status register and let the loop handle the error flags.
                self.buffered_sr = sr;
            }
        }
    }

    #[cfg(any(usart_v3, usart_v4))]
    fn check_rx_flags(&mut self) -> Result<bool, Error> {
        let r = T::regs();
        let sr = r.isr().read();
        if sr.pe() {
            r.icr().write(|w| w.set_pe(true));
            return Err(Error::Parity);
        } else if sr.fe() {
            r.icr().write(|w| w.set_fe(true));
            return Err(Error::Framing);
        } else if sr.ne() {
            r.icr().write(|w| w.set_ne(true));
            return Err(Error::Noise);
        } else if sr.ore() {
            r.icr().write(|w| w.set_ore(true));
            return Err(Error::Overrun);
        }
        Ok(sr.rxne())
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
        if self.check_rx_flags()? {
            Ok(unsafe { rdr(r).read_volatile() })
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let r = T::regs();
        for b in buffer {
            while !self.check_rx_flags()? {}
            unsafe { *b = rdr(r).read_volatile() }
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
        });

        let ch = &mut self.rx_dma;
        let request = ch.request();

        let buffer_len = buffer.len();

        // Start USART DMA
        // will not do anything yet because DMAR is not yet set
        // future which will complete when DMA Read request completes
        let transfer = unsafe { Transfer::new_read(ch, request, rdr(T::regs()), buffer, Default::default()) };

        // clear ORE flag just before enabling DMA Rx Request: can be mandatory for the second transfer
        if !self.detect_previous_overrun {
            let sr = sr(r).read();
            // This read also clears the error and idle interrupt flags on v1.
            unsafe { rdr(r).read_volatile() };
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
            unsafe { rdr(r).read_volatile() };
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

        if enable_idle_line_detection {
            // clear idle flag
            let sr = sr(r).read();
            // This read also clears the error and idle interrupt flags on v1.
            unsafe { rdr(r).read_volatile() };
            clear_interrupt_flags(r, sr);

            // enable idle interrupt
            r.cr1().modify(|w| {
                w.set_idleie(true);
            });
        }

        compiler_fence(Ordering::SeqCst);

        // future which completes when idle line or error is detected
        let abort = poll_fn(move |cx| {
            let s = T::state();

            s.rx_waker.register(cx.waker());

            let sr = sr(r).read();

            // This read also clears the error and idle interrupt flags on v1.
            unsafe { rdr(r).read_volatile() };
            clear_interrupt_flags(r, sr);

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

            if enable_idle_line_detection && sr.idle() {
                // Idle line detected
                return Poll::Ready(Ok(()));
            }

            Poll::Pending
        });

        // wait for the first of DMA request or idle line detected to completes
        // select consumes its arguments
        // when transfer is dropped, it will stop the DMA request
        let r = match select(transfer, abort).await {
            // DMA transfer completed first
            Either::Left(((), _)) => Ok(ReadCompletionEvent::DmaCompleted),

            // Idle line detected first
            Either::Right((Ok(()), transfer)) => Ok(ReadCompletionEvent::Idle(
                buffer_len - transfer.get_remaining_transfers() as usize,
            )),

            // error occurred
            Either::Right((Err(e), _)) => Err(e),
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

        match res {
            Ok(ReadCompletionEvent::DmaCompleted) => Ok(buffer_len),
            Ok(ReadCompletionEvent::Idle(n)) => Ok(n),
            Err(e) => Err(e),
        }
    }
}

impl<'d, T: BasicInstance, TxDma> Drop for UartTx<'d, T, TxDma> {
    fn drop(&mut self) {
        T::disable();
    }
}

impl<'d, T: BasicInstance, TxDma> Drop for UartRx<'d, T, TxDma> {
    fn drop(&mut self) {
        T::disable();
    }
}

impl<'d, T: BasicInstance, TxDma, RxDma> Uart<'d, T, TxDma, RxDma> {
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        tx_dma: impl Peripheral<P = TxDma> + 'd,
        rx_dma: impl Peripheral<P = RxDma> + 'd,
        config: Config,
    ) -> Self {
        // UartRx and UartTx have one refcount ea.
        T::enable();
        T::enable();
        T::reset();

        Self::new_inner(peri, rx, tx, tx_dma, rx_dma, config)
    }

    pub fn new_with_rtscts(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        tx_dma: impl Peripheral<P = TxDma> + 'd,
        rx_dma: impl Peripheral<P = RxDma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(cts, rts);

        // UartRx and UartTx have one refcount ea.
        T::enable();
        T::enable();
        T::reset();

        rts.set_as_af(rts.af_num(), AFType::OutputPushPull);
        cts.set_as_af(cts.af_num(), AFType::Input);
        T::regs().cr3().write(|w| {
            w.set_rtse(true);
            w.set_ctse(true);
        });
        Self::new_inner(peri, rx, tx, tx_dma, rx_dma, config)
    }

    #[cfg(not(any(usart_v1, usart_v2)))]
    pub fn new_with_de(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        de: impl Peripheral<P = impl DePin<T>> + 'd,
        tx_dma: impl Peripheral<P = TxDma> + 'd,
        rx_dma: impl Peripheral<P = RxDma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(de);

        // UartRx and UartTx have one refcount ea.
        T::enable();
        T::enable();
        T::reset();

        de.set_as_af(de.af_num(), AFType::OutputPushPull);
        T::regs().cr3().write(|w| {
            w.set_dem(true);
        });
        Self::new_inner(peri, rx, tx, tx_dma, rx_dma, config)
    }

    fn new_inner(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        tx_dma: impl Peripheral<P = TxDma> + 'd,
        rx_dma: impl Peripheral<P = RxDma> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, rx, tx, tx_dma, rx_dma);

        let r = T::regs();

        // Some chips do not have swap_rx_tx bit
        cfg_if::cfg_if! {
            if #[cfg(any(usart_v3, usart_v4))] {
                if config.swap_rx_tx {
                    let (rx, tx) = (tx, rx);
                    rx.set_as_af(rx.af_num(), AFType::Input);
                    tx.set_as_af(tx.af_num(), AFType::OutputPushPull);
                } else {
                    rx.set_as_af(rx.af_num(), AFType::Input);
                    tx.set_as_af(tx.af_num(), AFType::OutputPushPull);
                }
            } else {
                rx.set_as_af(rx.af_num(), AFType::Input);
                tx.set_as_af(tx.af_num(), AFType::OutputPushPull);
            }
        }

        configure(r, &config, T::frequency(), T::KIND, true, true);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

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
                #[cfg(any(usart_v1, usart_v2))]
                buffered_sr: stm32_metapac::usart::regs::Sr(0),
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
    /// particularly useful when having two tasks correlating to
    /// transmitting and receiving.
    pub fn split(self) -> (UartTx<'d, T, TxDma>, UartRx<'d, T, RxDma>) {
        (self.tx, self.rx)
    }
}

fn configure(r: Regs, config: &Config, pclk_freq: Hertz, kind: Kind, enable_rx: bool, enable_tx: bool) {
    if !enable_rx && !enable_tx {
        panic!("USART: At least one of RX or TX should be enabled");
    }

    #[cfg(not(usart_v4))]
    static DIVS: [(u16, ()); 1] = [(1, ())];

    #[cfg(usart_v4)]
    static DIVS: [(u16, vals::Presc); 12] = [
        (1, vals::Presc::DIV1),
        (2, vals::Presc::DIV2),
        (4, vals::Presc::DIV4),
        (6, vals::Presc::DIV6),
        (8, vals::Presc::DIV8),
        (10, vals::Presc::DIV10),
        (12, vals::Presc::DIV12),
        (16, vals::Presc::DIV16),
        (32, vals::Presc::DIV32),
        (64, vals::Presc::DIV64),
        (128, vals::Presc::DIV128),
        (256, vals::Presc::DIV256),
    ];

    let (mul, brr_min, brr_max) = match kind {
        #[cfg(any(usart_v3, usart_v4))]
        Kind::Lpuart => (256, 0x300, 0x10_0000),
        Kind::Uart => (1, 0x10, 0x1_0000),
    };

    fn calculate_brr(baud: u32, pclk: u32, presc: u32, mul: u32) -> u32 {
        // The calculation to be done to get the BRR is `mul * pclk / presc / baud`
        // To do this in 32-bit only we can't multiply `mul` and `pclk`
        let clock = pclk / presc;

        // The mul is applied as the last operation to prevent overflow
        let brr = clock / baud * mul;

        // The BRR calculation will be a bit off because of integer rounding.
        // Because we multiplied our inaccuracy with mul, our rounding now needs to be in proportion to mul.
        let rounding = ((clock % baud) * mul + (baud / 2)) / baud;

        brr + rounding
    }

    #[cfg(not(usart_v1))]
    let mut over8 = false;
    let mut found_brr = None;
    for &(presc, _presc_val) in &DIVS {
        let brr = calculate_brr(config.baudrate, pclk_freq.0, presc as u32, mul);
        trace!(
            "USART: presc={}, div=0x{:08x} (mantissa = {}, fraction = {})",
            presc,
            brr,
            brr >> 4,
            brr & 0x0F
        );

        if brr < brr_min {
            #[cfg(not(usart_v1))]
            if brr * 2 >= brr_min && kind == Kind::Uart && !cfg!(usart_v1) {
                over8 = true;
                r.brr().write_value(regs::Brr(((brr << 1) & !0xF) | (brr & 0x07)));
                #[cfg(usart_v4)]
                r.presc().write(|w| w.set_prescaler(_presc_val));
                found_brr = Some(brr);
                break;
            }
            panic!("USART: baudrate too high");
        }

        if brr < brr_max {
            r.brr().write_value(regs::Brr(brr));
            #[cfg(usart_v4)]
            r.presc().write(|w| w.set_prescaler(_presc_val));
            found_brr = Some(brr);
            break;
        }
    }

    let brr = found_brr.expect("USART: baudrate too low");

    #[cfg(not(usart_v1))]
    let oversampling = if over8 { "8 bit" } else { "16 bit" };
    #[cfg(usart_v1)]
    let oversampling = "default";
    trace!(
        "Using {} oversampling, desired baudrate: {}, actual baudrate: {}",
        oversampling,
        config.baudrate,
        pclk_freq.0 / brr * mul
    );

    r.cr2().write(|w| {
        w.set_stop(match config.stop_bits {
            StopBits::STOP0P5 => vals::Stop::STOP0P5,
            StopBits::STOP1 => vals::Stop::STOP1,
            StopBits::STOP1P5 => vals::Stop::STOP1P5,
            StopBits::STOP2 => vals::Stop::STOP2,
        });

        #[cfg(any(usart_v3, usart_v4))]
        w.set_swap(config.swap_rx_tx);
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
        #[cfg(not(usart_v1))]
        w.set_over8(vals::Over8::from_bits(over8 as _));
    });

    #[cfg(not(usart_v1))]
    r.cr3().modify(|w| {
        w.set_onebit(config.assume_noise_free);
    });
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

    impl embedded_hal_nb::serial::Error for Error {
        fn kind(&self) -> embedded_hal_nb::serial::ErrorKind {
            match *self {
                Self::Framing => embedded_hal_nb::serial::ErrorKind::FrameFormat,
                Self::Noise => embedded_hal_nb::serial::ErrorKind::Noise,
                Self::Overrun => embedded_hal_nb::serial::ErrorKind::Overrun,
                Self::Parity => embedded_hal_nb::serial::ErrorKind::Parity,
                Self::BufferTooLong => embedded_hal_nb::serial::ErrorKind::Other,
            }
        }
    }

    impl<'d, T: BasicInstance, TxDma, RxDma> embedded_hal_nb::serial::ErrorType for Uart<'d, T, TxDma, RxDma> {
        type Error = Error;
    }

    impl<'d, T: BasicInstance, TxDma> embedded_hal_nb::serial::ErrorType for UartTx<'d, T, TxDma> {
        type Error = Error;
    }

    impl<'d, T: BasicInstance, RxDma> embedded_hal_nb::serial::ErrorType for UartRx<'d, T, RxDma> {
        type Error = Error;
    }

    impl<'d, T: BasicInstance, RxDma> embedded_hal_nb::serial::Read for UartRx<'d, T, RxDma> {
        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            self.nb_read()
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
    use embedded_io_async::{ErrorType, Write};

    use super::*;

    impl<T, TxDma, RxDma> ErrorType for Uart<'_, T, TxDma, RxDma>
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

    impl<T, TxDma> ErrorType for UartTx<'_, T, TxDma>
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

#[cfg(feature = "nightly")]
pub use buffered::*;

#[cfg(feature = "nightly")]
pub use crate::usart::buffered::InterruptHandler as BufferedInterruptHandler;
#[cfg(feature = "nightly")]
mod buffered;

#[cfg(not(gpdma))]
mod ringbuffered;
#[cfg(not(gpdma))]
pub use ringbuffered::RingBufferedUartRx;

use self::sealed::Kind;

#[cfg(any(usart_v1, usart_v2))]
fn tdr(r: crate::pac::usart::Usart) -> *mut u8 {
    r.dr().as_ptr() as _
}

#[cfg(any(usart_v1, usart_v2))]
fn rdr(r: crate::pac::usart::Usart) -> *mut u8 {
    r.dr().as_ptr() as _
}

#[cfg(any(usart_v1, usart_v2))]
fn sr(r: crate::pac::usart::Usart) -> crate::pac::common::Reg<regs::Sr, crate::pac::common::RW> {
    r.sr()
}

#[cfg(any(usart_v1, usart_v2))]
#[allow(unused)]
fn clear_interrupt_flags(_r: Regs, _sr: regs::Sr) {
    // On v1 the flags are cleared implicitly by reads and writes to DR.
}

#[cfg(any(usart_v3, usart_v4))]
fn tdr(r: Regs) -> *mut u8 {
    r.tdr().as_ptr() as _
}

#[cfg(any(usart_v3, usart_v4))]
fn rdr(r: Regs) -> *mut u8 {
    r.rdr().as_ptr() as _
}

#[cfg(any(usart_v3, usart_v4))]
fn sr(r: Regs) -> crate::pac::common::Reg<regs::Isr, crate::pac::common::R> {
    r.isr()
}

#[cfg(any(usart_v3, usart_v4))]
#[allow(unused)]
fn clear_interrupt_flags(r: Regs, sr: regs::Isr) {
    r.icr().write(|w| *w = regs::Icr(sr.0));
}

pub(crate) mod sealed {
    use embassy_sync::waitqueue::AtomicWaker;

    use super::*;

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum Kind {
        Uart,
        #[cfg(any(usart_v3, usart_v4))]
        Lpuart,
    }

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
        const KIND: Kind;
        type Interrupt: interrupt::typelevel::Interrupt;

        fn regs() -> Regs;
        fn state() -> &'static State;

        #[cfg(feature = "nightly")]
        fn buffered_state() -> &'static buffered::State;
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

macro_rules! impl_usart {
    ($inst:ident, $irq:ident, $kind:expr) => {
        impl sealed::BasicInstance for crate::peripherals::$inst {
            const KIND: Kind = $kind;
            type Interrupt = crate::interrupt::typelevel::$irq;

            fn regs() -> Regs {
                unsafe { Regs::from_ptr(crate::pac::$inst.as_ptr()) }
            }

            fn state() -> &'static crate::usart::sealed::State {
                static STATE: crate::usart::sealed::State = crate::usart::sealed::State::new();
                &STATE
            }

            #[cfg(feature = "nightly")]
            fn buffered_state() -> &'static buffered::State {
                static STATE: buffered::State = buffered::State::new();
                &STATE
            }
        }

        impl BasicInstance for peripherals::$inst {}
    };
}

foreach_interrupt!(
    ($inst:ident, usart, LPUART, $signal_name:ident, $irq:ident) => {
        impl_usart!($inst, $irq, Kind::Lpuart);
    };

    ($inst:ident, usart, $block:ident, $signal_name:ident, $irq:ident) => {
        impl_usart!($inst, $irq, Kind::Uart);

        impl sealed::FullInstance for peripherals::$inst {
            fn regs_uart() -> crate::pac::usart::Usart {
                crate::pac::$inst
            }
        }

        impl FullInstance for peripherals::$inst {}
    };
);
