// required-features: eth
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;
use common::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_net::iface::Iface;
use embassy_net::tcp::TcpSocket;
use embassy_net::wire::Ipv4Addr;
use embassy_net::{Pool, StackStorage, StaticPool};
use embassy_stm32::eth::{Ethernet, PacketQueue};
use embassy_stm32::peripherals::ETH;
#[cfg(feature = "stop")]
use embassy_stm32::rcc::{StopMode, WakeGuard};
use embassy_stm32::rng::Rng;
use embassy_stm32::{bind_interrupts, eth, peripherals, rng};
use embassy_time::{Duration, with_timeout};
use panic_probe as _;
use static_cell::StaticCell;

teleprobe_meta::timeout!(120);

#[cfg(not(any(feature = "stm32h563zi", feature = "stm32f767zi", feature = "stm32f207zg")))]
bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler<ETH>;
    HASH_RNG => rng::InterruptHandler<peripherals::RNG>;
});
#[cfg(any(feature = "stm32h563zi", feature = "stm32f767zi", feature = "stm32f207zg"))]
bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler<ETH>;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

const SERVER_ADDRESS: Ipv4Addr = Ipv4Addr::new(192, 168, 2, 2);
const DOWNLOAD_PORT: u16 = 4321;
const DOWNLOAD_BYTES: usize = 256 * 1024;

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let mut p = init();
    info!("Hello World!");

    // Generate random seed.
    let seed = {
        let mut rng = Rng::new(p.RNG.reborrow(), Irqs);
        let mut seed = [0; 8];
        rng.blocking_fill_bytes(&mut seed);
        u64::from_le_bytes(seed)
    };

    // Ensure different boards get different MAC
    // so running tests concurrently doesn't break (they're all in the same LAN)
    #[cfg(feature = "stm32f429zi")]
    let n = 1;
    #[cfg(feature = "stm32h755zi")]
    let n = 2;
    #[cfg(feature = "stm32h563zi")]
    let n = 3;
    #[cfg(feature = "stm32f767zi")]
    let n = 4;
    #[cfg(feature = "stm32f207zg")]
    let n = 5;
    #[cfg(feature = "stm32h753zi")]
    let n = 6;

    let mac_addr = [0x00, n, 0xDE, 0xAD, 0xBE, 0xEF];

    // F2 runs out of RAM
    #[cfg(feature = "stm32f207zg")]
    const PACKET_QUEUE_SIZE: usize = 2;
    #[cfg(not(feature = "stm32f207zg"))]
    const PACKET_QUEUE_SIZE: usize = 4;

    // The same packet queue is reused by every driver instance, like an application
    // that stops and restarts Ethernet would.
    static PACKETS: StaticCell<PacketQueue<PACKET_QUEUE_SIZE, PACKET_QUEUE_SIZE>> = StaticCell::new();
    let packets = PACKETS.init(PacketQueue::new());

    // F2 runs out of RAM
    #[cfg(feature = "stm32f207zg")]
    const POOL_COUNT: usize = 4;
    #[cfg(not(feature = "stm32f207zg"))]
    const POOL_COUNT: usize = 16;
    // The ethernet DMA needs 8-byte aligned buffers, sized in multiples of 8.
    static POOL: StaticPool<1520, POOL_COUNT, 8> = StaticPool::new();

    #[cfg(feature = "stop")]
    let _guard = WakeGuard::new(StopMode::Stop1);

    for round in 0..2u64 {
        run_round(&mut p, &mut *packets, &POOL, mac_addr, seed.wrapping_add(round), round).await;
        assert_dma_stopped();
        info!("Round {}: dropped Ethernet", round);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

async fn run_round<const TX: usize, const RX: usize>(
    p: &mut embassy_stm32::Peripherals,
    packets: &mut PacketQueue<TX, RX>,
    pool: &'static impl Pool,
    mac_addr: [u8; 6],
    seed: u64,
    round: u64,
) {
    info!("Round {}: initializing Ethernet", round);

    let mut device = unwrap!(Ethernet::new(
        packets,
        p.ETH.reborrow(),
        p.PA1.reborrow(),
        p.PA7.reborrow(),
        p.PC4.reborrow(),
        p.PC5.reborrow(),
        p.PG13.reborrow(),
        #[cfg(not(feature = "stm32h563zi"))]
        p.PB13.reborrow(),
        #[cfg(feature = "stm32h563zi")]
        p.PB15.reborrow(),
        p.PG11.reborrow(),
        mac_addr,
        p.ETH_SMA.reborrow(),
        p.PA2.reborrow(),
        p.PC1.reborrow(),
        Irqs,
    ));

    let mut storage = StackStorage::new();
    let (stack, mut runner) = embassy_net::Stack::new(&mut storage, pool, seed);
    let eth = unwrap!(stack.add_iface_borrowed(&mut device));
    unwrap!(eth.set_dhcpv4(Some(Default::default())));

    match select(runner.run(), check(eth)).await {
        Either::First(never) => match never {},
        Either::Second(()) => {}
    }

    // The download is still streaming here, so the driver is dropped with frames in
    // flight in both directions.
    info!("Round {}: dropping Ethernet", round);
}

/// Check that dropping the driver left both DMA channels stopped, so it was safe to
/// release the descriptors and buffers.
fn assert_dma_stopped() {
    let dma = embassy_stm32::pac::ETH.ethernet_dma();

    #[cfg(any(feature = "stm32f429zi", feature = "stm32f767zi", feature = "stm32f207zg"))]
    {
        use embassy_stm32::pac::eth::vals::{Rps, Tps};
        let sr = dma.dmasr().read();
        defmt::assert!(sr.tps() == Tps::Stopped, "TX DMA not stopped after drop");
        defmt::assert!(sr.rps() == Rps::Stopped, "RX DMA not stopped after drop");
    }

    #[cfg(not(any(feature = "stm32f429zi", feature = "stm32f767zi", feature = "stm32f207zg")))]
    {
        let dsr = dma.dmadsr().read();
        defmt::assert!(dsr.tps0() == 0, "TX DMA not stopped after drop");
        defmt::assert!(dsr.rps0() == 0, "RX DMA not stopped after drop");
    }
}

/// Get an address over DHCP, then download some data from the perf server.
async fn check(eth: Iface<'_>) {
    info!("Waiting for DHCP up...");
    unwrap!(
        with_timeout(Duration::from_secs(20), eth.wait_config_up()).await,
        "DHCP timed out"
    );
    info!("IP addressing up!");

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut socket = unwrap!(TcpSocket::new(eth.stack(), &mut rx_buffer, &mut tx_buffer));
    socket.set_timeout(Some(Duration::from_secs(10)));

    info!("connecting to {:?}:{}...", SERVER_ADDRESS, DOWNLOAD_PORT);
    unwrap!(socket.connect((SERVER_ADDRESS, DOWNLOAD_PORT)).await);

    let mut buf = [0; 1024];
    let mut total = 0;
    while total < DOWNLOAD_BYTES {
        match unwrap!(socket.read(&mut buf).await) {
            0 => defmt::panic!("read EOF"),
            n => total += n,
        }
    }
    info!("downloaded {} bytes", total);
}
