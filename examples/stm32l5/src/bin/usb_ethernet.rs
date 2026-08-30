#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_net::StackStorage;
use embassy_net::tcp::TcpListener;
use embassy_stm32::rng::Rng;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals, rng, usb};
use embassy_usb::class::cdc_ncm::embassy_net::{Device, Runner, State as NetState};
use embassy_usb::class::cdc_ncm::{CdcNcmClass, State};
use embassy_usb::{Builder, UsbDevice};
use embedded_io_async::Write;
use panic_probe as _;
use static_cell::StaticCell;

type MyDriver = Driver<'static, embassy_stm32::peripherals::USB>;

const MTU: usize = 1514;

bind_interrupts!(struct Irqs {
    USB_FS => usb::InterruptHandler<peripherals::USB>;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, MyDriver>) -> ! {
    device.run().await
}

#[embassy_executor::task]
async fn usb_ncm_task(class: Runner<'static, MyDriver>) -> ! {
    class.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = true;
        config.rcc.sys = Sysclk::Pll1R;
        config.rcc.pll = Some(Pll {
            // 80Mhz clock (16 / 1 * 10 / 2)
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div1,
            mul: PllMul::Mul10,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::Div2),
        });
        config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true }); // needed for USB
        config.rcc.mux.clk48sel = mux::Clk48sel::Hsi48;
    }
    let p = embassy_stm32::init(config);

    // Create the driver, from the HAL.
    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-Ethernet example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Create embassy-usb DeviceBuilder using the driver and config.
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
    let mut builder = Builder::new(
        driver,
        config,
        &mut CONFIG_DESC.init([0; 256])[..],
        &mut BOS_DESC.init([0; 256])[..],
        &mut [], // no msos descriptors
        &mut CONTROL_BUF.init([0; 128])[..],
    );

    // Our MAC addr.
    let our_mac_addr = [0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];
    // Host's MAC addr. This is the MAC the host "thinks" its USB-to-ethernet adapter has.
    let host_mac_addr = [0x88, 0x88, 0x88, 0x88, 0x88, 0x88];

    // Create classes on the builder.
    static STATE: StaticCell<State> = StaticCell::new();
    let class = CdcNcmClass::new(&mut builder, STATE.init(State::new()), host_mac_addr, 64);

    // Build the builder.
    let usb = builder.build();

    spawner.spawn(unwrap!(usb_task(usb)));

    static NET_STATE: StaticCell<NetState<4, 4>> = StaticCell::new();
    let (runner, device) = class.into_embassy_net_device::<4, 4>(NET_STATE.init(NetState::new()), our_mac_addr, MTU);
    spawner.spawn(unwrap!(usb_ncm_task(runner)));

    // Generate random seed
    let mut rng = Rng::new(p.RNG, Irqs);
    let seed = rng.next_u64();

    // Init network stack
    static STACK: StaticCell<StackStorage> = StaticCell::new();
    let (stack, runner) = embassy_net::Stack::new(STACK.init(StackStorage::new()), seed);

    // Add the network interface to the stack.
    static DEVICE: StaticCell<Device<'static>> = StaticCell::new();
    let iface = unwrap!(stack.add_iface(DEVICE.init(device)));
    iface.set_dhcpv4(Some(Default::default()));

    spawner.spawn(unwrap!(net_task(runner)));

    // And now we can use it!

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    let mut listener = unwrap!(TcpListener::new(stack));
    unwrap!(listener.listen(1234));

    loop {
        info!("Listening on TCP:1234...");
        let mut socket = match listener.accept(&mut rx_buffer, &mut tx_buffer).await {
            Ok(socket) => socket,
            Err(e) => {
                warn!("accept error: {:?}", e);
                continue;
            }
        };
        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

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
