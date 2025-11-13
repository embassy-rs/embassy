use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_hal_internal::atomic_ring_buffer::RingBuffer;
use embassy_hal_internal::Peri;
use embassy_sync::waitqueue::AtomicWaker;

use super::*;
use crate::interrupt;

// ============================================================================
// STATIC STATE MANAGEMENT
// ============================================================================

/// State for buffered LPUART operations
pub struct State {
    rx_waker: AtomicWaker,
    rx_buf: RingBuffer,
    tx_waker: AtomicWaker,
    tx_buf: RingBuffer,
    tx_done: AtomicBool,
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
            rx_waker: AtomicWaker::new(),
            rx_buf: RingBuffer::new(),
            tx_waker: AtomicWaker::new(),
            tx_buf: RingBuffer::new(),
            tx_done: AtomicBool::new(true),
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
    info: Info,
    state: &'static State,
    _tx_pin: Peri<'a, AnyPin>,
}

/// Buffered LPUART RX driver
pub struct BufferedLpuartRx<'a> {
    info: Info,
    state: &'static State,
    _rx_pin: Peri<'a, AnyPin>,
}

// ============================================================================
// BUFFERED LPUART IMPLEMENTATION
// ============================================================================

impl<'a> BufferedLpuart<'a> {
    /// Create a new buffered LPUART instance
    pub fn new<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
        // Configure pins
        tx_pin.as_tx();
        rx_pin.as_rx();

        // Convert pins to AnyPin
        let tx_pin: Peri<'a, AnyPin> = tx_pin.into();
        let rx_pin: Peri<'a, AnyPin> = rx_pin.into();

        let state = T::buffered_state();

        // Initialize the peripheral
        Self::init::<T>(Some(&tx_pin), Some(&rx_pin), None, None, tx_buffer, rx_buffer, config)?;

        Ok(Self {
            tx: BufferedLpuartTx {
                info: T::info(),
                state,
                _tx_pin: tx_pin,
            },
            rx: BufferedLpuartRx {
                info: T::info(),
                state,
                _rx_pin: rx_pin,
            },
        })
    }

    /// Create a new buffered LPUART with flexible pin configuration
    pub fn new_with_pins<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Option<Peri<'a, impl TxPin<T>>>,
        rx_pin: Option<Peri<'a, impl RxPin<T>>>,
        rts_pin: Option<Peri<'a, impl RtsPin<T>>>,
        cts_pin: Option<Peri<'a, impl CtsPin<T>>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
        // Configure pins if provided
        let tx_pin = tx_pin.map(|pin| {
            pin.as_tx();
            let converted: Peri<'a, AnyPin> = pin.into();
            converted
        });

        let rx_pin = rx_pin.map(|pin| {
            pin.as_rx();
            let converted: Peri<'a, AnyPin> = pin.into();
            converted
        });

        let rts_pin = rts_pin.map(|pin| {
            pin.as_rts();
            let converted: Peri<'a, AnyPin> = pin.into();
            converted
        });

        let cts_pin = cts_pin.map(|pin| {
            pin.as_cts();
            let converted: Peri<'a, AnyPin> = pin.into();
            converted
        });

        let state = T::buffered_state();

        // Initialize the peripheral
        Self::init::<T>(
            tx_pin.as_ref(),
            rx_pin.as_ref(),
            rts_pin.as_ref(),
            cts_pin.as_ref(),
            tx_buffer,
            rx_buffer,
            config,
        )?;

        // Create TX and RX instances
        let (tx, rx) = if let (Some(tx_pin), Some(rx_pin)) = (tx_pin, rx_pin) {
            (
                BufferedLpuartTx {
                    info: T::info(),
                    state,
                    _tx_pin: tx_pin,
                },
                BufferedLpuartRx {
                    info: T::info(),
                    state,
                    _rx_pin: rx_pin,
                },
            )
        } else {
            return Err(Error::InvalidArgument);
        };

        Ok(Self { tx, rx })
    }

    fn init<T: Instance>(
        _tx: Option<&Peri<'a, AnyPin>>,
        _rx: Option<&Peri<'a, AnyPin>>,
        _rts: Option<&Peri<'a, AnyPin>>,
        _cts: Option<&Peri<'a, AnyPin>>,
        tx_buffer: &'a mut [u8],
        rx_buffer: &'a mut [u8],
        mut config: Config,
    ) -> Result<()> {
        let regs = T::info().regs;
        let state = T::buffered_state();

        // Check if already initialized
        if state.initialized.load(Ordering::Relaxed) {
            return Err(Error::InvalidArgument);
        }

        // Initialize ring buffers
        assert!(!tx_buffer.is_empty());
        unsafe { state.tx_buf.init(tx_buffer.as_mut_ptr(), tx_buffer.len()) }

        assert!(!rx_buffer.is_empty());
        unsafe { state.rx_buf.init(rx_buffer.as_mut_ptr(), rx_buffer.len()) }

        // Mark as initialized
        state.initialized.store(true, Ordering::Relaxed);

        // Enable TX and RX for buffered operation
        config.enable_tx = true;
        config.enable_rx = true;

        // Perform standard initialization
        perform_software_reset(regs);
        disable_transceiver(regs);
        configure_baudrate(regs, config.baudrate_bps, config.clock)?;
        configure_frame_format(regs, &config);
        configure_control_settings(regs, &config);
        configure_fifo(regs, &config);
        clear_all_status_flags(regs);
        configure_flow_control(regs, &config);
        configure_bit_order(regs, config.msb_firs);

        // Enable interrupts for buffered operation
        cortex_m::interrupt::free(|_| {
            regs.ctrl().modify(|_, w| {
                w.rie()
                    .enabled() // RX interrupt
                    .orie()
                    .enabled() // Overrun interrupt
                    .peie()
                    .enabled() // Parity error interrupt
                    .feie()
                    .enabled() // Framing error interrupt
                    .neie()
                    .enabled() // Noise error interrupt
            });
        });

        // Enable the transceiver
        enable_transceiver(regs, config.enable_tx, config.enable_rx);

        // Enable the interrupt
        // unsafe {
        //     // TODO: Used the propper interrupt enable method for the specific LPUART instance
        //     // T::Interrupt::enable();
        // }

        Ok(())
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
    /// Create a new TX-only buffered LPUART
    pub fn new<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        tx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
        tx_pin.as_tx();
        let tx_pin: Peri<'a, AnyPin> = tx_pin.into();

        let info = T::info();
        let state = T::buffered_state();

        // Check if already initialized
        if state.initialized.load(Ordering::Relaxed) {
            return Err(Error::InvalidArgument);
        }

        // Initialize TX ring buffer only
        unsafe {
            let tx_buf = &state.tx_buf as *const _ as *mut RingBuffer;
            (*tx_buf).init(tx_buffer.as_mut_ptr(), tx_buffer.len());
        }

        state.initialized.store(true, Ordering::Relaxed);

        // Initialize with TX only
        BufferedLpuart::init::<T>(
            Some(&tx_pin),
            None,
            None,
            None,
            tx_buffer,
            &mut [], // Empty RX buffer
            config,
        )?;

        Ok(Self {
            info,
            state,
            _tx_pin: tx_pin,
        })
    }

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
                        self.info.regs.ctrl().modify(|_, w| w.tie().enabled());
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
            let fifo_empty = self.info.regs.water().read().txcount().bits() == 0;
            let tc_complete = self.info.regs.stat().read().tc().is_complete();

            if tx_empty && fifo_empty && tc_complete {
                Poll::Ready(Ok(()))
            } else {
                // Enable appropriate interrupt
                cortex_m::interrupt::free(|_| {
                    if !tx_empty {
                        self.info.regs.ctrl().modify(|_, w| w.tie().enabled());
                    } else {
                        self.info.regs.ctrl().modify(|_, w| w.tcie().enabled());
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
                self.info.regs.ctrl().modify(|_, w| w.tie().enabled());
            });
        }

        Ok(written)
    }
}

// ============================================================================
// BUFFERED RX IMPLEMENTATION
// ============================================================================

impl<'a> BufferedLpuartRx<'a> {
    /// Create a new RX-only buffered LPUART
    pub fn new<T: Instance>(
        _inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'a,
        rx_buffer: &'a mut [u8],
        config: Config,
    ) -> Result<Self> {
        rx_pin.as_rx();
        let rx_pin: Peri<'a, AnyPin> = rx_pin.into();

        let info = T::info();
        let state = T::buffered_state();

        // Check if already initialized
        if state.initialized.load(Ordering::Relaxed) {
            return Err(Error::InvalidArgument);
        }

        // Initialize RX ring buffer only
        unsafe {
            let rx_buf = &state.rx_buf as *const _ as *mut RingBuffer;
            (*rx_buf).init(rx_buffer.as_mut_ptr(), rx_buffer.len());
        }

        state.initialized.store(true, Ordering::Relaxed);

        // Initialize with RX only
        BufferedLpuart::init::<T>(
            None,
            Some(&rx_pin),
            None,
            None,
            &mut [], // Empty TX buffer
            rx_buffer,
            config,
        )?;

        Ok(Self {
            info,
            state,
            _rx_pin: rx_pin,
        })
    }

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
                self.info.regs.ctrl().modify(|_, w| w.rie().disabled());
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
                self.info.regs.ctrl().modify(|_, w| w.rie().enabled());
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
            self.info.regs.ctrl().modify(|_, w| w.rie().disabled());
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
            self.info.regs.ctrl().modify(|_, w| w.rie().enabled());
        });

        Ok(read)
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
        let regs = T::info().regs;
        let state = T::buffered_state();

        // Check if this instance is initialized
        if !state.initialized.load(Ordering::Relaxed) {
            return;
        }

        let ctrl = regs.ctrl().read();
        let stat = regs.stat().read();
        let has_fifo = regs.param().read().rxfifo().bits() > 0;

        // Handle overrun error
        if stat.or().is_overrun() {
            regs.stat().write(|w| w.or().clear_bit_by_one());
            state.rx_waker.wake();
            return;
        }

        // Clear other error flags
        if stat.pf().is_parity() {
            regs.stat().write(|w| w.pf().clear_bit_by_one());
        }
        if stat.fe().is_error() {
            regs.stat().write(|w| w.fe().clear_bit_by_one());
        }
        if stat.nf().is_noise() {
            regs.stat().write(|w| w.nf().clear_bit_by_one());
        }

        // Handle RX data
        if ctrl.rie().is_enabled() && (has_data(regs) || stat.idle().is_idle()) {
            let mut pushed_any = false;
            let mut writer = state.rx_buf.writer();

            if has_fifo {
                // Read from FIFO
                while regs.water().read().rxcount().bits() > 0 {
                    let byte = (regs.data().read().bits() & 0xFF) as u8;
                    if writer.push_one(byte) {
                        pushed_any = true;
                    } else {
                        // Buffer full, stop reading
                        break;
                    }
                }
            } else {
                // Read single byte
                if regs.stat().read().rdrf().is_rxdata() {
                    let byte = (regs.data().read().bits() & 0xFF) as u8;
                    if writer.push_one(byte) {
                        pushed_any = true;
                    }
                }
            }

            if pushed_any {
                state.rx_waker.wake();
            }

            // Clear idle flag if set
            if stat.idle().is_idle() {
                regs.stat().write(|w| w.idle().clear_bit_by_one());
            }
        }

        // Handle TX data
        if ctrl.tie().is_enabled() {
            let mut sent_any = false;
            let mut reader = state.tx_buf.reader();

            // Send data while TX buffer is ready and we have data
            while regs.stat().read().tdre().is_no_txdata() {
                if let Some(byte) = reader.pop_one() {
                    regs.data().write(|w| w.bits(u32::from(byte)));
                    sent_any = true;
                } else {
                    // No more data to send
                    break;
                }
            }

            if sent_any {
                state.tx_waker.wake();
            }

            // If buffer is empty, switch to TC interrupt or disable
            if state.tx_buf.is_empty() {
                cortex_m::interrupt::free(|_| {
                    regs.ctrl().modify(|_, w| w.tie().disabled().tcie().enabled());
                });
            }
        }

        // Handle transmission complete
        if ctrl.tcie().is_enabled() && regs.stat().read().tc().is_complete() {
            state.tx_done.store(true, Ordering::Release);
            state.tx_waker.wake();

            // Disable TC interrupt
            cortex_m::interrupt::free(|_| {
                regs.ctrl().modify(|_, w| w.tcie().disabled());
            });
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
