#![no_std]
#![no_main]

use bt_hci::cmd::controller_baseband::Reset;
use bt_hci::controller::{Controller, ControllerCmdSync};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::rcc::Config as RccConfig;
use embassy_stm32_wpan::TlMbox;
use embassy_stm32_wpan::lhci::LhciC1DeviceInformationCcrp;
use embassy_stm32_wpan::sub::ble::ControllerAdapter;
use embassy_stm32_wpan::sub::mm;
use panic_probe as _;
use stm32wb_hci::aci::durations::{AdvInterval, PreferredConnInterval};
use stm32wb_hci::aci::flags::Role;
use stm32wb_hci::aci::gap::{GapDeleteAdType, GapInit, GapSetDiscoverable, GapUpdateAdvData};
use stm32wb_hci::aci::gatt::GattInit;
use stm32wb_hci::aci::hal::{HalSetTxPowerLevel, HalWriteConfigData};
use stm32wb_hci::aci::ranges::PaLevel;
use stm32wb_hci::aci::values::{AdvertisingType, ConfigDataOffset, OwnAddressType, Privacy};
use stm32wb_hci::adv_data::{AdvData, AdvFlags};
use stm32wb_hci::event::BleEvent;

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

const BLE_GAP_DEVICE_NAME_LENGTH: u8 = 7;

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
    let (ble, mm) = TlMbox::wait_ready(p.IPCC, Irqs, config)
        .await
        .unwrap()
        .init_ble(Default::default())
        .await
        .unwrap();

    spawner.spawn(run_mm_queue(mm).unwrap());

    let ble = ControllerAdapter::new(ble);

    join(
        async {
            loop {
                let mut buf = unwrap!(ble.alloc_buf());
                match ble.read(&mut buf).await {
                    Ok(bt_hci::ControllerToHostPacket::Event(packet)) => match BleEvent::from_packet(packet) {
                        Ok(event) => info!("event: {}", event),
                        Err(e) => error!("undecodable event: {}", e),
                    },
                    Ok(packet) => info!("pkt: {}", packet),
                    Err(e) => error!("read failed: {}", e),
                }
            }
        },
        async {
            info!("resetting BLE...");
            let response = ble.exec(&Reset::new()).await;
            defmt::info!("{}", response);

            info!("config public address...");
            let address = get_bd_addr();
            let command = unwrap!(HalWriteConfigData::entry(ConfigDataOffset::PublicAddress, &address));
            let response = ble.exec(&command).await;
            defmt::info!("{}", response);

            info!("config random address...");
            let address = get_random_addr();
            let command = unwrap!(HalWriteConfigData::entry(
                ConfigDataOffset::StaticRandomAddress,
                &address
            ));
            let response = ble.exec(&command).await;
            defmt::info!("{}", response);

            info!("config identity root...");
            let command = unwrap!(HalWriteConfigData::entry(ConfigDataOffset::IdentityRoot, &BLE_CFG_IRK));
            let response = ble.exec(&command).await;
            defmt::info!("{}", response);

            info!("config encryption root...");
            let command = unwrap!(HalWriteConfigData::entry(
                ConfigDataOffset::EncryptionRoot,
                &BLE_CFG_ERK
            ));
            let response = ble.exec(&command).await;
            defmt::info!("{}", response);

            info!("config tx power level...");
            // PA level 0x19 is 0 dBm.
            let response = ble
                .exec(&HalSetTxPowerLevel::new(false, unwrap!(PaLevel::new(0x19))))
                .await;
            defmt::info!("{}", response);

            info!("GATT init...");
            let response = ble.exec(&GattInit::new()).await;
            defmt::info!("{}", response);

            info!("GAP init...");
            let response = ble
                .exec(&GapInit::new(
                    Role::PERIPHERAL,
                    Privacy::Disabled,
                    BLE_GAP_DEVICE_NAME_LENGTH,
                ))
                .await;
            defmt::info!("{}", response);

            info!("set discoverable...");
            let interval = unwrap!(AdvInterval::from_millis(250));
            let command = unwrap!(GapSetDiscoverable::try_new(
                AdvertisingType::NonConnectableUndirected,
                interval,
                interval,
                OwnAddressType::Public,
                0, // no filter accept list
                &[],
                &[],
                PreferredConnInterval::OMITTED,
                PreferredConnInterval::OMITTED,
            ));
            let response = ble.exec(&command).await;
            defmt::info!("{}", response);

            // remove some advertisement to decrease the packet size
            info!("delete tx power ad type...");
            let response = ble.exec(&GapDeleteAdType::new(AD_TYPE_TX_POWER_LEVEL)).await;
            defmt::info!("{}", response);

            info!("delete conn interval ad type...");
            let response = ble
                .exec(&GapDeleteAdType::new(AD_TYPE_PERIPHERAL_CONN_INTERVAL_RANGE))
                .await;
            defmt::info!("{}", response);

            info!("update advertising data...");
            let data = unwrap!(eddystone_advertising_data());
            let response = ble.exec(&unwrap!(GapUpdateAdvData::try_new(data.as_bytes()))).await;
            defmt::info!("{}", response);

            info!("update advertising data type...");
            let data = unwrap!(AdvData::<31>::new().complete_uuid16_list(&[EDDYSTONE_UUID]));
            let response = ble.exec(&unwrap!(GapUpdateAdvData::try_new(data.as_bytes()))).await;
            defmt::info!("{}", response);

            info!("update advertising data flags...");
            // BLE general discoverable, without BR/EDR support
            let data =
                unwrap!(AdvData::<31>::new().flags(AdvFlags::LE_GENERAL_DISCOVERABLE | AdvFlags::BR_EDR_NOT_SUPPORTED));
            let response = ble.exec(&unwrap!(GapUpdateAdvData::try_new(data.as_bytes()))).await;
            defmt::info!("{}", response);

            // cortex_m::asm::bkpt();
        },
    )
    .await;
}

#[embassy_executor::task]
async fn run_mm_queue(mut memory_manager: mm::MemoryManager<'static>) {
    memory_manager.run_queue().await;
}

fn get_bd_addr() -> [u8; 6] {
    let mut bytes = [0u8; 6];

    let lhci_info = LhciC1DeviceInformationCcrp::new();
    bytes[0] = (lhci_info.uid64 & 0xff) as u8;
    bytes[1] = ((lhci_info.uid64 >> 8) & 0xff) as u8;
    bytes[2] = ((lhci_info.uid64 >> 16) & 0xff) as u8;
    bytes[3] = lhci_info.device_type_id;
    bytes[4] = (lhci_info.st_company_id & 0xff) as u8;
    bytes[5] = (lhci_info.st_company_id >> 8 & 0xff) as u8;

    bytes
}

fn get_random_addr() -> [u8; 6] {
    let mut bytes = [0u8; 6];

    let lhci_info = LhciC1DeviceInformationCcrp::new();
    bytes[0] = (lhci_info.uid64 & 0xff) as u8;
    bytes[1] = ((lhci_info.uid64 >> 8) & 0xff) as u8;
    bytes[2] = ((lhci_info.uid64 >> 16) & 0xff) as u8;
    bytes[3] = 0;
    bytes[4] = 0x6E;
    bytes[5] = 0xED;

    bytes
}

const BLE_CFG_IRK: [u8; 16] = [
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
];
const BLE_CFG_ERK: [u8; 16] = [
    0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65, 0x43, 0x21, 0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65, 0x43, 0x21,
];

const AD_TYPE_TX_POWER_LEVEL: u8 = 0x0A;
const AD_TYPE_PERIPHERAL_CONN_INTERVAL_RANGE: u8 = 0x12;
const EDDYSTONE_UUID: u16 = 0xFEAA;

fn eddystone_advertising_data() -> Result<AdvData<31>, stm32wb_hci::wire::TooLong> {
    const EDDYSTONE_URL: &[u8] = b"www.rust-lang.com";

    let mut frame = [0u8; 3 + EDDYSTONE_URL.len()];
    frame[0] = 0x10; // URL frame type
    frame[1] = 22_i8 as u8; // calibrated TX power at 0m
    frame[2] = 0x03; // eddystone url prefix = https
    frame[3..].copy_from_slice(EDDYSTONE_URL);

    AdvData::new().service_data_uuid16(EDDYSTONE_UUID, &frame)
}
