//! Buffered Lpuart driver powered by `bbqueue`

#![deny(clippy::undocumented_unsafe_blocks)]

use core::marker::PhantomData;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU8, AtomicU32, Ordering, fence};

use bbqueue::BBQueue;
use bbqueue::prod_cons::stream::{StreamGrantR, StreamGrantW};
use bbqueue::traits::coordination::cas::AtomicCoord;
use bbqueue::traits::notifier::maitake::MaiNotSpsc;
use bbqueue::traits::storage::Storage;
use embassy_hal_internal::Peri;
use grounded::uninit::GroundedCell;
use maitake_sync::WaitCell;
use nxp_pac::lpuart::vals::Tc;
use paste::paste;

use super::{DataBits, IdleConfig, Info, MsbFirst, Parity, RxPin, StopBits, TxPin, TxPins};
use crate::clocks::periph_helpers::{Div4, LpuartClockSel};
use crate::clocks::{PoweredClock, WakeGuard};
use crate::dma::{DMA_MAX_TRANSFER_SIZE, DmaChannel, DmaRequest, EnableInterrupt};
use crate::gpio::AnyPin;
use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::lpuart::{Instance, RxPins};

/// Error Type
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum BbqError {
    /// Errors from LPUart setup
    Basic(super::Error),
    /// Could not initialize a new instance as the current instance is already in use
    Busy,
    /// Attempted to create an Rx half with Tx parts, or a Tx half with Rx parts
    WrongParts,
    /// Requested an [`RxMode::MaxFrame`] too large for the provided buffer
    MaxFrameTooLarge,
}

/// RX Reception mode
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub enum BbqRxMode {
    /// Default mode, attempts to utilize the ring buffer as maximally as possible.
    ///
    /// In this mode, the interrupt will use whatever space is available, up to 1/4
    /// the total ring buffer size, or the max DMA transfer size, whichever is smaller.
    /// however this may mean that if we are at the "end" of the ring buffer,
    /// some transfers may be smaller, meaning we need to "reload" the interrupt
    /// more often.
    ///
    /// At slower UART rates (like 115_200), this is probably acceptable, as we have
    /// roughly 347us to service the "end of transfer" interrupt and reload the next
    /// DMA transfer. However at higher speeds (like 4_000_000), this time shrinks to
    /// 10us, meaning that critical sections (including defmt logging) may cause us to
    /// lose data.
    ///
    /// If you know your maximum frame/burst size, you can instead use [`RxMode::MaxFrame`],
    /// which will never allow "short" grants, with the trade off that we may reduce the
    /// total usable capacity temporarily if we need to wrap around the ring buffer early.
    #[default]
    Efficiency,

    /// Max Frame mode, ensures that dma transfers always have exactly `size` bytes available
    ///
    /// In this mode, we will always make DMA transfers of the given size. This is intended for
    /// cases where we are receving bursts of data <= `size`, ideally with a short gap between
    /// bursts. This means that we will receive an IDLE interrupt, and switch over receiving grants
    /// in the quiet period, avoiding potentially latency-sensitive DMA transfer updates while
    /// data is still being transferred. This is especially useful at higher baudrates.
    ///
    /// The tradeoff here is that we can temporarily "waste" up to `(size - 1)` bytes if we
    /// are forced to wrap-around the ring buffer early. For example if there is only 1023 bytes
    /// in the ring buffer before it wraps around, and `size = 1024`, we will be forced to wrap
    /// around the ring early, skipping that capacity. In some cases, where the required 1024
    /// bytes are not available at the beginning of the ring buffer either, we will not begin
    /// a transfer at all, potentially losing data if capacity is not freed up before the next
    /// transfer starts (each time the ring buffer is drained, we will automatically re-start
    /// receiving if enough capacity is made available).
    ///
    /// `size` must be <= (capacity / 4).
    MaxFrame { size: usize },
}

/// Lpuart config
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct BbqConfig {
    /// Power state required for this peripheral
    pub power: PoweredClock,
    /// Clock source
    pub source: LpuartClockSel,
    /// Clock divisor
    pub div: Div4,
    /// Baud rate in bits per second
    pub baudrate_bps: u32,
    /// Parity configuration
    pub parity_mode: Option<Parity>,
    /// Number of data bits
    pub data_bits_count: DataBits,
    /// MSB First or LSB First configuration
    pub msb_first: MsbFirst,
    /// Number of stop bits
    pub stop_bits_count: StopBits,
    /// RX IDLE configuration
    pub rx_idle_config: IdleConfig,
}

impl Default for BbqConfig {
    fn default() -> Self {
        Self {
            baudrate_bps: 115_200u32,
            parity_mode: None,
            data_bits_count: DataBits::DATA8,
            msb_first: MsbFirst::LSB_FIRST,
            stop_bits_count: StopBits::ONE,
            rx_idle_config: IdleConfig::IDLE_1,
            power: PoweredClock::AlwaysEnabled,
            source: LpuartClockSel::FroLfDiv,
            div: Div4::no_div(),
        }
    }
}

impl From<BbqConfig> for super::Config {
    fn from(value: BbqConfig) -> Self {
        let mut cfg = super::Config::default();
        let BbqConfig {
            power,
            source,
            div,
            baudrate_bps,
            parity_mode,
            data_bits_count,
            msb_first,
            stop_bits_count,
            rx_idle_config,
        } = value;

        // User selectable
        cfg.power = power;
        cfg.source = source;
        cfg.div = div;
        cfg.baudrate_bps = baudrate_bps;
        cfg.parity_mode = parity_mode;
        cfg.data_bits_count = data_bits_count;
        cfg.msb_first = msb_first;
        cfg.stop_bits_count = stop_bits_count;
        cfg.rx_idle_config = rx_idle_config;

        // Manually set
        cfg.tx_fifo_watermark = 0;
        cfg.rx_fifo_watermark = 0;
        cfg.swap_txd_rxd = false;

        cfg
    }
}

/// A `bbqueue` powered buffered Lpuart
pub struct LpuartBbq {
    // TODO: Don't just make these pub, we don't *really* handle dropping/recreation
    // of separate parts at the moment.
    /// The TX half of the LPUART
    tx: LpuartBbqTx,
    /// The RX half of the LPUART
    rx: LpuartBbqRx,
}

#[derive(Copy, Clone)]
struct BbqVtable {
    lpuart_init: fn(bool, bool, bool, bool, super::Config) -> Result<Option<WakeGuard>, super::Error>,
    int_pend: fn(),
    int_unpend: fn(),
    int_disable: fn(),
    dma_rx_cb: fn(),
    int_enable: unsafe fn(),
}

impl BbqVtable {
    fn for_lpuart<T: BbqInstance>() -> Self {
        Self {
            int_pend: T::Interrupt::pend,
            int_unpend: T::Interrupt::unpend,
            int_disable: T::Interrupt::disable,
            int_enable: T::Interrupt::enable,
            dma_rx_cb: T::dma_rx_complete_cb,
            lpuart_init: super::Lpuart::<'static, super::Blocking>::init::<T>,
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
enum WhichHalf {
    Rx,
    Tx,
}

pub struct BbqHalfParts {
    // resources
    buffer: &'static mut [u8],
    dma_ch: DmaChannel<'static>,
    pin: Peri<'static, AnyPin>,

    // type erasure
    which: WhichHalf,
    dma_req: u8,
    mux: crate::pac::port::vals::Mux,
    info: &'static Info,
    state: &'static BbqState,
    vtable: BbqVtable,
}

pub struct BbqParts {
    // resources
    tx_buffer: &'static mut [u8],
    tx_dma_ch: DmaChannel<'static>,
    tx_pin: Peri<'static, AnyPin>,
    rx_buffer: &'static mut [u8],
    rx_dma_ch: DmaChannel<'static>,
    rx_pin: Peri<'static, AnyPin>,

    // type erasure
    tx_dma_req: u8,
    tx_mux: crate::pac::port::vals::Mux,
    rx_dma_req: u8,
    rx_mux: crate::pac::port::vals::Mux,
    info: &'static Info,
    state: &'static BbqState,
    vtable: BbqVtable,
}

impl BbqParts {
    pub fn new<T: BbqInstance, Tx: TxPin<T>, Rx: RxPin<T>>(
        _inner: Peri<'static, T>,
        _irq: impl Binding<T::Interrupt, BbqInterruptHandler<T>> + 'static,
        tx_pin: Peri<'static, Tx>,
        tx_buffer: &'static mut [u8],
        tx_dma_ch: impl Into<DmaChannel<'static>>,
        rx_pin: Peri<'static, Rx>,
        rx_buffer: &'static mut [u8],
        rx_dma_ch: impl Into<DmaChannel<'static>>,
    ) -> Result<Self, BbqError> {
        Ok(Self {
            tx_buffer,
            tx_dma_ch: tx_dma_ch.into(),
            tx_pin: tx_pin.into(),
            rx_buffer,
            rx_dma_ch: rx_dma_ch.into(),
            rx_pin: rx_pin.into(),
            tx_dma_req: T::TX_DMA_REQUEST.number(),
            tx_mux: Tx::MUX,
            rx_dma_req: T::RX_DMA_REQUEST.number(),
            rx_mux: Rx::MUX,
            info: T::info(),
            state: T::bbq_state(),
            vtable: BbqVtable::for_lpuart::<T>(),
        })
    }
}

impl BbqHalfParts {
    pub fn pin(&mut self) -> Peri<'_, AnyPin> {
        self.pin.reborrow()
    }

    pub fn new_tx_half<T: BbqInstance, P: TxPin<T>>(
        _inner: Peri<'static, T>,
        _irq: impl Binding<T::Interrupt, BbqInterruptHandler<T>> + 'static,
        tx_pin: Peri<'static, P>,
        buffer: &'static mut [u8],
        dma_ch: impl Into<DmaChannel<'static>>,
    ) -> Self {
        Self {
            buffer,
            dma_ch: dma_ch.into(),
            pin: tx_pin.into(),
            mux: P::MUX,
            info: T::info(),
            state: T::bbq_state(),
            dma_req: T::TX_DMA_REQUEST.number(),
            vtable: BbqVtable::for_lpuart::<T>(),
            which: WhichHalf::Tx,
        }
    }

    pub fn new_rx_half<T: BbqInstance, P: RxPin<T>>(
        _inner: Peri<'static, T>,
        _irq: impl Binding<T::Interrupt, BbqInterruptHandler<T>> + 'static,
        tx_pin: Peri<'static, P>,
        buffer: &'static mut [u8],
        dma_ch: impl Into<DmaChannel<'static>>,
    ) -> Self {
        Self {
            buffer,
            dma_ch: dma_ch.into(),
            pin: tx_pin.into(),
            mux: P::MUX,
            info: T::info(),
            state: T::bbq_state(),
            dma_req: T::RX_DMA_REQUEST.number(),
            vtable: BbqVtable::for_lpuart::<T>(),
            which: WhichHalf::Rx,
        }
    }
}

impl LpuartBbq {
    /// Create a new LpuartBbq with both transmit and receive halves
    pub fn new(parts: BbqParts, config: BbqConfig, mode: BbqRxMode) -> Result<Self, BbqError> {
        // Get state for this instance, and try to move from the "uninit" to "initing" state
        parts.state.uninit_to_initing()?;

        // Set as TX/RX pin mode
        any_as_tx(&parts.tx_pin, parts.tx_mux);
        any_as_rx(&parts.rx_pin, parts.rx_mux);

        // Configure UART peripheral
        // TODO make this a specific Bbq mode instead of using blocking
        // TODO support CTS/RTS pins?

        let _wg = (parts.vtable.lpuart_init)(true, true, false, false, config.into()).map_err(BbqError::Basic)?;

        // Setup the TX state
        //
        // SAFETY: We have ensured we are in the INITING state, and interrupts are not yet active.
        unsafe {
            LpuartBbqTx::initialize_tx_state(parts.state, parts.tx_dma_ch, parts.tx_buffer, parts.tx_dma_req);
        }

        // Setup the RX state
        let len = parts.rx_buffer.len();
        // SAFETY: We have ensured we are in the INITING state, and the interrupt is not yet active.
        unsafe {
            LpuartBbqRx::initialize_rx_state(
                parts.state,
                parts.info,
                parts.rx_dma_ch,
                parts.vtable.dma_rx_cb,
                parts.rx_buffer,
                parts.rx_dma_req,
            );
        }

        // Update our state to "initialized", and that we have the TXDMA + RXDMA channels present
        // Okay to just store: we have exclusive access
        let max_size = (len / 4).min(DMA_MAX_TRANSFER_SIZE);
        let rx_mode_bits = match mode {
            BbqRxMode::Efficiency => (max_size as u32) << 16,
            BbqRxMode::MaxFrame { size } => {
                if size > max_size {
                    return Err(BbqError::MaxFrameTooLarge);
                }
                let size = (size as u32) << 16;
                size | STATE_RXDMA_MODE_MAXFRAME
            }
        };
        let new_state = STATE_INITED | STATE_TXDMA_PRESENT | STATE_RXDMA_PRESENT | rx_mode_bits;
        parts.state.state.store(new_state, Ordering::Release);

        // SAFETY: We have ensured that our ISR is present via the IRQ token, and we have
        // initialized the shared state machine sufficiently that it can execute correctly
        // when triggered.
        unsafe {
            // Clear any stale interrupt flags
            (parts.vtable.int_unpend)();
            // Enable the LPUART interrupt
            (parts.vtable.int_enable)();
            // Immediately pend the interrupt, this will "load" the DMA transfer as the
            // ISR will notice that there is no active grant. This means that we start
            // receiving immediately without additional user interaction.
            (parts.vtable.int_pend)();
        }

        Ok(Self {
            tx: LpuartBbqTx {
                state: parts.state,
                info: parts.info,
                vtable: parts.vtable,
                mux: parts.tx_mux,
                _tx_pins: TxPins {
                    tx_pin: parts.tx_pin,
                    cts_pin: None,
                },
                _wg: _wg.clone(),
            },
            rx: LpuartBbqRx {
                state: parts.state,
                info: parts.info,
                vtable: parts.vtable,
                mux: parts.rx_mux,
                _rx_pins: RxPins {
                    rx_pin: parts.rx_pin,
                    rts_pin: None,
                },
                _wg,
            },
        })
    }

    /// Write some data to the buffer. See [`LpuartBbqTx::write`] for more information
    pub fn write(&mut self, buf: &[u8]) -> impl Future<Output = Result<usize, BbqError>> {
        self.tx.write(buf)
    }

    /// Read some data from the buffer. See [`LpuartBbqRx::read`] for more information
    pub fn read(&mut self, buf: &mut [u8]) -> impl Future<Output = Result<usize, BbqError>> {
        self.rx.read(buf)
    }

    /// Wait for all bytes in the outgoing buffer to be flushed asynchronously.
    ///
    /// See [`LpuartBbqTx::flush`] for more information
    pub fn flush(&mut self) -> impl Future<Output = ()> {
        self.tx.flush()
    }

    /// Busy wait until all transmitting has completed
    ///
    /// See [`LpuartBbqTx::blocking_flush`] for more information
    pub fn blocking_flush(&mut self) {
        self.tx.blocking_flush();
    }

    /// Teardown the LpuartBbq, retrieving the original parts
    pub fn teardown(self) -> BbqParts {
        let Self { tx, rx } = self;
        let tx_parts = tx.teardown();
        let rx_parts = rx.teardown();
        BbqParts {
            tx_buffer: tx_parts.buffer,
            tx_dma_ch: tx_parts.dma_ch,
            tx_pin: tx_parts.pin,
            rx_buffer: rx_parts.buffer,
            rx_dma_ch: rx_parts.dma_ch,
            rx_pin: rx_parts.pin,
            tx_dma_req: tx_parts.dma_req,
            tx_mux: tx_parts.mux,
            rx_dma_req: rx_parts.dma_req,
            rx_mux: rx_parts.mux,
            info: tx_parts.info,
            state: tx_parts.state,
            vtable: tx_parts.vtable,
        }
    }
}

/// A `bbqueue` powered Lpuart TX Half
pub struct LpuartBbqTx {
    state: &'static BbqState,
    info: &'static Info,
    vtable: BbqVtable,
    mux: crate::pac::port::vals::Mux,
    _tx_pins: TxPins<'static>,
    _wg: Option<WakeGuard>,
}

impl LpuartBbqTx {
    /// ## SAFETY
    ///
    /// This function must only be called in the "INITING" state, and BEFORE
    /// enabling interrupts, meaning we have exclusive access to the TX components
    /// of the given BbqState.
    unsafe fn initialize_tx_state(
        state: &'static BbqState,
        dma: DmaChannel<'static>,
        tx_buffer: &'static mut [u8],
        request_num: u8,
    ) {
        // Enable the DMA interrupt to handle "transfer complete" interrupts
        dma.enable_interrupt();

        // Setup the TX bbqueue instance, store the DMA channel and bbqueue in the
        // BbqState storage location.
        //
        // TODO: We could probably be more clever and setup the DMA transfer request
        // number ONCE in init, then just do a minimal-reload. This would allow us to
        // avoid storing the txdma_num, and save some effort in the ISR.
        let cont = Container::from(tx_buffer);

        // SAFETY: We have exclusive access to the shared TX components, and the interrupt
        // is not yet enabled. We move ownership of these resources to the shared area.
        unsafe {
            state.tx_queue.get().write(BBQueue::new_with_storage(cont));
            state.txdma.get().write(dma);
            state.txdma_num.store(request_num, Ordering::Release);
        }
    }

    /// Create a new LpuartBbq with only the transmit half
    ///
    /// NOTE: Dropping the `LpuartBbqTx` will *permanently* leak the TX buffer, DMA channel, and tx pin.
    /// Call [LpuartBbqTx::teardown] to reclaim these resources.
    pub fn new(parts: BbqHalfParts, config: BbqConfig) -> Result<Self, BbqError> {
        // Are these the right parts?
        if parts.which != WhichHalf::Tx {
            return Err(BbqError::WrongParts);
        }

        // Get state for this instance, and try to move from the "uninit" to "initing" state
        parts.state.uninit_to_initing()?;

        // Set as TX pin mode
        any_as_tx(&parts.pin, parts.mux);

        // Configure UART peripheral
        // TODO make this a specific Bbq mode instead of using blocking
        // TODO support CTS pin?
        let _wg = (parts.vtable.lpuart_init)(true, false, false, false, config.into()).map_err(BbqError::Basic)?;

        // Setup the TX Half state
        //
        // SAFETY: We have ensured we are in the INITING state, and the interrupt is not yet active.
        unsafe {
            Self::initialize_tx_state(parts.state, parts.dma_ch, parts.buffer, parts.dma_req);
        }

        // Update our state to "initialized", and that we have the TXDMA channel present
        // Okay to just store: we have exclusive access
        let new_state = STATE_INITED | STATE_TXDMA_PRESENT;
        parts.state.state.store(new_state, Ordering::Release);

        // SAFETY: We have properly initialized the shared storage, and ensured that
        // our ISR is installed with the Irq token.
        unsafe {
            // Clear any stale interrupt flags
            (parts.vtable.int_unpend)();
            // Enable the LPUART interrupt
            (parts.vtable.int_enable)();
            // NOTE: Unlike RX, we don't begin transmitting immediately, we move
            // from Idle -> Transmitting the first time the user calls write.
        }

        Ok(Self {
            state: parts.state,
            info: parts.info,
            vtable: parts.vtable,
            _tx_pins: TxPins {
                tx_pin: parts.pin,
                cts_pin: None,
            },
            _wg,
            mux: parts.mux,
        })
    }

    /// Write some data to the outgoing transmit buffer
    ///
    /// This method waits until some data is able to be written to the internal buffer,
    /// and returns the number of bytes from `buf` consumed.
    ///
    /// This does NOT guarantee all bytes of `buf` have been buffered, only the amount returned.
    ///
    /// This does NOT guarantee the bytes have been written to the wire. See [`Self::flush()`].
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, BbqError> {
        // TODO: we could have a version of this that gives the user the grant directly
        // to reduce the effort of copying.

        // SAFETY: The existence of a LpuartBbqTx ensures that the `tx_queue` has been
        // initialized. The tx_queue is safe to access in a shared manner after initialization.
        let tx_queue = unsafe { &*self.state.tx_queue.get() };

        let prod = tx_queue.stream_producer();
        let mut wgr = prod.wait_grant_max_remaining(buf.len()).await;
        let to_copy = buf.len().min(wgr.len());
        wgr[..to_copy].copy_from_slice(&buf[..to_copy]);
        wgr.commit(to_copy);
        (self.vtable.int_pend)();

        Ok(to_copy)
    }

    /// Wait for all bytes in the outgoing buffer to be flushed asynchronously.
    ///
    /// When this method completes, the outgoing buffer is empty.
    pub async fn flush(&mut self) {
        // Discard the result on wait_for as we never close the waiter.
        let _ = self
            .state
            .tx_flushed
            .wait_for(|| {
                // We are idle when there is no TXGR active
                (self.state.state.load(Ordering::Acquire) & STATE_TXGR_ACTIVE) == 0
            })
            .await;
    }

    /// Busy wait until all transmitting has completed
    ///
    /// When this method completes, the outgoing buffer is empty.
    pub fn blocking_flush(&mut self) {
        while (self.state.state.load(Ordering::Acquire) & STATE_TXGR_ACTIVE) != 0 {}
    }

    /// Teardown the Tx handle, reclaiming the parts.
    pub fn teardown(self) -> BbqHalfParts {
        // First, disable relevant interrupts
        let state = critical_section::with(|_cs| {
            self.info.regs.ctrl().modify(|w| w.set_tcie(false));
            // Clear the TXDMA present bit to prevent the ISR from touching anything.
            // Relaxed is okay here because CS::with has Acq/Rel semantics on entry and exit
            self.state.state.fetch_and(!STATE_TXDMA_PRESENT, Ordering::Relaxed)
        });

        // If there is an active grant, the TX DMA may be active. Stop it and release the grant
        if (state & STATE_TXGR_ACTIVE) != 0 {
            // SAFETY: We have unset TXDMA_PRESENT and disabled TCIE: we now have exclusive
            // access to the shared tx resources.
            unsafe {
                // Take DMA channel by mut ref
                let txdma = &mut *self.state.txdma.get();

                // Stop the DMA
                self.info.regs().baud().modify(|w| w.set_tdmae(false));
                txdma.disable_request();
                txdma.clear_done();
                fence(Ordering::Acquire);

                // Then take the grant by ownership, and drop it, which releases the grant
                _ = self.state.txgr.get().read();
            }
            self.state.state.fetch_and(!STATE_TXGR_ACTIVE, Ordering::AcqRel);
        }

        // Get a reference to the tx_queue to retrieve the Container
        //
        // SAFETY: We have unset TXDMA_PRESENT and disabled TCIE: we now have exclusive
        // access to the shared tx resources.
        let (ptr, len) = unsafe {
            let tx_queue = &*self.state.tx_queue.get();
            tx_queue.storage().ptr_len()
        };

        // Now, drop the queue in place. This is sound because as the LpuartBbqTx, we have exclusive
        // access to the "producer" half, and by disabling the interrupt and notching out the state
        // bits, we know the ISR will no longer touch the consumer part.
        //
        // Also, take the DmaChannel by ownership this time.
        //
        // SAFETY: We have unset TXDMA_PRESENT and disabled TCIE: we now have exclusive
        // access to the shared tx resources.
        let tx_dma = unsafe {
            core::ptr::drop_in_place(self.state.tx_queue.get());
            // Defensive coding: purge the tx_queue just in case. This doesn't zero the
            // whole buffer, only the tracking pointers.
            core::ptr::write_bytes(self.state.tx_queue.get(), 0, 1);
            let mut dma = self.state.txdma.get().read();
            dma.clear_callback();
            dma
        };

        // Re-magic the mut slice from the storage we have now reclaimed by dropping the
        // tx_queue.
        //
        // SAFETY: We have unset TXDMA_PRESENT and disabled TCIE: we now have exclusive
        // access to the shared tx resources.
        let tx_buffer = unsafe { core::slice::from_raw_parts_mut(ptr.as_ptr(), len) };

        // Now, if this was the last part of the lpuart, we are responsible for peripheral
        // cleanup.
        if (state & !(STATE_TXGR_ACTIVE | STATE_TXDMA_PRESENT)) == STATE_INITED {
            (self.vtable.int_disable)();
            super::disable_peripheral(self.info);
            self.state.state.store(STATE_UNINIT, Ordering::Relaxed);
        }

        let (tx_pin, _) = self._tx_pins.take();

        BbqHalfParts {
            buffer: tx_buffer,
            dma_ch: tx_dma,
            pin: tx_pin,
            dma_req: self.state.txdma_num.load(Ordering::Relaxed),
            mux: self.mux,
            info: self.info,
            state: self.state,
            vtable: self.vtable,
            which: WhichHalf::Tx,
        }
    }
}

pub struct LpuartBbqRx {
    state: &'static BbqState,
    info: &'static Info,
    vtable: BbqVtable,
    mux: crate::pac::port::vals::Mux,
    _rx_pins: RxPins<'static>,
    _wg: Option<WakeGuard>,
}

impl LpuartBbqRx {
    /// ## SAFETY
    ///
    /// This function must only be called in the "INITING" state, and BEFORE
    /// enabling interrupts, meaning we have exclusive access to the RX components
    /// of the given BbqState.
    unsafe fn initialize_rx_state(
        state: &'static BbqState,
        _info: &'static Info,
        mut dma: DmaChannel<'static>,
        rx_callback: fn(),
        rx_buffer: &'static mut [u8],
        request_num: u8,
    ) {
        // Set the callback to our completion handler, so our LPUART interrupt gets called to
        // complete the transfer and reload
        //
        // TODO: Right now we only do this on RX, we might want to also handle this on TX as well
        // so we have more time to reload, but for now we'll naturally get the "transfer complete"
        // interrupt when the TX fifo empties, and we are less latency sensitive on TX than RX.
        //
        // SAFETY: We have exclusive ownership of the DmaChannel, and are able to overwrite the
        // existing callback, if any.
        unsafe {
            dma.set_callback(rx_callback);
        }

        // Enable the DMA interrupt to handle "transfer complete" interrupts
        dma.enable_interrupt();

        // Setup the RX bbqueue instance, store the DMA channel and bbqueue in the
        // BbqState storage location.
        //
        // TODO: We could probably be more clever and setup the DMA transfer request
        // number ONCE in init, then just do a minimal-reload. This would allow us to
        // avoid storing the rxdma_num, and save some effort in the ISR.
        let cont = Container::from(rx_buffer);

        // SAFETY: We have exclusive access to the shared RX components, and the interrupt
        // is not yet enabled. We move ownership of these resources to the shared area.
        unsafe {
            state.rx_queue.get().write(BBQueue::new_with_storage(cont));
            state.rxdma.get().write(dma);
            state.rxdma_num.store(request_num, Ordering::Release);
        }

        // TODO: Do we actually want these interrupts enabled? We probably do, so we can
        // clear the errors, but I'm not sure if any of these actually stall the receive.
        //
        // That being said, I've observed the RX line being floating (e.g. if the sender
        // is in reset or disconnected) causing ~infinite "framing errors", which causes
        // an interrupt storm since we don't *disable* the interrupt. We probably need to
        // think about how/if we handle these kinds of errors.
        //
        // info.regs().ctrl().modify(|w| {
        //     // overrun
        //     w.set_orie(true);
        //     // noise
        //     w.set_neie(true);
        //     // framing
        //     w.set_feie(true);
        // });
    }

    /// Create a new LpuartBbq with only the receive half
    ///
    /// NOTE: Dropping the `LpuartBbqRx` will *permanently* leak the TX buffer, DMA channel, and tx pin.
    /// Call [LpuartBbqTx::teardown] to reclaim these resources.
    pub fn new(parts: BbqHalfParts, config: BbqConfig, mode: BbqRxMode) -> Result<Self, BbqError> {
        // Are these the right parts?
        if parts.which != WhichHalf::Rx {
            return Err(BbqError::WrongParts);
        }

        // Get state for this instance, and try to move from the "uninit" to "initing" state
        parts.state.uninit_to_initing()?;

        // Set RX pin mode
        any_as_rx(&parts.pin, parts.mux);

        // Configure UART peripheral
        // TODO make this a specific Bbq mode instead of using blocking
        // TODO support RTS pin?
        let _wg = (parts.vtable.lpuart_init)(false, true, false, false, config.into()).map_err(BbqError::Basic)?;

        // Setup the RX half state
        let len = parts.buffer.len();

        // SAFETY: We have ensured that we are in the INITING state, and the interrupt is not yet active.
        unsafe {
            Self::initialize_rx_state(
                parts.state,
                parts.info,
                parts.dma_ch,
                parts.vtable.dma_rx_cb,
                parts.buffer,
                parts.dma_req,
            );
        }

        // Update our state to "initialized", and that we have the RXDMA channel present
        // Okay to just store: we have exclusive access
        let max_size = (len / 4).min(DMA_MAX_TRANSFER_SIZE);
        let rx_mode_bits = match mode {
            BbqRxMode::Efficiency => (max_size as u32) << 16,
            BbqRxMode::MaxFrame { size } => {
                if size > max_size {
                    return Err(BbqError::MaxFrameTooLarge);
                }
                let size = (size as u32) << 16;
                size | STATE_RXDMA_MODE_MAXFRAME
            }
        };
        let new_state = STATE_INITED | STATE_RXDMA_PRESENT | rx_mode_bits;
        parts.state.state.store(new_state, Ordering::Release);

        // SAFETY: We have ensured that our ISR is present via the IRQ token, and we have
        // initialized the shared state machine sufficiently that it can execute correctly
        // when triggered.
        unsafe {
            // Clear any stale interrupt flags
            (parts.vtable.int_unpend)();
            // Enable the LPUART interrupt
            (parts.vtable.int_enable)();
            // Immediately pend the interrupt, this will "load" the DMA transfer as the
            // ISR will notice that there is no active grant. This means that we start
            // receiving immediately without additional user interaction.
            (parts.vtable.int_pend)();
        }

        Ok(Self {
            state: parts.state,
            info: parts.info,
            vtable: parts.vtable,
            mux: parts.mux,
            _rx_pins: RxPins {
                rx_pin: parts.pin,
                rts_pin: None,
            },
            _wg,
        })
    }

    /// Read some data from the incoming receive buffer
    ///
    /// This method waits until some data is able to be read from the internal buffer,
    /// and returns the number of bytes from `buf` written.
    ///
    /// This does NOT guarantee all bytes of `buf` have been written, only the amount returned.
    ///
    /// When receiving, this method must be called somewhat regularly to ensure that the incoming
    /// buffer does not become over full.
    ///
    /// In this case, data will be discarded until this read method is called and capacity is made
    /// available.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, BbqError> {
        // TODO: we could have a version of this that gives the user the grant directly
        // to reduce the effort of copying.

        // SAFETY: The existence of a LpuartBbqRx ensures that the `rx_queue` has been
        // initialized. The rx_queue is safe to access in a shared manner after initialization.
        let queue = unsafe { &*self.state.rx_queue.get() };
        let cons = queue.stream_consumer();
        let rgr = cons.wait_read().await;
        let to_copy = buf.len().min(rgr.len());
        buf[..to_copy].copy_from_slice(&rgr[..to_copy]);
        rgr.release(to_copy);

        // If NO rx_dma is active, that means we stalled, so pend the interrupt to
        // restart it now that we've freed space.
        if (self.state.state.load(Ordering::Acquire) & STATE_RXGR_ACTIVE) == 0 {
            (self.vtable.int_pend)();
        }

        Ok(to_copy)
    }

    /// Teardown the Rx handle, reclaiming the DMA channel, receive buffer, and Rx pin.
    pub fn teardown(self) -> BbqHalfParts {
        // First, mark the RXDMA as not present to halt the ISR from processing the state
        // machine
        let rx_state_bits = STATE_RXDMA_PRESENT
            | STATE_RXGR_ACTIVE
            | STATE_RXDMA_COMPLETE
            | STATE_RXDMA_MODE_MAXFRAME
            | STATE_RXGR_LEN_MASK;
        let state = self.state.state.fetch_and(!rx_state_bits, Ordering::AcqRel);

        // Then, disable receive-relevant interrupts
        critical_section::with(|_cs| {
            self.info.regs.ctrl().modify(|w| {
                w.set_ilie(false);
                w.set_neie(false);
                w.set_feie(false);
                w.set_orie(false);
            });
        });

        // If there is an active grant, the RX DMA may be active. Stop it and release the grant
        if (state & STATE_RXGR_ACTIVE) != 0 {
            // SAFETY: We have unset RXDMA_PRESENT and disabled all RX interrupts: we now have exclusive
            // access to the shared rx resources.
            unsafe {
                // Take DMA channel by mut ref
                let rxdma = &mut *self.state.rxdma.get();

                // Stop the DMA
                self.info.regs().baud().modify(|w| w.set_rdmae(false));
                rxdma.disable_request();
                rxdma.clear_done();
                fence(Ordering::Acquire);

                // Then take the grant by ownership, and drop it, which releases the grant
                _ = self.state.rxgr.get().read();
            }
        }

        // Get a reference to the rx_queue to retrieve the Container
        //
        // SAFETY: We have unset RXDMA_PRESENT and disabled all RX interrupts: we now have exclusive
        // access to the shared rx resources.
        let (ptr, len) = unsafe {
            let rx_queue = &*self.state.rx_queue.get();
            rx_queue.storage().ptr_len()
        };

        // Now, drop the queue in place. This is sound because as the LpuartBbqRx, we have exclusive
        // access to the "consumer" half, and by disabling the interrupt and notching out the state
        // bits, we know the ISR will no longer touch the producer part.
        //
        // Also, take the DmaChannel by ownership this time.
        //
        // SAFETY: We have unset RXDMA_PRESENT and disabled all RX interrupts: we now have exclusive
        // access to the shared rx resources.
        let rx_dma = unsafe {
            core::ptr::drop_in_place(self.state.rx_queue.get());
            // Defensive coding: purge the rx_queue just in case. This doesn't zero the
            // whole buffer, only the tracking pointers.
            core::ptr::write_bytes(self.state.rx_queue.get(), 0, 1);
            let mut dma = self.state.rxdma.get().read();
            dma.clear_callback();
            dma
        };

        // Re-magic the mut slice from the storage we have now reclaimed by dropping the
        // rx_queue.
        //
        // SAFETY: We have unset RXDMA_PRESENT and disabled all RX interrupts: we now have exclusive
        // access to the shared rx resources.
        let rx_buffer = unsafe { core::slice::from_raw_parts_mut(ptr.as_ptr(), len) };

        // Now, if this was the last part of the lpuart, we are responsible for peripheral
        // cleanup.
        if (state & !rx_state_bits) == STATE_INITED {
            super::disable_peripheral(self.info);
            self.state.state.store(STATE_UNINIT, Ordering::Relaxed);
        }

        let (rx_pin, _) = self._rx_pins.take();
        BbqHalfParts {
            buffer: rx_buffer,
            dma_ch: rx_dma,
            pin: rx_pin,
            dma_req: self.state.rxdma_num.load(Ordering::Relaxed),
            mux: self.mux,
            info: self.info,
            state: self.state,
            vtable: self.vtable,
            which: WhichHalf::Rx,
        }
    }
}

// A wrapper type representing a `&'static mut [u8]` buffer
struct Container {
    ptr: NonNull<u8>,
    len: usize,
}

impl Storage for Container {
    /// SAFETY: The length and ptr destination of the Container are never changed.
    unsafe fn ptr_len(&self) -> (NonNull<u8>, usize) {
        (self.ptr, self.len)
    }
}

impl From<&'static mut [u8]> for Container {
    fn from(value: &'static mut [u8]) -> Self {
        Self {
            len: value.len(),
            // SAFETY: The input slice is guaranteed to contain a non-null value
            ptr: unsafe { NonNull::new_unchecked(value.as_mut_ptr()) },
        }
    }
}

/// interrupt handler.
pub struct BbqInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

const STATE_UNINIT: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0000;
const STATE_INITING: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0001;
const STATE_INITED: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0011;
const STATE_RXGR_ACTIVE: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0100;
const STATE_TXGR_ACTIVE: u32 = 0b0000_0000_0000_0000_0000_0000_0000_1000;
const STATE_RXDMA_PRESENT: u32 = 0b0000_0000_0000_0000_0000_0000_0001_0000;
const STATE_TXDMA_PRESENT: u32 = 0b0000_0000_0000_0000_0000_0000_0010_0000;
const STATE_RXDMA_COMPLETE: u32 = 0b0000_0000_0000_0000_0000_0000_0100_0000;
const STATE_RXDMA_MODE_MAXFRAME: u32 = 0b0000_0000_0000_0000_0000_0000_1000_0000;
const STATE_RXGR_LEN_MASK: u32 = 0b1111_1111_1111_1111_0000_0000_0000_0000;

struct BbqState {
    /// 0bGGGG_GGGG_GGGG_GGGG_xxxx_xxxx_MDTR_PCAI
    ///                                        ^^--> 0b00: uninit, 0b01: initing, 0b11 init'd.
    ///                                       ^----> 0b0: No Rx grant, 0b1: Rx grant active
    ///                                      ^-----> 0b0: No Tx grant, 0b1: Tx grant active
    ///                                    ^-------> 0b0: No Rx DMA present, 0b1: Rx DMA present
    ///                                   ^--------> 0b0: No Tx DMA present, 0b1: Tx DMA present
    ///                                  ^---------> 0b0: Rx DMA not complete, 0b1: Rx DMA complete
    ///                                 ^----------> 0b0: RxMode "Efficiency", 0b1: RxMode "Max Frame"
    ///   ^^^^_^^^^_^^^^_^^^^----------------------> 16-bit: RX Grant size
    state: AtomicU32,

    /// The "outgoing" bbqueue buffer
    ///
    /// Only valid when state is STATE_INITED + STATE_TXDMA_PRESENT.
    tx_queue: GroundedCell<BBQueue<Container, AtomicCoord, MaiNotSpsc>>,
    /// The "outgoing" transmit grant, which DMA will read from.
    ///
    /// Only valid when state is STATE_INITED + STATE_TXDMA_PRESENT + STATE_TXGR_ACTIVE.
    txgr: GroundedCell<StreamGrantR<&'static BBQueue<Container, AtomicCoord, MaiNotSpsc>>>,
    /// The "outgoing" DMA channel.
    ///
    /// Only valid when state is STATE_INITED + STATE_TXDMA_PRESENT.
    txdma: GroundedCell<DmaChannel<'static>>,
    /// The "outgoing" DMA request number.
    ///
    /// Only valid when state is STATE_INITED + STATE_TXDMA_PRESENT.
    txdma_num: AtomicU8,

    /// The "incoming" bbqueue buffer
    ///
    /// Only valid when state is STATE_INITED + STATE_RXDMA_PRESENT.
    rx_queue: GroundedCell<BBQueue<Container, AtomicCoord, MaiNotSpsc>>,
    /// The "incoming" receive grant, which DMA will write to.
    ///
    /// Only valid when state is STATE_INITED + STATE_RXDMA_PRESENT + STATE_RXGR_ACTIVE.
    rxgr: GroundedCell<StreamGrantW<&'static BBQueue<Container, AtomicCoord, MaiNotSpsc>>>,
    /// The "incoming" DMA channel.
    ///
    /// Only valid when state is STATE_INITED + STATE_RXDMA_PRESENT.
    rxdma: GroundedCell<DmaChannel<'static>>,
    /// The "incoming" DMA request number.
    ///
    /// Only valid when state is STATE_INITED + STATE_RXDMA_PRESENT.
    rxdma_num: AtomicU8,

    /// Waiter for the outgoing buffer to be flushed
    tx_flushed: WaitCell,
}

impl BbqState {
    const fn new() -> Self {
        Self {
            state: AtomicU32::new(0),
            tx_queue: GroundedCell::uninit(),
            rx_queue: GroundedCell::uninit(),
            rxgr: GroundedCell::uninit(),
            txgr: GroundedCell::uninit(),
            txdma: GroundedCell::uninit(),
            txdma_num: AtomicU8::new(0),
            rxdma: GroundedCell::uninit(),
            rxdma_num: AtomicU8::new(0),
            tx_flushed: WaitCell::new(),
        }
    }

    /// Attempt to move from the "uninit" state to the "initing" state. Returns an
    /// error if we are not in the "uninit" state.
    fn uninit_to_initing(&'static self) -> Result<(), BbqError> {
        self.state
            .compare_exchange(STATE_UNINIT, STATE_INITING, Ordering::AcqRel, Ordering::Acquire)
            .map(drop)
            .map_err(|_| BbqError::Busy)
    }

    /// Complete an active TX DMA transfer. Called from ISR context.
    ///
    /// After calling, the transmit half of the driver will be in the idle state.
    ///
    /// ## SAFETY
    ///
    /// * The HAL driver must be initialized
    /// * The TXDMA must be present
    /// * A write grant must be active
    /// * We must be in ISR context
    unsafe fn finalize_write(&'static self, info: &'static Info) {
        // SAFETY: With the function-level safety requirements met, we are free to modify
        // shared tx state.
        unsafe {
            // Load the active TX grant, taking it "by ownership"
            let txgr = self.txgr.get().read();
            // Get the TX DMA, taking it by &mut ref
            let txdma = &mut *self.txdma.get();

            // Stop the DMA
            info.regs().baud().modify(|w| w.set_tdmae(false));
            txdma.disable_request();
            txdma.clear_done();
            // TODO: Some other way of ensuring the DMA is completely stopped?
            fence(Ordering::Acquire);

            // The max transfer length was the lesser of capacity / 4 or the max DMA transfer size
            // in a single transaction. This is because the `read()` used to create this grant may
            // be larger, if more bytes were available.
            let max_len = (&*self.tx_queue.get()).capacity() / 4;
            let xfer = txgr.len().min(max_len).min(DMA_MAX_TRANSFER_SIZE);

            // Release the number of transferred bytes, making them available to the user to reuse,
            // and waking the write waiter if there is one present (e.g. if we were previously full).
            txgr.release(xfer);
        }
        // Mark the TXGR as inactive, signifying "idle"
        self.state.fetch_and(!STATE_TXGR_ACTIVE, Ordering::AcqRel);
    }

    /// Complete an active RX DMA transfer. Called from ISR context.
    ///
    /// After calling, the receive half of the driver will be in the idle state.
    ///
    /// ## SAFETY
    ///
    /// * The HAL driver must be initialized
    /// * The RXDMA must be present
    /// * A read grant must be active
    /// * We must be in ISR context
    unsafe fn finalize_read(&'static self, info: &'static Info) {
        // SAFETY: With the function-level safety requirements met, we are free to modify
        // shared rx state.
        unsafe {
            // Load the active RX grant, taking it by ownership
            let rxgr = self.rxgr.get().read();
            // Get the RX DMA, taking it by &mut ref
            let rxdma = &mut *self.rxdma.get();

            // Stop the active DMA.
            // The DMA may NOT be done yet if this was an idle interrupt
            info.regs().baud().modify(|w| w.set_rdmae(false));
            rxdma.disable_request();
            rxdma.clear_done();

            // Fence to ensure all DMA written bytes are complete, and we can see
            // any writes to the DADDR
            fence(Ordering::AcqRel);

            // Calculate the number of bytes written using the current write address of the
            // DMA channel, minus our starting address.
            let daddr = rxdma.daddr() as usize;
            let sstrt = rxgr.as_ptr() as usize;
            let ttl = daddr.wrapping_sub(sstrt).min(rxgr.len());

            // Commit these bytes, making them visible to the user, and waking any pending
            // waiters if any (e.g. if we were previously empty)
            rxgr.commit(ttl);
        }
        // Mark the RXGR inactive, signifying idle
        self.state.fetch_and(!STATE_RXGR_ACTIVE, Ordering::AcqRel);
    }

    /// Attempt to start an active write transfer. Called from ISR context.
    ///
    /// Returns true if a transfer was started, and returns false if no transfer
    /// was started (e.g. the outgoing buffer is completely drained).
    ///
    /// ## SAFETY
    ///
    /// * The HAL driver must be initialized
    /// * The TXDMA must be present
    /// * A write grant must NOT be active
    /// * We must be in ISR context
    unsafe fn start_write_transfer(&'static self, info: &'static Info) -> bool {
        // Get the tx queue, by & ref
        //
        // SAFETY: TXDMA_PRESENT bit being enabled means the tx_queue has been initialized.
        // The tx_queue is safe to access in a shared manner after initialization.
        let tx_queue = unsafe { &*self.tx_queue.get() };
        let Ok(rgr) = tx_queue.stream_consumer().read() else {
            // Nothing to do!
            return false;
        };

        // SAFETY: With the function-level safety requirements met, we are free to modify
        // shared tx state.
        unsafe {
            // Take the TXDMA by &mut ref
            let txdma = &mut *self.txdma.get();

            // Initialize the transfer from the bbqueue grant to DMA
            //
            // TODO: Most of this setup is redundant/repeated, we could save some effort
            // since most DMA transfer parameters are the same.
            let source = DmaRequest::from_number_unchecked(self.txdma_num.load(Ordering::Relaxed));
            txdma.disable_request();
            txdma.clear_done();
            txdma.clear_interrupt();
            txdma.set_request_source(source);

            let peri_addr = info.regs().data().as_ptr().cast::<u8>();

            // NOTE: we limit the max transfer size to 1/4 the capacity for latency reasons,
            // so we can make buffer space available for further writing by the application
            // as soon as possible, as the buffer space is not made available until after
            // the transfer completes.
            let max_len = (&*self.tx_queue.get()).capacity() / 4;
            let len = rgr.len().min(max_len).min(DMA_MAX_TRANSFER_SIZE);
            txdma.setup_write_to_peripheral(&rgr[..len], peri_addr, EnableInterrupt::Yes);

            // Enable the DMA transfer
            info.regs().baud().modify(|w| w.set_tdmae(true));
            txdma.enable_request();

            // Store (by ownership) the outgoing read grant to the bbqueue state
            self.txgr.get().write(rgr);

            // Mark the TXGR as active, signifying the "transmitting" state
            self.state.fetch_or(STATE_TXGR_ACTIVE, Ordering::AcqRel);
        }

        // wait until the system is not reporting TC complete, to ensure we don't
        // immediately retrigger an interrupt.
        //
        // TODO: I'm not sure this actually ever happens, this is a defensive check
        while info.regs.stat().read().tc() == Tc::COMPLETE {}

        true
    }

    /// Attempt to start an active read transfer. Called from ISR context.
    ///
    /// Returns true if a transfer was started, and returns false if no transfer
    /// was started (e.g. the incoming buffer is completely full).
    ///
    /// ## SAFETY
    ///
    /// * The HAL driver must be initialized
    /// * The RXDMA must be present
    /// * A write grant must NOT be active
    /// * We must be in ISR context
    unsafe fn start_read_transfer(&'static self, info: &'static Info) -> bool {
        // SAFETY: RXDMA_PRESENT bit being enabled means the rx_queue has been initialized.
        // The rx_queue is safe to access in a shared manner after initialization.
        let rx_queue = unsafe { &*self.rx_queue.get() };

        // Determine the size and kind of grant to request
        let state = self.state.load(Ordering::Relaxed);
        let len = (state >> 16) as usize;
        let is_max_frame = (state & STATE_RXDMA_MODE_MAXFRAME) != 0;
        let prod = rx_queue.stream_producer();

        let grant_res = if is_max_frame {
            prod.grant_exact(len)
        } else {
            prod.grant_max_remaining(len)
        };

        let Ok(mut wgr) = grant_res else {
            // If we can't get a grant, that's a problem. Return false to note we didn't
            // start one, and hope the user frees space soon. See the `read` method for
            // how read transfers are restarted in this case.
            return false;
        };

        // SAFETY: With the function-level safety requirements met, we are free to modify
        // shared rx state.
        unsafe {
            // Initialize the transfer from the DMA to the bbqueue grant
            //
            // TODO: Most of this setup is redundant/repeated, we could save some effort
            // since most DMA transfer parameters are the same.
            let rxdma = &mut *self.rxdma.get();
            let source = DmaRequest::from_number_unchecked(self.rxdma_num.load(Ordering::Relaxed));
            rxdma.disable_request();
            rxdma.clear_done();
            rxdma.clear_interrupt();
            rxdma.set_request_source(source);

            let peri_addr = info.regs().data().as_ptr().cast::<u8>();
            rxdma.setup_read_from_peripheral(peri_addr, &mut wgr, EnableInterrupt::Yes);

            // Enable the DMA transfer
            info.regs().baud().modify(|w| w.set_rdmae(true));
            rxdma.enable_request();

            // Store (by ownership) the incoming write grant to the bbqueue state
            self.rxgr.get().write(wgr);

            // Mark the RXGR as active, signifying the "receiving" state
            self.state.fetch_or(STATE_RXGR_ACTIVE, Ordering::AcqRel);
        }

        true
    }
}

#[allow(private_bounds, private_interfaces)]
pub trait BbqInstance: Instance {
    /// The BBQ specific state storage
    fn bbq_state() -> &'static BbqState;
    /// A callback for the DMA handler to call that marks RXDMA as complete and
    /// pends the LPUART interrupt for further processing.
    fn dma_rx_complete_cb();
}

macro_rules! impl_instance {
    ($($n:expr);* $(;)?) => {
        $(
            paste!{
                #[allow(private_interfaces)]
                impl BbqInstance for crate::peripherals::[<LPUART $n>] {
                    fn bbq_state() -> &'static BbqState {
                        static STATE: BbqState = BbqState::new();
                        &STATE
                    }

                    fn dma_rx_complete_cb() {
                        let state = Self::bbq_state();
                        // Mark the DMA as complete
                        state.state.fetch_or(STATE_RXDMA_COMPLETE, Ordering::AcqRel);
                        // Pend the UART interrupt to handle the switchover
                        Self::Interrupt::pend();
                    }
                }
            }
        )*
    };
}

impl_instance!(0; 1; 2; 3; 4; 5);

// Basically the on_interrupt handler, but as a free function so it doesn't get
// monomorphized.
//
// SAFETY: Should only be called by the `on_interrupt` function in ISR context, with
// the shared BbqState properly initialized in the INITED state
unsafe fn handler(info: &'static Info, state: &'static BbqState) {
    let regs = info.regs();
    let ctrl = regs.ctrl().read();
    let stat = regs.stat().read();

    // Just clear any errors - TODO, signal these to the consumer?
    // For now, we just clear + discard errors if they occur.
    let or = stat.or();
    let pf = stat.pf();
    let fe = stat.pf();
    let nf = stat.nf();
    let idle = stat.idle();
    regs.stat().modify(|w| {
        w.set_or(or);
        w.set_pf(pf);
        w.set_fe(fe);
        w.set_nf(nf);
        w.set_idle(idle);
    });

    //
    // RX state machine
    //

    // Check DMA complete or idle interrupt occurred - we need to stop
    // the current RX transfer in either case.
    let pre_clear = state.state.fetch_and(!STATE_RXDMA_COMPLETE, Ordering::AcqRel);

    // SAFETY NOTE: The RXDMA_PRESENT bit is used to mediate whether the interrupt should
    // act the shared RX data. This is used by functions like `teardown` to disable interrupt
    // access to shared data when tearing down.
    let rx_present = (pre_clear & STATE_RXDMA_PRESENT) != 0;
    if rx_present {
        let rx_active = (pre_clear & STATE_RXGR_ACTIVE) != 0;
        let dma_complete = (pre_clear & STATE_RXDMA_COMPLETE) != 0;
        if rx_active && (idle || dma_complete) {
            // State change, move from Receiving -> Idle
            //
            // SAFETY: The HAL driver is initialized, we checked that RXDMA_PRESENT is set, we
            // checked that RXGR_ACTIVE is set, we are in ISR context
            unsafe {
                state.finalize_read(info);
            }
        }

        // If we are now idle, attempt to "reload" the transfer and being receiving again ASAP.
        // Only do this if RXDMA is present. We re-load from state to ensure we see when
        // `finalize_read` just cleared the bit.
        let rx_idle = (state.state.load(Ordering::Acquire) & STATE_RXGR_ACTIVE) == 0;
        if rx_idle {
            // Either Idle -> Receiving or Idle -> Idle
            //
            // SAFETY: The HAL driver is initialized, we checked that RXDMA_PRESENT is set, we
            // checked there isn't a write grant active, and we are in ISR context.
            unsafe {
                let started = state.start_read_transfer(info);
                // Enable ILIE if we started a transfer, otherwise (keep) disabled.
                // ILIE - Idle Line Interrupt Enable
                regs.ctrl().modify(|w| w.set_ilie(started));
            }
        }
    }

    //
    // TX state machine
    //

    // SAFETY NOTE: The TXDMA_PRESENT bit is used to mediate whether the interrupt should
    // act the shared TX data. This is used by functions like `teardown` to disable interrupt
    // access to shared data when tearing down.
    let tx_state = state.state.load(Ordering::Acquire);
    let tx_present = (tx_state & STATE_TXDMA_PRESENT) != 0;
    if tx_present {
        // Handle TX data - TCIE is only enabled if we are transmitting, and we only
        // check that the outgoing transfer is complete. In the future, we might
        // try to do this a bit earlier if the DMA completes but we haven't yet
        // drained the TX fifo yet.
        let txie_set = ctrl.tcie();
        let tc_complete = regs.stat().read().tc() == Tc::COMPLETE;
        let txgr_present = (tx_state & STATE_TXGR_ACTIVE) != 0;

        let tx_did_finish = txie_set && tc_complete && txgr_present;
        if tx_did_finish {
            // State change, move from Transmitting -> Idle
            //
            // SAFETY: The driver has been initialized, we've checked TXDMA_PRESENT is set,
            // we've checked TXGR_ACTIVE is set, we are in ISR context.
            unsafe {
                state.finalize_write(info);
            }
        }

        // If we are now idle, attempt to "reload" the transfer and begin transmitting again.
        // Only do this if TXDMA is present.
        let tx_idle = (state.state.load(Ordering::Acquire) & STATE_TXGR_ACTIVE) == 0;
        if tx_idle {
            // Either Idle -> Transmitting or Idle -> Idle
            //
            // SAFETY: The driver has been initialized, we've checked TXDMA_PRESENT is set,
            // we've checked TXGR_ACTIVE is NOT set, we are in ISR context.
            unsafe {
                let started = state.start_write_transfer(info);
                // Enable tcie if we started a transfer, otherwise (keep) disabled.
                // TCIE - Transfer Complete Interrupt Enable
                regs.ctrl().modify(|w| w.set_tcie(started));

                // Did we go from "transmitting" to "idle" in this ISR? If so, wake any "flush" waiters.
                if tx_did_finish && !started {
                    state.tx_flushed.wake();
                }
            }
        }
    }
}

impl<T: BbqInstance> Handler<T::Interrupt> for BbqInterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();
        let info = T::info();
        let state = T::bbq_state();

        // SAFETY: Interrupts are only enabled when state is valid, we are calling the handler
        // from ISR context.
        unsafe {
            handler(info, state);
        }
    }
}

use crate::gpio::SealedPin;

fn any_as_tx(pin: &Peri<'_, AnyPin>, mux: crate::pac::port::vals::Mux) {
    pin.set_pull(crate::gpio::Pull::Disabled);
    pin.set_slew_rate(crate::gpio::SlewRate::Fast.into());
    pin.set_drive_strength(crate::gpio::DriveStrength::Normal.into());
    pin.set_function(mux);
    pin.set_enable_input_buffer(false);
}

fn any_as_rx(pin: &Peri<'_, AnyPin>, mux: crate::pac::port::vals::Mux) {
    pin.set_pull(crate::gpio::Pull::Disabled);
    pin.set_function(mux);
    pin.set_enable_input_buffer(true);
}
