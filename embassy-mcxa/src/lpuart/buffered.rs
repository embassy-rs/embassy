use core::marker::PhantomData;

use embassy_hal_internal::Peri;

use super::*;
use crate::clocks::WakeGuard;
use crate::interrupt::typelevel::Interrupt;
use crate::interrupt::{self};

/// Async buffered mode where the bytes are pumped by the interrupt handler.
pub struct Buffered;
impl sealed::Sealed for Buffered {}
impl Mode for Buffered {}

impl<'a> Lpuart<'a, Buffered> {
    /// Common initialization logic
    fn init_buffered<T: Instance>(
        tx_buffer: Option<&'a mut [u8]>,
        rx_buffer: Option<&'a mut [u8]>,
        config: Config,
        enable_tx: bool,
        enable_rx: bool,
        enable_rts: bool,
        enable_cts: bool,
    ) -> Result<Option<WakeGuard>, Error> {
        // Initialize buffers
        if let Some(tx_buffer) = tx_buffer {
            if tx_buffer.is_empty() {
                return Err(Error::InvalidArgument);
            }
            unsafe { T::state().tx_buf.init(tx_buffer.as_mut_ptr(), tx_buffer.len()) };
        }

        if let Some(rx_buffer) = rx_buffer {
            if rx_buffer.is_empty() {
                return Err(Error::InvalidArgument);
            }
            unsafe { T::state().rx_buf.init(rx_buffer.as_mut_ptr(), rx_buffer.len()) };
        }

        // Enable clocks and initialize hardware
        let conf = LpuartConfig {
            power: config.power,
            source: config.source,
            div: config.div,
            instance: T::CLOCK_INSTANCE,
        };
        let parts = unsafe { enable_and_reset::<T>(&conf).map_err(Error::ClockSetup)? };

        Self::init::<T>(enable_tx, enable_rx, enable_cts, enable_rts, config)?;

        // Enable interrupts for buffered operation
        cortex_m::interrupt::free(|_| {
            T::info().regs().ctrl().modify(|w| {
                w.set_rie(true); // RX interrupt
                w.set_orie(true); // Overrun interrupt
                w.set_peie(true); // Parity error interrupt
                w.set_feie(true); // Framing error interrupt
                w.set_neie(true); // Noise error interrupt
            });
        });

        // Enable interrupt
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Ok(parts.wake_guard)
    }

    /// Helper for full-duplex initialization
    fn new_inner_buffered<T: Instance>(
        tx_pin: Peri<'a, AnyPin>,
        rx_pin: Peri<'a, AnyPin>,
        rts_pin: Option<Peri<'a, AnyPin>>,
        cts_pin: Option<Peri<'a, AnyPin>>,
        tx_buffer: &'a mut [u8],
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self, Error> {
        let wg = Self::init_buffered::<T>(
            Some(tx_buffer),
            Some(rx_buffer),
            config,
            true,
            true,
            rts_pin.is_some(),
            cts_pin.is_some(),
        )?;

        Ok(Self {
            tx: LpuartTx::new_inner::<T>(tx_pin, cts_pin, Buffered, wg.clone()),
            rx: LpuartRx::new_inner::<T>(rx_pin, rts_pin, Buffered, wg),
        })
    }

    /// Create a new full duplex buffered LPUART.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    ///
    /// ## SAFETY
    ///
    /// You must NOT call `core::mem::forget` on `Lpuart` if `rx_buffer` or `tx_buffer` are not
    /// `'static`. This will cause memory corruption.
    pub fn new_buffered<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self, Error> {
        tx_pin.as_tx();
        rx_pin.as_rx();

        Self::new_inner_buffered::<T>(tx_pin.into(), rx_pin.into(), None, None, tx_buffer, rx_buffer, config)
    }

    /// Create a new buffered LPUART instance with RTS/CTS flow control.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    ///
    /// ## SAFETY
    ///
    /// You must NOT call `core::mem::forget` on `Lpuart` if `rx_buffer` or `tx_buffer` are not
    /// `'static`. This will cause memory corruption.
    pub fn new_buffered_with_rtscts<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        rts_pin: Peri<'a, impl RtsPin<T>>,
        cts_pin: Peri<'a, impl CtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self, Error> {
        tx_pin.as_tx();
        rx_pin.as_rx();
        rts_pin.as_rts();
        cts_pin.as_cts();

        Self::new_inner_buffered::<T>(
            tx_pin.into(),
            rx_pin.into(),
            Some(rts_pin.into()),
            Some(cts_pin.into()),
            tx_buffer,
            rx_buffer,
            config,
        )
    }

    /// Create a new buffered LPUART with only RTS flow control (RX flow control).
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    ///
    /// ## SAFETY
    ///
    /// You must NOT call `core::mem::forget` on `Lpuart` if `rx_buffer` or `tx_buffer` are not
    /// `'static`. This will cause memory corruption.
    pub fn new_buffered_with_rts<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        rts_pin: Peri<'a, impl RtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self, Error> {
        tx_pin.as_tx();
        rx_pin.as_rx();
        rts_pin.as_rts();

        Self::new_inner_buffered::<T>(
            tx_pin.into(),
            rx_pin.into(),
            Some(rts_pin.into()),
            None,
            tx_buffer,
            rx_buffer,
            config,
        )
    }

    /// Create a new buffered LPUART with only CTS flow control (TX flow control).
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    ///
    /// ## SAFETY
    ///
    /// You must NOT call `core::mem::forget` on `Lpuart` if `rx_buffer` or `tx_buffer` are not
    /// `'static`. This will cause memory corruption.
    pub fn new_buffered_with_cts<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        cts_pin: Peri<'a, impl CtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self, Error> {
        tx_pin.as_tx();
        rx_pin.as_rx();
        cts_pin.as_cts();

        Self::new_inner_buffered::<T>(
            tx_pin.into(),
            rx_pin.into(),
            None,
            Some(cts_pin.into()),
            tx_buffer,
            rx_buffer,
            config,
        )
    }
}

impl<'a> LpuartTx<'a, Buffered> {
    /// Helper for TX-only initialization
    fn new_inner_buffered<T: Instance>(
        tx_pin: Peri<'a, AnyPin>,
        cts_pin: Option<Peri<'a, AnyPin>>,
        tx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self, Error> {
        let wg = Lpuart::<'a, Buffered>::init_buffered::<T>(
            Some(tx_buffer),
            None,
            config,
            true,
            false,
            false,
            cts_pin.is_some(),
        )?;

        Ok(Self::new_inner::<T>(tx_pin, cts_pin, Buffered, wg))
    }

    /// Create a new TX-only LPUART.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    ///
    /// ## SAFETY
    ///
    /// You must NOT call `core::mem::forget` on `LpuartTx` if `tx_buffer` is not `'static`.
    /// This will potentially send "garbage" data via the UART.
    pub fn new<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self, Error> {
        tx_pin.as_tx();

        let res = Self::new_inner_buffered::<T>(tx_pin.into(), None, tx_buffer, config)?;

        Ok(res)
    }

    /// Create a new TX-only buffered LPUART with CTS flow control.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    ///
    /// ## SAFETY
    ///
    /// You must NOT call `core::mem::forget` on `LpuartTx` if `tx_buffer` is not `'static`.
    /// This will potentially send "garbage" data via the UART.
    pub fn new_with_cts<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        cts_pin: Peri<'a, impl CtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self, Error> {
        tx_pin.as_tx();
        cts_pin.as_cts();

        let res = Self::new_inner_buffered::<T>(tx_pin.into(), Some(cts_pin.into()), tx_buffer, config)?;

        // Enable interrupt
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Ok(res)
    }

    /// Write data asynchronously
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        // Wait for space in the buffer
        self.state.tx_waker.wait_for(|| !self.state.tx_buf.is_full()).await?;

        // We now know there is space, so do a normal try_write
        Ok(self.try_write(buf))
    }

    /// Flush the TX buffer and wait for transmission to complete
    pub async fn flush(&mut self) -> Result<(), Error> {
        // Wait for TX buffer to empty and transmission to complete
        Ok(self
            .state
            .tx_waker
            .wait_for(|| {
                let tx_empty = self.state.tx_buf.is_empty();
                let fifo_empty = self.info.regs().water().read().txcount() == 0;
                let tc_complete = self.info.regs().stat().read().tc() == Tc::COMPLETE;
                tx_empty && fifo_empty && tc_complete
            })
            .await?)
    }

    /// Try to write without blocking
    ///
    /// May return 0 if the provided buf is zero, or there are no bytes available
    pub fn try_write(&mut self, buf: &[u8]) -> usize {
        if buf.is_empty() {
            return 0;
        }

        // SAFETY: Ring buffer is initialized on `new`, there can only be one `LpuartTx`
        // live at once, exclusive access is guaranteed by `&mut self`.
        let mut writer = unsafe { self.state.tx_buf.writer() };
        let starting_len = buf.len();
        let mut window = buf;

        // This will usually run once, and at most twice, if we are in a wrap-around
        // condition.
        loop {
            // Get destination
            let dst = writer.push_slice();
            // Copy what is possible
            let to_copy = dst.len().min(window.len());
            let (now, later) = window.split_at(to_copy);
            dst[..to_copy].copy_from_slice(now);

            // Update the "to send" window to only contain the remaining unsent bytes
            window = later;
            // Update the ring buffer with the pushed bytes
            writer.push_done(to_copy);

            // If the copy is complete, or the buffer is full, we are done copying
            if to_copy == 0 || window.is_empty() {
                break;
            }
        }

        let written = starting_len - window.len();

        if written > 0 {
            // Enable TX interrupt to start transmission
            cortex_m::interrupt::free(|_| {
                self.info.regs().ctrl().modify(|w| w.set_tie(true));
            });
        }

        written
    }
}

impl<'a> LpuartRx<'a, Buffered> {
    /// Helper for RX-only initialization
    fn new_inner_buffered<T: Instance>(
        rx_pin: Peri<'a, AnyPin>,
        rts_pin: Option<Peri<'a, AnyPin>>,
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self, Error> {
        let wg = Lpuart::<'a, Buffered>::init_buffered::<T>(
            None,
            Some(rx_buffer),
            config,
            false,
            true,
            rts_pin.is_some(),
            false,
        )?;

        Ok(Self::new_inner::<T>(rx_pin, rts_pin, Buffered, wg))
    }

    /// Create a new RX-only buffered LPUART.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    ///
    /// ## SAFETY
    ///
    /// You must NOT call `core::mem::forget` on `LpuartRx` if `rx_buffer` is not `'static`.
    /// This will cause memory corruption.
    pub fn new<T: Instance>(
        _inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self, Error> {
        rx_pin.as_rx();

        let res = Self::new_inner_buffered::<T>(rx_pin.into(), None, rx_buffer, config)?;

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
    ///
    /// ## SAFETY
    ///
    /// You must NOT call `core::mem::forget` on `LpuartRx` if `rx_buffer` is not `'static`.
    /// This will cause memory corruption.
    pub fn new_with_rts<T: Instance>(
        _inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        rts_pin: Peri<'a, impl RtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self, Error> {
        rx_pin.as_rx();
        rts_pin.as_rts();

        let res = Self::new_inner_buffered::<T>(rx_pin.into(), Some(rts_pin.into()), rx_buffer, config)?;

        // Enable interrupt
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Ok(res)
    }

    /// Read data asynchronously
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if buf.is_empty() {
            return Ok(0);
        }

        // Wait for some data to be available
        self.state.rx_waker.wait_for(|| !self.state.rx_buf.is_empty()).await?;

        // Now do a normal try_read, we know it will return a non-zero amount
        Ok(self.try_read(buf))
    }

    /// Try to read without blocking
    ///
    /// May return zero bytes if none are available, or the provided buffer is
    /// of zero length.
    pub fn try_read(&mut self, buf: &mut [u8]) -> usize {
        if buf.is_empty() {
            return 0;
        }

        // SAFETY: Ring buffer is initialized on `new`, there can only be one `LpuartRx`
        // live at once, exclusive access is guaranteed by `&mut self`.
        let mut reader = unsafe { self.state.rx_buf.reader() };
        let starting_len = buf.len();
        let mut window = buf;

        // This will usually run once, and at most twice, if we are in a wrap-around
        // condition.
        loop {
            // Get source amount
            let src = reader.pop_slice();
            // Determine the amount to copy, do so
            let to_copy = src.len().min(window.len());
            let (now, later) = window.split_at_mut(to_copy);
            now.copy_from_slice(&src[..to_copy]);
            // Tell the ring buffer the space is now free
            reader.pop_done(to_copy);
            // The "to recv" window is the amount we didn't just fill
            window = later;

            // If we copied nothing or there are no bytes left to be copied,
            // then we are done
            if to_copy == 0 || window.is_empty() {
                break;
            }
        }

        starting_len - window.len()
    }
}

/// Buffered UART interrupt handler.
pub struct BufferedInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> crate::interrupt::typelevel::Handler<T::Interrupt> for BufferedInterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();

        let regs = T::info().regs();
        let state = T::state();

        let ctrl = regs.ctrl().read();
        let stat = regs.stat().read();
        let param = regs.param().read();
        let has_rx_fifo = param.rxfifo() > 0;
        let has_tx_fifo = param.txfifo() > 0;

        // Handle overrun error
        if stat.or() {
            regs.stat().write(|w| w.set_or(true));
            T::PERF_INT_WAKE_INCR();
            state.rx_waker.wake();
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
        if ctrl.rie() && (has_rx_data_pending(T::info()) || stat.idle()) {
            let mut pushed_any = false;
            let mut writer = unsafe { state.rx_buf.writer() };

            if has_rx_fifo {
                // Read from FIFO as long as there is data available and
                // somewhere to put it
                while regs.water().read().rxcount() > 0 && !state.rx_buf.is_full() {
                    let byte = regs.data().read().0 as u8;
                    writer.push_one(byte);
                    pushed_any = true;
                }
            } else {
                // Read single byte if possible
                if regs.stat().read().rdrf() && !state.rx_buf.is_full() {
                    let byte = (regs.data().read().0 & 0xFF) as u8;
                    writer.push_one(byte);
                    pushed_any = true;
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
            let mut reader = unsafe { state.tx_buf.reader() };
            let to_pop = if has_tx_fifo {
                // tx fifo size is 2^param.txfifo, we want to pop enough to fill
                // the fifo, minus whatever is in there now.
                (1 << param.txfifo()) - regs.water().read().txcount()
            } else {
                if regs.stat().read().tdre() != Tdre::TXDATA {
                    1
                } else {
                    0
                }
            };

            // Send data while TX buffer is ready and we have data
            for _ in 0..to_pop {
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
            T::PERF_INT_WAKE_INCR();
            state.tx_waker.wake();

            // Disable TC interrupt
            cortex_m::interrupt::free(|_| {
                regs.ctrl().modify(|w| w.set_tcie(false));
            });
        }
    }
}

impl embedded_io_async::Write for LpuartTx<'_, Buffered> {
    async fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        self.write(buf).await
    }

    async fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        self.flush().await
    }
}

impl embedded_io_async::Read for LpuartRx<'_, Buffered> {
    async fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        self.read(buf).await
    }
}

impl embedded_io_async::Write for Lpuart<'_, Buffered> {
    async fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        self.tx.write(buf).await
    }

    async fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        self.tx.flush().await
    }
}

impl embedded_io_async::Read for Lpuart<'_, Buffered> {
    async fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        self.rx.read(buf).await
    }
}
