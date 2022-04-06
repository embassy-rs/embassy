#![no_std]
#![no_main]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use core::mem;
use defmt::{info, panic, unwrap};
use embassy::executor::Spawner;
use embassy::util::Forever;
use embassy_nrf::pac;
use embassy_nrf::usb::Driver;
use embassy_nrf::Peripherals;
use embassy_nrf::{interrupt, peripherals};
use embassy_usb::driver::{ReadError, WriteError};
use embassy_usb::{Config, UsbDevice, UsbDeviceBuilder};
use embassy_usb_serial::{CdcAcmClass, State};

use defmt_rtt as _; // global logger
use panic_probe as _;

type MyDriver = Driver<'static, peripherals::USBD>;

#[embassy::task]
async fn usb_task(mut device: UsbDevice<'static, MyDriver>) {
    device.run().await;
}

#[embassy::task]
async fn echo_task(mut class: CdcAcmClass<'static, MyDriver>) {
    loop {
        class.wait_connection().await;
        info!("Connected");
        let _ = echo(&mut class).await;
        info!("Disconnected");
    }
}

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    let clock: pac::CLOCK = unsafe { mem::transmute(()) };
    let power: pac::POWER = unsafe { mem::transmute(()) };

    info!("Enabling ext hfosc...");
    clock.tasks_hfclkstart.write(|w| unsafe { w.bits(1) });
    while clock.events_hfclkstarted.read().bits() != 1 {}

    info!("Waiting for vbus...");
    while !power.usbregstatus.read().vbusdetect().is_vbus_present() {}
    info!("vbus OK");

    // Create the driver, from the HAL.
    let irq = interrupt::take!(USBD);
    let driver = Driver::new(p.USBD, irq);

    // Create embassy-usb Config
    let config = Config::new(0xc0de, 0xcafe);

    struct Resources {
        device_descriptor: [u8; 256],
        config_descriptor: [u8; 256],
        bos_descriptor: [u8; 256],
        control_buf: [u8; 7],
        serial_state: State<'static>,
    }
    static RESOURCES: Forever<Resources> = Forever::new();
    let res = RESOURCES.put(Resources {
        device_descriptor: [0; 256],
        config_descriptor: [0; 256],
        bos_descriptor: [0; 256],
        control_buf: [0; 7],
        serial_state: State::new(),
    });

    // Create embassy-usb DeviceBuilder using the driver and config.
    let mut builder = UsbDeviceBuilder::new(
        driver,
        config,
        &mut res.device_descriptor,
        &mut res.config_descriptor,
        &mut res.bos_descriptor,
        &mut res.control_buf,
    );

    // Create classes on the builder.
    let class = CdcAcmClass::new(&mut builder, &mut res.serial_state, 64);

    // Build the builder.
    let usb = builder.build();

    unwrap!(spawner.spawn(usb_task(usb)));
    unwrap!(spawner.spawn(echo_task(class)));
}

struct Disconnected {}

impl From<ReadError> for Disconnected {
    fn from(val: ReadError) -> Self {
        match val {
            ReadError::BufferOverflow => panic!("Buffer overflow"),
            ReadError::Disabled => Disconnected {},
        }
    }
}

impl From<WriteError> for Disconnected {
    fn from(val: WriteError) -> Self {
        match val {
            WriteError::BufferOverflow => panic!("Buffer overflow"),
            WriteError::Disabled => Disconnected {},
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
