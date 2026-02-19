use core::marker::PhantomData;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU8, Ordering};

use bbqueue::BBQueue;
use bbqueue::prod_cons::stream::StreamGrantW;
use bbqueue::traits::coordination::cas::AtomicCoord;
use bbqueue::traits::notifier::maitake::MaiNotSpsc;
use bbqueue::traits::storage::Storage;
use embassy_hal_internal::Peri;
use grounded::uninit::GroundedCell;
use nxp_pac::lpuart::vals::Tc;

use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::lpuart::{Instance, Lpuart};
use crate::{clocks::WakeGuard, interrupt::typelevel::Handler};

use super::{Config, Info, State, TxPin, TxPins};
use paste::paste;

pub enum Error {
    Basic(super::Error),
}

pub struct LpuartBbqTx<'a> {
    info: &'static Info,
    state: &'static BbqState,
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
        config: Config,
    ) -> Result<Self, Error> {
        let state = T::bbq_state();
        match state.state.compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => {}
            Err(_) => todo!(),
        }
        let cont = Container::from(tx_buffer);
        <T as Instance>::Interrupt::unpend();
        unsafe {
            state.queue.get().write(BBQueue::new_with_storage(cont));
            <T as Instance>::Interrupt::enable();
        }

        let _wg =
            Lpuart::<super::blocking::Blocking>::init::<T>(true, false, false, false, config).map_err(Error::Basic)?;

        Ok(Self {
            info: T::info(),
            state,
            _tx_pins: TxPins {
                tx_pin: tx_pin.into(),
                cts_pin: None,
            },
            _wg,
            _phantom: PhantomData,
        })
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

struct BbqState {
    state: AtomicU8,
    // 0bxxxx_PxAI
    //          -- -> 0b00: uninit, 0b01: initing, 0b11 init'd.
    //        - ----> 0b0: Write grant active

    queue: GroundedCell<BBQueue<Container, AtomicCoord, MaiNotSpsc>>,
    wgr: GroundedCell<StreamGrantW<&'static BBQueue<Container, AtomicCoord, MaiNotSpsc>>>,
}

impl BbqState {
    const fn new() -> Self {
        Self {
            state: AtomicU8::new(0),
            queue: GroundedCell::uninit(),
            wgr: GroundedCell::uninit(),
        }
    }
}

#[allow(private_bounds)]
pub trait BbqInstance: Instance {
    fn bbq_state() -> &'static BbqState;
}

macro_rules! impl_instance {
    ($($n:expr);* $(;)?) => {
        $(
            paste!{
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
            // TODO?
            // T::PERF_INT_WAKE_INCR();
            // state.rx_waker.wake();
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

        // // Handle TX data
        if ctrl.tie() {
            if stat.tc() == Tc::COMPLETE {
                // todo?
            }
        //     let mut sent_any = false;
        //     let mut reader = unsafe { state.tx_buf.reader() };
        //     let to_pop = if has_tx_fifo {
        //         // tx fifo size is 2^param.txfifo, we want to pop enough to fill
        //         // the fifo, minus whatever is in there now.
        //         (1 << param.txfifo()) - regs.water().read().txcount()
        //     } else {
        //         if regs.stat().read().tdre() != Tdre::TXDATA {
        //             1
        //         } else {
        //             0
        //         }
        //     };

        //     // Send data while TX buffer is ready and we have data
        //     for _ in 0..to_pop {
        //         if let Some(byte) = reader.pop_one() {
        //             regs.data().write(|w| w.0 = u32::from(byte));
        //             sent_any = true;
        //         } else {
        //             // No more data to send
        //             break;
        //         }
        //     }

        //     if sent_any {
        //         T::PERF_INT_WAKE_INCR();
        //         state.tx_waker.wake();
        //     }

        //     // If buffer is empty, switch to TC interrupt or disable
        //     if state.tx_buf.is_empty() {
        //         cortex_m::interrupt::free(|_| {
        //             regs.ctrl().modify(|w| {
        //                 w.set_tie(false);
        //                 w.set_tcie(true);
        //             });
        //         });
        //     }
        }

        // // Handle transmission complete
        // if ctrl.tcie() && regs.stat().read().tc() == Tc::COMPLETE {
        //     T::PERF_INT_WAKE_INCR();
        //     state.tx_waker.wake();

        //     // Disable TC interrupt
        //     cortex_m::interrupt::free(|_| {
        //         regs.ctrl().modify(|w| w.set_tcie(false));
        //     });
        // }
    }
}
