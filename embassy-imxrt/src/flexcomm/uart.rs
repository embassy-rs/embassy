//! Universal Asynchronous Receiver Transmitter (UART) driver.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU8, Ordering, compiler_fence};
use core::task::Poll;

use embassy_futures::select::{Either, select};
use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use paste::paste;

use crate::dma::AnyChannel;
use crate::flexcomm::Clock;
use crate::gpio::{AnyPin, GpioPin as Pin};
use crate::interrupt::typelevel::Interrupt;
use crate::iopctl::{DriveMode, DriveStrength, Inverter, IopctlPin, Pull, SlewRate};
use crate::pac::usart0::cfg::{Clkpol, Datalen, Loop, Paritysel as Parity, Stoplen, Syncen, Syncmst};
use crate::pac::usart0::ctl::Cc;
use crate::sealed::Sealed;
use crate::{dma, interrupt};

/// Driver move trait.
#[allow(private_bounds)]
pub trait Mode: Sealed {}

/// Blocking mode.
pub struct Blocking;
impl Sealed for Blocking {}
impl Mode for Blocking {}

/// Async mode.
pub struct Async;
impl Sealed for Async {}
impl Mode for Async {}

/// Uart driver.
pub struct Uart<'a, M: Mode> {
    tx: UartTx<'a, M>,
    rx: UartRx<'a, M>,
}

/// Uart TX driver.
pub struct UartTx<'a, M: Mode> {
    info: Info,
    tx_dma: Option<Peri<'a, AnyChannel>>,
    _phantom: PhantomData<(&'a (), M)>,
}

/// Uart RX driver.
pub struct UartRx<'a, M: Mode> {
    info: Info,
    rx_dma: Option<Peri<'a, AnyChannel>>,
    _phantom: PhantomData<(&'a (), M)>,
}

/// UART config
#[derive(Clone, Copy)]
pub struct Config {
    /// Baudrate of the Uart
    pub baudrate: u32,
    /// data length
    pub data_bits: Datalen,
    /// Parity
    pub parity: Parity,
    /// Stop bits
    pub stop_bits: Stoplen,
    /// Polarity of the clock
    pub clock_polarity: Clkpol,
    /// Sync/ Async operation selection
    pub operation: Syncen,
    /// Sync master/slave mode selection (only applicable in sync mode)
    pub sync_mode_master_select: Syncmst,
    /// USART continuous Clock generation enable in synchronous master mode.
    pub continuous_clock: Cc,
    /// Normal/ loopback mode
    pub loopback_mode: Loop,
    /// Clock type
    pub clock: Clock,
}

impl Default for Config {
    /// Default configuration for single channel sampling.
    fn default() -> Self {
        Self {
            baudrate: 115_200,
            data_bits: Datalen::Bit8,
            parity: Parity::NoParity,
            stop_bits: Stoplen::Bit1,
            clock_polarity: Clkpol::FallingEdge,
            operation: Syncen::AsynchronousMode,
            sync_mode_master_select: Syncmst::Slave,
            continuous_clock: Cc::ClockOnCharacter,
            loopback_mode: Loop::Normal,
            clock: crate::flexcomm::Clock::Sfro,
        }
    }
}

/// Uart Errors
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Read error
    Read,

    /// Buffer overflow
    Overrun,

    /// Noise error
    Noise,

    /// Framing error
    Framing,

    /// Parity error
    Parity,

    /// Failure
    Fail,

    /// Invalid argument
    InvalidArgument,

    /// Uart baud rate cannot be supported with the given clock
    UnsupportedBaudrate,

    /// RX FIFO Empty
    RxFifoEmpty,

    /// TX FIFO Full
    TxFifoFull,

    /// TX Busy
    TxBusy,
}
/// shorthand for -> `Result<T>`
pub type Result<T> = core::result::Result<T, Error>;

impl<'a, M: Mode> UartTx<'a, M> {
    fn new_inner<T: Instance>(tx_dma: Option<Peri<'a, AnyChannel>>) -> Self {
        let uarttx = Self {
            info: T::info(),
            tx_dma,
            _phantom: PhantomData,
        };
        uarttx.info.refcnt.fetch_add(1, Ordering::Relaxed);
        uarttx
    }
}

impl<'a, M: Mode> Drop for UartTx<'a, M> {
    fn drop(&mut self) {
        if self.info.refcnt.fetch_sub(1, Ordering::Relaxed) == 1 {
            while self.info.regs.stat().read().txidle().bit_is_clear() {}

            self.info.regs.fifointenclr().modify(|_, w| {
                w.txerr()
                    .set_bit()
                    .rxerr()
                    .set_bit()
                    .txlvl()
                    .set_bit()
                    .rxlvl()
                    .set_bit()
            });

            self.info
                .regs
                .fifocfg()
                .modify(|_, w| w.dmatx().clear_bit().dmarx().clear_bit());

            self.info.regs.cfg().modify(|_, w| w.enable().disabled());
        }
    }
}

impl<'a> UartTx<'a, Blocking> {
    /// Create a new UART which can only send data
    /// Unidirectional Uart - Tx only
    pub fn new_blocking<T: Instance>(_inner: Peri<'a, T>, tx: Peri<'a, impl TxPin<T>>, config: Config) -> Result<Self> {
        tx.as_tx();

        let _tx = tx.into();
        Uart::<Blocking>::init::<T>(Some(_tx), None, None, None, config)?;

        Ok(Self::new_inner::<T>(None))
    }

    fn write_byte_internal(&mut self, byte: u8) -> Result<()> {
        // SAFETY: unsafe only used for .bits()
        self.info
            .regs
            .fifowr()
            .write(|w| unsafe { w.txdata().bits(u16::from(byte)) });

        Ok(())
    }

    fn blocking_write_byte(&mut self, byte: u8) -> Result<()> {
        while self.info.regs.fifostat().read().txnotfull().bit_is_clear() {}

        // Prevent the compiler from reordering write_byte_internal()
        // before the loop above.
        compiler_fence(Ordering::Release);

        self.write_byte_internal(byte)
    }

    fn write_byte(&mut self, byte: u8) -> Result<()> {
        if self.info.regs.fifostat().read().txnotfull().bit_is_clear() {
            Err(Error::TxFifoFull)
        } else {
            self.write_byte_internal(byte)
        }
    }

    /// Transmit the provided buffer blocking execution until done.
    pub fn blocking_write(&mut self, buf: &[u8]) -> Result<()> {
        for x in buf {
            self.blocking_write_byte(*x)?;
        }

        Ok(())
    }

    /// Transmit the provided buffer. Non-blocking version, bails out
    /// if it would block.
    pub fn write(&mut self, buf: &[u8]) -> Result<()> {
        for x in buf {
            self.write_byte(*x)?;
        }

        Ok(())
    }

    /// Flush UART TX blocking execution until done.
    pub fn blocking_flush(&mut self) -> Result<()> {
        while self.info.regs.stat().read().txidle().bit_is_clear() {}
        Ok(())
    }

    /// Flush UART TX.
    pub fn flush(&mut self) -> Result<()> {
        if self.info.regs.stat().read().txidle().bit_is_clear() {
            Err(Error::TxBusy)
        } else {
            Ok(())
        }
    }
}

impl<'a, M: Mode> UartRx<'a, M> {
    fn new_inner<T: Instance>(rx_dma: Option<Peri<'a, AnyChannel>>) -> Self {
        let uartrx = Self {
            info: T::info(),
            rx_dma,
            _phantom: PhantomData,
        };
        uartrx.info.refcnt.fetch_add(1, Ordering::Relaxed);
        uartrx
    }
}

impl<'a, M: Mode> Drop for UartRx<'a, M> {
    fn drop(&mut self) {
        if self.info.refcnt.fetch_sub(1, Ordering::Relaxed) == 1 {
            while self.info.regs.stat().read().rxidle().bit_is_clear() {}

            self.info.regs.fifointenclr().modify(|_, w| {
                w.txerr()
                    .set_bit()
                    .rxerr()
                    .set_bit()
                    .txlvl()
                    .set_bit()
                    .rxlvl()
                    .set_bit()
            });

            self.info
                .regs
                .fifocfg()
                .modify(|_, w| w.dmatx().clear_bit().dmarx().clear_bit());

            self.info.regs.cfg().modify(|_, w| w.enable().disabled());
        }
    }
}

impl<'a> UartRx<'a, Blocking> {
    /// Create a new blocking UART which can only receive data
    pub fn new_blocking<T: Instance>(_inner: Peri<'a, T>, rx: Peri<'a, impl RxPin<T>>, config: Config) -> Result<Self> {
        rx.as_rx();

        let _rx = rx.into();
        Uart::<Blocking>::init::<T>(None, Some(_rx), None, None, config)?;

        Ok(Self::new_inner::<T>(None))
    }
}

impl UartRx<'_, Blocking> {
    fn read_byte_internal(&mut self) -> Result<u8> {
        if self.info.regs.fifostat().read().rxerr().bit_is_set() {
            self.info.regs.fifocfg().modify(|_, w| w.emptyrx().set_bit());
            self.info.regs.fifostat().modify(|_, w| w.rxerr().set_bit());
            Err(Error::Read)
        } else if self.info.regs.stat().read().parityerrint().bit_is_set() {
            self.info.regs.stat().modify(|_, w| w.parityerrint().clear_bit_by_one());
            Err(Error::Parity)
        } else if self.info.regs.stat().read().framerrint().bit_is_set() {
            self.info.regs.stat().modify(|_, w| w.framerrint().clear_bit_by_one());
            Err(Error::Framing)
        } else if self.info.regs.stat().read().rxnoiseint().bit_is_set() {
            self.info.regs.stat().modify(|_, w| w.rxnoiseint().clear_bit_by_one());
            Err(Error::Noise)
        } else {
            let byte = self.info.regs.fiford().read().rxdata().bits() as u8;
            Ok(byte)
        }
    }

    fn read_byte(&mut self) -> Result<u8> {
        if self.info.regs.fifostat().read().rxnotempty().bit_is_clear() {
            Err(Error::RxFifoEmpty)
        } else {
            self.read_byte_internal()
        }
    }

    fn blocking_read_byte(&mut self) -> Result<u8> {
        while self.info.regs.fifostat().read().rxnotempty().bit_is_clear() {}

        // Prevent the compiler from reordering read_byte_internal()
        // before the loop above.
        compiler_fence(Ordering::Acquire);

        self.read_byte_internal()
    }

    /// Read from UART RX. Non-blocking version, bails out if it would
    /// block.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<()> {
        for b in buf.iter_mut() {
            *b = self.read_byte()?;
        }

        Ok(())
    }

    /// Read from UART RX blocking execution until done.
    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<()> {
        for b in buf.iter_mut() {
            *b = self.blocking_read_byte()?;
        }

        Ok(())
    }
}

impl<'a, M: Mode> Uart<'a, M> {
    fn init<T: Instance>(
        tx: Option<Peri<'a, AnyPin>>,
        rx: Option<Peri<'a, AnyPin>>,
        rts: Option<Peri<'a, AnyPin>>,
        cts: Option<Peri<'a, AnyPin>>,
        config: Config,
    ) -> Result<()> {
        T::enable(config.clock);
        T::into_usart();

        let regs = T::info().regs;

        if tx.is_some() {
            regs.fifocfg().modify(|_, w| w.emptytx().set_bit().enabletx().enabled());

            // clear FIFO error
            regs.fifostat().write(|w| w.txerr().set_bit());
        }

        if rx.is_some() {
            regs.fifocfg().modify(|_, w| w.emptyrx().set_bit().enablerx().enabled());

            // clear FIFO error
            regs.fifostat().write(|w| w.rxerr().set_bit());
        }

        if rts.is_some() && cts.is_some() {
            regs.cfg().modify(|_, w| w.ctsen().enabled());
        }

        Self::set_baudrate_inner::<T>(config.baudrate, config.clock)?;
        Self::set_uart_config::<T>(config);

        Ok(())
    }

    fn get_fc_freq(clock: Clock) -> Result<u32> {
        match clock {
            Clock::Sfro => Ok(16_000_000),
            Clock::Ffro => Ok(48_000_000),
            // We only support Sfro and Ffro now.
            _ => Err(Error::InvalidArgument),
        }
    }

    fn set_baudrate_inner<T: Instance>(baudrate: u32, clock: Clock) -> Result<()> {
        // Get source clock frequency according to clock type.
        let source_clock_hz = Self::get_fc_freq(clock)?;

        if baudrate == 0 {
            return Err(Error::InvalidArgument);
        }

        let regs = T::info().regs;

        // If synchronous master mode is enabled, only configure the BRG value.
        if regs.cfg().read().syncen().is_synchronous_mode() {
            // Master
            if regs.cfg().read().syncmst().is_master() {
                // Calculate the BRG value
                let brgval = (source_clock_hz / baudrate) - 1;

                // SAFETY: unsafe only used for .bits()
                regs.brg().write(|w| unsafe { w.brgval().bits(brgval as u16) });
            }
        } else {
            // Smaller values of OSR can make the sampling position within a
            // data bit less accurate and may potentially cause more noise
            // errors or incorrect data.
            let (_, osr, brg) = (8..16).rev().fold(
                (u32::MAX, u32::MAX, u32::MAX),
                |(best_diff, best_osr, best_brg), osrval| {
                    // Compare source_clock_hz agaist with ((osrval + 1) * baudrate) to make sure
                    // (source_clock_hz / ((osrval + 1) * baudrate)) is not less than 0.
                    if source_clock_hz < ((osrval + 1) * baudrate) {
                        (best_diff, best_osr, best_brg)
                    } else {
                        let brgval = (source_clock_hz / ((osrval + 1) * baudrate)) - 1;
                        // We know brgval will not be less than 0 now, it should have already been a valid u32 value,
                        // then compare it agaist with 65535.
                        if brgval > 65535 {
                            (best_diff, best_osr, best_brg)
                        } else {
                            // Calculate the baud rate based on the BRG value
                            let candidate = source_clock_hz / ((osrval + 1) * (brgval + 1));

                            // Calculate the difference between the
                            // current baud rate and the desired baud rate
                            let diff = (candidate as i32 - baudrate as i32).unsigned_abs();

                            // Check if the current calculated difference is the best so far
                            if diff < best_diff {
                                (diff, osrval, brgval)
                            } else {
                                (best_diff, best_osr, best_brg)
                            }
                        }
                    }
                },
            );

            // Value over range
            if brg > 65535 {
                return Err(Error::UnsupportedBaudrate);
            }

            // SAFETY: unsafe only used for .bits()
            regs.osr().write(|w| unsafe { w.osrval().bits(osr as u8) });

            // SAFETY: unsafe only used for .bits()
            regs.brg().write(|w| unsafe { w.brgval().bits(brg as u16) });
        }

        Ok(())
    }

    fn set_uart_config<T: Instance>(config: Config) {
        let regs = T::info().regs;

        regs.cfg().modify(|_, w| w.enable().disabled());

        regs.cfg().modify(|_, w| {
            w.datalen()
                .variant(config.data_bits)
                .stoplen()
                .variant(config.stop_bits)
                .paritysel()
                .variant(config.parity)
                .loop_()
                .variant(config.loopback_mode)
                .syncen()
                .variant(config.operation)
                .clkpol()
                .variant(config.clock_polarity)
        });

        regs.cfg().modify(|_, w| w.enable().enabled());
    }

    /// Split the Uart into a transmitter and receiver, which is particularly
    /// useful when having two tasks correlating to transmitting and receiving.
    pub fn split(self) -> (UartTx<'a, M>, UartRx<'a, M>) {
        (self.tx, self.rx)
    }

    /// Split the Uart into a transmitter and receiver by mutable reference,
    /// which is particularly useful when having two tasks correlating to
    /// transmitting and receiving.
    pub fn split_ref(&mut self) -> (&mut UartTx<'a, M>, &mut UartRx<'a, M>) {
        (&mut self.tx, &mut self.rx)
    }
}

impl<'a> Uart<'a, Blocking> {
    /// Create a new blocking UART
    pub fn new_blocking<T: Instance>(
        _inner: Peri<'a, T>,
        tx: Peri<'a, impl TxPin<T>>,
        rx: Peri<'a, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self> {
        tx.as_tx();
        rx.as_rx();

        let tx = tx.into();
        let rx = rx.into();

        Self::init::<T>(Some(tx), Some(rx), None, None, config)?;

        Ok(Self {
            tx: UartTx::new_inner::<T>(None),
            rx: UartRx::new_inner::<T>(None),
        })
    }

    /// Read from UART RX blocking execution until done.
    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<()> {
        self.rx.blocking_read(buf)
    }

    /// Read from UART RX. Non-blocking version, bails out if it would
    /// block.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<()> {
        self.rx.read(buf)
    }

    /// Transmit the provided buffer blocking execution until done.
    pub fn blocking_write(&mut self, buf: &[u8]) -> Result<()> {
        self.tx.blocking_write(buf)
    }

    /// Transmit the provided buffer. Non-blocking version, bails out
    /// if it would block.
    pub fn write(&mut self, buf: &[u8]) -> Result<()> {
        self.tx.write(buf)
    }

    /// Flush UART TX blocking execution until done.
    pub fn blocking_flush(&mut self) -> Result<()> {
        self.tx.blocking_flush()
    }

    /// Flush UART TX.
    pub fn flush(&mut self) -> Result<()> {
        self.tx.flush()
    }
}

impl<'a> UartTx<'a, Async> {
    /// Create a new DMA enabled UART which can only send data
    pub fn new_async<T: Instance>(
        _inner: Peri<'a, T>,
        tx: Peri<'a, impl TxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        tx_dma: Peri<'a, impl TxDma<T>>,
        config: Config,
    ) -> Result<Self> {
        tx.as_tx();

        let _tx = tx.into();
        Uart::<Async>::init::<T>(Some(_tx), None, None, None, config)?;

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Ok(Self::new_inner::<T>(Some(tx_dma.into())))
    }

    /// Transmit the provided buffer asynchronously.
    pub async fn write(&mut self, buf: &[u8]) -> Result<()> {
        let regs = self.info.regs;

        // Disable DMA on completion/cancellation
        let _dma_guard = OnDrop::new(|| {
            regs.fifocfg().modify(|_, w| w.dmatx().disabled());
        });

        for chunk in buf.chunks(dma::MAX_CHUNK_SIZE) {
            regs.fifocfg().modify(|_, w| w.dmatx().enabled());

            let ch = self.tx_dma.as_mut().unwrap().reborrow();
            let transfer = unsafe { dma::write(ch, chunk, regs.fifowr().as_ptr() as *mut u8) };

            let res = select(
                transfer,
                poll_fn(|cx| {
                    UART_WAKERS[self.info.index].register(cx.waker());

                    self.info.regs.intenset().write(|w| {
                        w.framerren()
                            .set_bit()
                            .parityerren()
                            .set_bit()
                            .rxnoiseen()
                            .set_bit()
                            .aberren()
                            .set_bit()
                    });

                    let stat = self.info.regs.stat().read();

                    self.info.regs.stat().write(|w| {
                        w.framerrint()
                            .clear_bit_by_one()
                            .parityerrint()
                            .clear_bit_by_one()
                            .rxnoiseint()
                            .clear_bit_by_one()
                            .aberr()
                            .clear_bit_by_one()
                    });

                    if stat.framerrint().bit_is_set() {
                        Poll::Ready(Err(Error::Framing))
                    } else if stat.parityerrint().bit_is_set() {
                        Poll::Ready(Err(Error::Parity))
                    } else if stat.rxnoiseint().bit_is_set() {
                        Poll::Ready(Err(Error::Noise))
                    } else if stat.aberr().bit_is_set() {
                        Poll::Ready(Err(Error::Fail))
                    } else {
                        Poll::Pending
                    }
                }),
            )
            .await;

            match res {
                Either::First(()) | Either::Second(Ok(())) => (),
                Either::Second(e) => return e,
            }
        }

        Ok(())
    }

    /// Flush UART TX asynchronously.
    pub async fn flush(&mut self) -> Result<()> {
        self.wait_on(
            |me| {
                if me.info.regs.stat().read().txidle().bit_is_set() {
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Pending
                }
            },
            |me| {
                me.info.regs.intenset().write(|w| w.txidleen().set_bit());
            },
        )
        .await
    }

    /// Calls `f` to check if we are ready or not.
    /// If not, `g` is called once the waker is set (to eg enable the required interrupts).
    async fn wait_on<F, U, G>(&mut self, mut f: F, mut g: G) -> U
    where
        F: FnMut(&mut Self) -> Poll<U>,
        G: FnMut(&mut Self),
    {
        poll_fn(|cx| {
            // Register waker before checking condition, to ensure that wakes/interrupts
            // aren't lost between f() and g()
            UART_WAKERS[self.info.index].register(cx.waker());
            let r = f(self);

            if r.is_pending() {
                g(self);
            }

            r
        })
        .await
    }
}

impl<'a> UartRx<'a, Async> {
    /// Create a new DMA enabled UART which can only receive data
    pub fn new_async<T: Instance>(
        _inner: Peri<'a, T>,
        rx: Peri<'a, impl RxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        rx_dma: Peri<'a, impl RxDma<T>>,
        config: Config,
    ) -> Result<Self> {
        rx.as_rx();

        let _rx = rx.into();
        Uart::<Async>::init::<T>(None, Some(_rx), None, None, config)?;

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Ok(Self::new_inner::<T>(Some(rx_dma.into())))
    }

    /// Read from UART RX asynchronously.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<()> {
        let regs = self.info.regs;

        // Disable DMA on completion/cancellation
        let _dma_guard = OnDrop::new(|| {
            regs.fifocfg().modify(|_, w| w.dmarx().disabled());
        });

        for chunk in buf.chunks_mut(dma::MAX_CHUNK_SIZE) {
            regs.fifocfg().modify(|_, w| w.dmarx().enabled());

            let ch = self.rx_dma.as_mut().unwrap().reborrow();
            let transfer = unsafe { dma::read(ch, regs.fiford().as_ptr() as *const u8, chunk) };

            let res = select(
                transfer,
                poll_fn(|cx| {
                    UART_WAKERS[self.info.index].register(cx.waker());

                    self.info.regs.intenset().write(|w| {
                        w.framerren()
                            .set_bit()
                            .parityerren()
                            .set_bit()
                            .rxnoiseen()
                            .set_bit()
                            .aberren()
                            .set_bit()
                    });

                    let stat = self.info.regs.stat().read();

                    self.info.regs.stat().write(|w| {
                        w.framerrint()
                            .clear_bit_by_one()
                            .parityerrint()
                            .clear_bit_by_one()
                            .rxnoiseint()
                            .clear_bit_by_one()
                            .aberr()
                            .clear_bit_by_one()
                    });

                    if stat.framerrint().bit_is_set() {
                        Poll::Ready(Err(Error::Framing))
                    } else if stat.parityerrint().bit_is_set() {
                        Poll::Ready(Err(Error::Parity))
                    } else if stat.rxnoiseint().bit_is_set() {
                        Poll::Ready(Err(Error::Noise))
                    } else if stat.aberr().bit_is_set() {
                        Poll::Ready(Err(Error::Fail))
                    } else {
                        Poll::Pending
                    }
                }),
            )
            .await;

            match res {
                Either::First(()) | Either::Second(Ok(())) => (),
                Either::Second(e) => return e,
            }
        }

        Ok(())
    }
}

impl<'a> Uart<'a, Async> {
    /// Create a new DMA enabled UART
    pub fn new_async<T: Instance>(
        _inner: Peri<'a, T>,
        tx: Peri<'a, impl TxPin<T>>,
        rx: Peri<'a, impl RxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        tx_dma: Peri<'a, impl TxDma<T>>,
        rx_dma: Peri<'a, impl RxDma<T>>,
        config: Config,
    ) -> Result<Self> {
        tx.as_tx();
        rx.as_rx();

        let tx = tx.into();
        let rx = rx.into();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self::init::<T>(Some(tx), Some(rx), None, None, config)?;

        Ok(Self {
            tx: UartTx::new_inner::<T>(Some(tx_dma.into())),
            rx: UartRx::new_inner::<T>(Some(rx_dma.into())),
        })
    }

    /// Create a new DMA enabled UART with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts<T: Instance>(
        _inner: Peri<'a, T>,
        tx: Peri<'a, impl TxPin<T>>,
        rx: Peri<'a, impl RxPin<T>>,
        rts: Peri<'a, impl RtsPin<T>>,
        cts: Peri<'a, impl CtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        tx_dma: Peri<'a, impl TxDma<T>>,
        rx_dma: Peri<'a, impl RxDma<T>>,
        config: Config,
    ) -> Result<Self> {
        tx.as_tx();
        rx.as_rx();
        rts.as_rts();
        cts.as_cts();

        let tx = tx.into();
        let rx = rx.into();
        let rts = rts.into();
        let cts = cts.into();

        Self::init::<T>(Some(tx), Some(rx), Some(rts), Some(cts), config)?;

        Ok(Self {
            tx: UartTx::new_inner::<T>(Some(tx_dma.into())),
            rx: UartRx::new_inner::<T>(Some(rx_dma.into())),
        })
    }

    /// Read from UART RX.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<()> {
        self.rx.read(buf).await
    }

    /// Transmit the provided buffer.
    pub async fn write(&mut self, buf: &[u8]) -> Result<()> {
        self.tx.write(buf).await
    }

    /// Flush UART TX.
    pub async fn flush(&mut self) -> Result<()> {
        self.tx.flush().await
    }
}

impl embedded_hal_02::serial::Read<u8> for UartRx<'_, Blocking> {
    type Error = Error;

    fn read(&mut self) -> core::result::Result<u8, nb::Error<Self::Error>> {
        let mut buf = [0; 1];

        match self.read(&mut buf) {
            Ok(_) => Ok(buf[0]),
            Err(Error::RxFifoEmpty) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }
}

impl embedded_hal_02::serial::Write<u8> for UartTx<'_, Blocking> {
    type Error = Error;

    fn write(&mut self, word: u8) -> core::result::Result<(), nb::Error<Self::Error>> {
        match self.write(&[word]) {
            Ok(_) => Ok(()),
            Err(Error::TxFifoFull) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }

    fn flush(&mut self) -> core::result::Result<(), nb::Error<Self::Error>> {
        match self.flush() {
            Ok(_) => Ok(()),
            Err(Error::TxBusy) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }
}

impl embedded_hal_02::blocking::serial::Write<u8> for UartTx<'_, Blocking> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> core::result::Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> core::result::Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl embedded_hal_02::serial::Read<u8> for Uart<'_, Blocking> {
    type Error = Error;

    fn read(&mut self) -> core::result::Result<u8, nb::Error<Self::Error>> {
        embedded_hal_02::serial::Read::read(&mut self.rx)
    }
}

impl embedded_hal_02::serial::Write<u8> for Uart<'_, Blocking> {
    type Error = Error;

    fn write(&mut self, word: u8) -> core::result::Result<(), nb::Error<Self::Error>> {
        embedded_hal_02::serial::Write::write(&mut self.tx, word)
    }

    fn flush(&mut self) -> core::result::Result<(), nb::Error<Self::Error>> {
        embedded_hal_02::serial::Write::flush(&mut self.tx)
    }
}

impl embedded_hal_02::blocking::serial::Write<u8> for Uart<'_, Blocking> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> core::result::Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> core::result::Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl embedded_hal_nb::serial::Error for Error {
    fn kind(&self) -> embedded_hal_nb::serial::ErrorKind {
        match *self {
            Self::Framing => embedded_hal_nb::serial::ErrorKind::FrameFormat,
            Self::Overrun => embedded_hal_nb::serial::ErrorKind::Overrun,
            Self::Parity => embedded_hal_nb::serial::ErrorKind::Parity,
            Self::Noise => embedded_hal_nb::serial::ErrorKind::Noise,
            _ => embedded_hal_nb::serial::ErrorKind::Other,
        }
    }
}

impl embedded_hal_nb::serial::ErrorType for UartRx<'_, Blocking> {
    type Error = Error;
}

impl embedded_hal_nb::serial::ErrorType for UartTx<'_, Blocking> {
    type Error = Error;
}

impl embedded_hal_nb::serial::ErrorType for Uart<'_, Blocking> {
    type Error = Error;
}

impl embedded_hal_nb::serial::Read for UartRx<'_, Blocking> {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let mut buf = [0; 1];

        match self.read(&mut buf) {
            Ok(_) => Ok(buf[0]),
            Err(Error::RxFifoEmpty) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }
}

impl embedded_hal_nb::serial::Write for UartTx<'_, Blocking> {
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        match self.write(&[word]) {
            Ok(_) => Ok(()),
            Err(Error::TxFifoFull) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        match self.flush() {
            Ok(_) => Ok(()),
            Err(Error::TxBusy) => Err(nb::Error::WouldBlock),
            Err(e) => Err(nb::Error::Other(e)),
        }
    }
}

impl embedded_hal_nb::serial::Read for Uart<'_, Blocking> {
    fn read(&mut self) -> core::result::Result<u8, nb::Error<Self::Error>> {
        embedded_hal_02::serial::Read::read(&mut self.rx)
    }
}

impl embedded_hal_nb::serial::Write for Uart<'_, Blocking> {
    fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
        self.blocking_write(&[char]).map_err(nb::Error::Other)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.blocking_flush().map_err(nb::Error::Other)
    }
}

struct Info {
    regs: &'static crate::pac::usart0::RegisterBlock,
    index: usize,
    refcnt: AtomicU8,
}

trait SealedInstance {
    fn info() -> Info;
    fn index() -> usize;
}

/// UART interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

const UART_COUNT: usize = 8;
static UART_WAKERS: [AtomicWaker; UART_COUNT] = [const { AtomicWaker::new() }; UART_COUNT];

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let waker = &UART_WAKERS[T::index()];
        let regs = T::info().regs;
        let stat = regs.intstat().read();

        if stat.txidle().bit_is_set()
            || stat.framerrint().bit_is_set()
            || stat.parityerrint().bit_is_set()
            || stat.rxnoiseint().bit_is_set()
            || stat.aberrint().bit_is_set()
        {
            regs.intenclr().write(|w| {
                w.txidleclr()
                    .set_bit()
                    .framerrclr()
                    .set_bit()
                    .parityerrclr()
                    .set_bit()
                    .rxnoiseclr()
                    .set_bit()
                    .aberrclr()
                    .set_bit()
            });
        }

        waker.wake();
    }
}

/// UART instance trait.
#[allow(private_bounds)]
pub trait Instance: crate::flexcomm::IntoUsart + SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this UART instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_instance {
    ($($n:expr),*) => {
        $(
            paste!{
                impl SealedInstance for crate::peripherals::[<FLEXCOMM $n>] {
                    fn info() -> Info {
                        Info {
                            regs: unsafe { &*crate::pac::[<Usart $n>]::ptr() },
                            index: $n,
			    refcnt: AtomicU8::new(0),
                        }
                    }

                    #[inline]
                    fn index() -> usize {
                        $n
                    }
                }

                impl Instance for crate::peripherals::[<FLEXCOMM $n>] {
                    type Interrupt = crate::interrupt::typelevel::[<FLEXCOMM $n>];
                }
            }
        )*
    };
}

impl_instance!(0, 1, 2, 3, 4, 5, 6, 7);

impl<T: Pin> Sealed for T {}

/// io configuration trait for Uart Tx configuration
pub trait TxPin<T: Instance>: Pin + Sealed + PeripheralType {
    /// convert the pin to appropriate function for Uart Tx  usage
    fn as_tx(&self);
}

/// io configuration trait for Uart Rx configuration
pub trait RxPin<T: Instance>: Pin + Sealed + PeripheralType {
    /// convert the pin to appropriate function for Uart Rx  usage
    fn as_rx(&self);
}

/// io configuration trait for Uart Cts
pub trait CtsPin<T: Instance>: Pin + Sealed + PeripheralType {
    /// convert the pin to appropriate function for Uart Cts usage
    fn as_cts(&self);
}

/// io configuration trait for Uart Rts
pub trait RtsPin<T: Instance>: Pin + Sealed + PeripheralType {
    /// convert the pin to appropriate function for Uart Rts usage
    fn as_rts(&self);
}

macro_rules! impl_pin_trait {
    ($fcn:ident, $mode:ident, $($pin:ident, $fn:ident),*) => {
        paste! {
            $(
                impl [<$mode:camel Pin>]<crate::peripherals::$fcn> for crate::peripherals::$pin {
                    fn [<as_ $mode>](&self) {
                        // UM11147 table 507 pg 495
                        self.set_function(crate::iopctl::Function::$fn)
                            .set_pull(Pull::None)
                            .enable_input_buffer()
                            .set_slew_rate(SlewRate::Standard)
                            .set_drive_strength(DriveStrength::Normal)
                            .disable_analog_multiplex()
                            .set_drive_mode(DriveMode::PushPull)
                            .set_input_inverter(Inverter::Disabled);
                    }
                }
            )*
        }
    };
}

// FLEXCOMM0
impl_pin_trait!(FLEXCOMM0, tx, PIO0_1, F1, PIO3_1, F5);
impl_pin_trait!(FLEXCOMM0, rx, PIO0_2, F1, PIO3_2, F5);
impl_pin_trait!(FLEXCOMM0, cts, PIO0_3, F1, PIO3_3, F5);
impl_pin_trait!(FLEXCOMM0, rts, PIO0_4, F1, PIO3_4, F5);

// FLEXCOMM1
impl_pin_trait!(FLEXCOMM1, tx, PIO0_8, F1, PIO7_26, F1);
impl_pin_trait!(FLEXCOMM1, rx, PIO0_9, F1, PIO7_27, F1);
impl_pin_trait!(FLEXCOMM1, cts, PIO0_10, F1, PIO7_28, F1);
impl_pin_trait!(FLEXCOMM1, rts, PIO0_11, F1, PIO7_29, F1);

// FLEXCOMM2
impl_pin_trait!(FLEXCOMM2, tx, PIO0_15, F1, PIO7_30, F5);
impl_pin_trait!(FLEXCOMM2, rx, PIO0_16, F1, PIO7_31, F5);
impl_pin_trait!(FLEXCOMM2, cts, PIO0_17, F1, PIO4_8, F5);
impl_pin_trait!(FLEXCOMM2, rts, PIO0_18, F1);

// FLEXCOMM3
impl_pin_trait!(FLEXCOMM3, tx, PIO0_22, F1);
impl_pin_trait!(FLEXCOMM3, rx, PIO0_23, F1);
impl_pin_trait!(FLEXCOMM3, cts, PIO0_24, F1);
impl_pin_trait!(FLEXCOMM3, rts, PIO0_25, F1);

// FLEXCOMM4
impl_pin_trait!(FLEXCOMM4, tx, PIO0_29, F1);
impl_pin_trait!(FLEXCOMM4, rx, PIO0_30, F1);
impl_pin_trait!(FLEXCOMM4, cts, PIO0_31, F1);
impl_pin_trait!(FLEXCOMM4, rts, PIO1_0, F1);

// FLEXCOMM5
impl_pin_trait!(FLEXCOMM5, tx, PIO1_4, F1, PIO3_16, F5);
impl_pin_trait!(FLEXCOMM5, rx, PIO1_5, F1, PIO3_17, F5);
impl_pin_trait!(FLEXCOMM5, cts, PIO1_6, F1, PIO3_18, F5);
impl_pin_trait!(FLEXCOMM5, rts, PIO1_7, F1, PIO3_23, F5);

// FLEXCOMM6
impl_pin_trait!(FLEXCOMM6, tx, PIO3_26, F1);
impl_pin_trait!(FLEXCOMM6, rx, PIO3_27, F1);
impl_pin_trait!(FLEXCOMM6, cts, PIO3_28, F1);
impl_pin_trait!(FLEXCOMM6, rts, PIO3_29, F1);

// FLEXCOMM7
impl_pin_trait!(FLEXCOMM7, tx, PIO4_1, F1);
impl_pin_trait!(FLEXCOMM7, rx, PIO4_2, F1);
impl_pin_trait!(FLEXCOMM7, cts, PIO4_3, F1);
impl_pin_trait!(FLEXCOMM7, rts, PIO4_4, F1);

/// UART Tx DMA trait.
#[allow(private_bounds)]
pub trait TxDma<T: Instance>: crate::dma::Channel {}

/// UART Rx DMA trait.
#[allow(private_bounds)]
pub trait RxDma<T: Instance>: crate::dma::Channel {}

macro_rules! impl_dma {
    ($fcn:ident, $mode:ident, $dma:ident) => {
        paste! {
            impl [<$mode Dma>]<crate::peripherals::$fcn> for crate::peripherals::$dma {}
        }
    };
}

impl_dma!(FLEXCOMM0, Rx, DMA0_CH0);
impl_dma!(FLEXCOMM0, Tx, DMA0_CH1);

impl_dma!(FLEXCOMM1, Rx, DMA0_CH2);
impl_dma!(FLEXCOMM1, Tx, DMA0_CH3);

impl_dma!(FLEXCOMM2, Rx, DMA0_CH4);
impl_dma!(FLEXCOMM2, Tx, DMA0_CH5);

impl_dma!(FLEXCOMM3, Rx, DMA0_CH6);
impl_dma!(FLEXCOMM3, Tx, DMA0_CH7);

impl_dma!(FLEXCOMM4, Rx, DMA0_CH8);
impl_dma!(FLEXCOMM4, Tx, DMA0_CH9);

impl_dma!(FLEXCOMM5, Rx, DMA0_CH10);
impl_dma!(FLEXCOMM5, Tx, DMA0_CH11);

impl_dma!(FLEXCOMM6, Rx, DMA0_CH12);
impl_dma!(FLEXCOMM6, Tx, DMA0_CH13);

impl_dma!(FLEXCOMM7, Rx, DMA0_CH14);
impl_dma!(FLEXCOMM7, Tx, DMA0_CH15);
