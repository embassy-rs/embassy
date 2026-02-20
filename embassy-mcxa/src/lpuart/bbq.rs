use core::marker::PhantomData;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering, fence};

use bbqueue::BBQueue;
use bbqueue::prod_cons::stream::{StreamGrantR, StreamGrantW};
use bbqueue::traits::coordination::cas::AtomicCoord;
use bbqueue::traits::notifier::maitake::MaiNotSpsc;
use bbqueue::traits::storage::Storage;
use embassy_hal_internal::Peri;
use grounded::uninit::GroundedCell;
use nxp_pac::lpuart::vals::{Dozeen, Tc};

use crate::dma::{Channel, DMA_MAX_TRANSFER_SIZE, DmaChannel, DmaRequest, EnableInterrupt};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::lpuart::{Instance, Lpuart};
use crate::{clocks::WakeGuard, interrupt::typelevel::Handler};

use super::{Config, Info, TxPin, TxPins};
use paste::paste;

#[derive(Debug)]
pub enum Error {
    Basic(super::Error),
}

pub struct LpuartBbqTx<'a> {
    info: &'static Info,
    state: &'static BbqState,
    int_pend: fn(),
    _tx_pins: TxPins<'a>,
    _wg: Option<WakeGuard>,
    _phantom: PhantomData<&'a ()>,
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
        let tx_buf_len = tx_buffer.len();
        let state = T::bbq_state();
        tx_pin.as_tx();
        match state.state.compare_exchange(STATE_UNINIT, STATE_INITING, Ordering::AcqRel, Ordering::Acquire) {
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
        state.txdma_num.store(T::TxDmaRequest::REQUEST_NUMBER, Ordering::Release);
        state.tx_queue_len.store(tx_buf_len, Ordering::Release);
        unsafe {
            <T as Instance>::Interrupt::unpend();
            <T as Instance>::Interrupt::enable();
        }

        Ok(Self {
            info: T::info(),
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
        let queue = unsafe {
            &*self.state.tx_queue.get()
        };
        let prod = queue.stream_producer();
        let mut wgr = prod.wait_grant_max_remaining(buf.len()).await;
        let to_copy = buf.len().min(wgr.len());
        wgr[..to_copy].copy_from_slice(&buf[..to_copy]);
        wgr.commit(to_copy);
        // defmt::info!("Wrote, pending");
        // critical_section::with(|_| {
        //     self.info.regs.ctrl().modify(|w| {
        //         w.set_dozeen(Dozeen::ENABLED);
        //         w.set_te(true);
        //     });
        // });
        (self.int_pend)();

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
const _STATE_RXGR_ACTIVE: u8 = 0b0000_0100;
const STATE_TXGR_ACTIVE: u8 = 0b0000_1000;
const _STATE_RXDMA_PRESENT: u8 = 0b0001_0000;
const STATE_TXDMA_PRESENT: u8 = 0b0010_0000;

struct BbqState {
    state: AtomicU8,
    // 0bxxTR_PCAI
    //          ^^--> 0b00: uninit, 0b01: initing, 0b11 init'd.
    //         ^----> 0b0: No Rx grant, 0b1: Rx grant active
    //        ^-----> 0b0: No Tx grant, 0b1: Tx grant active
    //      ^-------> 0b0: No Rx DMA present, 0b1: Rx DMA present
    //     ^--------> 0b0: No Tx DMA present, 0b1: Tx DMA present

    tx_queue: GroundedCell<BBQueue<Container, AtomicCoord, MaiNotSpsc>>,
    // TODO: we should have a capacity field for bbqueue, this is redundant
    tx_queue_len: AtomicUsize,
    _rxgr: GroundedCell<StreamGrantW<&'static BBQueue<Container, AtomicCoord, MaiNotSpsc>>>,
    txgr: GroundedCell<StreamGrantR<&'static BBQueue<Container, AtomicCoord, MaiNotSpsc>>>,
    txdma: GroundedCell<DmaChannel<'static>>,
    _rxdma: GroundedCell<DmaChannel<'static>>,

    txdma_num: AtomicU8,
    _rxdma_num: AtomicU8,
}

impl BbqState {
    const fn new() -> Self {
        Self {
            state: AtomicU8::new(0),
            tx_queue: GroundedCell::uninit(),
            _rxgr: GroundedCell::uninit(),
            txgr: GroundedCell::uninit(),
            txdma: GroundedCell::uninit(),
            txdma_num: AtomicU8::new(0),
            _rxdma: GroundedCell::uninit(),
            _rxdma_num: AtomicU8::new(0),
            tx_queue_len: AtomicUsize::new(0),
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
            let max_len = self.tx_queue_len.load(Ordering::Relaxed) / 4;
            let xfer = txgr.len().min(max_len).min(DMA_MAX_TRANSFER_SIZE);
            txgr.release(xfer);
        }
        self.state.fetch_and(!STATE_TXGR_ACTIVE, Ordering::AcqRel);
    }

    unsafe fn start_write_transfer(&'static self, info: &'static Info) -> bool {
        let queue = unsafe { &*self.tx_queue.get() };
        let Ok(rgr) = queue.stream_consumer().read() else {
            // defmt::info!(":(");
            return false;
        };

        unsafe {
            let txdma = &mut *self.txdma.get();
            txdma.disable_request();
            txdma.clear_done();
            txdma.clear_interrupt();
            txdma.set_request_source(self.txdma_num.load(Ordering::Relaxed));
            let peri_addr = info.regs().data().as_ptr().cast::<u8>();
            let max_len = self.tx_queue_len.load(Ordering::Relaxed) / 4;
            let len = rgr.len().min(max_len).min(DMA_MAX_TRANSFER_SIZE);
            // defmt::info!("max len: {=usize}", len);

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
}

#[allow(private_bounds, private_interfaces)]
pub trait BbqInstance: Instance {
    fn bbq_state() -> &'static BbqState;
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

        // Handle overrun error
        if stat.or() {
            regs.stat().write(|w| w.set_or(true));
            // TODO? How do we report errors?
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

        // // Handle RX data
        // if ctrl.rie() && (has_rx_data_pending(T::info()) || stat.idle()) {
        //     let mut pushed_any = false;
        //     let mut writer = unsafe { state.rx_buf.writer() };

        //     if has_rx_fifo {
        //         // Read from FIFO as long as there is data available and
        //         // somewhere to put it
        //         while regs.water().read().rxcount() > 0 && !state.rx_buf.is_full() {
        //             let byte = regs.data().read().0 as u8;
        //             writer.push_one(byte);
        //             pushed_any = true;
        //         }
        //     } else {
        //         // Read single byte if possible
        //         if regs.stat().read().rdrf() && !state.rx_buf.is_full() {
        //             let byte = (regs.data().read().0 & 0xFF) as u8;
        //             writer.push_one(byte);
        //             pushed_any = true;
        //         }
        //     }

        //     if pushed_any {
        //         T::PERF_INT_WAKE_INCR();
        //         state.rx_waker.wake();
        //     }

        //     // Clear idle flag if set
        //     if stat.idle() {
        //         regs.stat().write(|w| w.set_idle(true));
        //     }
        // }

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
