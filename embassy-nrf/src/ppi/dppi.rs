use core::marker::PhantomData;

use super::{ChannelNumber, ConfigurableChannel, Event, Ppi, PpiGroup, Task};
use crate::Peri;

const DPPI_ENABLE_BIT: u32 = 0x8000_0000;
const DPPI_CHANNEL_MASK: u32 = 0x0000_00FF;

#[cfg(not(feature = "_nrf54l"))]
pub(crate) fn regs() -> crate::pac::dppic::Dppic {
    crate::pac::DPPIC
}

/// Power domain of a peripheral register, from its address.
///
/// On nRF54L, address bits [23:18] are the APB bus number, which is the power domain. DPPI
/// channels are local to a power domain: a channel can only connect events and tasks of
/// peripherals in the same domain as its DPPIC. Use [`ppib`](crate::ppib) to connect domains.
#[cfg(feature = "_nrf54l")]
fn domain(addr: *const ()) -> u32 {
    (addr as u32 >> 18) & 0x3F
}

impl<'d> Ppi<'d, 1, 1> {
    /// Configure PPI channel to trigger `task` on `event`.
    pub fn new_one_to_one(ch: Peri<'d, impl ConfigurableChannel>, event: Event<'d>, task: Task<'d>) -> Self {
        Self::new_one_to_one_inner(ChannelNumber::new(ch), event, task)
    }

    pub(crate) fn new_one_to_one_inner(ch: ChannelNumber, event: Event<'d>, task: Task<'d>) -> Self {
        Ppi::new_many_to_many_inner(ch, [event], [task])
    }
}

impl<'d> Ppi<'d, 1, 2> {
    /// Configure PPI channel to trigger both `task1` and `task2` on `event`.
    pub fn new_one_to_two(
        ch: Peri<'d, impl ConfigurableChannel>,
        event: Event<'d>,
        task1: Task<'d>,
        task2: Task<'d>,
    ) -> Self {
        Self::new_one_to_two_inner(ChannelNumber::new(ch), event, task1, task2)
    }

    pub(crate) fn new_one_to_two_inner(ch: ChannelNumber, event: Event<'d>, task1: Task<'d>, task2: Task<'d>) -> Self {
        Ppi::new_many_to_many_inner(ch, [event], [task1, task2])
    }
}

impl<'d, const EVENT_COUNT: usize, const TASK_COUNT: usize> Ppi<'d, EVENT_COUNT, TASK_COUNT> {
    /// Configure a DPPI channel to trigger all `tasks` when any of the `events` fires.
    pub fn new_many_to_many(
        ch: Peri<'d, impl ConfigurableChannel>,
        events: [Event<'d>; EVENT_COUNT],
        tasks: [Task<'d>; TASK_COUNT],
    ) -> Self {
        Self::new_many_to_many_inner(ChannelNumber::new(ch), events, tasks)
    }

    fn new_many_to_many_inner(
        ch: ChannelNumber,
        events: [Event<'d>; EVENT_COUNT],
        tasks: [Task<'d>; TASK_COUNT],
    ) -> Self {
        #[cfg(feature = "_nrf54l")]
        {
            let d = domain(ch.regs().as_ptr() as _);
            for task in &tasks {
                assert!(
                    domain(task.0.as_ptr() as _) == d,
                    "PPI task is not in the channel's power domain. Use a channel from the task's domain, or connect the domains with a ppib::Ppib."
                );
            }
            for event in &events {
                assert!(
                    domain(event.0.as_ptr() as _) == d,
                    "PPI event is not in the channel's power domain. Use a channel from the event's domain, or connect the domains with a ppib::Ppib."
                );
            }
        }

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

        Self {
            ch,
            events,
            tasks,
            _phantom: PhantomData,
        }
    }
}

impl<'d, const EVENT_COUNT: usize, const TASK_COUNT: usize> Ppi<'d, EVENT_COUNT, TASK_COUNT> {
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

impl<const EVENT_COUNT: usize, const TASK_COUNT: usize> Ppi<'static, EVENT_COUNT, TASK_COUNT> {
    /// Persist the channel's configuration for the rest of the program's lifetime. This method
    /// should be preferred over [`core::mem::forget()`] because the `'static` bound prevents
    /// accidental reuse of the underlying peripheral.
    pub fn persist(self) {
        core::mem::forget(self);
    }
}

impl<'d, const EVENT_COUNT: usize, const TASK_COUNT: usize> Drop for Ppi<'d, EVENT_COUNT, TASK_COUNT> {
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

impl<'d> PpiGroup<'d> {
    /// Add a PPI channel to this group.
    ///
    /// If the channel is already in the group, this is a no-op.
    pub fn add_channel<const EVENT_COUNT: usize, const TASK_COUNT: usize>(
        &mut self,
        ch: &Ppi<'_, EVENT_COUNT, TASK_COUNT>,
    ) {
        let r = self.g.regs();
        let ng = self.g.number();
        let nc = ch.ch.number();
        #[cfg(feature = "_nrf54l")]
        assert!(
            r.as_ptr() == ch.ch.regs().as_ptr(),
            "PPI group and channel are in different power domains"
        );

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
    pub fn remove_channel<const EVENT_COUNT: usize, const TASK_COUNT: usize>(
        &mut self,
        ch: &Ppi<'_, EVENT_COUNT, TASK_COUNT>,
    ) {
        let r = self.g.regs();
        let ng = self.g.number();
        let nc = ch.ch.number();
        #[cfg(feature = "_nrf54l")]
        assert!(
            r.as_ptr() == ch.ch.regs().as_ptr(),
            "PPI group and channel are in different power domains"
        );

        let en = r.subscribe_chg(ng).en().read();
        let dis = r.subscribe_chg(ng).dis().read();
        r.subscribe_chg(ng).en().write_value(Default::default());
        r.subscribe_chg(ng).dis().write_value(Default::default());

        r.chg(ng).modify(|w| w.set_ch(nc, false));

        r.subscribe_chg(ng).en().write_value(en);
        r.subscribe_chg(ng).dis().write_value(dis);
    }
}
