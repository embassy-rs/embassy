#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::mem;

use defmt::*;
use embassy_executor::Spawner;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Stack, StackResources};
use embassy_nrf::rng::Rng;
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_nrf::usb::Driver;
use embassy_nrf::{bind_interrupts, pac, peripherals, rng, usb};
use embassy_usb::class::cdc_ncm::embassy_net::{Device, Runner, State as NetState};
use embassy_usb::class::cdc_ncm::{CdcNcmClass, State};
use embassy_usb::{Builder, Config, UsbDevice};
use embedded_io_async::Write;
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<peripherals::USBD>;
    POWER_CLOCK => usb::vbus_detect::InterruptHandler;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

type MyDriver = Driver<'static, peripherals::USBD, HardwareVbusDetect>;

const MTU: usize = 1514;

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, MyDriver>) -> ! {
    device.run().await
}

#[embassy_executor::task]
async fn usb_ncm_task(class: Runner<'static, MyDriver, MTU>) -> ! {
    class.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<Device<'static, MTU>>) -> ! {
    stack.run().await
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
    config.product = Some("USB-Ethernet example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Required for Windows support.
    config.composite_with_iads = true;
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;

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

    // Our MAC addr.
    let our_mac_addr = [0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];
    // Host's MAC addr. This is the MAC the host "thinks" its USB-to-ethernet adapter has.
    let host_mac_addr = [0x88, 0x88, 0x88, 0x88, 0x88, 0x88];

    // Create classes on the builder.
    let class = CdcNcmClass::new(&mut builder, make_static!(State::new()), host_mac_addr, 64);

    // Build the builder.
    let usb = builder.build();

    unwrap!(spawner.spawn(usb_task(usb)));

    let (runner, device) = class.into_embassy_net_device::<MTU, 4, 4>(make_static!(NetState::new()), our_mac_addr);
    unwrap!(spawner.spawn(usb_ncm_task(runner)));

    let config = embassy_net::Config::dhcpv4(Default::default());
    // let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
    //    address: Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
    //    dns_servers: Vec::new(),
    //    gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    // });

    // Generate random seed
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut seed = [0; 8];
    rng.blocking_fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    // Init network stack
    let stack = &*make_static!(Stack::new(
        device,
        config,
        make_static!(StackResources::<2>::new()),
        seed
    ));

    unwrap!(spawner.spawn(net_task(stack)));

    // And now we can use it!

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

        info!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        info!("Received connection from {:?}", socket.remote_endpoint());

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    warn!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("read error: {:?}", e);
                    break;
                }
            };

            info!("rxd {:02x}", &buf[..n]);

            match socket.write_all(&buf[..n]).await {
                Ok(()) => {}
                Err(e) => {
                    warn!("write error: {:?}", e);
                    break;
                }
            };
        }
    }
}
