//! BLE Eddystone URL beacon example.
#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::panic;

use bluetooth_hci::{
    event::command::ReturnParameters,
    host::{AdvertisingFilterPolicy, EncryptionKey, Hci, OwnAddressType},
    BdAddr,
};
use core::time::Duration;
use cortex_m_rt::entry;
use embassy::executor::{task, Executor};
use embassy::util::Forever;
use embassy_stm32wb55::ble::Ble;
use embassy_stm32wb55::interrupt;
use stm32wb55::event::Stm32Wb5xError;
use stm32wb55::{
    gap::{
        AdvertisingDataType, AdvertisingType, Commands as GapCommands, DiscoverableParameters, Role,
    },
    gatt::{
        CharacteristicHandle, Commands as GattCommads, ServiceHandle,
        UpdateCharacteristicValueParameters,
    },
    hal::{Commands as HalCommands, ConfigData, PowerLevel},
};
use stm32wb_hal::flash::FlashExt;
use stm32wb_hal::prelude::*;
use stm32wb_hal::rcc::{
    ApbDivider, Config, HDivider, HseDivider, PllConfig, PllSrc, RfWakeupClock, RtcClkSrc,
    StopWakeupClock, SysClkSrc,
};
use stm32wb_hal::tl_mbox::lhci::LhciC1DeviceInformationCcrp;
use stm32wb_hal::tl_mbox::shci::ShciBleInitCmdParam;
use stm32wb_hal::tl_mbox::TlMbox;
use stm32wb_hal::{pwr, rtc, stm32};

type BleError =
    embassy_stm32wb55::ble::BleError<bluetooth_hci::host::uart::Error<(), Stm32Wb5xError>>;

// Setup Eddystone beacon to advertise this URL:
// https://www.rust-lang.org
const EDDYSTONE_URL_PREFIX: EddystoneUrlScheme = EddystoneUrlScheme::Https;
const EDDYSTONE_URL: &[u8] = b"www.rust-lang.com";

/// Advertisement interval in milliseconds.
const ADV_INTERVAL_MS: u64 = 250;

/// TX power at 0 m range. Used for range approximation.
const CALIBRATED_TX_POWER_AT_0_M: u8 = -22_i8 as u8;

/// Transmission power for the beacon.
const TX_POWER: PowerLevel = PowerLevel::ZerodBm;

#[derive(Debug)]
pub struct BleContext {
    service_handle: ServiceHandle,
    dev_name_handle: CharacteristicHandle,
    appearance_handle: CharacteristicHandle,
}

#[task]
async fn run(dp: stm32::Peripherals, _cp: cortex_m::Peripherals) {
    // Allow using debugger and RTT during WFI/WFE (sleep)
    dp.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });
    dp.RCC.ahb1enr.modify(|_, w| w.dma1en().set_bit());

    let mut rcc = dp.RCC.constrain();
    rcc.set_stop_wakeup_clock(StopWakeupClock::HSI16);

    // Fastest clock configuration.
    // * External low-speed crystal is used (LSE)
    // * 32 MHz HSE with PLL
    // * 64 MHz CPU1, 32 MHz CPU2
    // * 64 MHz for APB1, APB2
    // * HSI as a clock source after wake-up from low-power mode
    let clock_config = Config::new(SysClkSrc::Pll(PllSrc::Hse(HseDivider::NotDivided)))
        .with_lse()
        .cpu1_hdiv(HDivider::NotDivided)
        .cpu2_hdiv(HDivider::Div2)
        .apb1_div(ApbDivider::NotDivided)
        .apb2_div(ApbDivider::NotDivided)
        .pll_cfg(PllConfig {
            m: 2,
            n: 12,
            r: 3,
            q: Some(4),
            p: Some(3),
        })
        .rtc_src(RtcClkSrc::Lse)
        .rf_wkp_sel(RfWakeupClock::Lse);

    let mut rcc = rcc.apply_clock_config(clock_config, &mut dp.FLASH.constrain().acr);

    // RTC is required for proper operation of BLE stack
    let _rtc = rtc::Rtc::rtc(dp.RTC, &mut rcc);

    let mut ipcc = dp.IPCC.constrain();
    let mbox = TlMbox::tl_init(&mut rcc, &mut ipcc);

    pwr::set_cpu2(false);

    let config = ShciBleInitCmdParam {
        p_ble_buffer_address: 0,
        ble_buffer_size: 0,
        num_attr_record: 68,
        num_attr_serv: 8,
        attr_value_arr_size: 1344,
        num_of_links: 8,
        extended_packet_length_enable: 1,
        pr_write_list_size: 0x3A,
        mb_lock_count: 0x79,
        att_mtu: 156,
        slave_sca: 500,
        master_sca: 0,
        ls_source: 1,
        max_conn_event_length: 0xFFFFFFFF,
        hs_startup_time: 0x148,
        viterbi_enable: 1,
        ll_only: 0,
        hw_version: 0,
    };

    defmt::info!("Initializing BLE");

    let mut ble = Ble::init(
        interrupt::take!(IPCC_C1_RX_IT),
        interrupt::take!(IPCC_C1_TX_IT),
        config,
        mbox,
        ipcc,
    )
    .await
    .unwrap();

    defmt::info!("BLE Initialized");

    init_gap_and_gatt(&mut ble).await.unwrap();
    init_eddystone(&mut ble).await.unwrap();

    defmt::info!("Eddystone Beacon Ready");
}

async fn init_gap_and_gatt(ble: &mut Ble) -> Result<(), BleError> {
    ble.perform_command(|rc| {
        rc.write_config_data(&ConfigData::public_address(get_bd_addr()).build())
    })
    .await?;
    ble.perform_command(|rc| {
        rc.write_config_data(&ConfigData::random_address(get_random_addr()).build())
    })
    .await?;
    ble.perform_command(|rc| rc.write_config_data(&ConfigData::identity_root(&get_irk()).build()))
        .await?;
    ble.perform_command(|rc| {
        rc.write_config_data(&ConfigData::encryption_root(&get_erk()).build())
    })
    .await?;
    ble.perform_command(|rc| rc.set_tx_power_level(TX_POWER))
        .await?;
    ble.perform_command(|rc| rc.init_gatt()).await?;

    let return_params = ble
        .perform_command(|rc| rc.init_gap(Role::PERIPHERAL, false, BLE_GAP_DEVICE_NAME_LENGTH))
        .await?;
    let ble_context = if let ReturnParameters::Vendor(
        stm32wb55::event::command::ReturnParameters::GapInit(stm32wb55::event::command::GapInit {
            service_handle,
            dev_name_handle,
            appearance_handle,
            ..
        }),
    ) = return_params
    {
        BleContext {
            service_handle,
            dev_name_handle,
            appearance_handle,
        }
    } else {
        defmt::error!("Unexpected response to init_gap command");
        return Err(BleError::UnexpectedEvent);
    };

    ble.perform_command(|rc| {
        rc.update_characteristic_value(&UpdateCharacteristicValueParameters {
            service_handle: ble_context.service_handle,
            characteristic_handle: ble_context.dev_name_handle,
            offset: 0,
            value: b"BEACON",
        })
        .map_err(|_| nb::Error::Other(()))
    })
    .await?;

    Ok(())
}

async fn init_eddystone(ble: &mut Ble) -> Result<(), BleError> {
    // Disable scan response
    ble.perform_command(|rc| {
        rc.le_set_scan_response_data(&[])
            .map_err(|_| nb::Error::Other(()))
    })
    .await?;

    ble.perform_command(|rc| {
        let params = DiscoverableParameters {
            advertising_type: AdvertisingType::NonConnectableUndirected,
            advertising_interval: Some((
                Duration::from_millis(ADV_INTERVAL_MS),
                Duration::from_millis(ADV_INTERVAL_MS),
            )),
            address_type: OwnAddressType::Public,
            filter_policy: AdvertisingFilterPolicy::AllowConnectionAndScan,
            // Local name should be empty for the device to be recognized as an Eddystone beacon
            local_name: None,
            advertising_data: &[],
            conn_interval: (None, None),
        };

        rc.set_discoverable(&params)
            .map_err(|_| nb::Error::Other(()))
    })
    .await?;

    ble.perform_command(|rc| rc.delete_ad_type(AdvertisingDataType::TxPowerLevel))
        .await?;

    ble.perform_command(|rc| rc.delete_ad_type(AdvertisingDataType::PeripheralConnectionInterval))
        .await?;

    ble.perform_command(|rc| {
        let url_len = EDDYSTONE_URL.len();

        let mut service_data = [0u8; 24];
        service_data[0] = 6 + url_len as u8;
        service_data[1] = AdvertisingDataType::ServiceData as u8;

        // 16-bit Eddystone UUID
        service_data[2] = 0xAA;
        service_data[3] = 0xFE;

        service_data[4] = 0x10; // URL frame type
        service_data[5] = CALIBRATED_TX_POWER_AT_0_M;
        service_data[6] = EDDYSTONE_URL_PREFIX as u8;

        service_data[7..(7 + url_len)].copy_from_slice(EDDYSTONE_URL);

        rc.update_advertising_data(&service_data[..])
            .map_err(|_| nb::Error::Other(()))
    })
    .await?;

    ble.perform_command(|rc| {
        let service_uuid_list = [
            3_u8,
            AdvertisingDataType::UuidCompleteList16 as u8,
            0xAA,
            0xFE,
        ];

        rc.update_advertising_data(&service_uuid_list[..])
            .map_err(|_| nb::Error::Other(()))
    })
    .await?;

    ble.perform_command(|rc| {
        let flags = [
            2,
            AdvertisingDataType::Flags as u8,
            (0x02 | 0x04) as u8, // BLE general discoverable, without BR/EDR support.
        ];

        rc.update_advertising_data(&flags[..])
            .map_err(|_| nb::Error::Other(()))
    })
    .await?;

    Ok(())
}

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    defmt::info!("Starting");

    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let executor = EXECUTOR.put(Executor::new(cortex_m::asm::sev));
    executor.spawn(run(dp, cp)).unwrap();

    loop {
        executor.run();
        cortex_m::asm::wfe();
    }
}

// == Utils ==
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

#[derive(Copy, Clone)]
#[allow(dead_code)] // For unused enum variants
enum EddystoneUrlScheme {
    HttpWww = 0x00,
    HttpsWww = 0x01,
    Http = 0x02,
    Https = 0x03,
}

const BLE_GAP_DEVICE_NAME_LENGTH: u8 = 7;
