//! Temperature sensor interface.

use crate::interrupt;
use crate::pac;
use crate::peripherals::TEMP;

use core::marker::PhantomData;
use core::task::Poll;
use embassy::interrupt::InterruptExt;
use embassy::util::Unborrow;
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::{drop::OnDrop, unborrow};
use fixed::types::I30F2;
use futures::future::poll_fn;

/// Integrated temperature sensor.
pub struct Temp<'d> {
    _temp: PhantomData<&'d TEMP>,
    _irq: interrupt::TEMP,
}

static WAKER: AtomicWaker = AtomicWaker::new();

impl<'d> Temp<'d> {
    pub fn new(
        _t: impl Unborrow<Target = TEMP> + 'd,
        irq: impl Unborrow<Target = interrupt::TEMP> + 'd,
    ) -> Self {
        unborrow!(_t, irq);

        // Enable interrupt that signals temperature values
        irq.disable();
        irq.set_handler(|_| {
            let t = Self::regs();
            t.intenclr.write(|w| w.datardy().clear());
            WAKER.wake();
        });
        irq.enable();
        Self {
            _temp: PhantomData,
            _irq: irq,
        }
    }

    /// Perform an asynchronous temperature measurement. The returned future
    /// can be awaited to obtain the measurement.
    ///
    /// If the future is dropped, the measurement is cancelled.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let mut t = Temp::new(p.TEMP, interrupt::take!(TEMP));
    /// let v: u16 = t.read().await.to_num::<u16>();
    /// ```
    pub async fn read(&mut self) -> I30F2 {
        // In case the future is dropped, stop the task and reset events.
        let on_drop = OnDrop::new(|| {
            let t = Self::regs();
            t.tasks_stop.write(|w| unsafe { w.bits(1) });
            t.events_datardy.reset();
        });

        let t = Self::regs();
        t.intenset.write(|w| w.datardy().set());
        unsafe { t.tasks_start.write(|w| w.bits(1)) };

        let value = poll_fn(|cx| {
            WAKER.register(cx.waker());
            if t.events_datardy.read().bits() == 0 {
                return Poll::Pending;
            } else {
                t.events_datardy.reset();
                let raw = t.temp.read().bits();
                Poll::Ready(I30F2::from_bits(raw as i32))
            }
        })
        .await;
        on_drop.defuse();
        value
    }

    fn regs() -> &'static pac::temp::RegisterBlock {
        unsafe { &*pac::TEMP::ptr() }
    }
}
