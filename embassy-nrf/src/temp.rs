//! Temperature sensor interface.

use crate::interrupt;
use crate::pac;
use crate::peripherals::TEMP;

use core::future::Future;
use core::marker::PhantomData;
use embassy::channel::signal::Signal;
use embassy::interrupt::InterruptExt;
use embassy::util::Unborrow;
use embassy_hal_common::{drop::OnDrop, unborrow};
use fixed::types::I30F2;

/// Integrated temperature sensor.
pub struct Temp<'d> {
    _temp: PhantomData<&'d TEMP>,
    _irq: interrupt::TEMP,
}

static IRQ: Signal<I30F2> = Signal::new();

impl<'d> Temp<'d> {
    pub fn new(
        _t: impl Unborrow<Target = TEMP> + 'd,
        irq: impl Unborrow<Target = interrupt::TEMP> + 'd,
    ) -> Self {
        unborrow!(_t, irq);

        let t = Self::regs();

        // Enable interrupt that signals temperature values
        t.intenset.write(|w| w.datardy().set());
        irq.disable();
        irq.set_handler(|_| {
            let t = Self::regs();
            t.events_datardy.reset();
            let raw = t.temp.read().bits();
            IRQ.signal(I30F2::from_bits(raw as i32));
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
    pub fn read(&mut self) -> impl Future<Output = I30F2> {
        // In case the future is dropped, stop the task and reset events.
        let on_drop = OnDrop::new(|| {
            let t = Self::regs();
            unsafe {
                t.tasks_stop.write(|w| w.bits(1));
            }
            t.events_datardy.reset();
        });

        let t = Self::regs();
        // Empty signal channel and start measurement.
        IRQ.reset();
        unsafe { t.tasks_start.write(|w| w.bits(1)) };

        async move {
            let value = IRQ.wait().await;
            on_drop.defuse();
            value
        }
    }

    fn regs() -> &'static pac::temp::RegisterBlock {
        unsafe { &*pac::TEMP::ptr() }
    }
}
