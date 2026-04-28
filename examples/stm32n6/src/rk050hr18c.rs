//! RK050HR18C-B01 panel driver (5" 800x480 parallel RGB LCD on the STM32N6570-DK).
//!
//! Owns the three control GPIOs (NRST, ON/OFF, backlight) and exposes the panel
//! timings from the STM32CubeN6 BSP.

use embassy_stm32::Peri;
use embassy_stm32::gpio::{Level, Output, Pin, Speed};
use embassy_stm32::ltdc::{LtdcConfiguration, PolarityActive, PolarityEdge};
use embassy_time::Timer;

pub const WIDTH: u16 = 800;
pub const HEIGHT: u16 = 480;

pub const H_SYNC: u16 = 1;
pub const H_BACK_PORCH: u16 = 46;
pub const H_FRONT_PORCH: u16 = 210;
pub const V_SYNC: u16 = 1;
pub const V_BACK_PORCH: u16 = 23;
pub const V_FRONT_PORCH: u16 = 22;

/// Panel LTDC timing — matches the RK050HR18C-B01 datasheet & STM32CubeN6 BSP.
pub const LTDC_CONFIG: LtdcConfiguration = LtdcConfiguration {
    active_width: WIDTH,
    active_height: HEIGHT,
    h_back_porch: H_BACK_PORCH,
    h_front_porch: H_FRONT_PORCH,
    v_back_porch: V_BACK_PORCH,
    v_front_porch: V_FRONT_PORCH,
    h_sync: H_SYNC,
    v_sync: V_SYNC,
    h_sync_polarity: PolarityActive::ActiveLow,
    v_sync_polarity: PolarityActive::ActiveLow,
    data_enable_polarity: PolarityActive::ActiveLow,
    pixel_clock_polarity: PolarityEdge::RisingEdge,
};

pub struct Rk050Hr18c<'d> {
    reset: Output<'d>,
    enable: Output<'d>,
    backlight: Output<'d>,
}

impl<'d> Rk050Hr18c<'d> {
    pub fn new(reset: Peri<'d, impl Pin>, enable: Peri<'d, impl Pin>, backlight: Peri<'d, impl Pin>) -> Self {
        Self {
            reset: Output::new(reset, Level::Low, Speed::Low),
            enable: Output::new(enable, Level::Low, Speed::Low),
            backlight: Output::new(backlight, Level::Low, Speed::Low),
        }
    }

    /// Power the panel on: assert enable, pulse reset low for 10 ms, wait 120 ms for
    /// panel startup, then turn the backlight on.
    pub async fn power_on(&mut self) {
        self.enable.set_high();
        Timer::after_millis(10).await;
        self.reset.set_low();
        Timer::after_millis(10).await;
        self.reset.set_high();
        Timer::after_millis(120).await;
        self.backlight.set_high();
    }
}
