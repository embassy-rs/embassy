#![no_std]
#![no_main]

use core::net::Ipv6Addr;

use defmt::*;
use embassy_executor::Spawner;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Ipv6Cidr, StackResources, StaticConfigV6};
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::peripherals::RNG;
use embassy_stm32::rcc::WPAN_DEFAULT;
use embassy_stm32::rng::InterruptHandler as RngInterruptHandler;
use embassy_stm32_wpan::TlMbox;
use embassy_stm32_wpan::mac::{Driver, DriverState, Runner};
use embassy_stm32_wpan::sub::mm;
use embassy_time::{Duration, Timer};
use heapless::Vec;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
    RNG => RngInterruptHandler<RNG>;
});

#[embassy_executor::task]
async fn run_mm_queue(mut memory_manager: mm::MemoryManager<'static>) -> ! {
    memory_manager.run_queue().await
}

#[embassy_executor::task]
async fn run_mac(runner: &'static Runner<'static>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn run_net(mut runner: embassy_net::Runner<'static, Driver<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::main(executor = "embassy_stm32::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    /*
        How to make this work:

        - Obtain a NUCLEO-STM32WB55 from your preferred supplier.
        - Download and Install STM32CubeProgrammer.
        - Download stm32wb5x_FUS_fw.bin, stm32wb5x_BLE_Mac_802_15_4_fw.bin, and Release_Notes.html from
          gh:STMicroelectronics/STM32CubeWB@2234d97/Projects/STM32WB_Copro_Wireless_Binaries/STM32WB5x
        - Open STM32CubeProgrammer
        - On the right-hand pane, click "firmware upgrade" to upgrade the st-link firmware.
        - Once complete, click connect to connect to the device.
        - On the left hand pane, click the RSS signal icon to open "Firmware Upgrade Services".
        - In the Release_Notes.html, find the memory address that corresponds to your device for the stm32wb5x_FUS_fw.bin file
        - Select that file, the memory address, "verify download", and then "Firmware Upgrade".
        - Once complete, in the Release_Notes.html, find the memory address that corresponds to your device for the
          stm32wb5x_BLE_Mac_802_15_4_fw.bin file. It should not be the same memory address.
        - Select that file, the memory address, "verify download", and then "Firmware Upgrade".
        - Select "Start Wireless Stack".
        - Disconnect from the device.
        - Run this example.

        Note: extended stack versions are not supported at this time. Do not attempt to install a stack with "extended" in the name.
    */

    let mut config = embassy_stm32::Config::default();
    config.rcc = WPAN_DEFAULT;
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let config = Config::default();
    let mut mbox = TlMbox::init(p.IPCC, Irqs, config).await;

    spawner.spawn(run_mm_queue(mbox.mm_subsystem).unwrap());

    let result = mbox.sys_subsystem.shci_c2_mac_802_15_4_init().await;
    info!("initialized mac: {}", result);

    static DRIVER_STATE: StaticCell<DriverState> = StaticCell::new();
    static RUNNER: StaticCell<Runner> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

    let driver_state = DRIVER_STATE.init(DriverState::new(mbox.mac_subsystem));

    let (driver, mac_runner, mut control) = Driver::new(
        driver_state,
        0x1122u16.to_be_bytes().try_into().unwrap(),
        0xACDE480000000001u64.to_be_bytes().try_into().unwrap(),
    );

    // TODO: rng does not work for some reason
    // Generate random seed.
    // let mut rng = Rng::new(p.RNG, Irqs);
    let seed = [0; 8];
    // let _ = rng.async_fill_bytes(&mut seed).await;
    let seed = u64::from_le_bytes(seed);

    info!("seed generated");

    // Init network stack
    let ipv6_addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff);

    let config = embassy_net::Config::ipv6_static(StaticConfigV6 {
        address: Ipv6Cidr::new(ipv6_addr, 104),
        gateway: None,
        dns_servers: Vec::new(),
    });

    let (stack, eth_runner) = embassy_net::new(driver, config, RESOURCES.init(StackResources::new()), seed);

    // wpan runner
    spawner.spawn(run_mac(RUNNER.init(mac_runner)).unwrap());

    // Launch network task
    spawner.spawn(unwrap!(run_net(eth_runner)));

    info!("Network task initialized");

    control.init_link([0x1A, 0xAA]).await;

    // Ensure DHCP configuration is up before trying connect
    stack.wait_config_up().await;

    info!("Network up");

    // Then we can use it!
    let mut rx_meta = [PacketMetadata::EMPTY];
    let mut rx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY];
    let mut tx_buffer = [0; 4096];

    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);

    let remote_endpoint = (Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2fb), 8000);

    let send_buf = [0u8; 20];

    socket.bind((ipv6_addr, 8000)).unwrap();
    socket.send_to(&send_buf, remote_endpoint).await.unwrap();

    Timer::after(Duration::from_secs(2)).await;

    cortex_m::asm::bkpt();
}
