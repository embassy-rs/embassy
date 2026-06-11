//! Async touch sampling task — queues samples for the UI task via Embassy channel.
//!
//! **Interrupt-driven:** the task sleeps on the `CTP_INT` EXTI line (PE6,
//! active-low) and only starts I2C traffic once the controller signals a touch
//! event. While a contact is active it keeps polling at a short cadence so
//! pointer moves and the final release are tracked even when the controller
//! does not emit a distinct INT edge for every state change (the failure mode
//! that broke the earlier select-on-INT approach in the UI loop).
//!
//! A `Watch` was lossy: the UI task often read a later idle sample `(799,0)` while
//! the touch task had already logged `touch down (0,23)`. A bounded channel keeps
//! press/release ordering intact. Idle samples are no longer queued at all — only
//! press samples and one final release sample per contact.

use defmt::{debug, info};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::mode::Async;
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

/// Poll cadence while a contact is active (pointer-move tracking).
const TOUCH_ACTIVE_POLL_MS: u64 = 5;
/// Reads tolerated after an INT edge before the wake-up counts as spurious.
/// Covers controllers whose status register lags the INT assertion.
const TOUCH_SETTLE_READS: u8 = 10;
/// Queue depth — short taps must survive until the UI task drains.
const TOUCH_QUEUE_DEPTH: usize = 16;

static TOUCH_CHAN: Channel<CriticalSectionRawMutex, TouchBoardSample, TOUCH_QUEUE_DEPTH> = Channel::new();

/// Receiver for the UI task (sole LVGL owner).
pub fn receiver()
-> embassy_sync::channel::Receiver<'static, CriticalSectionRawMutex, TouchBoardSample, TOUCH_QUEUE_DEPTH> {
    TOUCH_CHAN.receiver()
}

/// Embassy task: sleep on `CTP_INT`, then sample the panel over I2C while the
/// contact lasts and enqueue every sample (press moves + final release).
#[embassy_executor::task]
pub async fn run_touch_int_task(
    mut i2c: embassy_stm32::i2c::I2c<'static, embassy_stm32::mode::Blocking, embassy_stm32::i2c::Master>,
    mut touch_int: ExtiInput<'static, Async>,
) -> ! {
    let sender = TOUCH_CHAN.sender();
    let mut last_x = 0i32;
    let mut last_y = 0i32;

    info!("oxivgl touch task: interrupt-driven via CTP_INT");

    loop {
        // Idle: zero I2C traffic until the controller asserts CTP_INT.
        if touch_int.is_high() {
            touch_int.wait_for_falling_edge().await;
        } else {
            // Line still asserted from the previous pass (held INT or contact
            // already active) — throttle so a stuck line cannot spam the bus.
            Timer::after(Duration::from_millis(TOUCH_ACTIVE_POLL_MS)).await;
        }
        touch_dbg::bump_int_wakeups();
        debug!("oxivgl touch int wake int_low={}", touch_int.is_low());

        // Active: poll until the contact ends or the wake-up proves spurious.
        let mut was_pressed = false;
        let mut settle_reads_left = TOUCH_SETTLE_READS;

        loop {
            let t = rvt50_board::read_touch(&mut i2c);

            if t.pressed {
                last_x = t.x as i32;
                last_y = t.y as i32;
            }

            // Idle reads park at the panel edge; keep releases on the last
            // contact point so LVGL can finish click hit-testing.
            let sample = TouchBoardSample {
                x: last_x,
                y: last_y,
                pressed: t.pressed,
                i2c_ok: t.i2c_ok,
                raw_status: t.raw_status,
            };

            touch_dbg::publish_touch(sample.x, sample.y, sample.pressed, sample.i2c_ok, sample.raw_status);

            if t.pressed {
                if !was_pressed {
                    info!(
                        "oxivgl touch down x={} y={} raw=0x{:02x}",
                        sample.x, sample.y, t.raw_status
                    );
                }
                was_pressed = true;
                sender.send(sample).await;
            } else if was_pressed {
                // Contact ended: deliver exactly one release sample, then re-arm.
                info!("oxivgl touch up x={} y={}", sample.x, sample.y);
                sender.send(sample).await;
                break;
            } else {
                settle_reads_left -= 1;
                if settle_reads_left == 0 {
                    debug!(
                        "oxivgl touch int spurious i2c_ok={} raw=0x{:02x}",
                        t.i2c_ok, t.raw_status
                    );
                    break;
                }
            }

            Timer::after(Duration::from_millis(TOUCH_ACTIVE_POLL_MS)).await;
        }
    }
}
