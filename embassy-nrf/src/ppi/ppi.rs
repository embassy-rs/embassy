use embassy_hal_common::into_ref;

use super::{Channel, ConfigurableChannel, Event, Ppi, StaticChannel, Task};
use crate::{pac, Peripheral};

impl Task {
    fn reg_val(&self) -> u32 {
        self.0.as_ptr() as _
    }
}
impl Event {
    fn reg_val(&self) -> u32 {
        self.0.as_ptr() as _
    }
}

pub(crate) fn regs() -> &'static pac::ppi::RegisterBlock {
    unsafe { &*pac::PPI::ptr() }
}

#[cfg(not(feature = "nrf51"))] // Not for nrf51 because of the fork task
impl<'d, C: StaticChannel> Ppi<'d, C, 0, 1> {
    /// Configure PPI channel to trigger `task`.
    pub fn new_zero_to_one(ch: impl Peripheral<P = C> + 'd, task: Task) -> Self {
        into_ref!(ch);

        let r = regs();
        let n = ch.number();
        r.fork[n].tep.write(|w| unsafe { w.bits(task.reg_val()) });

        Self { ch }
    }
}

impl<'d, C: ConfigurableChannel> Ppi<'d, C, 1, 1> {
    /// Configure PPI channel to trigger `task` on `event`.
    pub fn new_one_to_one(ch: impl Peripheral<P = C> + 'd, event: Event, task: Task) -> Self {
        into_ref!(ch);

        let r = regs();
        let n = ch.number();
        r.ch[n].eep.write(|w| unsafe { w.bits(event.reg_val()) });
        r.ch[n].tep.write(|w| unsafe { w.bits(task.reg_val()) });

        Self { ch }
    }
}

#[cfg(not(feature = "nrf51"))] // Not for nrf51 because of the fork task
impl<'d, C: ConfigurableChannel> Ppi<'d, C, 1, 2> {
    /// Configure PPI channel to trigger both `task1` and `task2` on `event`.
    pub fn new_one_to_two(ch: impl Peripheral<P = C> + 'd, event: Event, task1: Task, task2: Task) -> Self {
        into_ref!(ch);

        let r = regs();
        let n = ch.number();
        r.ch[n].eep.write(|w| unsafe { w.bits(event.reg_val()) });
        r.ch[n].tep.write(|w| unsafe { w.bits(task1.reg_val()) });
        r.fork[n].tep.write(|w| unsafe { w.bits(task2.reg_val()) });

        Self { ch }
    }
}

impl<'d, C: Channel, const EVENT_COUNT: usize, const TASK_COUNT: usize> Ppi<'d, C, EVENT_COUNT, TASK_COUNT> {
    /// Enables the channel.
    pub fn enable(&mut self) {
        let n = self.ch.number();
        regs().chenset.write(|w| unsafe { w.bits(1 << n) });
    }

    /// Disables the channel.
    pub fn disable(&mut self) {
        let n = self.ch.number();
        regs().chenclr.write(|w| unsafe { w.bits(1 << n) });
    }
}

impl<'d, C: Channel, const EVENT_COUNT: usize, const TASK_COUNT: usize> Drop for Ppi<'d, C, EVENT_COUNT, TASK_COUNT> {
    fn drop(&mut self) {
        self.disable();

        let r = regs();
        let n = self.ch.number();
        r.ch[n].eep.write(|w| unsafe { w.bits(0) });
        r.ch[n].tep.write(|w| unsafe { w.bits(0) });
        r.fork[n].tep.write(|w| unsafe { w.bits(0) });
    }
}
