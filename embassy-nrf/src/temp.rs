//! Temperature sensor interface.

use core::future::poll_fn;
use core::task::Poll;

use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use fixed::types::I30F2;

use crate::interrupt::InterruptExt;
use crate::peripherals::TEMP;
use crate::{interrupt, pac, Peripheral};

/// Integrated temperature sensor.
pub struct Temp<'d> {
    _irq: PeripheralRef<'d, interrupt::TEMP>,
}

static WAKER: AtomicWaker = AtomicWaker::new();

impl<'d> Temp<'d> {
    pub fn new(_t: impl Peripheral<P = TEMP> + 'd, irq: impl Peripheral<P = interrupt::TEMP> + 'd) -> Self {
        into_ref!(_t, irq);

        // Enable interrupt that signals temperature values
        irq.disable();
        irq.set_handler(|_| {
            let t = Self::regs();
            t.intenclr.write(|w| w.datardy().clear());
            WAKER.wake();
        });
        irq.enable();
        Self { _irq: irq }
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
