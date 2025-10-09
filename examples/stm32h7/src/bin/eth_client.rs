#![no_std]
#![no_main]

use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_stm32::eth::{Ethernet, GenericPhy, PacketQueue};
use embassy_stm32::peripherals::ETH;
use embassy_stm32::rng::Rng;
use embassy_stm32::{Config, bind_interrupts, eth, peripherals, rng};
use embassy_time::Timer;
use embedded_io_async::Write;
use embedded_nal_async::{TcpConnect, UnconnectedUdp};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;

enum TcpState {
    Connecting,
    Connected,
}

type MessageType = Mutex<ThreadModeRawMutex, Option<TcpState>>;
static MESSAGE: MessageType = Mutex::new(None);

bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

type Device = Ethernet<'static, ETH, GenericPhy>;

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.hsi48 = Some(Default::default()); // needed for RNG
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
    }
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    // Generate random seed.
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut seed = [0; 8];
    rng.fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];

    static PACKETS: StaticCell<PacketQueue<4, 4>> = StaticCell::new();

    let device = Ethernet::new(
        PACKETS.init(PacketQueue::<4, 4>::new()),
        p.ETH,
        Irqs,
        p.PA1,
        p.PA2,
        p.PC1,
        p.PA7,
        p.PC4,
        p.PC5,
        p.PG13,
        p.PB13,
        p.PG11,
        GenericPhy::new_auto(),
        mac_addr,
    );

    let config = embassy_net::Config::dhcpv4(Default::default());
    //let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
    //    address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 18, 64), 24),
    //    dns_servers: Vec::new(),
    //    gateway: Some(Ipv4Address::new(192, 168, 18, 1)),
    //});

    // Init network stack
    static RESOURCES: StaticCell<StackResources<4>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(device, config, RESOURCES.init(StackResources::new()), seed);

    // Launch network task
    spawner.spawn(unwrap!(net_task(runner)));

    // Ensure DHCP configuration is up before trying connect
    stack.wait_config_up().await;
    let config_v4 = unwrap!(stack.config_v4());

    info!("Network task initialized");

    let local_socket_address: SocketAddr = SocketAddrV4::new(config_v4.address.address().into(), 8001).into();
    info!("udp local address: {}", local_socket_address);
    let broadcast_socket_address: SocketAddr = SocketAddrV4::new(config_v4.address.broadcast().unwrap().into(), 8001).into();
    info!("udp broadcast address: {}", broadcast_socket_address);

    // You need to start a server on the host machine, for example: `nc -b -l 8001`
    let udp_address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 18, 41), 8001));

    // You need to start a server on the host machine, for example: `nc -l 8000`
    let tcp_address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 18, 41), 8000));
    
    spawner.spawn(unwrap!(broadcast_task(stack, local_socket_address, broadcast_socket_address, udp_address, &MESSAGE)));
    spawner.spawn(unwrap!(tcp_communication_task(stack, tcp_address, &MESSAGE)));

    loop {
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn tcp_communication_task(stack: Stack<'static>, tcp_address: SocketAddr, message: &'static MessageType) -> ! {

    let state: TcpClientState<1, 1024, 1024> = TcpClientState::new();
    let client = TcpClient::new(stack, &state);

    loop {
        info!("connecting...");
        {
            *(message.lock().await) = Some(TcpState::Connecting);
        }
        let r = client.connect(tcp_address).await;
        if let Err(e) = r {
            info!("tcp connect error: {:?}", e);
            Timer::after_secs(1).await;
            continue;
        }

        let mut connection = r.unwrap();
        info!("tcp connected!");
        {
            *(message.lock().await) = Some(TcpState::Connected);
        }

        loop {
            let r = connection.write_all(b"Hello\n").await;
            if let Err(e) = r {
                info!("tcp write error: {:?}", e);
                break;
            }
            Timer::after_secs(1).await;
        }
    }
}

#[embassy_executor::task]
async fn broadcast_task(stack: Stack<'static>, local_socket_address: SocketAddr, broadcast_socket_address: SocketAddr, udp_address: SocketAddr, message: &'static MessageType) -> ! {

    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; 4096];

    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
    socket.bind(8001).unwrap();

    loop {
        let mut message_unlocked = message.lock().await;
        if let Some(message_ref) = message_unlocked.as_mut() {

            match message_ref {
                TcpState::Connecting => {
                    let r = socket.send(local_socket_address, broadcast_socket_address, b"UDP: Waiting for TCP connect\n").await;
                    if let Err(e) = r {
                        info!("udp broadcast error: {:?}", e);
                    }
                }
                TcpState::Connected => {
                    let r = socket.send(local_socket_address, udp_address, b"UDP: TCP connection OK\n").await;
                    if let Err(e) = r {
                        info!("udp write error: {:?}", e);
                    }
                }
            }
        }
        // release the mutex
        drop(message_unlocked);

        Timer::after_secs(1).await;
    }
}