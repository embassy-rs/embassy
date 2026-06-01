//! BLE Numeric-Comparison Bonded Example (mirrors ST `BLE_Privacy_Peripheral`)
//!
//! Demonstrates MITM-protected bonding using LE Secure Connections Numeric
//! Comparison, with **controller privacy enabled** so iOS can reconnect using
//! RPAs. Bonds are persisted to flash via `BLEPLAT_NvmStore` and restored on boot.
//!
//! Security model (aligned with ST `BLE_Privacy_Peripheral`):
//! - `aci_gap_init` with privacy enabled → peripheral advertises with an RPA
//! - Resolving list restored from bonds (mode `0x01`); advertising filter stays open
//! - `IoCapability::DisplayYesNo` + SC Optional → Numeric Comparison
//! - Identity address type **Public** (ST `CFG_BD_ADDRESS_DEVICE`)
//!
//! Hardware: STM32WBA65 or compatible.
//!
//! ## iPhone testing
//!
//! 1. Flash and open RTT (`probe-rs run --chip STM32WBA65RI`).
//! 2. In **nRF Connect** or **LightBlue**: connect to "Embassy-Bond".
//! 3. Read characteristic **0xBEF0** (service 0xBEEF) to trigger pairing.
//! 4. RTT shows `CONFIRM ON PHONE: NNNNNN` (iOS often auto-confirms).
//! 5. Expect `PAIRING COMPLETE` and `Link encrypted`.
//! 6. Disconnect in the app, reconnect — expect `Link encrypted` without a new
//!    pairing prompt (disconnect handler refreshes resolving list + FAL).
//!    If reconnect fails, set `ERASE_BONDS_ON_BOOT = true` once and re-pair.
//!
//! **iOS Settings → Bluetooth does not show generic BLE advertisers** — use
//! nRF Connect or LightBlue to scan for "Embassy-Bond".
//!
//! If the app scan is empty, a stale bond may remain in device flash (reflash
//! does not erase the bond NVM page). Set `ERASE_BONDS_ON_BOOT` to `true` once,
//! rebuild, flash, then set it back to `false` for reconnect testing.
//!
//! **nRF Connect hangs on "Connecting"** after you reflash the board: the phone
//! still has an old bond but the MCU does not. In nRF Connect: disconnect →
//! menu on the device → **Forget bond** / remove from cache, then scan again.
//! Or set `ERASE_BONDS_ON_BOOT = true` and pair fresh on both sides.
//!
//! ## Invariant
//!
//! `set_authentication_requirements` is called exactly once at boot.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::peripherals::{AES, PKA, RNG};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts, rcc};
use embassy_stm32_wpan::bluetooth::gap::types::OwnAddressType;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::bluetooth::gap_init::{AddressType, GapInitParams, IoCapability as GapIoCapability};
use embassy_stm32_wpan::bluetooth::gatt::{CharProperties, GattEventMask, SecurityPermissions, ServiceType, Uuid};
use embassy_stm32_wpan::bluetooth::hci::AdvFilterPolicy;
use embassy_stm32_wpan::bluetooth::security::{
    IdentityAddressType, IoCapability, SecureConnectionsSupport, SecurityParams,
};
use embassy_stm32_wpan::{
    HighInterruptHandler, LowInterruptHandler, Platform, erase_bond_nvm_flash, new_platform, set_nvm_base_address,
};
use stm32wb_hci::Event;
use stm32wb_hci::event::{Encryption, EncryptionChange};
use stm32wb_hci::vendor::event::{GapPairingComplete, GapPairingStatus, VendorEvent};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    AES => aes::InterruptHandler<AES>;
    PKA => pka::InterruptHandler<PKA>;
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
});

/// GATT service/characteristic UUIDs for the demo "bonded data" service
const BOND_SERVICE_UUID: u16 = 0xBEEF;
const BOND_CHAR_UUID: u16 = 0xBEF0;

/// Erase bond NVM **before** BLE init. Set `true` once to clear stale bonds, flash, then `false`.
const ERASE_BONDS_ON_BOOT: bool = false;

#[embassy_executor::task]
async fn ble_runner_task(platform: &'static Platform) {
    platform.run_ble().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc = rcc::Config::new_wpan();

    let p = embassy_stm32::init(config);

    info!("Embassy STM32WBA6 BLE Pairing Code Example");

    // Configure the last 8KB page of BANK_2 for bond storage.
    // Must be called before new_platform! triggers BleStack_Init.
    set_nvm_base_address(0x081F_E000);
    if ERASE_BONDS_ON_BOOT {
        if erase_bond_nvm_flash() {
            info!("Bond NVM flash page erased — next boot should not restore bonds");
        } else {
            warn!("Bond NVM flash erase failed");
        }
    }

    let (platform, runtime) = new_platform!(
        Rng::new(p.RNG, Irqs),
        Pka::new(p.PKA, Irqs),
        Aes::new_blocking(p.AES, Irqs),
        8
    );

    spawner.spawn(ble_runner_task(platform).expect("ble runner"));

    // ST BLE_Privacy_Peripheral: public peripheral identity + controller privacy.
    let gap_params = GapInitParams {
        privacy_enabled: true,
        address_type: AddressType::Public,
        bd_addr: st_public_bd_addr(),
        device_name: b"Embassy-Bond",
        io_capability: GapIoCapability::DisplayYesNo,
        skip_authentication_setup: true,
        ..GapInitParams::default()
    };
    let mut ble = embassy_stm32_wpan::bluetooth::HCI::new_with_gap_params(platform, runtime, Irqs, gap_params)
        .await
        .expect("BLE init failed");

    // ── Security ──────────────────────────────────────────────────────────────
    let mut security = ble.security_manager();

    // DisplayYesNo + MITM + SC Optional — matches ST BLE_Privacy_Peripheral.
    let params = SecurityParams::new()
        .with_bonding(true)
        .with_mitm_protection(true)
        .with_io_capability(IoCapability::DisplayYesNo)
        .with_secure_connections(SecureConnectionsSupport::Optional)
        .with_key_size_range(7, 16)
        .with_identity_address_type(IdentityAddressType::Public);

    security
        .set_authentication_requirements(params)
        .expect("set security params");

    // ST: initialize whitelist when bonding is enabled.
    security
        .configure_filter_accept_list()
        .expect("configure_filter_accept_list");

    if ERASE_BONDS_ON_BOOT {
        security.clear_security_database().expect("clear_security_database");
        info!("Bond RAM database cleared (ERASE_BONDS_ON_BOOT)");
    }

    // Cold boot with bonds in NVM: program FAL + resolving list (ST privacy flow).
    match security.configure_filter_and_resolving_list() {
        Ok(count) => {
            if count > 0 {
                info!("Bond lists configured at boot ({} peer(s))", count);
            }
        }
        Err(e) => warn!("configure_filter_and_resolving_list at boot failed: {:?}", e),
    }

    // Dump the controller's view of the resolving list (debug). Useful for verifying that
    // the bond's IRK made it into the LL resolving list — if `peer_rpa` here matches the
    // identity address (instead of being a valid RPA), the LL has `peer_irk = 0` and will
    // silently drop incoming CONNECT_INDs from a bonded RPA-using peer like iOS.
    security.log_resolving_list_diagnostics();

    // ── GATT ──────────────────────────────────────────────────────────────────
    //
    // One characteristic that requires an authenticated, encrypted link.
    // Accessing it before pairing completes will return "Insufficient Authentication".
    let mut gatt = ble.gatt_server();

    let svc = gatt
        .add_service(Uuid::from_u16(BOND_SERVICE_UUID), ServiceType::Primary, 4)
        .expect("add service");

    let ch = gatt
        .add_characteristic(
            svc,
            Uuid::from_u16(BOND_CHAR_UUID),
            20,
            CharProperties::READ | CharProperties::WRITE,
            SecurityPermissions::ENCRY_READ | SecurityPermissions::ENCRY_WRITE,
            GattEventMask::ATTRIBUTE_MODIFIED,
            0,
            true,
        )
        .expect("add characteristic");

    gatt.update_characteristic_value(svc, ch, 0, b"Hello, bonded!")
        .expect("set initial value");

    // ── Advertising ───────────────────────────────────────────────────────────
    let adv_params = make_adv_params();

    fn make_adv_data() -> AdvData {
        let mut d = AdvData::new();
        d.add_flags(0x06).unwrap();
        d.add_name("Embassy-Bond").unwrap();
        d.add_service_uuid_16(BOND_SERVICE_UUID).unwrap();
        d
    }

    let mut scan_rsp = AdvData::new();
    scan_rsp.add_name("Embassy-Bond").expect("scan rsp name");

    ble.start_advertising(adv_params.clone(), make_adv_data(), Some(scan_rsp))
        .await
        .expect("start advertising");

    info!("Advertising as 'Embassy-Bond' — waiting for connection");

    // ── Event loop ────────────────────────────────────────────────────────────
    loop {
        let event = ble.read_event().await;

        if let Some(gap_event) = ble.process_event(&event) {
            match gap_event {
                GapEvent::Connected(conn) => {
                    if let Some(rpa) = conn.peer_rpa {
                        info!(
                            "Connected: handle=0x{:04X} peer={} peer_rpa={}",
                            conn.handle.0, conn.peer_address, rpa
                        );
                    } else {
                        info!("Connected: handle=0x{:04X} peer={}", conn.handle.0, conn.peer_address);
                    }
                }

                GapEvent::Disconnected { handle, reason } => {
                    info!("Disconnected: handle=0x{:04X} reason=0x{:02X}", handle.0, reason);
                    // Match ST BLE_Privacy_Peripheral: do nothing on disconnect. The boot-time
                    // configure_filter_and_resolving_list call already populated the controller
                    // resolving list, and the stack auto-resumes the previous advertising set
                    // after HCI_Disconnection_Complete. Re-appending bonds here (mode 0x04) or
                    // tearing down + restarting advertising loses controller-side resolution
                    // state and breaks RPA-based reconnect from iOS.
                }

                _ => {}
            }
        }

        match &event {
            // ── Numeric comparison (LE SC path — the normal iOS path) ───────
            // Both sides computed the same 6-digit value from the ECDH
            // exchange. We auto-confirm; iOS shows a popup asking the user
            // to confirm the same value shown here.
            Event::Vendor(VendorEvent::GapNumericComparisonValue(ev)) => {
                info!("");
                info!("╔══════════════════════════════════╗");
                info!("║  CONFIRM ON PHONE: {:06}          ║", ev.numeric_value);
                info!("╚══════════════════════════════════╝");
                info!("");
                if let Err(e) = security.numeric_comparison_response(ev.connection_handle.0, true) {
                    error!("numeric_comparison_response error: {:?}", e);
                }
            }

            // ── Pairing result ──────────────────────────────────────────────
            Event::Vendor(VendorEvent::GapPairingComplete(GapPairingComplete { conn_handle, status })) => {
                match status {
                    GapPairingStatus::Success => {
                        info!(
                            "PAIRING COMPLETE: handle=0x{:04X} — encrypted and bonded",
                            conn_handle.0
                        );
                        security.log_bonded_devices();
                    }
                    GapPairingStatus::Timeout(reason) => {
                        warn!("Pairing timed out: handle=0x{:04X} reason={:?}", conn_handle.0, reason);
                    }
                    GapPairingStatus::Failed(reason) => {
                        warn!(
                            "Pairing failed: handle=0x{:04X} reason={:?} — wrong code entered?",
                            conn_handle.0, reason
                        );
                    }
                    GapPairingStatus::EncryptionFailed(reason) => {
                        warn!("Encryption failed: handle=0x{:04X} reason={:?}", conn_handle.0, reason);
                    }
                }
            }

            // ── Encryption state (also fires on reconnect with stored bond) ─
            Event::EncryptionChange(EncryptionChange {
                conn_handle,
                encryption,
                ..
            }) => match encryption {
                Encryption::On | Encryption::OnAesCcmForBrEdr => {
                    info!("Link encrypted: handle=0x{:04X}", conn_handle.0);
                }
                Encryption::Off => {
                    warn!("Encryption disabled: handle=0x{:04X}", conn_handle.0);
                }
            },

            // ── Bond lost / LTK mismatch: allow re-pair and restart open advertising ─
            Event::Vendor(VendorEvent::GapBondLost) => {
                info!("Bond lost — allowing rebond and restarting advertising");
                let conn_handle = ble.connections().iter().next().map(|c| c.handle.0).unwrap_or(0);
                let _ = security.allow_rebond(conn_handle);
                if ble.is_advertising() {
                    let _ = ble.stop_advertising().await;
                }
                let mut scan_rsp = AdvData::new();
                scan_rsp.add_name("Embassy-Bond").expect("scan rsp name");
                let _ = ble
                    .start_advertising(make_adv_params(), make_adv_data(), Some(scan_rsp))
                    .await;
            }

            // ── Authenticated write received from bonded peer ─────────────
            Event::Vendor(VendorEvent::GattAttributeModified(attr)) => {
                info!(
                    "Authenticated write: conn=0x{:04X} attr=0x{:04X} data={:?}",
                    attr.conn_handle,
                    attr.attr_handle,
                    attr.data()
                );
            }

            _ => {
                debug!("Event: {:?}", event);
            }
        }
    }
}

/// ST-style public address (CFG_BD_ADDRESS_DEVICE = GAP_PUBLIC_ADDR).
fn st_public_bd_addr() -> [u8; 6] {
    let uid = embassy_stm32::uid::uid();
    [uid[0], uid[1], uid[2], 0xE1, 0x80, 0x00]
}

fn make_adv_params() -> AdvParams {
    AdvParams {
        interval_min: 0x0080,
        interval_max: 0x0080,
        adv_type: AdvType::ConnectableUndirected,
        // 0x02 = resolvable private address when controller privacy is enabled.
        own_addr_type: OwnAddressType::PrivateFallbackPublic,
        filter_policy: AdvFilterPolicy::All,
        channel_map: 0x07,
        // Use set_discoverable + le_set_advertise_enable (undirected path can hang centrals).
        privacy_undirected: false,
    }
}
