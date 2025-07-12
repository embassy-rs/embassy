use super::{Channel, ConfigurableChannel, Event, Ppi, Task};
use crate::{pac, Peri};

impl<'d> Task<'d> {
    fn reg_val(&self) -> u32 {
        self.0.as_ptr() as _
    }
}
impl<'d> Event<'d> {
    fn reg_val(&self) -> u32 {
        self.0.as_ptr() as _
    }
}

pub(crate) fn regs() -> pac::ppi::Ppi {
    pac::PPI
}

#[cfg(not(feature = "_nrf51"))] // Not for nrf51 because of the fork task
impl<'d, C: super::StaticChannel> Ppi<'d, C, 0, 1> {
    /// Configure PPI channel to trigger `task`.
    pub fn new_zero_to_one(ch: Peri<'d, C>, task: Task) -> Self {
        let r = regs();
        let n = ch.number();
        r.fork(n).tep().write_value(task.reg_val());

        Self { ch }
    }
}

impl<'d, C: ConfigurableChannel> Ppi<'d, C, 1, 1> {
    /// Configure PPI channel to trigger `task` on `event`.
    pub fn new_one_to_one(ch: Peri<'d, C>, event: Event<'d>, task: Task<'d>) -> Self {
        let r = regs();
        let n = ch.number();
        r.ch(n).eep().write_value(event.reg_val());
        r.ch(n).tep().write_value(task.reg_val());

        Self { ch }
    }
}

#[cfg(not(feature = "_nrf51"))] // Not for nrf51 because of the fork task
impl<'d, C: ConfigurableChannel> Ppi<'d, C, 1, 2> {
    /// Configure PPI channel to trigger both `task1` and `task2` on `event`.
    pub fn new_one_to_two(ch: Peri<'d, C>, event: Event<'d>, task1: Task<'d>, task2: Task<'d>) -> Self {
        let r = regs();
        let n = ch.number();
        r.ch(n).eep().write_value(event.reg_val());
        r.ch(n).tep().write_value(task1.reg_val());
        r.fork(n).tep().write_value(task2.reg_val());

        Self { ch }
    }
}

impl<'d, C: Channel, const EVENT_COUNT: usize, const TASK_COUNT: usize> Ppi<'d, C, EVENT_COUNT, TASK_COUNT> {
    /// Enables the channel.
    pub fn enable(&mut self) {
        let n = self.ch.number();
        regs().chenset().write(|w| w.set_ch(n, true));
    }

    /// Disables the channel.
    pub fn disable(&mut self) {
        let n = self.ch.number();
        regs().chenclr().write(|w| w.set_ch(n, true));
    }
}

impl<'d, C: Channel, const EVENT_COUNT: usize, const TASK_COUNT: usize> Drop for Ppi<'d, C, EVENT_COUNT, TASK_COUNT> {
    fn drop(&mut self) {
        self.disable();

        let r = regs();
        let n = self.ch.number();
        r.ch(n).eep().write_value(0);
        r.ch(n).tep().write_value(0);
        #[cfg(not(feature = "_nrf51"))]
        r.fork(n).tep().write_value(0);
    }
}
