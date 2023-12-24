#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::rcc::WPAN_DEFAULT;
use embassy_stm32_wpan::mac::commands::{AssociateRequest, DataRequest, GetRequest, ResetRequest, SetRequest};
use embassy_stm32_wpan::mac::event::MacEvent;
use embassy_stm32_wpan::mac::typedefs::{
    AddressMode, Capabilities, KeyIdMode, MacAddress, MacChannel, PanId, PibId, SecurityLevel,
};
use embassy_stm32_wpan::sub::mm;
use embassy_stm32_wpan::TlMbox;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

#[embassy_executor::task]
async fn run_mm_queue(memory_manager: mm::MemoryManager) {
    memory_manager.run_queue().await;
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
    let extended_address: u64 = 0xACDE480000000002;
    mbox.mac_subsystem
        .send_command(&SetRequest {
            pib_attribute_ptr: &extended_address as *const _ as *const u8,
            pib_attribute: PibId::ExtendedAddress,
        })
        .await
        .unwrap();
    defmt::info!("{:#x}", mbox.mac_subsystem.read().await.unwrap());

    info!("getting extended address");
    mbox.mac_subsystem
        .send_command(&GetRequest {
            pib_attribute: PibId::ExtendedAddress,
            ..Default::default()
        })
        .await
        .unwrap();

    {
        let evt = mbox.mac_subsystem.read().await.unwrap();
        info!("{:#x}", evt);

        if let MacEvent::MlmeGetCnf(evt) = evt {
            if evt.pib_attribute_value_len == 8 {
                let value = unsafe { core::ptr::read_unaligned(evt.pib_attribute_value_ptr as *const u64) };

                info!("value {:#x}", value)
            }
        }
    }

    info!("assocation request");
    let a = AssociateRequest {
        channel_number: MacChannel::Channel16,
        channel_page: 0,
        coord_addr_mode: AddressMode::Short,
        coord_address: MacAddress { short: [34, 17] },
        capability_information: Capabilities::ALLOCATE_ADDRESS,
        coord_pan_id: PanId([0x1A, 0xAA]),
        security_level: SecurityLevel::Unsecure,
        key_id_mode: KeyIdMode::Implicite,
        key_source: [0; 8],
        key_index: 152,
    };
    info!("{}", a);
    mbox.mac_subsystem.send_command(&a).await.unwrap();
    let short_addr = {
        let evt = mbox.mac_subsystem.read().await.unwrap();
        info!("{:#x}", evt);

        if let MacEvent::MlmeAssociateCnf(conf) = evt {
            conf.assoc_short_address
        } else {
            defmt::panic!()
        }
    };

    info!("setting short address");
    mbox.mac_subsystem
        .send_command(&SetRequest {
            pib_attribute_ptr: &short_addr as *const _ as *const u8,
            pib_attribute: PibId::ShortAddress,
        })
        .await
        .unwrap();
    {
        let evt = mbox.mac_subsystem.read().await.unwrap();
        info!("{:#x}", evt);
    }

    info!("sending data");
    let data = b"Hello from embassy!";
    mbox.mac_subsystem
        .send_command(
            DataRequest {
                src_addr_mode: AddressMode::Short,
                dst_addr_mode: AddressMode::Short,
                dst_pan_id: PanId([0x1A, 0xAA]),
                dst_address: MacAddress::BROADCAST,
                msdu_handle: 0x02,
                ack_tx: 0x00,
                gts_tx: false,
                security_level: SecurityLevel::Unsecure,
                ..Default::default()
            }
            .set_buffer(data),
        )
        .await
        .unwrap();
    {
        let evt = mbox.mac_subsystem.read().await.unwrap();
        info!("{:#x}", evt);
    }

    loop {
        match mbox.mac_subsystem.read().await {
            Ok(evt) => info!("{:#x}", evt),
            _ => continue,
        };
    }
}
