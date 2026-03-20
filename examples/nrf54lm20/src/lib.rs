#![no_std]

use embassy_nrf::config::{ClockSpeed, Config as NrfConfig, HfclkSource};
use embassy_nrf::usb::{self, Driver};
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_nrf::{bind_interrupts, peripherals, Peri};

bind_interrupts!(pub struct Irqs {
    USBHS => usb::InterruptHandler<peripherals::USBHS>;
    VREGUSB => usb::vbus_detect::InterruptHandler;
});

pub type UsbDriver<'d> = Driver<'d, HardwareVbusDetect>;

pub fn board_config() -> NrfConfig {
    let mut config = NrfConfig::default();
    config.hfclk_source = HfclkSource::ExternalXtal;
    config.clock_speed = ClockSpeed::CK64;
    config
}

pub fn init_board() -> embassy_nrf::Peripherals {
    embassy_nrf::init(board_config())
}

pub fn usb_driver(usb: Peri<'static, peripherals::USBHS>, ep_out_buffer: &'static mut [u8]) -> UsbDriver<'static> {
    Driver::new(
        usb,
        Irqs,
        HardwareVbusDetect::new(Irqs),
        ep_out_buffer,
        usb::Config::default(),
    )
}
