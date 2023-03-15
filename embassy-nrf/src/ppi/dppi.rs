use embassy_hal_common::into_ref;

use super::{Channel, ConfigurableChannel, Event, Ppi, Task};
use crate::{pac, Peripheral};

const DPPI_ENABLE_BIT: u32 = 0x8000_0000;
const DPPI_CHANNEL_MASK: u32 = 0x0000_00FF;

pub(crate) fn regs() -> &'static pac::dppic::RegisterBlock {
    unsafe { &*pac::DPPIC::ptr() }
}

impl<'d, C: ConfigurableChannel> Ppi<'d, C, 1, 1> {
    /// Configure PPI channel to trigger `task` on `event`.
    pub fn new_one_to_one(ch: impl Peripheral<P = C> + 'd, event: Event, task: Task) -> Self {
        Ppi::new_many_to_many(ch, [event], [task])
    }
}

impl<'d, C: ConfigurableChannel> Ppi<'d, C, 1, 2> {
    /// Configure PPI channel to trigger both `task1` and `task2` on `event`.
    pub fn new_one_to_two(ch: impl Peripheral<P = C> + 'd, event: Event, task1: Task, task2: Task) -> Self {
        Ppi::new_many_to_many(ch, [event], [task1, task2])
    }
}

impl<'d, C: ConfigurableChannel, const EVENT_COUNT: usize, const TASK_COUNT: usize>
    Ppi<'d, C, EVENT_COUNT, TASK_COUNT>
{
    /// Configure a DPPI channel to trigger all `tasks` when any of the `events` fires.
    pub fn new_many_to_many(
        ch: impl Peripheral<P = C> + 'd,
        events: [Event; EVENT_COUNT],
        tasks: [Task; TASK_COUNT],
    ) -> Self {
        into_ref!(ch);

        let val = DPPI_ENABLE_BIT | (ch.number() as u32 & DPPI_CHANNEL_MASK);
        for task in tasks {
            if unsafe { task.subscribe_reg().read_volatile() } != 0 {
                panic!("Task is already in use");
            }
            unsafe { task.subscribe_reg().write_volatile(val) }
        }
        for event in events {
            if unsafe { event.publish_reg().read_volatile() } != 0 {
                panic!("Event is already in use");
            }
            unsafe { event.publish_reg().write_volatile(val) }
        }

        Self { ch, events, tasks }
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

        for task in self.tasks {
            unsafe { task.subscribe_reg().write_volatile(0) }
        }
        for event in self.events {
            unsafe { event.publish_reg().write_volatile(0) }
        }
    }
}
