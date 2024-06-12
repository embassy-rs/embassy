// required-features: ble

#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use core::time::Duration;

use common::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::rcc::WPAN_DEFAULT;
use embassy_stm32_wpan::hci::host::uart::UartHci;
use embassy_stm32_wpan::hci::host::{AdvertisingFilterPolicy, EncryptionKey, HostHci, OwnAddressType};
use embassy_stm32_wpan::hci::types::AdvertisingType;
use embassy_stm32_wpan::hci::vendor::command::gap::{AdvertisingDataType, DiscoverableParameters, GapCommands, Role};
use embassy_stm32_wpan::hci::vendor::command::gatt::GattCommands;
use embassy_stm32_wpan::hci::vendor::command::hal::{ConfigData, HalCommands, PowerLevel};
use embassy_stm32_wpan::hci::BdAddr;
use embassy_stm32_wpan::lhci::LhciC1DeviceInformationCcrp;
use embassy_stm32_wpan::sub::mm;
use embassy_stm32_wpan::TlMbox;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

const BLE_GAP_DEVICE_NAME_LENGTH: u8 = 7;

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
    let mut mbox = TlMbox::init(p.IPCC, Irqs, config);

    spawner.spawn(run_mm_queue(mbox.mm_subsystem)).unwrap();

    let sys_event = mbox.sys_subsystem.read().await;
    info!("sys event: {}", sys_event.payload());

    let fw_info = mbox.sys_subsystem.wireless_fw_info().unwrap();
    let version_major = fw_info.version_major();
    let version_minor = fw_info.version_minor();
    let subversion = fw_info.subversion();

    let sram2a_size = fw_info.sram2a_size();
    let sram2b_size = fw_info.sram2b_size();

    info!(
        "version {}.{}.{} - SRAM2a {} - SRAM2b {}",
        version_major, version_minor, subversion, sram2a_size, sram2b_size
    );

    let _ = mbox.sys_subsystem.shci_c2_ble_init(Default::default()).await;

    info!("resetting BLE...");
    mbox.ble_subsystem.reset().await;
    let response = mbox.ble_subsystem.read().await.unwrap();
    info!("{}", response);

    info!("config public address...");
    mbox.ble_subsystem
        .write_config_data(&ConfigData::public_address(get_bd_addr()).build())
        .await;
    let response = mbox.ble_subsystem.read().await.unwrap();
    info!("{}", response);

    info!("config random address...");
    mbox.ble_subsystem
        .write_config_data(&ConfigData::random_address(get_random_addr()).build())
        .await;
    let response = mbox.ble_subsystem.read().await.unwrap();
    info!("{}", response);

    info!("config identity root...");
    mbox.ble_subsystem
        .write_config_data(&ConfigData::identity_root(&get_irk()).build())
        .await;
    let response = mbox.ble_subsystem.read().await.unwrap();
    info!("{}", response);

    info!("config encryption root...");
    mbox.ble_subsystem
        .write_config_data(&ConfigData::encryption_root(&get_erk()).build())
        .await;
    let response = mbox.ble_subsystem.read().await.unwrap();
    info!("{}", response);

    info!("config tx power level...");
    mbox.ble_subsystem.set_tx_power_level(PowerLevel::ZerodBm).await;
    let response = mbox.ble_subsystem.read().await.unwrap();
    info!("{}", response);

    info!("GATT init...");
    mbox.ble_subsystem.init_gatt().await;
    let response = mbox.ble_subsystem.read().await.unwrap();
    info!("{}", response);

    info!("GAP init...");
    mbox.ble_subsystem
        .init_gap(Role::PERIPHERAL, false, BLE_GAP_DEVICE_NAME_LENGTH)
        .await;
    let response = mbox.ble_subsystem.read().await.unwrap();
    info!("{}", response);

    // info!("set scan response...");
    // mbox.ble_subsystem.le_set_scan_response_data(&[]).await.unwrap();
    // let response = mbox.ble_subsystem.read().await.unwrap();
    // info!("{}", response);

    info!("set discoverable...");
    mbox.ble_subsystem
        .set_discoverable(&DiscoverableParameters {
            advertising_type: AdvertisingType::NonConnectableUndirected,
            advertising_interval: Some((Duration::from_millis(250), Duration::from_millis(250))),
            address_type: OwnAddressType::Public,
            filter_policy: AdvertisingFilterPolicy::AllowConnectionAndScan,
            local_name: None,
            advertising_data: &[],
            conn_interval: (None, None),
        })
        .await
        .unwrap();

    let response = mbox.ble_subsystem.read().await;
    info!("{}", response);

    // remove some advertisement to decrease the packet size
    info!("delete tx power ad type...");
    mbox.ble_subsystem
        .delete_ad_type(AdvertisingDataType::TxPowerLevel)
        .await;
    let response = mbox.ble_subsystem.read().await.unwrap();
    info!("{}", response);

    info!("delete conn interval ad type...");
    mbox.ble_subsystem
        .delete_ad_type(AdvertisingDataType::PeripheralConnectionInterval)
        .await;
    let response = mbox.ble_subsystem.read().await.unwrap();
    info!("{}", response);

    info!("update advertising data...");
    mbox.ble_subsystem
        .update_advertising_data(&eddystone_advertising_data())
        .await
        .unwrap();
    let response = mbox.ble_subsystem.read().await.unwrap();
    info!("{}", response);

    info!("update advertising data type...");
    mbox.ble_subsystem
        .update_advertising_data(&[3, AdvertisingDataType::UuidCompleteList16 as u8, 0xaa, 0xfe])
        .await
        .unwrap();
    let response = mbox.ble_subsystem.read().await.unwrap();
    info!("{}", response);

    info!("update advertising data flags...");
    mbox.ble_subsystem
        .update_advertising_data(&[
            2,
            AdvertisingDataType::Flags as u8,
            (0x02 | 0x04) as u8, // BLE general discoverable, without BR/EDR support
        ])
        .await
        .unwrap();
    let response = mbox.ble_subsystem.read().await.unwrap();
    info!("{}", response);

    info!("Test OK");
    cortex_m::asm::bkpt();
}

fn get_bd_addr() -> BdAddr {
    let mut bytes = [0u8; 6];

    let lhci_info = LhciC1DeviceInformationCcrp::new();
    bytes[0] = (lhci_info.uid64 & 0xff) as u8;
    bytes[1] = ((lhci_info.uid64 >> 8) & 0xff) as u8;
    bytes[2] = ((lhci_info.uid64 >> 16) & 0xff) as u8;
    bytes[3] = lhci_info.device_type_id;
    bytes[4] = (lhci_info.st_company_id & 0xff) as u8;
    bytes[5] = (lhci_info.st_company_id >> 8 & 0xff) as u8;

    BdAddr(bytes)
}

fn get_random_addr() -> BdAddr {
    let mut bytes = [0u8; 6];

    let lhci_info = LhciC1DeviceInformationCcrp::new();
    bytes[0] = (lhci_info.uid64 & 0xff) as u8;
    bytes[1] = ((lhci_info.uid64 >> 8) & 0xff) as u8;
    bytes[2] = ((lhci_info.uid64 >> 16) & 0xff) as u8;
    bytes[3] = 0;
    bytes[4] = 0x6E;
    bytes[5] = 0xED;

    BdAddr(bytes)
}

const BLE_CFG_IRK: [u8; 16] = [
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
];
const BLE_CFG_ERK: [u8; 16] = [
    0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65, 0x43, 0x21, 0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65, 0x43, 0x21,
];

fn get_irk() -> EncryptionKey {
    EncryptionKey(BLE_CFG_IRK)
}

fn get_erk() -> EncryptionKey {
    EncryptionKey(BLE_CFG_ERK)
}

fn eddystone_advertising_data() -> [u8; 24] {
    const EDDYSTONE_URL: &[u8] = b"www.rust-lang.com";

    let mut service_data = [0u8; 24];
    let url_len = EDDYSTONE_URL.len();

    service_data[0] = 6 + url_len as u8;
    service_data[1] = AdvertisingDataType::ServiceData as u8;

    // 16-bit eddystone uuid
    service_data[2] = 0xaa;
    service_data[3] = 0xFE;

    service_data[4] = 0x10; // URL frame type
    service_data[5] = 22_i8 as u8; // calibrated TX power at 0m
    service_data[6] = 0x03; // eddystone url prefix = https

    service_data[7..(7 + url_len)].copy_from_slice(EDDYSTONE_URL);

    service_data
}
