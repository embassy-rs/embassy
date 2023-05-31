#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::mem;

use defmt::{info, panic, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_nrf::usb::Driver;
use embassy_nrf::{bind_interrupts, pac, peripherals, usb};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config, UsbDevice};
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<peripherals::USBD>;
    POWER_CLOCK => usb::vbus_detect::InterruptHandler;
});

type MyDriver = Driver<'static, peripherals::USBD, HardwareVbusDetect>;

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, MyDriver>) {
    device.run().await;
}

#[embassy_executor::task]
async fn echo_task(mut class: CdcAcmClass<'static, MyDriver>) {
    loop {
        class.wait_connection().await;
        info!("Connected");
        let _ = echo(&mut class).await;
        info!("Disconnected");
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let clock: pac::CLOCK = unsafe { mem::transmute(()) };

    info!("Enabling ext hfosc...");
    clock.tasks_hfclkstart.write(|w| unsafe { w.bits(1) });
    while clock.events_hfclkstarted.read().bits() != 1 {}

    // Create the driver, from the HAL.
    let driver = Driver::new(p.USBD, Irqs, HardwareVbusDetect::new(Irqs));

    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-serial example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    let state = make_static!(State::new());

    // Create embassy-usb DeviceBuilder using the driver and config.
    let mut builder = Builder::new(
        driver,
        config,
        &mut make_static!([0; 256])[..],
        &mut make_static!([0; 256])[..],
        &mut make_static!([0; 256])[..],
        &mut make_static!([0; 128])[..],
        &mut make_static!([0; 128])[..],
    );

    // Create classes on the builder.
    let class = CdcAcmClass::new(&mut builder, state, 64);

    // Build the builder.
    let usb = builder.build();

    unwrap!(spawner.spawn(usb_task(usb)));
    unwrap!(spawner.spawn(echo_task(class)));
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo(class: &mut CdcAcmClass<'static, MyDriver>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
    }
}
