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
    ) -> Result<Option<WakeGuard>> {
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
    ) -> Result<Self> {
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
    pub fn new_buffered<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
        tx_pin.as_tx();
        rx_pin.as_rx();

        Self::new_inner_buffered::<T>(tx_pin.into(), rx_pin.into(), None, None, tx_buffer, rx_buffer, config)
    }

    /// Create a new buffered LPUART instance with RTS/CTS flow control.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
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
    ) -> Result<Self> {
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
    pub fn new_buffered_with_rts<T: Instance>(
        _inner: Peri<'a, T>,
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
    pub fn new_buffered_with_cts<T: Instance>(
        _inner: Peri<'a, T>,
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
    ) -> Result<Self> {
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
    pub fn new<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
        tx_pin.as_tx();

        let res = Self::new_inner_buffered::<T>(tx_pin.into(), None, tx_buffer, config)?;

        Ok(res)
    }

    /// Create a new TX-only buffered LPUART with CTS flow control.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_with_cts<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        cts_pin: Peri<'a, impl CtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
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
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // Wait for space in the buffer
        Ok(self
            .state
            .tx_waker
            .wait_for_value(|| {
                let mut writer = unsafe { self.state.tx_buf.writer() };
                let written = writer.push(|slice| {
                    let write_len = slice.len().min(buf.len());
                    slice[..write_len].copy_from_slice(&buf[..write_len]);
                    write_len
                });

                // Enable TX interrupt to start transmission
                cortex_m::interrupt::free(|_| {
                    self.info.regs().ctrl().modify(|w| {
                        w.set_tie(true);
                    });
                });

                if written != 0 { Some(written) } else { None }
            })
            .await?)
    }

    /// Flush the TX buffer and wait for transmission to complete
    pub async fn flush(&mut self) -> Result<()> {
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

impl<'a> LpuartRx<'a, Buffered> {
    /// Helper for RX-only initialization
    fn new_inner_buffered<T: Instance>(
        rx_pin: Peri<'a, AnyPin>,
        rts_pin: Option<Peri<'a, AnyPin>>,
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
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
    pub fn new<T: Instance>(
        _inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
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
    pub fn new_with_rts<T: Instance>(
        _inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        rts_pin: Peri<'a, impl RtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
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
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        // Try to read available data
        Ok(self
            .state
            .rx_waker
            .wait_for_value(|| {
                // Disable RX interrupt while reading from buffer
                let read = cortex_m::interrupt::free(|_| {
                    let mut reader = unsafe { self.state.rx_buf.reader() };
                    reader.pop(|data| {
                        let to_copy = core::cmp::min(data.len(), buf.len());
                        buf[..to_copy].copy_from_slice(&data[..to_copy]);
                        to_copy
                    })
                });

                if read > 0 { Some(read) } else { None }
            })
            .await?)
    }

    /// Try to read without blocking
    ///
    /// May return zero bytes if none are available, or the provided buffer is
    /// of zero length.
    pub fn try_read(&mut self, buf: &mut [u8]) -> usize {
        if buf.is_empty() {
            return 0;
        }

        // Disable RX interrupt while reading from buffer
        cortex_m::interrupt::free(|_| {
            let mut reader = unsafe { self.state.rx_buf.reader() };
            reader.pop(|data| {
                let to_copy = core::cmp::min(data.len(), buf.len());
                buf[..to_copy].copy_from_slice(&data[..to_copy]);
                to_copy
            })
        })
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
            let mut writer = unsafe { state.rx_buf.writer() };

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
            let mut reader = unsafe { state.tx_buf.reader() };

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
