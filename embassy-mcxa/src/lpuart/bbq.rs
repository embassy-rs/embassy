//! Buffered Lpuart driver powered by `bbqueue`

#![deny(clippy::undocumented_unsafe_blocks)]

use core::marker::PhantomData;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering, fence};

use bbqueue::BBQueue;
use bbqueue::prod_cons::stream::{StreamGrantR, StreamGrantW};
use bbqueue::traits::coordination::cas::AtomicCoord;
use bbqueue::traits::notifier::maitake::MaiNotSpsc;
use bbqueue::traits::storage::Storage;
use embassy_futures::select::{Either, select};
use embassy_hal_internal::Peri;
use grounded::uninit::GroundedCell;
use maitake_sync::WaitCell;
use nxp_pac::lpuart::Tc;

use super::{CtsPin, DataBits, IdleConfig, Info, MsbFirst, Parity, RtsPin, RxPin, StopBits, TxPin, TxPins};
use crate::clocks::periph_helpers::{Div4, LpuartClockSel};
use crate::clocks::{PoweredClock, WakeGuard};
use crate::dma::{
    DMA_MAX_TRANSFER_SIZE, DmaChannel, DmaRequest, InvalidParameters, PingPongSelector, Priority, TransferOptions,
};
use crate::gpio::{AnyPin, HasGpioInstance, PeriGpioExt};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::lpuart::{Instance, RxPins};
use crate::pac::lpuart::{Txctsc as TxCtsConfig, Txctssrc as TxCtsSource};

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
    /// Requested an invalid continuous RX half size for the provided buffer
    InvalidContinuousRxSize,
    /// Continuous DMA reused a half before it was published
    Overrun,
}

impl core::fmt::Display for BbqError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <Self as core::fmt::Debug>::fmt(self, f)
    }
}

impl core::error::Error for BbqError {}

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

    /// Continuous circular DMA with two fixed-size staging halves.
    ///
    /// DMA remains enabled while hardware wraps between the two halves. Idle,
    /// half-transfer, and major-transfer interrupts publish newly received bytes
    /// into the BBQueue without stopping or reprogramming the DMA channel.
    ///
    /// The staging ring consumes `2 * half_size` bytes from the supplied RX buffer;
    /// the remainder backs the BBQueue. The complete staging ring must fit within
    /// one DMA major loop and within one quarter of the remaining BBQueue capacity.
    ///
    /// Each completed half must be copied into the BBQueue before DMA reaches the
    /// next boundary. If the BBQueue lacks capacity or interrupt handling is delayed
    /// too long, reception stops and [`BbqError::Overrun`] remains sticky until this
    /// RX half is torn down and initialized again.
    Continuous { half_size: usize },
}

struct RxBufferLayout {
    queue: &'static mut [u8],
    continuous_dma: Option<&'static mut [u8]>,
    original_addr: usize,
    original_len: usize,
    mode_bits: u32,
}

fn prepare_rx_buffer(buffer: &'static mut [u8], mode: BbqRxMode) -> Result<RxBufferLayout, BbqError> {
    let original_addr = buffer.as_mut_ptr() as usize;
    let original_len = buffer.len();
    let max_size = (original_len / 4).min(DMA_MAX_TRANSFER_SIZE);

    match mode {
        BbqRxMode::Efficiency => Ok(RxBufferLayout {
            queue: buffer,
            continuous_dma: None,
            original_addr,
            original_len,
            mode_bits: (max_size as u32) << 16,
        }),
        BbqRxMode::MaxFrame { size } => {
            if size > max_size {
                return Err(BbqError::MaxFrameTooLarge);
            }

            Ok(RxBufferLayout {
                queue: buffer,
                continuous_dma: None,
                original_addr,
                original_len,
                mode_bits: ((size as u32) << 16) | STATE_RXDMA_MODE_MAXFRAME,
            })
        }
        BbqRxMode::Continuous { half_size } => {
            let Some(staging_len) = half_size.checked_mul(2) else {
                return Err(BbqError::InvalidContinuousRxSize);
            };

            if half_size == 0 || staging_len > DMA_MAX_TRANSFER_SIZE || staging_len >= original_len {
                return Err(BbqError::InvalidContinuousRxSize);
            }

            let (continuous_dma, queue) = buffer.split_at_mut(staging_len);
            let max_publish = (queue.len() / 4).min(DMA_MAX_TRANSFER_SIZE);
            if staging_len > max_publish {
                return Err(BbqError::InvalidContinuousRxSize);
            }

            Ok(RxBufferLayout {
                queue,
                continuous_dma: Some(continuous_dma),
                original_addr,
                original_len,
                mode_bits: ((staging_len as u32) << 16) | STATE_RXDMA_MODE_CONTINUOUS,
            })
        }
    }
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
    /// TX CTS source
    pub tx_cts_source: TxCtsSource,
    /// TX CTS configuration
    pub tx_cts_config: TxCtsConfig,
}

impl Default for BbqConfig {
    fn default() -> Self {
        Self {
            baudrate_bps: 115_200u32,
            parity_mode: None,
            data_bits_count: DataBits::Data8,
            msb_first: MsbFirst::LsbFirst,
            stop_bits_count: StopBits::One,
            rx_idle_config: IdleConfig::Idle1,
            power: PoweredClock::AlwaysEnabled,
            source: LpuartClockSel::FroLfDiv,
            div: Div4::no_div(),
            tx_cts_source: TxCtsSource::Cts,
            tx_cts_config: TxCtsConfig::Start,
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
            tx_cts_source,
            tx_cts_config,
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
        cfg.tx_cts_source = tx_cts_source;
        cfg.tx_cts_config = tx_cts_config;

        // Manually set
        cfg.tx_fifo_watermark = 0;
        cfg.rx_fifo_watermark = 0;
        cfg.swap_txd_rxd = false;

        cfg
    }
}

/// A `bbqueue` powered buffered Lpuart
pub struct LpuartBbq {
    /// The TX half of the LPUART
    tx: LpuartBbqTx,
    /// The RX half of the LPUART
    rx: LpuartBbqRx,
}

#[derive(Copy, Clone)]
struct BbqVtable {
    #[allow(clippy::type_complexity)]
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
    mux: crate::pac::port::Mux,
    info: &'static Info,
    state: &'static BbqState,
    vtable: BbqVtable,

    // flow control (optional)
    flow_pin: Option<Peri<'static, AnyPin>>,
    flow_mux: Option<crate::pac::port::Mux>,
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
    tx_mux: crate::pac::port::Mux,
    rx_dma_req: u8,
    rx_mux: crate::pac::port::Mux,
    info: &'static Info,
    state: &'static BbqState,
    vtable: BbqVtable,

    // flow control (optional)
    cts_pin: Option<Peri<'static, AnyPin>>,
    cts_mux: Option<crate::pac::port::Mux>,
    rts_pin: Option<Peri<'static, AnyPin>>,
    rts_mux: Option<crate::pac::port::Mux>,
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
            cts_pin: None,
            cts_mux: None,
            rts_pin: None,
            rts_mux: None,
        })
    }

    /// Create a full-duplex `BbqParts` with RTS/CTS hardware flow control.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_rtscts<T: BbqInstance, Tx: TxPin<T>, Rx: RxPin<T>, Cts: CtsPin<T>, Rts: RtsPin<T>>(
        _inner: Peri<'static, T>,
        _irq: impl Binding<T::Interrupt, BbqInterruptHandler<T>> + 'static,
        tx_pin: Peri<'static, Tx>,
        tx_buffer: &'static mut [u8],
        tx_dma_ch: impl Into<DmaChannel<'static>>,
        rx_pin: Peri<'static, Rx>,
        rx_buffer: &'static mut [u8],
        rx_dma_ch: impl Into<DmaChannel<'static>>,
        cts_pin: Peri<'static, Cts>,
        rts_pin: Peri<'static, Rts>,
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
            cts_pin: Some(cts_pin.into()),
            cts_mux: Some(Cts::MUX),
            rts_pin: Some(rts_pin.into()),
            rts_mux: Some(Rts::MUX),
        })
    }

    /// Access a borrow of the contained RX pin
    pub fn rx_pin(&mut self) -> Peri<'_, AnyPin> {
        self.rx_pin.reborrow()
    }

    /// Access a borrow of the contained TX pin
    pub fn tx_pin(&mut self) -> Peri<'_, AnyPin> {
        self.tx_pin.reborrow()
    }

    /// Access a borrow of both the RX and TX pin (in that order)
    pub fn pins(&mut self) -> (Peri<'_, AnyPin>, Peri<'_, AnyPin>) {
        let Self { tx_pin, rx_pin, .. } = self;
        (rx_pin.reborrow(), tx_pin.reborrow())
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
            flow_pin: None,
            flow_mux: None,
        }
    }

    /// Create a TX-only `BbqHalfParts` with CTS hardware flow control.
    pub fn new_tx_half_with_cts<T: BbqInstance, P: TxPin<T>, C: CtsPin<T>>(
        _inner: Peri<'static, T>,
        _irq: impl Binding<T::Interrupt, BbqInterruptHandler<T>> + 'static,
        tx_pin: Peri<'static, P>,
        buffer: &'static mut [u8],
        dma_ch: impl Into<DmaChannel<'static>>,
        cts_pin: Peri<'static, C>,
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
            flow_pin: Some(cts_pin.into()),
            flow_mux: Some(C::MUX),
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
            flow_pin: None,
            flow_mux: None,
        }
    }

    /// Create an RX-only `BbqHalfParts` with RTS hardware flow control.
    pub fn new_rx_half_with_rts<T: BbqInstance, P: RxPin<T>, R: RtsPin<T>>(
        _inner: Peri<'static, T>,
        _irq: impl Binding<T::Interrupt, BbqInterruptHandler<T>> + 'static,
        rx_pin: Peri<'static, P>,
        buffer: &'static mut [u8],
        dma_ch: impl Into<DmaChannel<'static>>,
        rts_pin: Peri<'static, R>,
    ) -> Self {
        Self {
            buffer,
            dma_ch: dma_ch.into(),
            pin: rx_pin.into(),
            mux: P::MUX,
            info: T::info(),
            state: T::bbq_state(),
            dma_req: T::RX_DMA_REQUEST.number(),
            vtable: BbqVtable::for_lpuart::<T>(),
            which: WhichHalf::Rx,
            flow_pin: Some(rts_pin.into()),
            flow_mux: Some(R::MUX),
        }
    }

    /// Setup Rx half while binding GPIO to the gpio pin.
    /// This allows later use of async functions on the pin.
    pub fn new_rx_half_async<T: BbqInstance, P: RxPin<T> + HasGpioInstance>(
        _inner: Peri<'static, T>,
        irq: impl Binding<T::Interrupt, BbqInterruptHandler<T>>
        + Binding<<P::Instance as crate::gpio::Instance>::Interrupt, crate::gpio::InterruptHandler<P::Instance>>
        + 'static,
        tx_pin: Peri<'static, P>,
        buffer: &'static mut [u8],
        dma_ch: impl Into<DmaChannel<'static>>,
    ) -> Self {
        Self {
            buffer,
            dma_ch: dma_ch.into(),
            pin: tx_pin.degrade_async(irq),
            mux: P::MUX,
            info: T::info(),
            state: T::bbq_state(),
            dma_req: T::RX_DMA_REQUEST.number(),
            vtable: BbqVtable::for_lpuart::<T>(),
            which: WhichHalf::Rx,
            flow_pin: None,
            flow_mux: None,
        }
    }

    /// Setup an RX-only half with RTS flow control while binding GPIO to the RX pin.
    /// This allows later use of async functions on the RX pin.
    pub fn new_rx_half_with_rts_async<T: BbqInstance, P: RxPin<T> + HasGpioInstance, R: RtsPin<T>>(
        _inner: Peri<'static, T>,
        irq: impl Binding<T::Interrupt, BbqInterruptHandler<T>>
        + Binding<<P::Instance as crate::gpio::Instance>::Interrupt, crate::gpio::InterruptHandler<P::Instance>>
        + 'static,
        rx_pin: Peri<'static, P>,
        buffer: &'static mut [u8],
        dma_ch: impl Into<DmaChannel<'static>>,
        rts_pin: Peri<'static, R>,
    ) -> Self {
        Self {
            buffer,
            dma_ch: dma_ch.into(),
            pin: rx_pin.degrade_async(irq),
            mux: P::MUX,
            info: T::info(),
            state: T::bbq_state(),
            dma_req: T::RX_DMA_REQUEST.number(),
            vtable: BbqVtable::for_lpuart::<T>(),
            which: WhichHalf::Rx,
            flow_pin: Some(rts_pin.into()),
            flow_mux: Some(R::MUX),
        }
    }
}

impl LpuartBbq {
    /// Create a new LpuartBbq with both transmit and receive halves
    pub fn new(parts: BbqParts, config: BbqConfig, mode: BbqRxMode) -> Result<Self, BbqError> {
        // Validate and split the RX allocation before changing shared peripheral state.
        let rx_layout = prepare_rx_buffer(parts.rx_buffer, mode)?;

        // Get state for this instance, and try to move from the "uninit" to "initing" state
        parts.state.uninit_to_initing()?;

        // Set as TX/RX pin mode
        any_as_tx(&parts.tx_pin, parts.tx_mux);
        any_as_rx(&parts.rx_pin, parts.rx_mux);

        // Configure optional flow-control pins (only when a mux is present; a
        // teardown-reclaimed pin arrives already configured with mux == None).
        if let (Some(cts), Some(mux)) = (&parts.cts_pin, parts.cts_mux) {
            any_as_cts(cts, mux);
        }
        if let (Some(rts), Some(mux)) = (&parts.rts_pin, parts.rts_mux) {
            any_as_rts(rts, mux);
        }
        let enable_cts = parts.cts_pin.is_some();
        let enable_rts = parts.rts_pin.is_some();

        // Configure UART peripheral
        // TODO make this a specific Bbq mode instead of using blocking

        let _wg =
            (parts.vtable.lpuart_init)(true, true, enable_cts, enable_rts, config.into()).map_err(BbqError::Basic)?;

        // Setup the TX state
        //
        // SAFETY: We have ensured we are in the INITING state, and interrupts are not yet active.
        unsafe {
            LpuartBbqTx::initialize_tx_state(parts.state, parts.tx_dma_ch, parts.tx_buffer, parts.tx_dma_req);
        }

        // Setup the RX state
        // SAFETY: We have ensured we are in the INITING state, and the interrupt is not yet active.
        unsafe {
            LpuartBbqRx::initialize_rx_state(
                parts.state,
                parts.rx_dma_ch,
                parts.vtable.dma_rx_cb,
                rx_layout.queue,
                rx_layout.continuous_dma,
                parts.rx_dma_req,
            );
        }

        // Update our state to "initialized", and that we have the TXDMA + RXDMA channels present
        // Okay to just store: we have exclusive access
        let new_state = STATE_INITED | STATE_TXDMA_PRESENT | STATE_RXDMA_PRESENT | rx_layout.mode_bits;
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
                    cts_pin: parts.cts_pin,
                },
                _wg: _wg.clone(),
            },
            rx: LpuartBbqRx {
                state: parts.state,
                info: parts.info,
                vtable: parts.vtable,
                mux: parts.rx_mux,
                buffer_addr: rx_layout.original_addr,
                buffer_len: rx_layout.original_len,
                _rx_pins: RxPins {
                    rx_pin: parts.rx_pin,
                    rts_pin: parts.rts_pin,
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

    /// Borrow split parts.
    pub fn split_ref(&mut self) -> (&mut LpuartBbqRx, &mut LpuartBbqTx) {
        let Self { tx, rx } = self;
        (rx, tx)
    }

    /// Split the LpuartBbq into separate TX and RX halves
    pub fn split(self) -> (LpuartBbqTx, LpuartBbqRx) {
        (self.tx, self.rx)
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
            cts_pin: tx_parts.flow_pin,
            cts_mux: tx_parts.flow_mux,
            rts_pin: rx_parts.flow_pin,
            rts_mux: rx_parts.flow_mux,
        }
    }
}

impl embedded_io_async::ErrorType for LpuartBbq {
    type Error = BbqError;
}

impl embedded_io_async::Write for LpuartBbq {
    fn write(&mut self, buf: &[u8]) -> impl Future<Output = Result<usize, Self::Error>> {
        self.write(buf)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush().await;
        Ok(())
    }
}

impl embedded_io_async::Read for LpuartBbq {
    fn read(&mut self, buf: &mut [u8]) -> impl Future<Output = Result<usize, Self::Error>> {
        self.read(buf)
    }
}

/// A `bbqueue` powered Lpuart TX Half
pub struct LpuartBbqTx {
    state: &'static BbqState,
    info: &'static Info,
    vtable: BbqVtable,
    mux: crate::pac::port::Mux,
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
    /// NOTE: Dropping the `LpuartBbqTx` shuts down TX DMA and (if no other half is live)
    /// disables the peripheral; the TX pin and clock gate are released via their own
    /// destructors. However, the TX backing buffer cannot be returned by `Drop` and is
    /// effectively leaked. Call [`LpuartBbqTx::teardown`] to reclaim the buffer along
    /// with the DMA channel and pin.
    pub fn new(parts: BbqHalfParts, config: BbqConfig) -> Result<Self, BbqError> {
        // Are these the right parts?
        if parts.which != WhichHalf::Tx {
            return Err(BbqError::WrongParts);
        }

        // Get state for this instance, and try to move from the "uninit" to "initing" state
        parts.state.uninit_to_initing()?;

        // Set as TX pin mode
        any_as_tx(&parts.pin, parts.mux);

        // Configure optional CTS pin (skip reconfig for a teardown-reclaimed pin, mux == None).
        if let (Some(cts), Some(mux)) = (&parts.flow_pin, parts.flow_mux) {
            any_as_cts(cts, mux);
        }
        let enable_cts = parts.flow_pin.is_some();

        // Configure UART peripheral
        // TODO make this a specific Bbq mode instead of using blocking
        let _wg = (parts.vtable.lpuart_init)(true, false, enable_cts, false, config.into()).map_err(BbqError::Basic)?;

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
                cts_pin: parts.flow_pin,
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
        while (self.state.state.load(Ordering::Acquire) & STATE_TXGR_ACTIVE) != 0 {
            core::hint::spin_loop()
        }
    }

    /// Stop the TX side: disable TCIE, halt TX DMA, drop the tx_queue in place,
    /// and (if this was the last live half) disable the peripheral.
    ///
    /// Returns the reclaimed buffer pointer/length and DMA channel. The caller is
    /// responsible for ensuring the side effects are not performed a second time
    /// on the same instance (either by consuming `self` and calling `mem::forget`,
    /// or by only invoking this from `Drop`, which by definition runs once).
    fn teardown_inner(&mut self) -> (NonNull<u8>, usize, DmaChannel<'static>) {
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

        // Now, if this was the last part of the lpuart, we are responsible for peripheral
        // cleanup.
        if (state & !(STATE_TXGR_ACTIVE | STATE_TXDMA_PRESENT)) == STATE_INITED {
            (self.vtable.int_disable)();
            super::disable_peripheral(self.info);
            self.state.state.store(STATE_UNINIT, Ordering::Relaxed);
        }

        (ptr, len, tx_dma)
    }

    /// Teardown the Tx handle, reclaiming the parts.
    pub fn teardown(mut self) -> BbqHalfParts {
        let (ptr, len, tx_dma) = self.teardown_inner();

        // Re-magic the mut slice from the storage we have now reclaimed by dropping the
        // tx_queue.
        //
        // SAFETY: teardown_inner has unset TXDMA_PRESENT and disabled TCIE: we now have
        // exclusive access to the shared tx resources, including the backing buffer.
        let tx_buffer = unsafe { core::slice::from_raw_parts_mut(ptr.as_ptr(), len) };

        // Move out the non-Copy fields by value before we mem::forget self below, so that
        // their destructors run normally (TxPins -> reclaims via take(), Option<WakeGuard>
        // -> releases the clock at end of scope).
        //
        // SAFETY: We unconditionally `mem::forget(self)` before returning, so reading
        // these fields here cannot cause a double-drop.
        let tx_pins: TxPins<'static> = unsafe { core::ptr::read(&self._tx_pins) };
        // SAFETY: see above.
        let _wg: Option<WakeGuard> = unsafe { core::ptr::read(&self._wg) };

        // Reclaim the data pin and (already-configured) CTS pin. The flow mux is
        // intentionally None: the reclaimed pin stays configured across a
        // teardown->rebuild cycle (take() forgets, so Drop never disabled it).
        let (data_pin, cts_pin) = tx_pins.take();

        let parts = BbqHalfParts {
            buffer: tx_buffer,
            dma_ch: tx_dma,
            pin: data_pin,
            dma_req: self.state.txdma_num.load(Ordering::Relaxed),
            mux: self.mux,
            info: self.info,
            state: self.state,
            vtable: self.vtable,
            which: WhichHalf::Tx,
            flow_pin: cts_pin,
            flow_mux: None,
        };

        // Prevent Drop::drop from re-running teardown_inner (which would re-enter the
        // already-completed cleanup, dropping the already-dropped tx_queue, etc.).
        core::mem::forget(self);

        parts
    }
}

impl Drop for LpuartBbqTx {
    fn drop(&mut self) {
        // Halt TX DMA, drop the tx_queue in place, and (if this is the last live half
        // of this LPUART) disable the peripheral. The returned DmaChannel is then
        // dropped immediately, which releases the channel. _tx_pins and _wg fields
        // are dropped automatically after this function returns.
        let _ = self.teardown_inner();
    }
}

use embedded_io_async::ErrorType;
impl embedded_io_async::Error for BbqError {
    fn kind(&self) -> embedded_io::ErrorKind {
        match self {
            BbqError::Basic(error) => error.kind(),
            BbqError::Busy => embedded_io::ErrorKind::Other,
            BbqError::WrongParts => embedded_io::ErrorKind::Other,
            BbqError::MaxFrameTooLarge => embedded_io::ErrorKind::OutOfMemory,
            BbqError::InvalidContinuousRxSize => embedded_io::ErrorKind::InvalidInput,
            BbqError::Overrun => embedded_io::ErrorKind::Other,
        }
    }
}
impl ErrorType for LpuartBbqTx {
    type Error = BbqError;
}

impl embedded_io_async::Write for LpuartBbqTx {
    fn write(&mut self, buf: &[u8]) -> impl Future<Output = Result<usize, Self::Error>> {
        self.write(buf)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush().await;
        Ok(())
    }
}

pub struct LpuartBbqRx {
    state: &'static BbqState,
    info: &'static Info,
    vtable: BbqVtable,
    mux: crate::pac::port::Mux,
    buffer_addr: usize,
    buffer_len: usize,
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
        mut dma: DmaChannel<'static>,
        rx_callback: fn(),
        rx_queue_buffer: &'static mut [u8],
        continuous_dma_buffer: Option<&'static mut [u8]>,
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
        let queue_storage = Container::from(rx_queue_buffer);

        // SAFETY: We have exclusive access to the shared RX components, and the interrupt
        // is not yet enabled. We move ownership of these resources to the shared area.
        unsafe {
            state.rx_queue.get().write(BBQueue::new_with_storage(queue_storage));
            // If a continuous DMA buffer is provided, store it in the shared state.
            if let Some(buffer) = continuous_dma_buffer {
                state.rx_dma_buffer.get().write(Container::from(buffer));
            }
            state.rx_published_pos.store(0, Ordering::Release);
            state.rx_publish_pending.store(false, Ordering::Release);
            state.rx_overrun.store(false, Ordering::Release);
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
    /// NOTE: Dropping the `LpuartBbqRx` shuts down RX DMA and (if no other half is live)
    /// disables the peripheral; the RX pin and clock gate are released via their own
    /// destructors. However, the RX backing buffer cannot be returned by `Drop` and is
    /// effectively leaked. Call [`LpuartBbqRx::teardown`] to reclaim the buffer along
    /// with the DMA channel and pin.
    pub fn new(parts: BbqHalfParts, config: BbqConfig, mode: BbqRxMode) -> Result<Self, BbqError> {
        // Are these the right parts?
        if parts.which != WhichHalf::Rx {
            return Err(BbqError::WrongParts);
        }

        // Validate and split the RX allocation before changing shared peripheral state.
        let rx_layout = prepare_rx_buffer(parts.buffer, mode)?;

        // Get state for this instance, and try to move from the "uninit" to "initing" state
        parts.state.uninit_to_initing()?;

        // Set RX pin mode
        any_as_rx(&parts.pin, parts.mux);

        // Configure optional RTS pin (skip reconfig for a teardown-reclaimed pin, mux == None).
        if let (Some(rts), Some(mux)) = (&parts.flow_pin, parts.flow_mux) {
            any_as_rts(rts, mux);
        }
        let enable_rts = parts.flow_pin.is_some();

        // Configure UART peripheral
        // TODO make this a specific Bbq mode instead of using blocking
        let _wg = (parts.vtable.lpuart_init)(false, true, false, enable_rts, config.into()).map_err(BbqError::Basic)?;

        // Setup the RX half state
        // SAFETY: We have ensured that we are in the INITING state, and the interrupt is not yet active.
        unsafe {
            Self::initialize_rx_state(
                parts.state,
                parts.dma_ch,
                parts.vtable.dma_rx_cb,
                rx_layout.queue,
                rx_layout.continuous_dma,
                parts.dma_req,
            );
        }

        // Update our state to "initialized", and that we have the RXDMA channel present
        // Okay to just store: we have exclusive access
        let new_state = STATE_INITED | STATE_RXDMA_PRESENT | rx_layout.mode_bits;
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
            buffer_addr: rx_layout.original_addr,
            buffer_len: rx_layout.original_len,
            _rx_pins: RxPins {
                rx_pin: parts.pin,
                rts_pin: parts.flow_pin,
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
    /// In efficiency and max-frame modes, data is discarded until this method frees
    /// capacity. In continuous mode, failing to publish a granted DMA half before the
    /// next boundary stops reception and returns [`BbqError::Overrun`].
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, BbqError> {
        // TODO: we could have a version of this that gives the user the grant directly
        // to reduce the effort of copying.

        // SAFETY: The existence of a LpuartBbqRx ensures that the `rx_queue` has been
        // initialized. The rx_queue is safe to access in a shared manner after initialization.
        let queue = unsafe { &*self.state.rx_queue.get() };
        let cons = queue.stream_consumer();
        if self.state.rx_overrun.load(Ordering::Acquire) {
            return Err(BbqError::Overrun);
        }

        // The overrun future is first so a simultaneously-ready error wins over
        // queued data. The flag is sticky until the RX half is reinitialized.
        let rgr = match select(
            self.state
                .rx_overrun_wait
                .wait_for(|| self.state.rx_overrun.load(Ordering::Acquire)),
            cons.wait_read(),
        )
        .await
        {
            Either::First(_) => return Err(BbqError::Overrun),
            Either::Second(rgr) => {
                if self.state.rx_overrun.load(Ordering::Acquire) {
                    drop(rgr);
                    return Err(BbqError::Overrun);
                }
                rgr
            }
        };
        let to_copy = buf.len().min(rgr.len());
        buf[..to_copy].copy_from_slice(&rgr[..to_copy]);
        rgr.release(to_copy);

        // If NO rx_dma is active, that means we stalled, so pend the interrupt to
        // restart it now that we've freed space.
        let state = self.state.state.load(Ordering::Acquire);
        if (state & STATE_RXGR_ACTIVE) == 0 || (state & STATE_RXDMA_MODE_CONTINUOUS) != 0 {
            (self.vtable.int_pend)();
        }

        Ok(to_copy)
    }

    /// Stop the RX side: disable RX interrupts, halt RX DMA, drop the rx_queue in place,
    /// and (if this was the last live half) disable the peripheral.
    ///
    /// Returns the reclaimed buffer pointer/length and DMA channel. The caller is
    /// responsible for ensuring the side effects are not performed a second time
    /// on the same instance (either by consuming `self` and calling `mem::forget`,
    /// or by only invoking this from `Drop`, which by definition runs once).
    fn teardown_inner(&mut self) -> (NonNull<u8>, usize, DmaChannel<'static>) {
        // First, mark the RXDMA as not present to halt the ISR from processing the state
        // machine
        let rx_state_bits = STATE_RXDMA_PRESENT
            | STATE_RXGR_ACTIVE
            | STATE_RXDMA_COMPLETE
            | STATE_RXDMA_MODE_MAXFRAME
            | STATE_RXDMA_MODE_CONTINUOUS
            | STATE_RXGR_LEN_MASK;
        let state = self.state.state.fetch_and(!rx_state_bits, Ordering::AcqRel);
        let continuous = (state & STATE_RXDMA_MODE_CONTINUOUS) != 0;

        // Then, disable receive-relevant interrupts
        critical_section::with(|_cs| {
            self.info.regs.ctrl().modify(|w| {
                w.set_ilie(false);
                w.set_neie(false);
                w.set_feie(false);
                w.set_orie(false);
            });
        });

        // Stop the peripheral request first, then wait for any accepted minor loop
        // to retire before returning the backing buffer to safe Rust.
        unsafe {
            // Take DMA channel by mut ref
            let rxdma = &mut *self.state.rxdma.get();

            // Stop the DMA
            self.info.regs().baud().modify(|w| w.set_rdmae(false));
            rxdma.clear_callback();
            rxdma.stop();

            if !continuous && (state & STATE_RXGR_ACTIVE) != 0 {
                // Grant-based modes take the grant by ownership and drop it.
                // Continuous mode writes a fixed staging ring and has no RX grant.
                _ = self.state.rxgr.get().read();
            }
        }
        fence(Ordering::Acquire);
        self.state.rx_published_pos.store(0, Ordering::Release);
        self.state.rx_publish_pending.store(false, Ordering::Release);
        self.state.rx_overrun.store(false, Ordering::Release);

        // Continuous mode splits the original allocation into staging and queue
        // regions. Preserve the original address and length carried by the RX handle
        // so teardown can return the complete allocation rather than only the queue.
        // SAFETY: Every constructor obtains this address from a non-null mutable slice.
        let ptr = unsafe { NonNull::new_unchecked(self.buffer_addr as *mut u8) };
        let len = self.buffer_len;

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
            self.state.rxdma.get().read()
        };

        // Now, if this was the last part of the lpuart, we are responsible for peripheral
        // cleanup.
        if (state & !rx_state_bits) == STATE_INITED {
            super::disable_peripheral(self.info);
            self.state.state.store(STATE_UNINIT, Ordering::Relaxed);
        }

        (ptr, len, rx_dma)
    }

    /// Teardown the Rx handle, reclaiming the DMA channel, receive buffer, and Rx pin.
    pub fn teardown(mut self) -> BbqHalfParts {
        let (ptr, len, rx_dma) = self.teardown_inner();

        // Re-magic the mut slice from the storage we have now reclaimed by dropping the
        // rx_queue.
        //
        // SAFETY: teardown_inner has unset RXDMA_PRESENT and disabled all RX interrupts:
        // we now have exclusive access to the shared rx resources, including the backing
        // buffer.
        let rx_buffer = unsafe { core::slice::from_raw_parts_mut(ptr.as_ptr(), len) };

        // Move out the non-Copy fields by value before we mem::forget self below, so that
        // their destructors run normally (RxPins -> reclaims via take(), Option<WakeGuard>
        // -> releases the clock at end of scope).
        //
        // SAFETY: We unconditionally `mem::forget(self)` before returning, so reading
        // these fields here cannot cause a double-drop.
        let rx_pins: RxPins<'static> = unsafe { core::ptr::read(&self._rx_pins) };
        // SAFETY: see above.
        let _wg: Option<WakeGuard> = unsafe { core::ptr::read(&self._wg) };

        // Reclaim the data pin and (already-configured) RTS pin. The flow mux is
        // intentionally None: the reclaimed pin stays configured across a
        // teardown->rebuild cycle (take() forgets, so Drop never disabled it).
        let (data_pin, rts_pin) = rx_pins.take();

        let parts = BbqHalfParts {
            buffer: rx_buffer,
            dma_ch: rx_dma,
            pin: data_pin,
            dma_req: self.state.rxdma_num.load(Ordering::Relaxed),
            mux: self.mux,
            info: self.info,
            state: self.state,
            vtable: self.vtable,
            which: WhichHalf::Rx,
            flow_pin: rts_pin,
            flow_mux: None,
        };

        // Prevent Drop::drop from re-running teardown_inner (which would re-enter the
        // already-completed cleanup, dropping the already-dropped rx_queue, etc.).
        core::mem::forget(self);

        parts
    }
}

impl Drop for LpuartBbqRx {
    fn drop(&mut self) {
        // Halt RX DMA, drop the rx_queue in place, and (if this is the last live half
        // of this LPUART) disable the peripheral. The returned DmaChannel is then
        // dropped immediately, which releases the channel. _rx_pins and _wg fields
        // are dropped automatically after this function returns.
        let _ = self.teardown_inner();
    }
}

impl embedded_io_async::ErrorType for LpuartBbqRx {
    type Error = BbqError;
}

impl embedded_io_async::Read for LpuartBbqRx {
    fn read(&mut self, buf: &mut [u8]) -> impl Future<Output = Result<usize, Self::Error>> {
        self.read(buf)
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

pub(crate) const STATE_UNINIT: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0000;
pub(crate) const STATE_INITING: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0001;
pub(crate) const STATE_INITED: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0011;
pub(crate) const STATE_RXGR_ACTIVE: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0100;
pub(crate) const STATE_TXGR_ACTIVE: u32 = 0b0000_0000_0000_0000_0000_0000_0000_1000;
pub(crate) const STATE_RXDMA_PRESENT: u32 = 0b0000_0000_0000_0000_0000_0000_0001_0000;
pub(crate) const STATE_TXDMA_PRESENT: u32 = 0b0000_0000_0000_0000_0000_0000_0010_0000;
pub(crate) const STATE_RXDMA_COMPLETE: u32 = 0b0000_0000_0000_0000_0000_0000_0100_0000;
pub(crate) const STATE_RXDMA_MODE_MAXFRAME: u32 = 0b0000_0000_0000_0000_0000_0000_1000_0000;
pub(crate) const STATE_RXDMA_MODE_CONTINUOUS: u32 = 0b0000_0000_0000_0000_0000_0001_0000_0000;
pub(crate) const STATE_RXGR_LEN_MASK: u32 = 0b1111_1111_1111_1111_0000_0000_0000_0000;

pub(crate) struct BbqState {
    /// 0bGGGG_GGGG_GGGG_GGGG_xxxx_xxxx_MDTR_PCAI
    ///                                        ^^--> 0b00: uninit, 0b01: initing, 0b11 init'd.
    ///                                       ^----> 0b0: No Rx grant, 0b1: Rx grant active
    ///                                      ^-----> 0b0: No Tx grant, 0b1: Tx grant active
    ///                                    ^-------> 0b0: No Rx DMA present, 0b1: Rx DMA present
    ///                                   ^--------> 0b0: No Tx DMA present, 0b1: Tx DMA present
    ///                                  ^---------> 0b0: Rx DMA not complete, 0b1: Rx DMA complete
    ///                                 ^----------> 0b1: RxMode "Max Frame"
    ///                                ^-----------> 0b1: RxMode "Continuous"
    ///   ^^^^_^^^^_^^^^_^^^^----------------------> 16-bit: RX Grant size
    pub(crate) state: AtomicU32,

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
    /// Fixed circular DMA staging buffer used only in continuous RX mode.
    ///
    /// Only valid when `STATE_RXDMA_MODE_CONTINUOUS` is set.
    rx_dma_buffer: GroundedCell<Container>,
    /// Next byte in the continuous DMA staging ring that has not yet been
    /// published to `rx_queue`.
    rx_published_pos: AtomicU32,
    /// A staging range could not be published because the BBQueue was full.
    rx_publish_pending: AtomicBool,
    /// Sticky continuous-DMA overrun indication.
    rx_overrun: AtomicBool,
    /// Wakes a blocked reader when a continuous-DMA overrun occurs.
    rx_overrun_wait: WaitCell,
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
    pub(crate) const fn new() -> Self {
        Self {
            state: AtomicU32::new(0),
            tx_queue: GroundedCell::uninit(),
            rx_queue: GroundedCell::uninit(),
            rx_dma_buffer: GroundedCell::uninit(),
            rx_published_pos: AtomicU32::new(0),
            rx_publish_pending: AtomicBool::new(false),
            rx_overrun: AtomicBool::new(false),
            rx_overrun_wait: WaitCell::new(),
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

    /// Record a terminal continuous-DMA overrun and wake a blocked reader.
    pub(crate) fn record_continuous_overrun(&'static self) {
        if !self.rx_overrun.swap(true, Ordering::AcqRel) {
            self.rx_overrun_wait.wake();
        }
    }

    /// Resolve an inconsistent partial-publish snapshot.
    ///
    /// A DMA boundary landing between the sampled values makes them legitimately
    /// inconsistent, so retry on the next pass instead of failing the transfer.
    fn partial_retry_or_overrun(&'static self, rxdma: &DmaChannel<'static>) -> Result<bool, ()> {
        if rxdma.ping_pong_boundary_pending() {
            self.rx_publish_pending.store(true, Ordering::Release);
            return Ok(false);
        }

        self.record_continuous_overrun();
        Err(())
    }

    /// Stop continuous RX after an overrun without releasing its resources.
    ///
    /// ## SAFETY
    ///
    /// * Continuous RX mode must be initialized.
    /// * This must only run from the serialized LPUART interrupt handler.
    unsafe fn stop_continuous_read(&'static self, info: &'static Info) {
        // SAFETY: RXDMA_PRESENT and ISR serialization give exclusive mutable
        // access to the channel while the transfer is stopped.
        let rxdma = unsafe { &mut *self.rxdma.get() };
        info.regs().baud().modify(|w| w.set_rdmae(false));
        rxdma.stop();
        self.rx_publish_pending.store(false, Ordering::Release);
        self.state.fetch_and(!STATE_RXGR_ACTIVE, Ordering::AcqRel);
    }

    /// Publish one complete DMA-granted buffer into the consumer BBQueue.
    ///
    /// Returns `Ok(false)` when the BBQueue is full; the buffer remains granted,
    /// so reaching the next DMA boundary turns into an overrun.
    ///
    /// ## SAFETY
    ///
    /// * Continuous RX mode must be initialized and active.
    /// * `buffer` must currently be granted by the DMA ping-pong state machine.
    /// * This must only run from the serialized LPUART interrupt handler.
    unsafe fn publish_continuous_buffer(&'static self, buffer: PingPongSelector) -> Result<bool, ()> {
        // SAFETY: Continuous-mode initialization writes these resources before
        // enabling interrupts, and teardown clears RXDMA_PRESENT first.
        let (dma_buffer, rx_queue, rxdma) =
            unsafe { (&*self.rx_dma_buffer.get(), &*self.rx_queue.get(), &*self.rxdma.get()) };

        let len = dma_buffer.len;
        if len < 2 || !len.is_multiple_of(2) {
            self.record_continuous_overrun();
            return Err(());
        }

        let half_len = len / 2;
        let published = self.rx_published_pos.load(Ordering::Acquire) as usize;
        let (boundary, next_published, valid_position) = match buffer {
            PingPongSelector::BufferA => (half_len, half_len, published <= half_len),
            PingPongSelector::BufferB => (len, 0, published >= half_len && published <= len),
        };
        if !valid_position {
            self.record_continuous_overrun();
            return Err(());
        }

        let available = boundary - published;
        if available == 0 {
            if rxdma.commit_ping_pong_buffer(buffer).is_err() {
                self.record_continuous_overrun();
                return Err(());
            }
            self.rx_published_pos.store(next_published as u32, Ordering::Release);
            self.rx_publish_pending.store(false, Ordering::Release);
            return Ok(true);
        }

        let prod = rx_queue.stream_producer();
        let Ok(mut wgr) = prod.grant_exact(available) else {
            self.rx_publish_pending.store(true, Ordering::Release);
            return Ok(false);
        };

        fence(Ordering::Acquire);
        // SAFETY: DMA has granted this completed buffer and cannot legally reuse
        // this exact range until `commit_ping_pong_buffer` succeeds below. The
        // slice deliberately excludes the opposite buffer that DMA is writing.
        let source = unsafe { core::slice::from_raw_parts(dma_buffer.ptr.as_ptr().add(published), available) };
        wgr[..available].copy_from_slice(source);

        // Complete all source reads before testing whether DMA wrapped back into
        // this half, which would mean the copy above raced against fresh writes.
        fence(Ordering::Release);
        if rxdma.ping_pong_boundary_pending() {
            drop(wgr);
            self.record_continuous_overrun();
            return Err(());
        }

        // Return ownership to DMA only after the source copy has completed.
        if rxdma.commit_ping_pong_buffer(buffer).is_err() {
            drop(wgr);
            self.record_continuous_overrun();
            return Err(());
        }

        self.rx_published_pos.store(next_published as u32, Ordering::Release);
        self.rx_publish_pending.store(false, Ordering::Release);
        wgr.commit(available);
        Ok(true)
    }

    /// Publish bytes already staged in the currently active half.
    ///
    /// If a DMA boundary appears while copying, the uncommitted BBQueue grant is
    /// discarded and the completed half is handled through the ownership state
    /// machine on the next ISR pass.
    ///
    /// ## SAFETY
    ///
    /// * Continuous RX mode must be initialized and active.
    /// * This must only run from the serialized LPUART interrupt handler.
    unsafe fn publish_continuous_partial(&'static self) -> Result<bool, ()> {
        // SAFETY: Continuous-mode initialization writes these resources before
        // enabling interrupts, and teardown clears RXDMA_PRESENT first.
        let (dma_buffer, rx_queue, rxdma) =
            unsafe { (&*self.rx_dma_buffer.get(), &*self.rx_queue.get(), &*self.rxdma.get()) };

        if rxdma.ping_pong_boundary_pending() {
            self.rx_publish_pending.store(true, Ordering::Release);
            return Ok(false);
        }
        let Some(status) = rxdma.ping_pong_status() else {
            self.record_continuous_overrun();
            return Err(());
        };
        match status.granted_buffer() {
            Ok(None) => {}
            Ok(Some(_)) => return Ok(false),
            Err(_) => {
                self.record_continuous_overrun();
                return Err(());
            }
        }

        let len = dma_buffer.len;
        if len < 2 || !len.is_multiple_of(2) {
            self.record_continuous_overrun();
            return Err(());
        }
        let start = dma_buffer.ptr.as_ptr() as usize;
        let end = start.saturating_add(len);
        let daddr = rxdma.daddr() as usize;
        if daddr < start || daddr > end {
            return self.partial_retry_or_overrun(rxdma);
        }

        let write_pos = daddr.wrapping_sub(start) % len;
        let published = self.rx_published_pos.load(Ordering::Acquire) as usize;
        if write_pos == published {
            self.rx_publish_pending.store(false, Ordering::Release);
            return Ok(true);
        }

        let half_len = len / 2;
        let in_current_buffer = match status.current {
            PingPongSelector::BufferA => published <= half_len && write_pos <= half_len,
            PingPongSelector::BufferB => published >= half_len && write_pos >= half_len,
        };
        if write_pos < published || !in_current_buffer {
            return self.partial_retry_or_overrun(rxdma);
        }

        let available = write_pos - published;
        let prod = rx_queue.stream_producer();
        let Ok(mut wgr) = prod.grant_exact(available) else {
            self.rx_publish_pending.store(true, Ordering::Release);
            return Ok(false);
        };

        fence(Ordering::Acquire);
        // SAFETY: This range precedes the sampled DADDR in the active half. A
        // boundary during the copy is detected before the grant is committed.
        // The slice excludes bytes at and after DADDR that DMA may still write.
        let source = unsafe { core::slice::from_raw_parts(dma_buffer.ptr.as_ptr().add(published), available) };
        wgr[..available].copy_from_slice(source);

        // Complete all source reads before deciding that DMA has not crossed
        // this half boundary during the copy.
        fence(Ordering::Release);
        let boundary_crossed = rxdma.ping_pong_boundary_pending() || rxdma.ping_pong_status() != Some(status);
        if boundary_crossed {
            drop(wgr);
            self.rx_publish_pending.store(true, Ordering::Release);
            return Ok(false);
        }

        self.rx_published_pos.store(write_pos as u32, Ordering::Release);
        self.rx_publish_pending.store(false, Ordering::Release);
        wgr.commit(available);
        Ok(true)
    }

    /// Start the fixed staging-ring transfer used by continuous RX mode.
    ///
    /// ## SAFETY
    ///
    /// * Continuous RX mode must be initialized but inactive.
    /// * This must only run from ISR context while RXDMA_PRESENT is set.
    unsafe fn start_continuous_read(&'static self, info: &'static Info) -> bool {
        // SAFETY: Continuous mode initialization writes these resources before
        // enabling the LPUART interrupt, and no transfer is active here.
        let (rxdma, dma_buffer) = unsafe { (&mut *self.rxdma.get(), &mut *self.rx_dma_buffer.get()) };
        // SAFETY: Initialization stored the request number from the typed RX DMA
        // request associated with this LPUART instance.
        let source = unsafe { DmaRequest::from_number_unchecked(self.rxdma_num.load(Ordering::Relaxed)) };

        // SAFETY: This ISR has exclusive access to the owned channel while
        // continuous RX is inactive.
        unsafe {
            rxdma.disable_request();
            rxdma.clear_done();
            rxdma.clear_interrupt();
            rxdma.set_request_source(source);
        }

        // SAFETY: The staging buffer is exclusively owned by BbqState and remains
        // valid until teardown stops DMA. No Rust reference escapes this function.
        let buffer = unsafe { core::slice::from_raw_parts_mut(dma_buffer.ptr.as_ptr(), dma_buffer.len) };
        let peri_addr = info.regs().data().as_ptr().cast::<u8>();
        // SAFETY: The LPUART data register and staging allocation remain valid
        // until teardown disables RDMAE and stops this channel.
        let setup = unsafe { rxdma.setup_ping_pong_read_from_peripheral(peri_addr, buffer, Priority::default()) };
        if setup.is_err() {
            return false;
        }

        self.rx_published_pos.store(0, Ordering::Release);
        self.rx_publish_pending.store(false, Ordering::Release);
        self.rx_overrun.store(false, Ordering::Release);
        info.regs().baud().modify(|w| w.set_rdmae(true));
        // SAFETY: The ping-pong TCD and request source were fully configured above.
        unsafe {
            rxdma.enable_request();
        }
        // Mark the RX as active in the shared state.
        self.state.fetch_or(STATE_RXGR_ACTIVE, Ordering::AcqRel);

        true
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
            if let Err(InvalidParameters) =
                txdma.setup_write_to_peripheral(&rgr[..len], peri_addr, false, TransferOptions::COMPLETE_INTERRUPT)
            {
                return false;
            }

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
        while info.regs.stat().read().tc() == Tc::Complete {}

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
            if let Err(InvalidParameters) =
                rxdma.setup_read_from_peripheral(peri_addr, &mut wgr, false, TransferOptions::COMPLETE_INTERRUPT)
            {
                return false;
            }

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

#[doc(hidden)]
#[macro_export]
macro_rules! impl_lpuart_bbq_instance {
    ($n:expr) => {
        paste::paste! {
            #[allow(private_interfaces)]
            impl $crate::lpuart::bbq::BbqInstance for $crate::peripherals::[<LPUART $n>] {
                fn bbq_state() -> &'static $crate::lpuart::bbq::BbqState {
                    static STATE: $crate::lpuart::bbq::BbqState = $crate::lpuart::bbq::BbqState::new();
                    &STATE
                }

                fn dma_rx_complete_cb() {
                    use $crate::_generated::interrupt::typelevel::Interrupt;

                    let state = Self::bbq_state();
                    // Mark the DMA as complete
                    state.state.fetch_or($crate::lpuart::bbq::STATE_RXDMA_COMPLETE, core::sync::atomic::Ordering::AcqRel);
                    // Pend the UART interrupt to handle the switchover
                    Self::Interrupt::pend();
                }
            }
        }
    };
}

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
    let fe = stat.fe();
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
        let continuous = (pre_clear & STATE_RXDMA_MODE_CONTINUOUS) != 0;

        if continuous {
            if rx_active {
                // SAFETY: Continuous mode owns a fixed staging ring, RXDMA is present
                // and active, and this is the serialized LPUART ISR.
                let result = unsafe {
                    let rxdma = &*state.rxdma.get();
                    match rxdma.ping_pong_status() {
                        Some(status) => match status.granted_buffer() {
                            // A boundary and an IDLE can be observed in the same ISR pass,
                            // so the tail already staged in the now-active half must be
                            // flushed here too or it stalls until the next boundary.
                            Ok(Some(buffer)) => match state.publish_continuous_buffer(buffer) {
                                Ok(true) => state.publish_continuous_partial(),
                                other => other,
                            },
                            Ok(None) => state.publish_continuous_partial(),
                            Err(_) => {
                                state.record_continuous_overrun();
                                Err(())
                            }
                        },
                        None => {
                            state.record_continuous_overrun();
                            Err(())
                        }
                    }
                };

                if result.is_err() || state.rx_overrun.load(Ordering::Acquire) {
                    // An overrun is terminal until this RX half is torn down and
                    // initialized again. Stop both the peripheral request source
                    // and DMA before waking/returning the user-facing error.
                    regs.ctrl().modify(|w| w.set_ilie(false));
                    // SAFETY: Continuous RX is active in the serialized UART ISR.
                    unsafe {
                        state.stop_continuous_read(info);
                    }
                }
            } else if !state.rx_overrun.load(Ordering::Acquire) {
                // Initial transition from Idle -> Receiving. Once started, continuous
                // mode remains active until teardown.
                //
                // SAFETY: RXDMA is present, no transfer is active, and this is ISR context.
                unsafe {
                    let started = state.start_continuous_read(info);
                    regs.ctrl().modify(|w| w.set_ilie(started));
                }
            }
        } else {
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
        let tc_complete = regs.stat().read().tc() == Tc::Complete;
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

fn any_as_tx(pin: &Peri<'_, AnyPin>, mux: crate::pac::port::Mux) {
    pin.set_pull(crate::gpio::Pull::Disabled);
    pin.set_slew_rate(crate::gpio::SlewRate::Fast.into());
    pin.set_drive_strength(crate::gpio::DriveStrength::Normal.into());
    pin.set_function(mux);
    pin.set_enable_input_buffer(false);
}

fn any_as_rx(pin: &Peri<'_, AnyPin>, mux: crate::pac::port::Mux) {
    pin.set_pull(crate::gpio::Pull::Disabled);
    pin.set_function(mux);
    pin.set_enable_input_buffer(true);
}

fn any_as_cts(pin: &Peri<'_, AnyPin>, mux: crate::pac::port::Mux) {
    pin.set_pull(crate::gpio::Pull::Disabled);
    pin.set_function(mux);
    pin.set_enable_input_buffer(true);
}

fn any_as_rts(pin: &Peri<'_, AnyPin>, mux: crate::pac::port::Mux) {
    pin.set_pull(crate::gpio::Pull::Disabled);
    pin.set_slew_rate(crate::gpio::SlewRate::Fast.into());
    pin.set_drive_strength(crate::gpio::DriveStrength::Normal.into());
    pin.set_function(mux);
    pin.set_enable_input_buffer(false);
}
