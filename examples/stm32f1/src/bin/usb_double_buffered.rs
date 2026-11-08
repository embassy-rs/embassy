//! Raw USB bulk echo using double-buffered bulk endpoints.
//!
//! Blue Pill: 8 MHz HSE and external D+ pull-up, as in usb_serial.rs.
//! Claim vendor interface 0 and use OUT 0x01 / IN 0x82 (64-byte packets).
//! A double-buffered endpoint uses both directions of its hardware endpoint,
//! so IN and OUT must use different endpoint numbers.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_time::Timer;
use embassy_usb::Builder;
use embassy_usb::driver::{Direction, Endpoint, EndpointAddress, EndpointIn, EndpointOut};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB_LP_CAN1_RX0 => usb::InterruptHandler<peripherals::USB>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::Div1,
            mul: PllMul::Mul9,
        });
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.ahb_pre = AHBPrescaler::Div1;
        config.rcc.apb1_pre = APBPrescaler::Div2;
        config.rcc.apb2_pre = APBPrescaler::Div1;
    }
    let mut p = embassy_stm32::init(config);
    {
        let _dp = Output::new(p.PA12.reborrow(), Level::Low, Speed::Low);
        Timer::after_millis(10).await;
    }
    let driver = usb::Driver::new(p.USB, p.PA12, p.PA11, Irqs);
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("STM32F103 double-buffered bulk echo");
    config.serial_number = Some("12345678");
    config.max_packet_size_0 = 64;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    // Large enough for the UTF-16 product string descriptor (72 bytes).
    let mut control_buf = [0; 128];
    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );
    let mut function = builder.function(0xff, 0, 0);
    let mut interface = function.interface();
    let mut alt = interface.alt_setting(0xff, 0, 0, None);
    let mut out = alt.endpoint_bulk_out_double_buffered(Some(EndpointAddress::from_parts(1, Direction::Out)), 64);
    let mut inp = alt.endpoint_bulk_in_double_buffered(Some(EndpointAddress::from_parts(2, Direction::In)), 64);
    drop(function);
    let mut usb = builder.build();
    let echo = async {
        loop {
            inp.wait_enabled().await;
            out.wait_enabled().await;
            loop {
                let mut packet = [0; 64];
                let Ok(len) = out.read(&mut packet).await else {
                    break;
                };
                if inp.write(&packet[..len]).await.is_err() {
                    break;
                }
            }
        }
    };
    join(usb.run(), echo).await;
}
