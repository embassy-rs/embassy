// required-features: eth, tls
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;
use common::*;
use defmt_rtt as _;
use embassy_crypto as _;
use embassy_executor::Spawner;
use embassy_net::{StackStorage, StaticPool};
use embassy_stm32::eth::{Ethernet, GenericPhy, PacketQueue, Sma};
use embassy_stm32::peripherals::{ETH, ETH_SMA};
#[cfg(feature = "stop")]
use embassy_stm32::rcc::{StopMode, WakeGuard};
use embassy_stm32::{bind_interrupts, eth};
use mcu_crypto_asm as _;
#[cfg(feature = "tls-sw")]
use mcu_crypto_sw_aes as _;
use panic_probe as _;
use static_cell::StaticCell;

teleprobe_meta::timeout!(60);

bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler<ETH>;
});

type Device = Ethernet<'static, ETH, GenericPhy<Sma<'static, ETH_SMA>>>;

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static>) -> ! {
    runner.run().await
}

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(spawner: Spawner) {
    let p = init();
    info!("Hello World!");

    // mcu-crypto-asm's bignum assembly uses VFP registers as scratch, so the
    // FPU must be accessible. run-from-RAM loaders skip cortex-m-rt's Reset
    // pre-main (which normally enables it), so do it here.
    unsafe {
        let cpacr = 0xE000_ED88 as *mut u32;
        cpacr.write(cpacr.read() | (0b1111 << 20));
        core::arch::asm!("dsb", "isb");
    }

    // Random seed for the network stack, drawn from the hardware RNG that
    // serves the `embassy-crypto` driver.
    let mut seed = [0; 8];
    embassy_crypto::rng_fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    // Unique MAC id (eth.rs uses 1..=6) so eth and tls tests can run
    // concurrently on different boards on the same LAN.
    let mac_addr = [0x00, 7, 0xDE, 0xAD, 0xBE, 0xEF];

    const PACKET_QUEUE_SIZE: usize = 4;
    static PACKETS: StaticCell<PacketQueue<PACKET_QUEUE_SIZE, PACKET_QUEUE_SIZE>> = StaticCell::new();

    let device = Ethernet::new(
        PACKETS.init(PacketQueue::<PACKET_QUEUE_SIZE, PACKET_QUEUE_SIZE>::new()),
        p.ETH,
        p.PA1,
        p.PA7,
        p.PC4,
        p.PC5,
        p.PG13,
        #[cfg(not(feature = "stm32h563zi"))]
        p.PB13,
        #[cfg(feature = "stm32h563zi")]
        p.PB15,
        p.PG11,
        mac_addr,
        p.ETH_SMA,
        p.PA2,
        p.PC1,
        Irqs,
    );

    static STACK: StaticCell<StackStorage> = StaticCell::new();
    // F2 runs out of RAM
    #[cfg(feature = "stm32f207zg")]
    const POOL_COUNT: usize = 4;
    #[cfg(not(feature = "stm32f207zg"))]
    const POOL_COUNT: usize = 16;
    // The ethernet DMA needs 8-byte aligned buffers, sized in multiples of 8.
    static POOL: StaticPool<1520, POOL_COUNT, 8> = StaticPool::new();
    let (stack, runner) = embassy_net::Stack::new(STACK.init(StackStorage::new()), &POOL, seed);

    static ETH: StaticCell<Device> = StaticCell::new();
    let eth = unwrap!(stack.add_iface_borrowed(ETH.init(device)));

    unwrap!(eth.set_dhcpv4(Some(Default::default())));

    #[cfg(feature = "stop")]
    let _guard = WakeGuard::new(StopMode::Stop1);

    spawner.spawn(unwrap!(net_task(runner)));

    perf_client::run_tls(
        eth,
        perf_client::Expected {
            down_kbps: 100,
            up_kbps: 100,
            updown_kbps: 100,
        },
    )
    .await;

    info!("Test OK");
    cortex_m::asm::bkpt();
}
