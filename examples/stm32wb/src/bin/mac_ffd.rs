#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::rcc::Config as RccConfig;
use embassy_stm32_wpan::TlMbox;
use embassy_stm32_wpan::net::commands::{AssociateResponse, ResetRequest, SetRequest, StartRequest};
use embassy_stm32_wpan::net::iface::{Controller, ControllerToHostPacket, ControllerToHostPacketBox, mcps, mlme};
use embassy_stm32_wpan::net::typedefs::{MacChannel, MacStatus, PanId, PibId, SecurityLevel};
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
    let extended_address: u64 = 0xACDE480000000001;
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

    info!("setting short address");
    let short_address: u16 = 0x1122;
    controller
        .write(&SetRequest {
            pib_attribute_ptr: &short_address as *const _ as *const u8,
            pib_attribute: PibId::ShortAddress,
        })
        .await
        .unwrap();
    {
        let pkt = controller.read().await.unwrap();

        defmt::info!("{:#x}", pkt.packet());
    }

    info!("setting association permit");
    let association_permit: bool = true;
    controller
        .write(&SetRequest {
            pib_attribute_ptr: &association_permit as *const _ as *const u8,
            pib_attribute: PibId::AssociationPermit,
        })
        .await
        .unwrap();
    {
        let pkt = controller.read().await.unwrap();

        defmt::info!("{:#x}", pkt.packet());
    }

    info!("setting TX power");
    let transmit_power: i8 = 2;
    controller
        .write(&SetRequest {
            pib_attribute_ptr: &transmit_power as *const _ as *const u8,
            pib_attribute: PibId::TransmitPower,
        })
        .await
        .unwrap();
    {
        let pkt = controller.read().await.unwrap();

        defmt::info!("{:#x}", pkt.packet());
    }

    info!("starting FFD device");
    controller
        .write(&StartRequest {
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
    {
        let pkt = controller.read().await.unwrap();

        defmt::info!("{:#x}", pkt.packet());
    }

    info!("setting RX on when idle");
    let rx_on_while_idle: bool = true;
    controller
        .write(&SetRequest {
            pib_attribute_ptr: &rx_on_while_idle as *const _ as *const u8,
            pib_attribute: PibId::RxOnWhenIdle,
        })
        .await
        .unwrap();
    {
        let pkt = controller.read().await.unwrap();

        defmt::info!("{:#x}", pkt.packet());
    }

    loop {
        let pkt = controller.read().await;

        if let Ok(pkt) = pkt {
            let evt = pkt.packet();

            defmt::info!("parsed mac event");
            defmt::info!("{:#x}", evt);

            match evt {
                ControllerToHostPacket::Mlme(mlme::Packet::Indication(mlme::IndicationPacket::Associate(
                    association,
                ))) => controller
                    .write(&AssociateResponse {
                        device_address: association.device_address,
                        assoc_short_address: [0x33, 0x44],
                        status: MacStatus::Success,
                        security_level: SecurityLevel::Unsecure,
                        ..Default::default()
                    })
                    .await
                    .unwrap(),
                ControllerToHostPacket::Mcps(mcps::Packet::Indication(mcps::IndicationPacket::Data(data_ind))) => {
                    let payload = data_ind.payload();
                    let ref_payload = b"Hello from embassy!";
                    info!("{}", payload);

                    if payload == ref_payload {
                        info!("success");
                    } else {
                        info!("ref payload: {}", ref_payload);
                    }
                }
                _ => {
                    defmt::info!("other mac event");
                }
            }
        } else {
            defmt::info!("failed to parse mac event");
        }
    }
}
