#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#[path = "../common.rs"]
mod common;

use defmt::{info, unwrap};
use embassy_executor::Executor;
use embassy_executor::_export::StaticCell;
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_rp::bind_interrupts;
use embassy_rp::pac::usb::Usb;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_time::{Duration, Timer};
use embassy_usb::Builder;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
static CHANNEL0: Channel<CriticalSectionRawMutex, bool, 1> = Channel::new();
static CHANNEL1: Channel<CriticalSectionRawMutex, bool, 1> = Channel::new();

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());

    // Create the driver, from the HAL.
    let driver = Driver::new(p.USB, Irqs);

    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-serial logger");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 8;

    // Required for windows compatiblity.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    let device_descriptor = make_static!([0; 256]);
    let config_descriptor = make_static!([0; 256]);
    let bos_descriptor = make_static!([0; 256]);
    let control_buf = make_static!([0; 64]);
    let state = make_static!(State::new());

    let mut builder = Builder::new(
        driver,
        config,
        device_descriptor,
        config_descriptor,
        bos_descriptor,
        control_buf,
    );

    // Create classes on the builder.
    let class = make_static!(CdcAcmClass::new(&mut builder, state, 8));
    let usb = make_static!(builder.build());

    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 = EXECUTOR1.init(Executor::new());
        executor1.run(|spawner| {
            unwrap!(spawner.spawn(run_usb(usb)));
        });
    });
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| unwrap!(spawner.spawn(core0_task())));
}

#[embassy_executor::task]
async fn core0_task() {
    info!("CORE0 is running");
    let ping = true;
    CHANNEL0.send(ping).await;
    let pong = CHANNEL1.recv().await;
    assert_eq!(ping, pong);

    info!("Test OK");
    cortex_m::asm::bkpt();
}

#[embassy_executor::task]
pub async fn run_usb(usb: &'static mut embassy_usb::UsbDevice<'static, Driver<'static, USB>>
) {
    info!("CORE1 is running");
    let ping = CHANNEL0.recv().await;
    CHANNEL1.send(ping).await;
    usb.run().await;
}
