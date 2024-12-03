// required-features: mac

#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::rcc::WPAN_DEFAULT;
use embassy_stm32_wpan::mac::commands::{AssociateRequest, GetRequest, ResetRequest, SetRequest};
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
    let mut config = config();
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
    {
        let evt = mbox.mac_subsystem.read().await.unwrap();
        info!("{:#x}", evt);
    }

    info!("setting extended address");
    let extended_address: u64 = 0xACDE480000000002;
    mbox.mac_subsystem
        .send_command(&SetRequest {
            pib_attribute_ptr: &extended_address as *const _ as *const u8,
            pib_attribute: PibId::ExtendedAddress,
        })
        .await
        .unwrap();
    {
        let evt = mbox.mac_subsystem.read().await.unwrap();
        info!("{:#x}", evt);
    }

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
    let short_addr = if let MacEvent::MlmeAssociateCnf(conf) = mbox.mac_subsystem.read().await.unwrap() {
        conf.assoc_short_address
    } else {
        defmt::panic!()
    };

    info!("{}", short_addr);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
