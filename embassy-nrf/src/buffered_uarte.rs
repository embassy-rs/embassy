//! Async buffered UART
//!
//! WARNING!!! The functionality provided here is intended to be used only
//! in situations where hardware flow control are available i.e. CTS and RTS.
//! This is a problem that should be addressed at a later stage and can be
//! fully explained at https://github.com/embassy-rs/embassy/issues/536.
//!
//! Note that discarding a future from a read or write operation may lead to losing
//! data. For example, when using `futures_util::future::select` and completion occurs
//! on the "other" future, you should capture the incomplete future and continue to use
//! it for the next read or write. This pattern is a consideration for all IO, and not
//! just serial communications.
//!
//! Please also see [crate::uarte] to understand when [BufferedUarte] should be used.

use core::cmp::min;
use core::marker::PhantomData;
use core::mem;
use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};
use embassy::interrupt::InterruptExt;
use embassy::io::{AsyncBufRead, AsyncWrite};
use embassy::util::Unborrow;
use embassy::waitqueue::WakerRegistration;
use embassy_hal_common::peripheral::{PeripheralMutex, PeripheralState, StateStorage};
use embassy_hal_common::ring_buffer::RingBuffer;
use embassy_hal_common::{low_power_wait_until, unborrow};

use crate::gpio::Pin as GpioPin;
use crate::pac;
use crate::ppi::{AnyConfigurableChannel, ConfigurableChannel, Event, Ppi, Task};
use crate::timer::Instance as TimerInstance;
use crate::timer::{Frequency, Timer};
use crate::uarte::{apply_workaround_for_enable_anomaly, Config, Instance as UarteInstance};

// Re-export SVD variants to allow user to directly set values
pub use pac::uarte0::{baudrate::BAUDRATE_A as Baudrate, config::PARITY_A as Parity};

#[derive(Copy, Clone, Debug, PartialEq)]
enum RxState {
    Idle,
    Receiving,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum TxState {
    Idle,
    Transmitting(usize),
}

pub struct State<'d, U: UarteInstance, T: TimerInstance>(StateStorage<StateInner<'d, U, T>>);
impl<'d, U: UarteInstance, T: TimerInstance> State<'d, U, T> {
    pub fn new() -> Self {
        Self(StateStorage::new())
    }
}

struct StateInner<'d, U: UarteInstance, T: TimerInstance> {
    phantom: PhantomData<&'d mut U>,
    timer: Timer<'d, T>,
    _ppi_ch1: Ppi<'d, AnyConfigurableChannel, 1, 2>,
    _ppi_ch2: Ppi<'d, AnyConfigurableChannel, 1, 1>,

    rx: RingBuffer<'d>,
    rx_state: RxState,
    rx_waker: WakerRegistration,

    tx: RingBuffer<'d>,
    tx_state: TxState,
    tx_waker: WakerRegistration,
}

/// Interface to a UARTE instance
pub struct BufferedUarte<'d, U: UarteInstance, T: TimerInstance> {
    inner: PeripheralMutex<'d, StateInner<'d, U, T>>,
}

impl<'d, U: UarteInstance, T: TimerInstance> Unpin for BufferedUarte<'d, U, T> {}

impl<'d, U: UarteInstance, T: TimerInstance> BufferedUarte<'d, U, T> {
    pub fn new(
        state: &'d mut State<'d, U, T>,
        _uarte: impl Unborrow<Target = U> + 'd,
        timer: impl Unborrow<Target = T> + 'd,
        ppi_ch1: impl Unborrow<Target = impl ConfigurableChannel + 'd> + 'd,
        ppi_ch2: impl Unborrow<Target = impl ConfigurableChannel + 'd> + 'd,
        irq: impl Unborrow<Target = U::Interrupt> + 'd,
        rxd: impl Unborrow<Target = impl GpioPin> + 'd,
        txd: impl Unborrow<Target = impl GpioPin> + 'd,
        cts: impl Unborrow<Target = impl GpioPin> + 'd,
        rts: impl Unborrow<Target = impl GpioPin> + 'd,
        config: Config,
        rx_buffer: &'d mut [u8],
        tx_buffer: &'d mut [u8],
    ) -> Self {
        unborrow!(ppi_ch1, ppi_ch2, irq, rxd, txd, cts, rts);

        let r = U::regs();

        let mut timer = Timer::new(timer);

        rxd.conf().write(|w| w.input().connect().drive().h0h1());
        r.psel.rxd.write(|w| unsafe { w.bits(rxd.psel_bits()) });

        txd.set_high();
        txd.conf().write(|w| w.dir().output().drive().h0h1());
        r.psel.txd.write(|w| unsafe { w.bits(txd.psel_bits()) });

        cts.conf().write(|w| w.input().connect().drive().h0h1());
        r.psel.cts.write(|w| unsafe { w.bits(cts.psel_bits()) });

        rts.set_high();
        rts.conf().write(|w| w.dir().output().drive().h0h1());
        r.psel.rts.write(|w| unsafe { w.bits(rts.psel_bits()) });

        r.baudrate.write(|w| w.baudrate().variant(config.baudrate));
        r.config.write(|w| w.parity().variant(config.parity));

        // Configure
        r.config.write(|w| {
            w.hwfc().bit(true);
            w.parity().variant(config.parity);
            w
        });
        r.baudrate.write(|w| w.baudrate().variant(config.baudrate));

        // Enable interrupts
        r.intenset.write(|w| w.endrx().set().endtx().set());

        // Disable the irq, let the Registration enable it when everything is set up.
        irq.disable();
        irq.pend();

        // Enable UARTE instance
        apply_workaround_for_enable_anomaly(&r);
        r.enable.write(|w| w.enable().enabled());

        // BAUDRATE register values are `baudrate * 2^32 / 16000000`
        // source: https://devzone.nordicsemi.com/f/nordic-q-a/391/uart-baudrate-register-values
        //
        // We want to stop RX if line is idle for 2 bytes worth of time
        // That is 20 bits (each byte is 1 start bit + 8 data bits + 1 stop bit)
        // This gives us the amount of 16M ticks for 20 bits.
        let timeout = 0x8000_0000 / (config.baudrate as u32 / 40);

        timer.set_frequency(Frequency::F16MHz);
        timer.cc(0).write(timeout);
        timer.cc(0).short_compare_clear();
        timer.cc(0).short_compare_stop();

        let mut ppi_ch1 = Ppi::new_one_to_two(
            ppi_ch1.degrade(),
            Event::from_reg(&r.events_rxdrdy),
            timer.task_clear(),
            timer.task_start(),
        );
        ppi_ch1.enable();

        let mut ppi_ch2 = Ppi::new_one_to_one(
            ppi_ch2.degrade(),
            timer.cc(0).event_compare(),
            Task::from_reg(&r.tasks_stoprx),
        );
        ppi_ch2.enable();

        Self {
            inner: unsafe {
                PeripheralMutex::new_unchecked(irq, &mut state.0, move || StateInner {
                    phantom: PhantomData,
                    timer,
                    _ppi_ch1: ppi_ch1,
                    _ppi_ch2: ppi_ch2,

                    rx: RingBuffer::new(rx_buffer),
                    rx_state: RxState::Idle,
                    rx_waker: WakerRegistration::new(),

                    tx: RingBuffer::new(tx_buffer),
                    tx_state: TxState::Idle,
                    tx_waker: WakerRegistration::new(),
                })
            },
        }
    }

    pub fn set_baudrate(&mut self, baudrate: Baudrate) {
        self.inner.with(|state| {
            let r = U::regs();

            let timeout = 0x8000_0000 / (baudrate as u32 / 40);
            state.timer.cc(0).write(timeout);
            state.timer.clear();

            r.baudrate.write(|w| w.baudrate().variant(baudrate));
        });
    }
}

impl<'d, U: UarteInstance, T: TimerInstance> AsyncBufRead for BufferedUarte<'d, U, T> {
    fn poll_fill_buf(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<embassy::io::Result<&[u8]>> {
        self.inner.with(|state| {
            compiler_fence(Ordering::SeqCst);
            trace!("poll_read");

            // We have data ready in buffer? Return it.
            let buf = state.rx.pop_buf();
            if !buf.is_empty() {
                trace!("  got {:?} {:?}", buf.as_ptr() as u32, buf.len());
                let buf: &[u8] = buf;
                let buf: &[u8] = unsafe { mem::transmute(buf) };
                return Poll::Ready(Ok(buf));
            }

            trace!("  empty");
            state.rx_waker.register(cx.waker());
            Poll::<embassy::io::Result<&[u8]>>::Pending
        })
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        self.inner.with(|state| {
            trace!("consume {:?}", amt);
            state.rx.pop(amt);
        });
        self.inner.pend();
    }
}

impl<'d, U: UarteInstance, T: TimerInstance> AsyncWrite for BufferedUarte<'d, U, T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<embassy::io::Result<usize>> {
        let poll = self.inner.with(|state| {
            trace!("poll_write: {:?}", buf.len());

            let tx_buf = state.tx.push_buf();
            if tx_buf.is_empty() {
                trace!("poll_write: pending");
                state.tx_waker.register(cx.waker());
                return Poll::Pending;
            }

            let n = min(tx_buf.len(), buf.len());
            tx_buf[..n].copy_from_slice(&buf[..n]);
            state.tx.push(n);

            trace!("poll_write: queued {:?}", n);

            compiler_fence(Ordering::SeqCst);

            Poll::Ready(Ok(n))
        });

        self.inner.pend();

        poll
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<embassy::io::Result<()>> {
        self.inner.with(|state| {
            trace!("poll_flush");

            if !state.tx.is_empty() {
                trace!("poll_flush: pending");
                state.tx_waker.register(cx.waker());
                return Poll::Pending;
            }

            Poll::Ready(Ok(()))
        })
    }
}

impl<'a, U: UarteInstance, T: TimerInstance> Drop for StateInner<'a, U, T> {
    fn drop(&mut self) {
        let r = U::regs();

        // TODO this probably deadlocks. do like Uarte instead.

        self.timer.stop();
        if let RxState::Receiving = self.rx_state {
            r.tasks_stoprx.write(|w| unsafe { w.bits(1) });
        }
        if let TxState::Transmitting(_) = self.tx_state {
            r.tasks_stoptx.write(|w| unsafe { w.bits(1) });
        }
        if let RxState::Receiving = self.rx_state {
            low_power_wait_until(|| r.events_endrx.read().bits() == 1);
        }
        if let TxState::Transmitting(_) = self.tx_state {
            low_power_wait_until(|| r.events_endtx.read().bits() == 1);
        }
    }
}

impl<'a, U: UarteInstance, T: TimerInstance> PeripheralState for StateInner<'a, U, T> {
    type Interrupt = U::Interrupt;
    fn on_interrupt(&mut self) {
        trace!("irq: start");
        let r = U::regs();

        loop {
            match self.rx_state {
                RxState::Idle => {
                    trace!("  irq_rx: in state idle");

                    let buf = self.rx.push_buf();
                    if !buf.is_empty() {
                        trace!("  irq_rx: starting {:?}", buf.len());
                        self.rx_state = RxState::Receiving;

                        // Set up the DMA read
                        r.rxd.ptr.write(|w|
                            // The PTR field is a full 32 bits wide and accepts the full range
                            // of values.
                            unsafe { w.ptr().bits(buf.as_ptr() as u32) });
                        r.rxd.maxcnt.write(|w|
                            // We're giving it the length of the buffer, so no danger of
                            // accessing invalid memory. We have verified that the length of the
                            // buffer fits in an `u8`, so the cast to `u8` is also fine.
                            //
                            // The MAXCNT field is at least 8 bits wide and accepts the full
                            // range of values.
                            unsafe { w.maxcnt().bits(buf.len() as _) });
                        trace!("  irq_rx: buf {:?} {:?}", buf.as_ptr() as u32, buf.len());

                        // Start UARTE Receive transaction
                        r.tasks_startrx.write(|w| unsafe { w.bits(1) });
                    }
                    break;
                }
                RxState::Receiving => {
                    trace!("  irq_rx: in state receiving");
                    if r.events_endrx.read().bits() != 0 {
                        self.timer.stop();

                        let n: usize = r.rxd.amount.read().amount().bits() as usize;
                        trace!("  irq_rx: endrx {:?}", n);
                        self.rx.push(n);

                        r.events_endrx.reset();

                        self.rx_waker.wake();
                        self.rx_state = RxState::Idle;
                    } else {
                        break;
                    }
                }
            }
        }

        loop {
            match self.tx_state {
                TxState::Idle => {
                    trace!("  irq_tx: in state Idle");
                    let buf = self.tx.pop_buf();
                    if !buf.is_empty() {
                        trace!("  irq_tx: starting {:?}", buf.len());
                        self.tx_state = TxState::Transmitting(buf.len());

                        // Set up the DMA write
                        r.txd.ptr.write(|w|
                            // The PTR field is a full 32 bits wide and accepts the full range
                            // of values.
                            unsafe { w.ptr().bits(buf.as_ptr() as u32) });
                        r.txd.maxcnt.write(|w|
                            // We're giving it the length of the buffer, so no danger of
                            // accessing invalid memory. We have verified that the length of the
                            // buffer fits in an `u8`, so the cast to `u8` is also fine.
                            //
                            // The MAXCNT field is 8 bits wide and accepts the full range of
                            // values.
                            unsafe { w.maxcnt().bits(buf.len() as _) });

                        // Start UARTE Transmit transaction
                        r.tasks_starttx.write(|w| unsafe { w.bits(1) });
                    }
                    break;
                }
                TxState::Transmitting(n) => {
                    trace!("  irq_tx: in state Transmitting");
                    if r.events_endtx.read().bits() != 0 {
                        r.events_endtx.reset();

                        trace!("  irq_tx: endtx {:?}", n);
                        self.tx.pop(n);
                        self.tx_waker.wake();
                        self.tx_state = TxState::Idle;
                    } else {
                        break;
                    }
                }
            }
        }
        trace!("irq: end");
    }
}
