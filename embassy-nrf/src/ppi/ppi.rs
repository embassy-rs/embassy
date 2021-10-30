use core::marker::PhantomData;

use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

use super::{Channel, ConfigurableChannel, Event, Ppi, StaticChannel, Task};
use crate::pac;

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

#[cfg(not(feature = "nrf51"))] // Not for nrf51 because of the fork task
impl<'d, C: StaticChannel> Ppi<'d, C, 0, 1> {
    pub fn new_zero_to_one(ch: impl Unborrow<Target = C> + 'd, task: Task) -> Self {
        unborrow!(ch);

        let r = unsafe { &*pac::PPI::ptr() };
        let n = ch.number();
        r.fork[n].tep.write(|w| unsafe { w.bits(task.reg_val()) });

        Self {
            ch,
            phantom: PhantomData,
        }
    }
}

impl<'d, C: ConfigurableChannel> Ppi<'d, C, 1, 1> {
    pub fn new_one_to_one(ch: impl Unborrow<Target = C> + 'd, event: Event, task: Task) -> Self {
        unborrow!(ch);

        let r = unsafe { &*pac::PPI::ptr() };
        let n = ch.number();
        r.ch[n].eep.write(|w| unsafe { w.bits(event.reg_val()) });
        r.ch[n].tep.write(|w| unsafe { w.bits(task.reg_val()) });

        Self {
            ch,
            phantom: PhantomData,
        }
    }
}

#[cfg(not(feature = "nrf51"))] // Not for nrf51 because of the fork task
impl<'d, C: ConfigurableChannel> Ppi<'d, C, 1, 2> {
    pub fn new_one_to_two(
        ch: impl Unborrow<Target = C> + 'd,
        event: Event,
        task1: Task,
        task2: Task,
    ) -> Self {
        unborrow!(ch);

        let r = unsafe { &*pac::PPI::ptr() };
        let n = ch.number();
        r.ch[n].eep.write(|w| unsafe { w.bits(event.reg_val()) });
        r.ch[n].tep.write(|w| unsafe { w.bits(task1.reg_val()) });
        r.fork[n].tep.write(|w| unsafe { w.bits(task2.reg_val()) });

        Self {
            ch,
            phantom: PhantomData,
        }
    }
}

impl<'d, C: Channel, const EVENT_COUNT: usize, const TASK_COUNT: usize> Drop
    for Ppi<'d, C, EVENT_COUNT, TASK_COUNT>
{
    fn drop(&mut self) {
        self.disable();

        let r = unsafe { &*pac::PPI::ptr() };
        let n = self.ch.number();
        r.ch[n].eep.write(|w| unsafe { w.bits(0) });
        r.ch[n].tep.write(|w| unsafe { w.bits(0) });
        r.fork[n].tep.write(|w| unsafe { w.bits(0) });
    }
}
