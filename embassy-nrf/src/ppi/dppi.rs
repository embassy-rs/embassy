use super::{Channel, ConfigurableChannel, Event, Group, Ppi, PpiGroup, Task};
use crate::Peri;

const DPPI_ENABLE_BIT: u32 = 0x8000_0000;
const DPPI_CHANNEL_MASK: u32 = 0x0000_00FF;

#[cfg(not(feature = "_nrf54l"))]
pub(crate) fn regs() -> crate::pac::dppic::Dppic {
    crate::pac::DPPIC
}

impl<'d, C: ConfigurableChannel> Ppi<'d, C, 1, 1> {
    /// Configure PPI channel to trigger `task` on `event`.
    pub fn new_one_to_one(ch: Peri<'d, C>, event: Event<'d>, task: Task<'d>) -> Self {
        Ppi::new_many_to_many(ch, [event], [task])
    }
}

impl<'d, C: ConfigurableChannel> Ppi<'d, C, 1, 2> {
    /// Configure PPI channel to trigger both `task1` and `task2` on `event`.
    pub fn new_one_to_two(ch: Peri<'d, C>, event: Event<'d>, task1: Task<'d>, task2: Task<'d>) -> Self {
        Ppi::new_many_to_many(ch, [event], [task1, task2])
    }
}

impl<'d, C: ConfigurableChannel, const EVENT_COUNT: usize, const TASK_COUNT: usize>
    Ppi<'d, C, EVENT_COUNT, TASK_COUNT>
{
    /// Configure a DPPI channel to trigger all `tasks` when any of the `events` fires.
    pub fn new_many_to_many(ch: Peri<'d, C>, events: [Event<'d>; EVENT_COUNT], tasks: [Task<'d>; TASK_COUNT]) -> Self {
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
        self.ch.regs().chenset().write(|w| w.0 = 1 << n);
    }

    /// Disables the channel.
    pub fn disable(&mut self) {
        let n = self.ch.number();
        self.ch.regs().chenclr().write(|w| w.0 = 1 << n);
    }
}

impl<C: Channel, const EVENT_COUNT: usize, const TASK_COUNT: usize> Ppi<'static, C, EVENT_COUNT, TASK_COUNT> {
    /// Persist the channel's configuration for the rest of the program's lifetime. This method
    /// should be preferred over [`core::mem::forget()`] because the `'static` bound prevents
    /// accidental reuse of the underlying peripheral.
    pub fn persist(self) {
        core::mem::forget(self);
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

impl<'d, G: Group> PpiGroup<'d, G> {
    /// Add a PPI channel to this group.
    ///
    /// If the channel is already in the group, this is a no-op.
    pub fn add_channel<C: Channel, const EVENT_COUNT: usize, const TASK_COUNT: usize>(
        &mut self,
        ch: &Ppi<'_, C, EVENT_COUNT, TASK_COUNT>,
    ) {
        let r = self.g.regs();
        let ng = self.g.number();
        let nc = ch.ch.number();

        let en = r.subscribe_chg(ng).en().read();
        let dis = r.subscribe_chg(ng).dis().read();
        r.subscribe_chg(ng).en().write_value(Default::default());
        r.subscribe_chg(ng).dis().write_value(Default::default());

        r.chg(ng).modify(|w| w.set_ch(nc, true));

        r.subscribe_chg(ng).en().write_value(en);
        r.subscribe_chg(ng).dis().write_value(dis);
    }

    /// Remove a PPI channel from this group.
    ///
    /// If the channel is already not in the group, this is a no-op.
    pub fn remove_channel<C: Channel, const EVENT_COUNT: usize, const TASK_COUNT: usize>(
        &mut self,
        ch: &Ppi<'_, C, EVENT_COUNT, TASK_COUNT>,
    ) {
        let r = self.g.regs();
        let ng = self.g.number();
        let nc = ch.ch.number();

        let en = r.subscribe_chg(ng).en().read();
        let dis = r.subscribe_chg(ng).dis().read();
        r.subscribe_chg(ng).en().write_value(Default::default());
        r.subscribe_chg(ng).dis().write_value(Default::default());

        r.chg(ng).modify(|w| w.set_ch(nc, false));

        r.subscribe_chg(ng).en().write_value(en);
        r.subscribe_chg(ng).dis().write_value(dis);
    }
}
