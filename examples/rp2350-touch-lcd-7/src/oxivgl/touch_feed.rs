//! Async GT911 sampling — INT wake, I2C poll, channel queue (same pattern as RVT50).

use crate::board::BoardI2c;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};

use crate::board;
use crate::oxivgl::indev::TouchSample;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TouchBoardSample {
    pub x: i32,
    pub y: i32,
    pub pressed: bool,
}

impl From<TouchBoardSample> for TouchSample {
    fn from(s: TouchBoardSample) -> Self {
        Self {
            x: s.x,
            y: s.y,
            pressed: s.pressed,
        }
    }
}

const TOUCH_ACTIVE_POLL_MS: u64 = 5;
const TOUCH_RELEASE_CONFIRM_READS: u8 = 8;
const TOUCH_SETTLE_READS: u8 = 10;
const TOUCH_QUEUE_DEPTH: usize = 16;

static TOUCH_CHAN: Channel<CriticalSectionRawMutex, TouchBoardSample, TOUCH_QUEUE_DEPTH> = Channel::new();

pub fn receiver()
-> embassy_sync::channel::Receiver<'static, CriticalSectionRawMutex, TouchBoardSample, TOUCH_QUEUE_DEPTH> {
    TOUCH_CHAN.receiver()
}

#[embassy_executor::task]
pub async fn run_touch_int_task(
    mut i2c: BoardI2c,
    mut touch_int: embassy_rp::gpio::Flex<'static>,
) -> ! {
    let sender = TOUCH_CHAN.sender();
    let mut last_x = 0i32;
    let mut last_y = 0i32;

    loop {
        if touch_int.is_high() {
            touch_int.wait_for_falling_edge().await;
        } else {
            Timer::after(Duration::from_millis(TOUCH_ACTIVE_POLL_MS)).await;
        }

        let mut was_pressed = false;
        let mut settle_reads_left = TOUCH_SETTLE_READS;
        let mut release_reads = 0u8;

        loop {
            let t = board::read_touch(&mut i2c);

            if !t.i2c_ok {
                if was_pressed {
                    Timer::after(Duration::from_millis(TOUCH_ACTIVE_POLL_MS)).await;
                    continue;
                }
                settle_reads_left = settle_reads_left.saturating_sub(1);
                if settle_reads_left == 0 {
                    break;
                }
                Timer::after(Duration::from_millis(TOUCH_ACTIVE_POLL_MS)).await;
                continue;
            }

            if t.pressed {
                last_x = t.x as i32;
                last_y = t.y as i32;
            }

            let sample = TouchBoardSample {
                x: last_x,
                y: last_y,
                pressed: t.pressed,
            };

            if t.pressed {
                was_pressed = true;
                release_reads = 0;
                sender.send(sample).await;
            } else if was_pressed {
                release_reads += 1;
                if release_reads >= TOUCH_RELEASE_CONFIRM_READS {
                    sender.send(sample).await;
                    break;
                }
            } else {
                settle_reads_left -= 1;
                if settle_reads_left == 0 {
                    break;
                }
            }

            Timer::after(Duration::from_millis(TOUCH_ACTIVE_POLL_MS)).await;
        }
    }
}
