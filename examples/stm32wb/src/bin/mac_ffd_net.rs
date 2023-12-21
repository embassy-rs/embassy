#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::rcc::WPAN_DEFAULT;
use embassy_stm32_wpan::mac::commands::{ResetRequest, SetRequest, StartRequest};
use embassy_stm32_wpan::mac::typedefs::{MacChannel, PanId, PibId};
use embassy_stm32_wpan::mac::{self, Runner};
use embassy_stm32_wpan::sub::mm;
use embassy_stm32_wpan::TlMbox;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

#[embassy_executor::task]
async fn run_mm_queue(memory_manager: mm::MemoryManager) {
    memory_manager.run_queue().await;
}

#[embassy_executor::task]
async fn run_mac(runner: &'static Runner<'static>) {
    runner.run().await;
}

#[embassy_executor::main]
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
        - In the examples folder for stm32wb, modify the memory.x file to match your target device.
        - Run this example.

        Note: extended stack versions are not supported at this time. Do not attempt to install a stack with "extended" in the name.
    */

    let mut config = embassy_stm32::Config::default();
    config.rcc = WPAN_DEFAULT;
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let config = Config::default();
    let mbox = TlMbox::init(p.IPCC, Irqs, config);

    spawner.spawn(run_mm_queue(mbox.mm_subsystem)).unwrap();

    let sys_event = mbox.sys_subsystem.read().await;
    info!("sys event: {}", sys_event.payload());

    core::mem::drop(sys_event);

    let result = mbox.sys_subsystem.shci_c2_mac_802_15_4_init().await;
    info!("initialized mac: {}", result);

    info!("resetting");
    mbox.mac_subsystem
        .send_command(&ResetRequest {
            set_default_pib: true,
            ..Default::default()
        })
        .await
        .unwrap();
    defmt::info!("{:#x}", mbox.mac_subsystem.read().await.unwrap());

    info!("setting extended address");
    let extended_address: u64 = 0xACDE480000000001;
    mbox.mac_subsystem
        .send_command(&SetRequest {
            pib_attribute_ptr: &extended_address as *const _ as *const u8,
            pib_attribute: PibId::ExtendedAddress,
        })
        .await
        .unwrap();
    defmt::info!("{:#x}", mbox.mac_subsystem.read().await.unwrap());

    info!("setting short address");
    let short_address: u16 = 0x1122;
    mbox.mac_subsystem
        .send_command(&SetRequest {
            pib_attribute_ptr: &short_address as *const _ as *const u8,
            pib_attribute: PibId::ShortAddress,
        })
        .await
        .unwrap();
    defmt::info!("{:#x}", mbox.mac_subsystem.read().await.unwrap());

    info!("setting association permit");
    let association_permit: bool = true;
    mbox.mac_subsystem
        .send_command(&SetRequest {
            pib_attribute_ptr: &association_permit as *const _ as *const u8,
            pib_attribute: PibId::AssociationPermit,
        })
        .await
        .unwrap();
    defmt::info!("{:#x}", mbox.mac_subsystem.read().await.unwrap());

    info!("setting TX power");
    let transmit_power: i8 = 2;
    mbox.mac_subsystem
        .send_command(&SetRequest {
            pib_attribute_ptr: &transmit_power as *const _ as *const u8,
            pib_attribute: PibId::TransmitPower,
        })
        .await
        .unwrap();
    defmt::info!("{:#x}", mbox.mac_subsystem.read().await.unwrap());

    info!("starting FFD device");
    mbox.mac_subsystem
        .send_command(&StartRequest {
            pan_id: PanId([0x1A, 0xAA]),
            channel_number: MacChannel::Channel16,
            beacon_order: 0x0F,
            superframe_order: 0x0F,
            pan_coordinator: true,
            battery_life_extension: false,
            ..Default::default()
        })
        .await
        .unwrap();
    defmt::info!("{:#x}", mbox.mac_subsystem.read().await.unwrap());

    info!("setting RX on when idle");
    let rx_on_while_idle: bool = true;
    mbox.mac_subsystem
        .send_command(&SetRequest {
            pib_attribute_ptr: &rx_on_while_idle as *const _ as *const u8,
            pib_attribute: PibId::RxOnWhenIdle,
        })
        .await
        .unwrap();
    defmt::info!("{:#x}", mbox.mac_subsystem.read().await.unwrap());

    static TX1: StaticCell<[u8; 127]> = StaticCell::new();
    static TX2: StaticCell<[u8; 127]> = StaticCell::new();
    static TX3: StaticCell<[u8; 127]> = StaticCell::new();
    static TX4: StaticCell<[u8; 127]> = StaticCell::new();
    static TX5: StaticCell<[u8; 127]> = StaticCell::new();
    let tx_queue = [
        TX1.init([0u8; 127]),
        TX2.init([0u8; 127]),
        TX3.init([0u8; 127]),
        TX4.init([0u8; 127]),
        TX5.init([0u8; 127]),
    ];

    static RUNNER: StaticCell<Runner> = StaticCell::new();
    let runner = RUNNER.init(Runner::new(mbox.mac_subsystem, tx_queue));

    spawner.spawn(run_mac(runner)).unwrap();

    let (driver, control) = mac::new(runner).await;

    let _ = driver;
    let _ = control;
}
