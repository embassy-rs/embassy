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
use embassy_stm32_wpan::TlMbox;
use embassy_stm32_wpan::mac::commands::{AssociateRequest, GetRequest, ResetRequest, SetRequest};
use embassy_stm32_wpan::mac::event::MacEvent;
use embassy_stm32_wpan::mac::typedefs::{
    AddressMode, Capabilities, KeyIdMode, MacAddress, MacChannel, PanId, PibId, SecurityLevel,
};
use embassy_stm32_wpan::sub::mm;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

#[embassy_executor::task]
async fn run_mm_queue(mut memory_manager: mm::MemoryManager<'static>) {
    memory_manager.run_queue().await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = config();
    config.rcc = WPAN_DEFAULT;

    let p = init_with_config(config);
    info!("Hello World!");

    let config = Config::default();
    let mbox = TlMbox::init(p.IPCC, Irqs, config).await;
    let mut sys = mbox.sys_subsystem;
    let (mut mac_rx, mut mac_tx) = mbox.mac_subsystem.split();

    spawner.spawn(run_mm_queue(mbox.mm_subsystem).unwrap());

    let result = sys.shci_c2_mac_802_15_4_init().await;
    info!("initialized mac: {}", result);

    info!("resetting");
    mac_tx
        .send_command(&ResetRequest {
            set_default_pib: true,
            ..Default::default()
        })
        .await
        .unwrap();
    {
        let evt = mac_rx.read().await.unwrap();
        info!("{:#x}", evt);
    }

    info!("setting extended address");
    let extended_address: u64 = 0xACDE480000000002;
    mac_tx
        .send_command(&SetRequest {
            pib_attribute_ptr: &extended_address as *const _ as *const u8,
            pib_attribute: PibId::ExtendedAddress,
        })
        .await
        .unwrap();
    {
        let evt = mac_rx.read().await.unwrap();
        info!("{:#x}", evt);
    }

    info!("getting extended address");
    mac_tx
        .send_command(&GetRequest {
            pib_attribute: PibId::ExtendedAddress,
            ..Default::default()
        })
        .await
        .unwrap();

    {
        let evt = mac_rx.read().await.unwrap();
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
    mac_tx.send_command(&a).await.unwrap();
    let short_addr = if let MacEvent::MlmeAssociateCnf(conf) = mac_rx.read().await.unwrap() {
        conf.assoc_short_address
    } else {
        defmt::panic!()
    };

    info!("{}", short_addr);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
