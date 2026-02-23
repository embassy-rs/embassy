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

use crate::dma::{Channel, DMA_MAX_TRANSFER_SIZE, DmaChannel, DmaRequest, EnableInterrupt};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::lpuart::{Instance, Lpuart, RxPins};
use crate::{clocks::WakeGuard, interrupt::typelevel::Handler};

use super::{Config, Info, RxPin, TxPin, TxPins};
use paste::paste;

#[derive(Debug)]
pub enum Error {
    Basic(super::Error),
}

pub struct LpuartBbqTx<'a> {
    state: &'static BbqState,
    int_pend: fn(),
    _tx_pins: TxPins<'a>,
    _wg: Option<WakeGuard>,
    _phantom: PhantomData<&'a mut ()>,
}

impl<'a> LpuartBbqTx<'a> {
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
        let state = T::bbq_state();
        tx_pin.as_tx();
        match state
            .state
            .compare_exchange(STATE_UNINIT, STATE_INITING, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => {}
            Err(_) => todo!(),
        }
        let cont = Container::from(tx_buffer);
        let dma = DmaChannel::new(tx_dma_ch);

        let _wg =
            Lpuart::<super::blocking::Blocking>::init::<T>(true, false, false, false, config).map_err(Error::Basic)?;

        dma.enable_interrupt();
        unsafe {
            state.tx_queue.get().write(BBQueue::new_with_storage(cont));
            state.txdma.get().write(dma);
        }
        let new_state = STATE_INITED | STATE_TXDMA_PRESENT;
        state.state.store(new_state, Ordering::Release);
        state
            .txdma_num
            .store(T::TxDmaRequest::REQUEST_NUMBER, Ordering::Release);
        unsafe {
            <T as Instance>::Interrupt::unpend();
            <T as Instance>::Interrupt::enable();
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
        let state = T::bbq_state();
        rx_pin.as_rx();
        match state
            .state
            .compare_exchange(STATE_UNINIT, STATE_INITING, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => {}
            Err(_) => todo!(),
        }
        let cont = Container::from(rx_buffer);
        let mut dma = DmaChannel::new(rx_dma_ch);

        let _wg =
            Lpuart::<super::blocking::Blocking>::init::<T>(false, true, false, false, config).map_err(Error::Basic)?;

        unsafe {
            dma.set_callback(T::dma_complete_cb);
        }

        dma.enable_interrupt();
        unsafe {
            state.rx_queue.get().write(BBQueue::new_with_storage(cont));
            state.rxdma.get().write(dma);
        }
        let new_state = STATE_INITED | STATE_RXDMA_PRESENT;
        state.state.store(new_state, Ordering::Release);
        state
            .rxdma_num
            .store(T::RxDmaRequest::REQUEST_NUMBER, Ordering::Release);
        T::info().regs().ctrl().modify(|w| {
            w.set_orie(true);
            w.set_neie(true);
            w.set_feie(true);
        });
        unsafe {
            <T as Instance>::Interrupt::unpend();
            <T as Instance>::Interrupt::enable();
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
        let queue = unsafe { &*self.state.rx_queue.get() };
        let cons = queue.stream_consumer();
        let rgr = cons.wait_read().await;
        let to_copy = buf.len().min(rgr.len());
        buf[..to_copy].copy_from_slice(&rgr[..to_copy]);
        rgr.release(to_copy);
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
    state: AtomicU8,
    // 0bxDTR_PCAI
    //          ^^--> 0b00: uninit, 0b01: initing, 0b11 init'd.
    //         ^----> 0b0: No Rx grant, 0b1: Rx grant active
    //        ^-----> 0b0: No Tx grant, 0b1: Tx grant active
    //      ^-------> 0b0: No Rx DMA present, 0b1: Rx DMA present
    //     ^--------> 0b0: No Tx DMA present, 0b1: Tx DMA present
    //    ^---------> 0b0: Rx DMA not complete, 0b1: Rx DMA complete
    tx_queue: GroundedCell<BBQueue<Container, AtomicCoord, MaiNotSpsc>>,
    rx_queue: GroundedCell<BBQueue<Container, AtomicCoord, MaiNotSpsc>>,
    rxgr: GroundedCell<StreamGrantW<&'static BBQueue<Container, AtomicCoord, MaiNotSpsc>>>,
    txgr: GroundedCell<StreamGrantR<&'static BBQueue<Container, AtomicCoord, MaiNotSpsc>>>,
    txdma: GroundedCell<DmaChannel<'static>>,
    rxdma: GroundedCell<DmaChannel<'static>>,

    txdma_num: AtomicU8,
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

    unsafe fn finalize_write(&'static self, info: &'static Info) {
        let mask = STATE_INITED | STATE_TXGR_ACTIVE | STATE_TXDMA_PRESENT;
        let load = self.state.load(Ordering::Acquire);
        assert_eq!(load & mask, mask);
        unsafe {
            let txgr = self.txgr.get().read();
            let txdma = &mut *self.txdma.get();
            assert!(txdma.is_done());
            // Stop the DMA
            info.regs().baud().modify(|w| w.set_tdmae(false));
            txdma.disable_request();
            txdma.clear_done();
            fence(Ordering::Acquire);
            let max_len = (&*self.tx_queue.get()).capacity() / 4;
            let xfer = txgr.len().min(max_len).min(DMA_MAX_TRANSFER_SIZE);
            txgr.release(xfer);
        }
        self.state.fetch_and(!STATE_TXGR_ACTIVE, Ordering::AcqRel);
    }

    unsafe fn finalize_read(&'static self, info: &'static Info) {
        let mask = STATE_INITED | STATE_RXGR_ACTIVE | STATE_RXDMA_PRESENT;
        let load = self.state.load(Ordering::Acquire);
        assert_eq!(load & mask, mask);

        unsafe {
            let rxgr = self.rxgr.get().read();
            let rxdma = &mut *self.rxdma.get();
            // The DMA may NOT be done yet if this was an idle interrupt
            // TODO, we could make this done if we enable "end of packet"
            // processing? I don't know how to do that.
            info.regs().baud().modify(|w| w.set_rdmae(false));
            rxdma.disable_request();
            rxdma.clear_done();
            fence(Ordering::AcqRel);

            let daddr = rxdma.daddr() as usize;
            let sstrt = rxgr.as_ptr() as usize;
            let ttl = daddr.wrapping_sub(sstrt).min(rxgr.len());
            // defmt::info!("committing {=usize}", ttl);
            rxgr.commit(ttl);
        }
        self.state.fetch_and(!STATE_RXGR_ACTIVE, Ordering::AcqRel);
    }

    unsafe fn start_write_transfer(&'static self, info: &'static Info) -> bool {
        let tx_queue = unsafe { &*self.tx_queue.get() };
        let Ok(rgr) = tx_queue.stream_consumer().read() else {
            // Nothing to do!
            return false;
        };

        unsafe {
            let txdma = &mut *self.txdma.get();
            txdma.disable_request();
            txdma.clear_done();
            txdma.clear_interrupt();
            txdma.set_request_source(self.txdma_num.load(Ordering::Relaxed));

            let peri_addr = info.regs().data().as_ptr().cast::<u8>();
            let max_len = (&*self.tx_queue.get()).capacity() / 4;
            let len = rgr.len().min(max_len).min(DMA_MAX_TRANSFER_SIZE);

            txdma.setup_write_to_peripheral(&rgr[..len], peri_addr, EnableInterrupt::Yes);
            info.regs().baud().modify(|w| w.set_tdmae(true));
            txdma.enable_request();
            self.txgr.get().write(rgr);
            self.state.fetch_or(STATE_TXGR_ACTIVE, Ordering::AcqRel);
        }

        // wait until the system is not reporting TC complete, to ensure we don't
        while info.regs.stat().read().tc() == Tc::COMPLETE {}

        true
    }

    unsafe fn start_read_transfer(&'static self, info: &'static Info) -> bool {
        let rx_queue = unsafe { &*self.rx_queue.get() };
        let max_len = rx_queue.capacity() / 4;
        let len = max_len.min(DMA_MAX_TRANSFER_SIZE);
        let Ok(mut wgr) = rx_queue.stream_producer().grant_max_remaining(len) else {
            // Nothing to do!
            panic!();
            return false;
        };

        unsafe {
            let rxdma = &mut *self.rxdma.get();
            rxdma.disable_request();
            rxdma.clear_done();
            rxdma.clear_interrupt();
            rxdma.set_request_source(self.rxdma_num.load(Ordering::Relaxed));

            let peri_addr = info.regs().data().as_ptr().cast::<u8>();
            let len = wgr.len();
            rxdma.setup_read_from_peripheral(peri_addr, &mut wgr, EnableInterrupt::Yes);

            info.regs().baud().modify(|w| w.set_rdmae(true));
            rxdma.enable_request();
            self.rxgr.get().write(wgr);
            self.state.fetch_or(STATE_RXGR_ACTIVE, Ordering::AcqRel);
        }

        true
    }
}

#[allow(private_bounds, private_interfaces)]
pub trait BbqInstance: Instance {
    fn bbq_state() -> &'static BbqState;
    fn dma_complete_cb();
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

                    fn dma_complete_cb() {
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

        let or = stat.or();
        let pf = stat.pf();
        let fe = stat.pf();
        let nf = stat.nf();
        let idle = stat.idle();

        // Just clear any errors - TODO, signal these to the consumer?
        regs.stat().modify(|w| {
            w.set_or(or);
            w.set_pf(pf);
            w.set_fe(fe);
            w.set_nf(nf);
            w.set_idle(idle);
        });

        if or || pf || fe || nf {
            defmt::error!("ERR {=bool} {=bool} {=bool} {=bool}", or, pf, fe, nf);
        }

        // Check DMA complete or idle interrupt occurred - we need to stop
        // the current RX transfer in either case.
        let pre_clear = state.state.fetch_and(!STATE_RXDMA_COMPLETE, Ordering::AcqRel);
        let dma_complete = (pre_clear & STATE_RXDMA_COMPLETE) != 0;

        if idle || dma_complete {
            defmt::warn!("FIN {=bool} {=bool}", idle, dma_complete);
            unsafe {
                state.finalize_read(info);
            }
        }

        let rx_idle = (state.state.load(Ordering::Acquire) & STATE_RXGR_ACTIVE) == 0;
        if rx_idle {
            unsafe {
                let started = state.start_read_transfer(info);
                regs.ctrl().modify(|w| w.set_ilie(started));
            }
        }

        // Handle TX data
        if ctrl.tcie() && regs.stat().read().tc() == Tc::COMPLETE {
            unsafe {
                state.finalize_write(info);
            }
        }

        let tx_idle = (state.state.load(Ordering::Acquire) & STATE_TXGR_ACTIVE) == 0;

        if tx_idle {
            unsafe {
                let started = state.start_write_transfer(info);
                // Enable tcie if we started a transfer, otherwise disable
                regs.ctrl().modify(|w| w.set_tcie(started));
            }
        }
    }
}
