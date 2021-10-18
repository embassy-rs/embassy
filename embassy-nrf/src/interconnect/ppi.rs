use super::{Channel, Event, Ppi, Task};
use crate::pac;

impl<'d, C: Channel + 'd, const EVENT_COUNT: usize, const TASK_COUNT: usize>
    Ppi<'d, C, EVENT_COUNT, TASK_COUNT>
{
    pub(super) fn enable_task(task: &Task, channel: &C, index: usize) {
        match (index, channel.is_task_configurable()) {
            (0, false) => Self::set_fork_task(Some(task), channel.number()), // Static channel with fork
            (0, true) => Self::set_main_task(Some(task), channel.number()), // Configurable channel without fork
            (1, true) => Self::set_fork_task(Some(task), channel.number()), // Configurable channel with fork
            _ => unreachable!("{}, {}", index, channel.is_task_configurable()), // Not available with the PPI, so should not be constructable
        }
    }

    pub(super) fn disable_task(_task: &Task, channel: &C, index: usize) {
        match (index, channel.is_task_configurable()) {
            (0, false) => Self::set_fork_task(None, channel.number()), // Static channel with fork
            (0, true) => Self::set_main_task(None, channel.number()), // Configurable channel without fork
            (1, true) => Self::set_fork_task(None, channel.number()), // Configurable channel with fork
            _ => unreachable!(), // Not available with the PPI, so should not be constructable
        }
    }

    pub(super) fn enable_event(event: &Event, channel: &C, _index: usize) {
        Self::set_event(Some(event), channel.number())
    }

    pub(super) fn disable_event(_event: &Event, channel: &C, _index: usize) {
        Self::set_event(None, channel.number())
    }

    fn set_main_task(task: Option<&Task>, channel: usize) {
        let r = unsafe { &*pac::PPI::ptr() };
        if let Some(task) = task {
            r.ch[channel]
                .tep
                .write(|w| unsafe { w.bits(task.0.as_ptr() as u32) })
        } else {
            r.ch[channel].tep.write(|w| unsafe { w.bits(0) })
        }
    }

    fn set_fork_task(task: Option<&Task>, channel: usize) {
        let r = unsafe { &*pac::PPI::ptr() };
        if let Some(task) = task {
            r.fork[channel]
                .tep
                .write(|w| unsafe { w.bits(task.0.as_ptr() as u32) })
        } else {
            r.fork[channel].tep.write(|w| unsafe { w.bits(0) })
        }
    }

    fn set_event(event: Option<&Event>, channel: usize) {
        let r = unsafe { &*pac::PPI::ptr() };
        if let Some(event) = event {
            r.ch[channel]
                .eep
                .write(|w| unsafe { w.bits(event.0.as_ptr() as u32) })
        } else {
            r.ch[channel].eep.write(|w| unsafe { w.bits(0) })
        }
    }
}
