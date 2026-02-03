use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_hal_internal::Peri;
use embassy_hal_internal::atomic_ring_buffer::RingBuffer;
use embassy_sync::waitqueue::AtomicWaker;

use super::*;
use crate::clocks::WakeGuard;
use crate::interrupt::typelevel::Interrupt;
use crate::interrupt::{self};

// ============================================================================
// STATIC STATE MANAGEMENT
// ============================================================================

/// State for buffered LPUART operations
pub struct State {
    tx_waker: AtomicWaker,
    tx_buf: RingBuffer,
    tx_done: AtomicBool,
    rx_waker: AtomicWaker,
    rx_buf: RingBuffer,
    initialized: AtomicBool,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    /// Create a new state instance
    pub const fn new() -> Self {
        Self {
            tx_waker: AtomicWaker::new(),
            tx_buf: RingBuffer::new(),
            tx_done: AtomicBool::new(true),
            rx_waker: AtomicWaker::new(),
            rx_buf: RingBuffer::new(),
            initialized: AtomicBool::new(false),
        }
    }
}
// ============================================================================
// BUFFERED DRIVER STRUCTURES
// ============================================================================

/// Buffered LPUART driver
pub struct BufferedLpuart<'a> {
    tx: BufferedLpuartTx<'a>,
    rx: BufferedLpuartRx<'a>,
}

/// Buffered LPUART TX driver
pub struct BufferedLpuartTx<'a> {
    info: &'static Info,
    state: &'static State,
    _tx_pin: Peri<'a, AnyPin>,
    _cts_pin: Option<Peri<'a, AnyPin>>,
    _wg: Option<WakeGuard>,
}

/// Buffered LPUART RX driver
pub struct BufferedLpuartRx<'a> {
    info: &'static Info,
    state: &'static State,
    _rx_pin: Peri<'a, AnyPin>,
    _rts_pin: Option<Peri<'a, AnyPin>>,
    _wg: Option<WakeGuard>,
}

// ============================================================================
// BUFFERED LPUART IMPLEMENTATION
// ============================================================================

impl<'a> BufferedLpuart<'a> {
    /// Common initialization logic
    fn init_common<T: Instance>(
        _inner: &Peri<'a, T>,
        tx_buffer: Option<&'a mut [u8]>,
        rx_buffer: Option<&'a mut [u8]>,
        config: &Config,
        enable_tx: bool,
        enable_rx: bool,
        enable_rts: bool,
        enable_cts: bool,
    ) -> Result<(&'static State, Option<WakeGuard>)> {
        let state = T::buffered_state();

        if state.initialized.load(Ordering::Relaxed) {
            return Err(Error::InvalidArgument);
        }

        // Initialize buffers
        if let Some(tx_buffer) = tx_buffer {
            if tx_buffer.is_empty() {
                return Err(Error::InvalidArgument);
            }
            unsafe { state.tx_buf.init(tx_buffer.as_mut_ptr(), tx_buffer.len()) };
        }

        if let Some(rx_buffer) = rx_buffer {
            if rx_buffer.is_empty() {
                return Err(Error::InvalidArgument);
            }
            unsafe { state.rx_buf.init(rx_buffer.as_mut_ptr(), rx_buffer.len()) };
        }

        state.initialized.store(true, Ordering::Relaxed);

        // Enable clocks and initialize hardware
        let conf = LpuartConfig {
            power: config.power,
            source: config.source,
            div: config.div,
            instance: T::CLOCK_INSTANCE,
        };
        let parts = unsafe { enable_and_reset::<T>(&conf).map_err(Error::ClockSetup)? };

        Self::init_hardware(
            T::info(),
            *config,
            parts.freq,
            enable_tx,
            enable_rx,
            enable_rts,
            enable_cts,
        )?;

        Ok((state, parts.wake_guard))
    }

    /// Helper for full-duplex initialization
    fn new_inner<T: Instance>(
        inner: Peri<'a, T>,
        tx_pin: Peri<'a, AnyPin>,
        rx_pin: Peri<'a, AnyPin>,
        rts_pin: Option<Peri<'a, AnyPin>>,
        cts_pin: Option<Peri<'a, AnyPin>>,
        tx_buffer: &'a mut [u8],
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<(BufferedLpuartTx<'a>, BufferedLpuartRx<'a>)> {
        let (state, wg) = Self::init_common::<T>(
            &inner,
            Some(tx_buffer),
            Some(rx_buffer),
            &config,
            true,
            true,
            rts_pin.is_some(),
            cts_pin.is_some(),
        )?;

        let tx = BufferedLpuartTx {
            info: T::info(),
            state,
            _tx_pin: tx_pin,
            _cts_pin: cts_pin,
            _wg: wg.clone(),
        };

        let rx = BufferedLpuartRx {
            info: T::info(),
            state,
            _rx_pin: rx_pin,
            _rts_pin: rts_pin,
            _wg: wg,
        };

        Ok((tx, rx))
    }

    /// Common hardware initialization logic
    fn init_hardware(
        info: &'static Info,
        config: Config,
        clock_freq: u32,
        enable_tx: bool,
        enable_rx: bool,
        enable_rts: bool,
        enable_cts: bool,
    ) -> Result<()> {
        // Perform standard initialization
        perform_software_reset(info);
        disable_transceiver(info);
        configure_baudrate(info, config.baudrate_bps, clock_freq)?;
        configure_frame_format(info, &config);
        configure_control_settings(info, &config);
        configure_fifo(info, &config);
        clear_all_status_flags(info);
        configure_flow_control(info, enable_rts, enable_cts, &config);
        configure_bit_order(info, config.msb_first);

        // Enable interrupts for buffered operation
        cortex_m::interrupt::free(|_| {
            info.regs().ctrl().modify(|w| {
                w.set_rie(true); // RX interrupt
                w.set_orie(true); // Overrun interrupt
                w.set_peie(true); // Parity error interrupt
                w.set_feie(true); // Framing error interrupt
                w.set_neie(true); // Noise error interrupt
            });
        });

        // Enable the transceiver
        enable_transceiver(info, enable_rx, enable_tx);

        Ok(())
    }

    /// Create a new full duplex buffered LPUART.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new<T: Instance>(
        inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
        tx_pin.as_tx();
        rx_pin.as_rx();

        let (tx, rx) = Self::new_inner::<T>(
            inner,
            tx_pin.into(),
            rx_pin.into(),
            None,
            None,
            tx_buffer,
            rx_buffer,
            config,
        )?;

        // Enable interrupt
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Ok(Self { tx, rx })
    }

    /// Create a new buffered LPUART instance with RTS/CTS flow control.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_with_rtscts<T: Instance>(
        inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        rts_pin: Peri<'a, impl RtsPin<T>>,
        cts_pin: Peri<'a, impl CtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
        tx_pin.as_tx();
        rx_pin.as_rx();
        rts_pin.as_rts();
        cts_pin.as_cts();

        let (tx, rx) = Self::new_inner::<T>(
            inner,
            tx_pin.into(),
            rx_pin.into(),
            Some(rts_pin.into()),
            Some(cts_pin.into()),
            tx_buffer,
            rx_buffer,
            config,
        )?;

        // Enable interrupt
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Ok(Self { tx, rx })
    }

    /// Create a new buffered LPUART with only RTS flow control (RX flow control).
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_with_rts<T: Instance>(
        inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        rts_pin: Peri<'a, impl RtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
        tx_pin.as_tx();
        rx_pin.as_rx();
        rts_pin.as_rts();

        let (tx, rx) = Self::new_inner::<T>(
            inner,
            tx_pin.into(),
            rx_pin.into(),
            Some(rts_pin.into()),
            None,
            tx_buffer,
            rx_buffer,
            config,
        )?;

        // Enable interrupt
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Ok(Self { tx, rx })
    }

    /// Create a new buffered LPUART with only CTS flow control (TX flow control).
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_with_cts<T: Instance>(
        inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        cts_pin: Peri<'a, impl CtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
        tx_pin.as_tx();
        rx_pin.as_rx();
        cts_pin.as_cts();

        let (tx, rx) = Self::new_inner::<T>(
            inner,
            tx_pin.into(),
            rx_pin.into(),
            None,
            Some(cts_pin.into()),
            tx_buffer,
            rx_buffer,
            config,
        )?;

        // Enable interrupt
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Ok(Self { tx, rx })
    }

    /// Split the buffered LPUART into separate TX and RX parts
    pub fn split(self) -> (BufferedLpuartTx<'a>, BufferedLpuartRx<'a>) {
        (self.tx, self.rx)
    }

    /// Get mutable references to TX and RX parts
    pub fn split_ref(&mut self) -> (&mut BufferedLpuartTx<'a>, &mut BufferedLpuartRx<'a>) {
        (&mut self.tx, &mut self.rx)
    }
}

// ============================================================================
// BUFFERED TX IMPLEMENTATION
// ============================================================================

impl<'a> BufferedLpuartTx<'a> {
    /// Helper for TX-only initialization
    fn new_inner<T: Instance>(
        inner: Peri<'a, T>,
        tx_pin: Peri<'a, AnyPin>,
        cts_pin: Option<Peri<'a, AnyPin>>,
        tx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<BufferedLpuartTx<'a>> {
        let (state, wg) = BufferedLpuart::init_common::<T>(
            &inner,
            Some(tx_buffer),
            None,
            &config,
            true,
            false,
            false,
            cts_pin.is_some(),
        )?;

        Ok(BufferedLpuartTx {
            info: T::info(),
            state,
            _tx_pin: tx_pin,
            _cts_pin: cts_pin,
            _wg: wg,
        })
    }

    /// Create a new TX-only LPUART.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new<T: Instance>(
        inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
        tx_pin.as_tx();

        let res = Self::new_inner::<T>(inner, tx_pin.into(), None, tx_buffer, config)?;

        // Enable interrupt
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Ok(res)
    }

    /// Create a new TX-only buffered LPUART with CTS flow control.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_with_cts<T: Instance>(
        inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        cts_pin: Peri<'a, impl CtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
        tx_pin.as_tx();
        cts_pin.as_cts();

        let res = Self::new_inner::<T>(inner, tx_pin.into(), Some(cts_pin.into()), tx_buffer, config)?;

        // Enable interrupt
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Ok(res)
    }
}

impl<'a> BufferedLpuartTx<'a> {
    /// Write data asynchronously
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut written = 0;

        for &byte in buf {
            // Wait for space in the buffer
            poll_fn(|cx| {
                self.state.tx_waker.register(cx.waker());

                let mut writer = unsafe { self.state.tx_buf.writer() };
                if writer.push_one(byte) {
                    // Enable TX interrupt to start transmission
                    cortex_m::interrupt::free(|_| {
                        self.info.regs().ctrl().modify(|w| w.set_tie(true));
                    });
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Pending
                }
            })
            .await?;

            written += 1;
        }

        Ok(written)
    }

    /// Flush the TX buffer and wait for transmission to complete
    pub async fn flush(&mut self) -> Result<()> {
        // Wait for TX buffer to empty and transmission to complete
        poll_fn(|cx| {
            self.state.tx_waker.register(cx.waker());

            let tx_empty = self.state.tx_buf.is_empty();
            let fifo_empty = self.info.regs().water().read().txcount() == 0;
            let tc_complete = self.info.regs().stat().read().tc() == Tc::COMPLETE;

            if tx_empty && fifo_empty && tc_complete {
                Poll::Ready(Ok(()))
            } else {
                // Enable appropriate interrupt
                cortex_m::interrupt::free(|_| {
                    if !tx_empty {
                        self.info.regs().ctrl().modify(|w| w.set_tie(true));
                    } else {
                        self.info.regs().ctrl().modify(|w| w.set_tcie(true));
                    }
                });
                Poll::Pending
            }
        })
        .await
    }

    /// Try to write without blocking
    pub fn try_write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut writer = unsafe { self.state.tx_buf.writer() };
        let mut written = 0;

        for &byte in buf {
            if writer.push_one(byte) {
                written += 1;
            } else {
                break;
            }
        }

        if written > 0 {
            // Enable TX interrupt to start transmission
            cortex_m::interrupt::free(|_| {
                self.info.regs().ctrl().modify(|w| w.set_tie(true));
            });
        }

        Ok(written)
    }
}

// ============================================================================
// BUFFERED RX IMPLEMENTATION
// ============================================================================

impl<'a> BufferedLpuartRx<'a> {
    /// Helper for RX-only initialization
    fn new_inner<T: Instance>(
        inner: Peri<'a, T>,
        rx_pin: Peri<'a, AnyPin>,
        rts_pin: Option<Peri<'a, AnyPin>>,
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<BufferedLpuartRx<'a>> {
        let (state, wg) = BufferedLpuart::init_common::<T>(
            &inner,
            None,
            Some(rx_buffer),
            &config,
            false,
            true,
            rts_pin.is_some(),
            false,
        )?;

        Ok(BufferedLpuartRx {
            info: T::info(),
            state,
            _rx_pin: rx_pin,
            _rts_pin: rts_pin,
            _wg: wg,
        })
    }

    /// Create a new RX-only buffered LPUART.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new<T: Instance>(
        inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
        rx_pin.as_rx();

        let res = Self::new_inner::<T>(inner, rx_pin.into(), None, rx_buffer, config)?;

        // Enable interrupt
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Ok(res)
    }

    /// Create a new RX-only buffered LPUART with RTS flow control.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_with_rts<T: Instance>(
        inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        rts_pin: Peri<'a, impl RtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
        rx_pin.as_rx();
        rts_pin.as_rts();

        let res = Self::new_inner::<T>(inner, rx_pin.into(), Some(rts_pin.into()), rx_buffer, config)?;

        // Enable interrupt
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Ok(res)
    }
}

impl<'a> BufferedLpuartRx<'a> {
    /// Read data asynchronously
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut read = 0;

        // Try to read available data
        poll_fn(|cx| {
            self.state.rx_waker.register(cx.waker());

            // Disable RX interrupt while reading from buffer
            cortex_m::interrupt::free(|_| {
                self.info.regs().ctrl().modify(|w| w.set_rie(false));
            });

            let mut reader = unsafe { self.state.rx_buf.reader() };
            let available = reader.pop(|data| {
                let to_copy = core::cmp::min(data.len(), buf.len() - read);
                if to_copy > 0 {
                    buf[read..read + to_copy].copy_from_slice(&data[..to_copy]);
                    read += to_copy;
                }
                to_copy
            });

            // Re-enable RX interrupt
            cortex_m::interrupt::free(|_| {
                self.info.regs().ctrl().modify(|w| w.set_rie(true));
            });

            if read > 0 {
                Poll::Ready(Ok(read))
            } else if available == 0 {
                Poll::Pending
            } else {
                Poll::Ready(Ok(0))
            }
        })
        .await
    }

    /// Try to read without blocking
    pub fn try_read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        // Disable RX interrupt while reading from buffer
        cortex_m::interrupt::free(|_| {
            self.info.regs().ctrl().modify(|w| w.set_rie(false));
        });

        let mut reader = unsafe { self.state.rx_buf.reader() };
        let read = reader.pop(|data| {
            let to_copy = core::cmp::min(data.len(), buf.len());
            if to_copy > 0 {
                buf[..to_copy].copy_from_slice(&data[..to_copy]);
            }
            to_copy
        });

        // Re-enable RX interrupt
        cortex_m::interrupt::free(|_| {
            self.info.regs().ctrl().modify(|w| w.set_rie(true));
        });

        Ok(read)
    }
}

// ============================================================================
// DROP TRAIT IMPLEMENTATIONS
// ============================================================================

impl<'a> Drop for BufferedLpuartTx<'a> {
    fn drop(&mut self) {
        self._tx_pin.set_as_disabled();
        if let Some(cts_pin) = &self._cts_pin {
            cts_pin.set_as_disabled();
        }
    }
}

impl<'a> Drop for BufferedLpuartRx<'a> {
    fn drop(&mut self) {
        self._rx_pin.set_as_disabled();
        if let Some(rts_pin) = &self._rts_pin {
            rts_pin.set_as_disabled();
        }
    }
}

// ============================================================================
// INTERRUPT HANDLER
// ============================================================================

/// Buffered UART interrupt handler
pub struct BufferedInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> crate::interrupt::typelevel::Handler<T::Interrupt> for BufferedInterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();
        unsafe {
            let regs = T::info().regs();
            let state = T::buffered_state();

            // Check if this instance is initialized
            if !state.initialized.load(Ordering::Relaxed) {
                return;
            }

            let ctrl = regs.ctrl().read();
            let stat = regs.stat().read();
            let has_fifo = regs.param().read().rxfifo() > 0;

            // Handle overrun error
            if stat.or() {
                regs.stat().write(|w| w.set_or(true));
                T::PERF_INT_WAKE_INCR();
                state.rx_waker.wake();
                return;
            }

            // Clear other error flags
            if stat.pf() {
                regs.stat().write(|w| w.set_pf(true));
            }
            if stat.fe() {
                regs.stat().write(|w| w.set_fe(true));
            }
            if stat.nf() {
                regs.stat().write(|w| w.set_nf(true));
            }

            // Handle RX data
            if ctrl.rie() && (has_data(T::info()) || stat.idle()) {
                let mut pushed_any = false;
                let mut writer = state.rx_buf.writer();

                if has_fifo {
                    // Read from FIFO
                    while regs.water().read().rxcount() > 0 {
                        let byte = (regs.data().read().0 & 0xFF) as u8;
                        if writer.push_one(byte) {
                            pushed_any = true;
                        } else {
                            // Buffer full, stop reading
                            break;
                        }
                    }
                } else {
                    // Read single byte
                    if regs.stat().read().rdrf() {
                        let byte = (regs.data().read().0 & 0xFF) as u8;
                        if writer.push_one(byte) {
                            pushed_any = true;
                        }
                    }
                }

                if pushed_any {
                    T::PERF_INT_WAKE_INCR();
                    state.rx_waker.wake();
                }

                // Clear idle flag if set
                if stat.idle() {
                    regs.stat().write(|w| w.set_idle(true));
                }
            }

            // Handle TX data
            if ctrl.tie() {
                let mut sent_any = false;
                let mut reader = state.tx_buf.reader();

                // Send data while TX buffer is ready and we have data
                while regs.stat().read().tdre() == Tdre::NO_TXDATA {
                    if let Some(byte) = reader.pop_one() {
                        regs.data().write(|w| w.0 = u32::from(byte));
                        sent_any = true;
                    } else {
                        // No more data to send
                        break;
                    }
                }

                if sent_any {
                    T::PERF_INT_WAKE_INCR();
                    state.tx_waker.wake();
                }

                // If buffer is empty, switch to TC interrupt or disable
                if state.tx_buf.is_empty() {
                    cortex_m::interrupt::free(|_| {
                        regs.ctrl().modify(|w| {
                            w.set_tie(false);
                            w.set_tcie(true);
                        });
                    });
                }
            }

            // Handle transmission complete
            if ctrl.tcie() && regs.stat().read().tc() == Tc::COMPLETE {
                state.tx_done.store(true, Ordering::Release);
                T::PERF_INT_WAKE_INCR();
                state.tx_waker.wake();

                // Disable TC interrupt
                cortex_m::interrupt::free(|_| {
                    regs.ctrl().modify(|w| w.set_tcie(false));
                });
            }
        }
    }
}

// ============================================================================
// EMBEDDED-IO ASYNC TRAIT IMPLEMENTATIONS
// ============================================================================

impl embedded_io_async::ErrorType for BufferedLpuartTx<'_> {
    type Error = Error;
}

impl embedded_io_async::ErrorType for BufferedLpuartRx<'_> {
    type Error = Error;
}

impl embedded_io_async::ErrorType for BufferedLpuart<'_> {
    type Error = Error;
}

impl embedded_io_async::Write for BufferedLpuartTx<'_> {
    async fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        self.write(buf).await
    }

    async fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        self.flush().await
    }
}

impl embedded_io_async::Read for BufferedLpuartRx<'_> {
    async fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        self.read(buf).await
    }
}

impl embedded_io_async::Write for BufferedLpuart<'_> {
    async fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        self.tx.write(buf).await
    }

    async fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        self.tx.flush().await
    }
}

impl embedded_io_async::Read for BufferedLpuart<'_> {
    async fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        self.rx.read(buf).await
    }
}
