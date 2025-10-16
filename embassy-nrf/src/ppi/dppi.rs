use super::{Channel, ConfigurableChannel, Event, Ppi, Task};
use crate::{Peri, pac};

const DPPI_ENABLE_BIT: u32 = 0x8000_0000;
const DPPI_CHANNEL_MASK: u32 = 0x0000_00FF;

#[derive(Copy, Clone)]
/// DPPI instance
pub enum DppiInstance {
    /// DPPI_00 instance
    Dppi00,
    /// DPPI_20 instance
    #[cfg(feature = "_nrf54l")]
    Dppi20,
    /// DPPI_30 instance
    #[cfg(feature = "_nrf54l")]
    Dppi30,
}

impl From<pac::dppic::Dppic> for DppiInstance {
    fn from(dppic: pac::dppic::Dppic) -> Self {
        match dppic {
            pac::DPPIC00 => DppiInstance::Dppi00,
            #[cfg(feature = "_nrf54l")]
            pac::DPPIC20 => DppiInstance::Dppi20,
            #[cfg(feature = "_nrf54l")]
            pac::DPPIC30 => DppiInstance::Dppi30,
            _ => panic!(),
        }
    }
}

#[cfg(feature = "_nrf54l")]
pub(crate) type PpiInstance = DppiInstance;

#[cfg(not(feature = "_nrf54l"))]
pub(crate) type PpiInstance = ();

pub(crate) fn regs(_inst: PpiInstance) -> pac::dppic::Dppic {
    #[cfg(not(feature = "_nrf54l"))]
    {
        pac::DPPIC
    }

    #[cfg(feature = "_nrf54l")]
    match _inst {
        DppiInstance::Dppi00 => pac::DPPIC00,
        DppiInstance::Dppi20 => pac::DPPIC20,
        DppiInstance::Dppi30 => pac::DPPIC30,
    }
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
        let inst = self.ch.inst();
        regs(inst).chenset().write(|w| w.0 = 1 << n);
    }

    /// Disables the channel.
    pub fn disable(&mut self) {
        let n = self.ch.number();
        let inst = self.ch.inst();
        regs(inst).chenclr().write(|w| w.0 = 1 << n);
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
