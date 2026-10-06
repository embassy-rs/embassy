#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::rcc::Config as RccConfig;
use embassy_stm32_wpan::TlMbox;
use embassy_stm32_wpan::net::commands::{AssociateRequest, DataRequest, GetRequest, ResetRequest, SetRequest};
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

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    /*
        How to make this work:

        - Obtain a NUCLEO-STM32WB55 from your preferred supplier.
        - Run the `fus_update` example: it installs the FUS and the wireless stack on its own,
          no external tool needed.
        - Run this example.

        Note: extended stack versions are not supported at this time. Do not attempt to install a stack with "extended" in the name.
    */

    let mut config = embassy_stm32::Config::default();
    config.rcc = RccConfig::new_wpan();
    let p = embassy_stm32::init(config);
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

    info!("setting short address");
    controller
        .write(&SetRequest {
            pib_attribute_ptr: &short_addr as *const _ as *const u8,
            pib_attribute: PibId::ShortAddress,
        })
        .await
        .unwrap();
    {
        let pkt = controller.read().await.unwrap();

        defmt::info!("{:#x}", pkt.packet());
    }

    info!("sending data");
    let data = b"Hello from embassy!";
    controller
        .write(
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
        let pkt = controller.read().await.unwrap();

        defmt::info!("{:#x}", pkt.packet());
    }

    loop {
        {
            let pkt = controller.read().await;

            match pkt {
                Ok(pkt) => info!("{:#x}", pkt.packet()),
                _ => continue,
            };
        }
    }
}
