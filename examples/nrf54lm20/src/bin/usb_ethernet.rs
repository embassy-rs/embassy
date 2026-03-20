#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv4Address, Ipv4Cidr, StackResources, StaticConfigV4};
use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::LinkState;
use embassy_nrf54lm20_examples::{UsbDriver, init_board, usb_driver};
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_ncm::{CdcNcmClass, Receiver, Sender, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config, UsbDevice};
use embedded_io_async::Write;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

type NetDevice = ch::Device<'static, 1514>;

static EP_OUT_BUFFER: StaticCell<[u8; 4096]> = StaticCell::new();
static NCM_SCRATCH: StaticCell<[u8; 2048]> = StaticCell::new();

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, UsbDriver<'static>>) -> ! {
    device.run().await
}

#[embassy_executor::task]
async fn usb_ncm_rx_task(
    mut rx_usb: Receiver<'static, UsbDriver<'static>>,
    state_chan: ch::StateRunner<'static>,
    mut rx_chan: ch::RxRunner<'static, 1514>,
) -> ! {
    let ntb = NCM_SCRATCH.init([0; 2048]);
    Timer::after(Duration::from_millis(100)).await;
    loop {
        trace!("WAITING for connection");
        state_chan.set_link_state(LinkState::Down);

        match rx_usb.wait_connection().await {
            Ok(()) => {}
            Err(e) => {
                warn!("wait_connection failed: {:?}", e);
                Timer::after(Duration::from_millis(10)).await;
                continue;
            }
        }

        trace!("Connected");
        state_chan.set_link_state(LinkState::Up);

        loop {
            let p = rx_chan.rx_buf().await;
            match rx_usb.read_packet_with_ntb(ntb, p).await {
                Ok(n) => rx_chan.rx_done(n),
                Err(e) => {
                    warn!("error reading packet: {:?}", e);
                    Timer::after(Duration::from_millis(10)).await;
                    break;
                }
            };
        }
    }
}

#[embassy_executor::task]
async fn usb_ncm_tx_task(
    mut tx_usb: Sender<'static, UsbDriver<'static>>,
    mut tx_chan: ch::TxRunner<'static, 1514>,
) -> ! {
    Timer::after(Duration::from_millis(100)).await;
    loop {
        tx_usb.wait_connection().await;

        loop {
            let p = tx_chan.tx_buf().await;
            match tx_usb.write_packet(p).await {
                Ok(()) => {}
                Err(EndpointError::Disabled) => {
                    trace!("TX path disabled, waiting for link");
                    tx_chan.tx_done();
                    Timer::after(Duration::from_millis(10)).await;
                    break;
                }
                Err(e) => {
                    warn!("Failed to TX packet: {:?}", e);
                    Timer::after(Duration::from_millis(10)).await;
                }
            }
            tx_chan.tx_done();
        }
    }
}

#[embassy_executor::task]
async fn heartbeat_task() -> ! {
    loop {
        Timer::after(Duration::from_secs(1)).await;
        info!("usb_ethernet alive");
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, NetDevice>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    cortex_m::asm::delay(64_000_000);
    info!("usb_ethernet start");
    let p = init_board();

    let driver = usb_driver(p.USBHS, &mut EP_OUT_BUFFER.init([0; 4096])[..]);
    info!("usb driver ready");

    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-Ethernet example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.composite_with_iads = true;
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;

    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static MSOS_DESC: StaticCell<[u8; 128]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
    let mut builder = Builder::new(
        driver,
        config,
        &mut CONFIG_DESC.init([0; 256])[..],
        &mut BOS_DESC.init([0; 256])[..],
        &mut MSOS_DESC.init([0; 128])[..],
        &mut CONTROL_BUF.init([0; 128])[..],
    );

    let our_mac_addr = [0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];
    let host_mac_addr = [0x88, 0x88, 0x88, 0x88, 0x88, 0x88];

    static STATE: StaticCell<State> = StaticCell::new();
    let class = CdcNcmClass::new(&mut builder, STATE.init(State::new()), host_mac_addr, 64);
    let usb = builder.build();
    info!("usb device built");

    spawner.spawn(unwrap!(usb_task(usb)));
    info!("usb task spawned");
    let (tx_usb, rx_usb) = class.split();
    static NET_STATE: StaticCell<ch::State<1514, 4, 4>> = StaticCell::new();
    let (runner, device) = ch::new(
        NET_STATE.init(ch::State::new()),
        ch::driver::HardwareAddress::Ethernet(our_mac_addr),
    );
    let (state_chan, rx_chan, tx_chan) = runner.split();
    spawner.spawn(unwrap!(usb_ncm_rx_task(rx_usb, state_chan, rx_chan)));
    spawner.spawn(unwrap!(usb_ncm_tx_task(tx_usb, tx_chan)));

    let config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(169, 254, 1, 61), 16),
        dns_servers: Default::default(),
        gateway: None,
    });
    static RESOURCES: StaticCell<StackResources<1>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(device, config, RESOURCES.init(StackResources::new()), 0x54_4c_4d20);
    spawner.spawn(unwrap!(net_task(runner)));
    spawner.spawn(unwrap!(heartbeat_task()));

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

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

            if let Err(e) = socket.write_all(&buf[..n]).await {
                warn!("write error: {:?}", e);
                break;
            }
        }
    }
}
