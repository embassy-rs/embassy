//! Async touch sampling task — queues samples for the UI task via Embassy channel.
//!
//! A `Watch` was lossy: the UI task often read a later idle sample `(799,0)` while
//! the touch task had already logged `touch down (0,23)`. A bounded channel keeps
//! press/release ordering intact.

use defmt::info;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};

use crate::oxivgl::indev::TouchSample;
use crate::oxivgl::touch_dbg;
use crate::rvt50_board;

/// Latest raw touch sample from the board (includes I2C metadata for debug).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TouchBoardSample {
    pub x: i32,
    pub y: i32,
    pub pressed: bool,
    pub i2c_ok: bool,
    pub raw_status: u8,
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

/// Touch sample cadence.
const TOUCH_POLL_MS: u64 = 5;
/// Queue depth — short taps must survive until the UI task drains.
const TOUCH_QUEUE_DEPTH: usize = 16;

static TOUCH_CHAN: Channel<CriticalSectionRawMutex, TouchBoardSample, TOUCH_QUEUE_DEPTH> = Channel::new();

/// Receiver for the UI task (sole LVGL owner).
pub fn receiver(
) -> embassy_sync::channel::Receiver<'static, CriticalSectionRawMutex, TouchBoardSample, TOUCH_QUEUE_DEPTH> {
    TOUCH_CHAN.receiver()
}

/// Embassy task: poll the capacitive panel over I2C and enqueue every sample.
#[embassy_executor::task]
pub async fn run_touch_poll_task(
    mut i2c: embassy_stm32::i2c::I2c<'static, embassy_stm32::mode::Blocking, embassy_stm32::i2c::Master>,
) -> ! {
    let sender = TOUCH_CHAN.sender();
    let mut was_pressed = false;
    let mut last_x = 0i32;
    let mut last_y = 0i32;

    loop {
        let t = rvt50_board::read_touch(&mut i2c);

        if t.pressed {
            last_x = t.x as i32;
            last_y = t.y as i32;
        }

        let sample = TouchBoardSample {
            x: last_x,
            y: last_y,
            pressed: t.pressed,
            i2c_ok: t.i2c_ok,
            raw_status: t.raw_status,
        };

        touch_dbg::publish_touch(
            sample.x,
            sample.y,
            sample.pressed,
            sample.i2c_ok,
            sample.raw_status,
        );

        if t.pressed && !was_pressed {
            info!(
                "oxivgl touch down x={} y={} raw=0x{:02x}",
                sample.x, sample.y, t.raw_status
            );
        } else if !t.pressed && was_pressed {
            info!("oxivgl touch up x={} y={}", sample.x, sample.y);
        }
        was_pressed = t.pressed;

        sender.send(sample).await;
        Timer::after(Duration::from_millis(TOUCH_POLL_MS)).await;
    }
}
