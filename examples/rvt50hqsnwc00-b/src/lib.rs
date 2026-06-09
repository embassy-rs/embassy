#![no_std]

pub mod can_bridge;
pub mod rvt50_board;
pub mod touch_can;
pub mod touch_config;

#[cfg(feature = "lvgl")]
pub mod lvgl;
