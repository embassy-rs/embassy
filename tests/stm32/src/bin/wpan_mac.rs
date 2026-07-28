// required-features: mac

#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::rcc::Config as RccConfig;
use embassy_stm32_wpan::TlMbox;
use embassy_stm32_wpan::net::commands::{AssociateRequest, GetRequest, ResetRequest, SetRequest};
use embassy_stm32_wpan::net::iface::{Controller, ControllerToHostPacket, ControllerToHostPacketBox, mlme};
use embassy_stm32_wpan::net::typedefs::{
    AddressMode, Capabilities, KeyIdMode, MacAddress, MacChannel, PanId, PibId, SecurityLevel,
};
use embassy_stm32_wpan::sub::mac::ControllerAdapter;
use embassy_stm32_wpan::sub::mm;
use panic_probe as _;

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

#[embassy_executor::task]
async fn run_mm_queue(mut memory_manager: mm::MemoryManager<'static>) {
    memory_manager.run_queue().await;
}

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(spawner: Spawner) {
    let mut config = config();
    config.rcc = RccConfig::new_wpan();

    let p = init_with_config(config);
    info!("Hello World!");

    let config = Config::default();
    let (mac, mm) = TlMbox::wait_ready(p.IPCC, Irqs, config)
        .await
        .unwrap()
        .init_mac()
        .await
        .unwrap();

    spawner.spawn(run_mm_queue(mm).unwrap());
    let controller = ControllerAdapter::new(mac);

    info!("resetting");
    controller
        .write(&ResetRequest {
            set_default_pib: true,
            ..Default::default()
        })
        .await
        .unwrap();

    {
        let pkt = controller.read().await.unwrap();

        defmt::info!("{:#x}", pkt.packet());
    }

    info!("setting extended address");
    let extended_address: u64 = 0xACDE480000000002;
    controller
        .write(&SetRequest {
            pib_attribute_ptr: &extended_address as *const _ as *const u8,
            pib_attribute: PibId::ExtendedAddress,
        })
        .await
        .unwrap();
    {
        let pkt = controller.read().await.unwrap();

        defmt::info!("{:#x}", pkt.packet());
    }

    info!("getting extended address");
    controller
        .write(&GetRequest {
            pib_attribute: PibId::ExtendedAddress,
            ..Default::default()
        })
        .await
        .unwrap();

    {
        let pkt = controller.read().await.unwrap();
        defmt::info!("{:#x}", pkt.packet());

        if let ControllerToHostPacket::Mlme(mlme::Packet::Confirm(mlme::ConfirmPacket::Get(evt))) = pkt.packet() {
            if evt.pib_attribute_value_len == 8 {
                let value = unsafe { core::ptr::read_unaligned(evt.pib_attribute_value_ptr as *const u64) };

                info!("value {:#x}", value)
            }
        } else {
            defmt::panic!();
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
    controller.write(&a).await.unwrap();
    let short_addr = {
        let pkt = controller.read().await.unwrap();
        defmt::info!("{:#x}", pkt.packet());

        if let ControllerToHostPacket::Mlme(mlme::Packet::Confirm(mlme::ConfirmPacket::Associate(conf))) = pkt.packet()
        {
            conf.assoc_short_address
        } else {
            defmt::panic!()
        }
    };

    info!("{}", short_addr);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
