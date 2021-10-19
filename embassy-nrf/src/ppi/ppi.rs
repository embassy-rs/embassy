use super::{Channel, Event, Ppi, Task};
use crate::pac;

impl<'d, C: Channel + 'd, const EVENT_COUNT: usize, const TASK_COUNT: usize>
    Ppi<'d, C, EVENT_COUNT, TASK_COUNT>
{
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

    /// Enables all tasks and events
    pub(super) fn enable_all(tasks: &[Task], events: &[Event], channel: &C) {
        // One configurable task, no fork
        if channel.is_task_configurable() && TASK_COUNT == 1 {
            Self::set_main_task(Some(&tasks[0]), channel.number());
        }

        // One configurable task, as fork
        if !channel.is_task_configurable() && TASK_COUNT == 1 {
            Self::set_fork_task(Some(&tasks[0]), channel.number());
        }

        // Two configurable tasks (main + fork)
        if TASK_COUNT == 2 {
            Self::set_main_task(Some(&tasks[0]), channel.number());
            Self::set_fork_task(Some(&tasks[1]), channel.number());
        }

        if EVENT_COUNT == 1 {
            Self::set_event(Some(&events[0]), channel.number());
        }
    }

    /// Disable all tasks and events
    pub(super) fn disable_all(&self) {
        if self.ch.is_task_configurable() {
            Self::set_main_task(None, self.ch.number());
        }

        if TASK_COUNT == 1 && !self.ch.is_task_configurable() || TASK_COUNT == 2 {
            Self::set_fork_task(None, self.ch.number());
        }

        if EVENT_COUNT == 1 {
            Self::set_event(None, self.ch.number());
        }
    }
}
