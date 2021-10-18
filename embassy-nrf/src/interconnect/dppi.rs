use super::{Channel, Event, Ppi, Task};

const DPPI_ENABLE_BIT: u32 = 0x8000_0000;
const DPPI_CHANNEL_MASK: u32 = 0x0000_00FF;
const REGISTER_DPPI_CONFIG_OFFSET: usize = 0x80 / core::mem::size_of::<u32>();

impl<'d, C: Channel, const EVENT_COUNT: usize, const TASK_COUNT: usize>
    Ppi<'d, C, EVENT_COUNT, TASK_COUNT>
{
    pub(super) fn enable_task(task: &Task, channel: &C, _index: usize) {
        unsafe {
            task.0
                .as_ptr()
                .add(REGISTER_DPPI_CONFIG_OFFSET)
                .write_volatile(DPPI_ENABLE_BIT | (channel.number() as u32 & DPPI_CHANNEL_MASK));
        }
    }

    pub(super) fn disable_task(task: &Task, _channel: &C, _index: usize) {
        unsafe {
            task.0
                .as_ptr()
                .add(REGISTER_DPPI_CONFIG_OFFSET)
                .write_volatile(0);
        }
    }

    pub(super) fn enable_event(event: &Event, channel: &C, _index: usize) {
        unsafe {
            event
                .0
                .as_ptr()
                .add(REGISTER_DPPI_CONFIG_OFFSET)
                .write_volatile(DPPI_ENABLE_BIT | (channel.number() as u32 & DPPI_CHANNEL_MASK));
        }
    }

    pub(super) fn disable_event(event: &Event, _channel: &C, _index: usize) {
        unsafe {
            event
                .0
                .as_ptr()
                .add(REGISTER_DPPI_CONFIG_OFFSET)
                .write_volatile(0);
        }
    }
}
