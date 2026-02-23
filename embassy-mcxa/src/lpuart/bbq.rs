use core::marker::PhantomData;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU8, Ordering, fence};

use bbqueue::BBQueue;
use bbqueue::prod_cons::stream::{StreamGrantR, StreamGrantW};
use bbqueue::traits::coordination::cas::AtomicCoord;
use bbqueue::traits::notifier::maitake::MaiNotSpsc;
use bbqueue::traits::storage::Storage;
use embassy_hal_internal::Peri;
use grounded::uninit::GroundedCell;
use nxp_pac::lpuart::vals::Tc;
use paste::paste;

use super::{Config, Info, RxPin, TxPin, TxPins};
use crate::clocks::WakeGuard;
use crate::dma::{Channel, DMA_MAX_TRANSFER_SIZE, DmaChannel, DmaRequest, EnableInterrupt};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::lpuart::{Instance, Lpuart, RxPins};

#[derive(Debug)]
pub enum Error {
    Basic(super::Error),
    Busy,
}

pub struct LpuartBbq<'a> {
    tx: LpuartBbqTx<'a>,
    rx: LpuartBbqRx<'a>,
}

pub struct LpuartBbqTx<'a> {
    state: &'static BbqState,
    int_pend: fn(),
    _tx_pins: TxPins<'a>,
    _wg: Option<WakeGuard>,
    _phantom: PhantomData<&'a mut ()>,
}

impl<'a> LpuartBbqTx<'a> {
    fn initialize_tx_state(
        state: &'static BbqState,
        tx_dma_ch: Peri<'static, impl Channel>,
        tx_buffer: &'static mut [u8],
        request_num: u8,
    ) {
        // Create DMA channel to use for TX
        let dma = DmaChannel::new(tx_dma_ch);
        // Enable the DMA interrupt to handle "transfer complete" interrupts
        dma.enable_interrupt();

        let cont = Container::from(tx_buffer);

        // Setup the TX bbqueue instance, store the DMA channel and bbqueue in the
        // BbqState storage location.
        //
        // TODO: We could probably be more clever and setup the DMA transfer request
        // number ONCE in init, then just do a minimal-reload. This would allow us to
        // avoid storing the rxdma_num, and save some effort in the ISR.
        unsafe {
            state.tx_queue.get().write(BBQueue::new_with_storage(cont));
            state.txdma.get().write(dma);
            state.txdma_num.store(request_num, Ordering::Release);
        }
    }

    pub fn new<T: BbqInstance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        _irq: impl Binding<T::Interrupt, BbqInterruptHandler<T>> + 'a,
        // TODO: something better for this
        tx_buffer: &'static mut [u8],
        // TODO: lifetime
        tx_dma_ch: Peri<'static, impl Channel>,
        config: Config,
    ) -> Result<Self, Error> {
        // Get state for this instance, and try to move from the "uninit" to "initing" state
        let state = T::bbq_state();
        state.uninit_to_initing()?;

        // Set as TX pin mode
        tx_pin.as_tx();

        // Configure UART peripheral
        // TODO make this a specific Bbq mode instead of using blocking
        // TODO support CTS pin?
        let _wg =
            Lpuart::<super::blocking::Blocking>::init::<T>(true, false, false, false, config).map_err(Error::Basic)?;

        // Setup the TX Half state
        Self::initialize_tx_state(state, tx_dma_ch, tx_buffer, T::TxDmaRequest::REQUEST_NUMBER);

        // Update our state to "initialized", and that we have the TXDMA channel present
        // Okay to just store: we have exclusive access
        let new_state = STATE_INITED | STATE_TXDMA_PRESENT;
        state.state.store(new_state, Ordering::Release);

        unsafe {
            // Clear any stale interrupt flags
            <T as Instance>::Interrupt::unpend();
            // Enable the LPUART interrupt
            <T as Instance>::Interrupt::enable();
            // NOTE: Unlike RX, we don't begin transmitting immediately, we move
            // from Idle -> Transmitting the first time the user calls write.
        }

        Ok(Self {
            state,
            int_pend: T::Interrupt::pend,
            _tx_pins: TxPins {
                tx_pin: tx_pin.into(),
                cts_pin: None,
            },
            _wg,
            _phantom: PhantomData,
        })
    }

    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        // TODO: we could have a version of this that gives the user the grant directly
        // to reduce the effort of copying.
        let tx_queue = unsafe { &*self.state.tx_queue.get() };
        let prod = tx_queue.stream_producer();
        let mut wgr = prod.wait_grant_max_remaining(buf.len()).await;
        let to_copy = buf.len().min(wgr.len());
        wgr[..to_copy].copy_from_slice(&buf[..to_copy]);
        wgr.commit(to_copy);
        (self.int_pend)();

        Ok(to_copy)
    }
}

pub struct LpuartBbqRx<'a> {
    state: &'static BbqState,
    int_pend: fn(),
    _rx_pins: RxPins<'a>,
    _wg: Option<WakeGuard>,
    _phantom: PhantomData<&'a mut ()>,
}

impl<'a> LpuartBbqRx<'a> {
    fn initialize_rx_state(
        state: &'static BbqState,
        info: &'static Info,
        rx_dma_ch: Peri<'static, impl Channel>,
        rx_callback: fn(),
        rx_buffer: &'static mut [u8],
        request_num: u8,
    ) {
        // Create DMA channel to use with RX
        let mut dma = DmaChannel::new(rx_dma_ch);

        // Set the callback to our completion handler, so our LPUART interrupt gets called to
        // complete the transfer and reload
        //
        // TODO: Right now we only do this on RX, we might want to also handle this on TX as well
        // so we have more time to reload, but for now we'll naturally get the "transfer complete"
        // interrupt when the TX fifo empties, and we are less latency sensitive on TX than RX.
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
        unsafe {
            state.rx_queue.get().write(BBQueue::new_with_storage(cont));
            state.rxdma.get().write(dma);
            state.rxdma_num.store(request_num, Ordering::Release);
        }

        // TODO: Do we actually want these interrupts enabled? We probably do, so we can
        // clear the errors, but I'm not sure if any of these actually stall the receive.
        info.regs().ctrl().modify(|w| {
            // overrun
            w.set_orie(true);
            // noise
            w.set_neie(true);
            // framing
            w.set_feie(true);
        });
    }

    pub fn new<T: BbqInstance>(
        _inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        _irq: impl Binding<T::Interrupt, BbqInterruptHandler<T>> + 'a,
        // TODO: something better for this
        rx_buffer: &'static mut [u8],
        // TODO: lifetime
        rx_dma_ch: Peri<'static, impl Channel>,
        config: Config,
    ) -> Result<Self, Error> {
        // Get state for this instance, and try to move from the "uninit" to "initing" state
        let state = T::bbq_state();
        state.uninit_to_initing()?;

        // Set RX pin mode
        rx_pin.as_rx();

        // Configure UART peripheral
        // TODO make this a specific Bbq mode instead of using blocking
        // TODO support RTS pin?
        let _wg =
            Lpuart::<super::blocking::Blocking>::init::<T>(false, true, false, false, config).map_err(Error::Basic)?;

        // Setup the RX half state
        Self::initialize_rx_state(
            state,
            T::info(),
            rx_dma_ch,
            T::dma_rx_complete_cb,
            rx_buffer,
            T::RxDmaRequest::REQUEST_NUMBER,
        );

        // Update our state to "initialized", and that we have the RXDMA channel present
        // Okay to just store: we have exclusive access
        let new_state = STATE_INITED | STATE_RXDMA_PRESENT;
        state.state.store(new_state, Ordering::Release);

        unsafe {
            // Clear any stale interrupt flags
            <T as Instance>::Interrupt::unpend();
            // Enable the LPUART interrupt
            <T as Instance>::Interrupt::enable();
            // Immediately pend the interrupt, this will "load" the DMA transfer as the
            // ISR will notice that there is no active grant. This means that we start
            // receiving immediately without additional user interaction.
            <T as Instance>::Interrupt::pend();
        }

        Ok(Self {
            state,
            int_pend: T::Interrupt::pend,
            _rx_pins: RxPins {
                rx_pin: rx_pin.into(),
                rts_pin: None,
            },
            _wg,
            _phantom: PhantomData,
        })
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        // TODO: we could have a version of this that gives the user the grant directly
        // to reduce the effort of copying.
        let queue = unsafe { &*self.state.rx_queue.get() };
        let cons = queue.stream_consumer();
        let rgr = cons.wait_read().await;
        let to_copy = buf.len().min(rgr.len());
        buf[..to_copy].copy_from_slice(&rgr[..to_copy]);
        rgr.release(to_copy);

        // If NO rx_dma is active, that means we stalled, so pend the interrupt to
        // restart it now that we've freed space.
        if (self.state.state.load(Ordering::Acquire) & STATE_RXGR_ACTIVE) == 0 {
            (self.int_pend)();
        }

        Ok(to_copy)
    }
}

struct Container {
    ptr: NonNull<u8>,
    len: usize,
}

impl Storage for Container {
    unsafe fn ptr_len(&self) -> (NonNull<u8>, usize) {
        (self.ptr, self.len)
    }
}

impl From<&'static mut [u8]> for Container {
    fn from(value: &'static mut [u8]) -> Self {
        Self {
            len: value.len(),
            ptr: unsafe { NonNull::new_unchecked(value.as_mut_ptr()) },
        }
    }
}

/// interrupt handler.
pub struct BbqInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

const STATE_UNINIT: u8 = 0b0000_0000;
const STATE_INITING: u8 = 0b0000_0001;
const STATE_INITED: u8 = 0b0000_0011;
const STATE_RXGR_ACTIVE: u8 = 0b0000_0100;
const STATE_TXGR_ACTIVE: u8 = 0b0000_1000;
const STATE_RXDMA_PRESENT: u8 = 0b0001_0000;
const STATE_TXDMA_PRESENT: u8 = 0b0010_0000;
const STATE_RXDMA_COMPLETE: u8 = 0b0100_0000;

struct BbqState {
    /// 0bxDTR_PCAI
    ///          ^^--> 0b00: uninit, 0b01: initing, 0b11 init'd.
    ///         ^----> 0b0: No Rx grant, 0b1: Rx grant active
    ///        ^-----> 0b0: No Tx grant, 0b1: Tx grant active
    ///      ^-------> 0b0: No Rx DMA present, 0b1: Rx DMA present
    ///     ^--------> 0b0: No Tx DMA present, 0b1: Tx DMA present
    ///    ^---------> 0b0: Rx DMA not complete, 0b1: Rx DMA complete
    state: AtomicU8,

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
}

impl BbqState {
    const fn new() -> Self {
        Self {
            state: AtomicU8::new(0),
            tx_queue: GroundedCell::uninit(),
            rx_queue: GroundedCell::uninit(),
            rxgr: GroundedCell::uninit(),
            txgr: GroundedCell::uninit(),
            txdma: GroundedCell::uninit(),
            txdma_num: AtomicU8::new(0),
            rxdma: GroundedCell::uninit(),
            rxdma_num: AtomicU8::new(0),
        }
    }

    /// Attempt to move from the "uninit" state to the "initing" state. Returns an
    /// error if we are not in the "uninit" state.
    fn uninit_to_initing(&'static self) -> Result<(), Error> {
        self.state
            .compare_exchange(STATE_UNINIT, STATE_INITING, Ordering::AcqRel, Ordering::Acquire)
            .map(drop)
            .map_err(|_| Error::Busy)
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
        let tx_queue = unsafe { &*self.tx_queue.get() };
        let Ok(rgr) = tx_queue.stream_consumer().read() else {
            // Nothing to do!
            return false;
        };

        unsafe {
            // Take the TXDMA by &mut ref
            let txdma = &mut *self.txdma.get();

            // Initialize the transfer from the bbqueue grant to DMA
            //
            // TODO: Most of this setup is redundant/repeated, we could save some effort
            // since most DMA transfer parameters are the same.
            txdma.disable_request();
            txdma.clear_done();
            txdma.clear_interrupt();
            txdma.set_request_source(self.txdma_num.load(Ordering::Relaxed));

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
    /// * The TXDMA must be present
    /// * A write grant must NOT be active
    /// * We must be in ISR context
    unsafe fn start_read_transfer(&'static self, info: &'static Info) -> bool {
        let rx_queue = unsafe { &*self.rx_queue.get() };

        // Limit the transfer size to 1/4 the capacity to limit max latency from
        // receiving bytes off the wire to making them available to the user for
        // draining, to avoid completely filling the buffer before the user has
        // a chance to drain and make the capacity available again.
        let max_len = (rx_queue.capacity() / 4).min(DMA_MAX_TRANSFER_SIZE);

        let Ok(mut wgr) = rx_queue.stream_producer().grant_max_remaining(max_len) else {
            // If we can't get a grant, that's a problem. Return false to note we didn't
            // start one, and hope the user frees space soon. See the `read` method for
            // how read transfers are restarted in this case.
            return false;
        };

        unsafe {
            // Initialize the transfer from the DMA to the bbqueue grant
            //
            // TODO: Most of this setup is redundant/repeated, we could save some effort
            // since most DMA transfer parameters are the same.
            let rxdma = &mut *self.rxdma.get();
            rxdma.disable_request();
            rxdma.clear_done();
            rxdma.clear_interrupt();
            rxdma.set_request_source(self.rxdma_num.load(Ordering::Relaxed));

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

impl<T: BbqInstance> Handler<T::Interrupt> for BbqInterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();

        let info = T::info();
        let regs = info.regs();
        let state = T::bbq_state();
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
        let dma_complete = (pre_clear & STATE_RXDMA_COMPLETE) != 0;

        if idle || dma_complete {
            // State change, move from Receiving -> Idle
            unsafe {
                state.finalize_read(info);
            }
        }

        // If we are now idle, attempt to "reload" the transfer and being receiving again ASAP.
        // Only do this if RXDMA is present.
        let rx_idle =
            (state.state.load(Ordering::Acquire) & (STATE_RXGR_ACTIVE | STATE_RXDMA_PRESENT)) == STATE_RXDMA_PRESENT;
        if rx_idle {
            // Either Idle -> Receiving or Idle -> Idle
            unsafe {
                let started = state.start_read_transfer(info);
                // Enable ILIE if we started a transfer, otherwise (keep) disabled.
                // ILIE - Idle Line Interrupt Enable
                regs.ctrl().modify(|w| w.set_ilie(started));
            }
        }

        //
        // TX state machine
        //

        // Handle TX data - TCIE is only enabled if we are transmitting, and we only
        // check that the outgoing transfer is complete. In the future, we might
        // try to do this a bit earlier if the DMA completes but we haven't yet
        // drained the TX fifo yet.
        if ctrl.tcie() && regs.stat().read().tc() == Tc::COMPLETE {
            // State change, move from Transmitting -> Idle
            unsafe {
                state.finalize_write(info);
            }
        }

        // If we are now idle, attempt to "reload" the transfer and begin transmitting again.
        // Only do this if TXDMA is present.
        let tx_idle =
            (state.state.load(Ordering::Acquire) & (STATE_TXGR_ACTIVE | STATE_TXDMA_PRESENT)) == STATE_TXDMA_PRESENT;
        if tx_idle {
            // Either Idle -> Transmitting or Idle -> Idle
            unsafe {
                let started = state.start_write_transfer(info);
                // Enable tcie if we started a transfer, otherwise (keep) disabled.
                // TCIE - Transfer Complete Interrupt Enable
                regs.ctrl().modify(|w| w.set_tcie(started));
            }
        }
    }
}
