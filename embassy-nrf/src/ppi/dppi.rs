use super::{Channel, Event, Ppi, Task};

const DPPI_ENABLE_BIT: u32 = 0x8000_0000;
const DPPI_CHANNEL_MASK: u32 = 0x0000_00FF;

impl<'d, C: Channel + 'd, const EVENT_COUNT: usize, const TASK_COUNT: usize>
    Ppi<'d, C, EVENT_COUNT, TASK_COUNT>
{
    pub(super) fn enable_task(task: &Task, channel: &C, _index: usize) {
        unsafe {
            if task.subscribe_reg().read_volatile() != 0 {
                panic!("Task is already in use");
            }
            task.subscribe_reg()
                .write_volatile(DPPI_ENABLE_BIT | (channel.number() as u32 & DPPI_CHANNEL_MASK));
        }
    }

    pub(super) fn disable_task(task: &Task, _channel: &C, _index: usize) {
        unsafe {
            task.subscribe_reg().write_volatile(0);
        }
    }

    pub(super) fn enable_event(event: &Event, channel: &C, _index: usize) {
        unsafe {
            if event.publish_reg().read_volatile() != 0 {
                panic!("Task is already in use");
            }
            event
                .publish_reg()
                .write_volatile(DPPI_ENABLE_BIT | (channel.number() as u32 & DPPI_CHANNEL_MASK));
        }
    }

    pub(super) fn disable_event(event: &Event, _channel: &C, _index: usize) {
        unsafe {
            event.publish_reg().write_volatile(0);
        }
    }

    /// Enables all tasks and events
    pub(super) fn enable_all(tasks: &[Task], events: &[Event], channel: &C) {
        for (index, task) in tasks.iter().enumerate() {
            Self::enable_task(task, channel, index);
        }
        for (index, event) in events.iter().enumerate() {
            Self::enable_event(event, channel, index);
        }
    }

    /// Disable all tasks and events
    pub(super) fn disable_all(&self) {
        for (index, task) in self.tasks.iter().enumerate() {
            Self::disable_task(task, &self.ch, index);
        }
        for (index, event) in self.events.iter().enumerate() {
            Self::disable_event(event, &self.ch, index);
        }
    }
}
