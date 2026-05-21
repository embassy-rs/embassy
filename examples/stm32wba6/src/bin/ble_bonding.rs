//! BLE Numeric-Comparison Bonded Example (mirrors ST `BLE_p2pServer`)
//!
//! Demonstrates MITM-protected bonding using LE Secure Connections Numeric
//! Comparison. The device displays a 6-digit value derived from the ECDH key
//! exchange; iOS auto-confirms via CoreBluetooth, Android shows a confirm prompt.
//! After pairing, the bond (LTK/IRK) is persisted to the last 8 KB page of
//! flash via `BLEPLAT_NvmStore` and restored on the next boot.
//!
//! Security model:
//! - `IoCapability::DisplayYesNo` + SC → Numeric Comparison
//! - MITM Required, Bonding enabled, key size 7..16
//! - `SecureConnectionsSupport::Optional` (matches the ST reference)
//! - Privacy disabled at the GAP layer; bonds are restored into the controller's
//!   Filter Accept List via `aci_gap_configure_filter_accept_list()` (the
//!   maintained `aci_gap_configure_whitelist()` from ST's reference)
//!
//! Hardware: STM32WBA65 or compatible.
//!
//! ## Known limitation: iOS bonded reconnect
//!
//! iOS connects as a central using **Resolvable Private Addresses** (RPAs).
//! When pairing completes, the ST BLE stack stores the bond keyed by iOS's
//! identity address (sent via SMP IRK distribution). On reconnect, iOS uses an
//! RPA again, so the host needs Controller Privacy + a properly populated
//! resolving list (with explicit IRKs) to map RPA → identity → LTK. That path
//! requires additional integration work — pulling IRKs from the bond DB and
//! calling `hci_le_add_device_to_resolving_list` directly — that this example
//! does not yet do. The visible symptom is HAL warning 0x06 ("SMP unexpected
//! LTK request") on reconnect, after which iOS will fall back to re-pairing.
//!
//! Bonded reconnect **does work** against centrals that use public or static
//! random identity addresses (most Android phones, nRF Connect on desktop).
//!
//! ## To test
//!
//! 1. Flash and open an RTT terminal (`probe-rs run --chip STM32WBA65RI`).
//! 2. nRF Connect on the central: scan, connect to "Embassy-Bond".
//! 3. Tap Read on characteristic BEF0 — pairing starts because the read
//!    requires authentication.
//! 4. The device logs "CONFIRM ON PHONE: NNNNNN"; iOS auto-accepts, Android
//!    prompts the user.
//! 5. Log shows "PAIRING COMPLETE" — link is encrypted and bonded.
//! 6. Disconnect, then reconnect — on Android/desktop centrals the LTK lookup
//!    succeeds and the link re-encrypts automatically (no pairing prompt). On
//!    iOS this currently re-pairs (see "Known limitation" above).
//!
//! ## Invariant
//!
//! `set_authentication_requirements` is called exactly once at boot. Calling it
//! again invalidates the BLE stack's bond-lookup state and triggers HAL firmware
//! warning 0x06 on the next reconnect.

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
use embassy_stm32_wpan::bluetooth::gatt::{CharProperties, GattEventMask, SecurityPermissions, ServiceType, Uuid};
use embassy_stm32_wpan::bluetooth::security::{IoCapability, SecureConnectionsSupport, SecurityParams};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform, set_nvm_base_address};
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

#[embassy_executor::task]
async fn rng_runner_task(platform: &'static Platform) {
    platform.run_rng().await
}

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

    let (platform, runtime) = new_platform!(
        Rng::new(p.RNG, Irqs),
        Aes::new_blocking(p.AES, Irqs),
        Pka::new_blocking(p.PKA, Irqs),
        8
    );

    spawner.spawn(rng_runner_task(platform).expect("rng runner"));
    spawner.spawn(ble_runner_task(platform).expect("ble runner"));

    // Use a fixed static random address. Matching ST's BLE_p2pServer pattern:
    // privacy disabled at the GAP layer, bonded reconnect works at the LL via
    // EDIV/RAND lookup (LE Legacy fallback), no resolving list / RPA resolution
    // needed on the host.
    let gap_params = embassy_stm32_wpan::bluetooth::gap_init::GapInitParams {
        bd_addr: [0x01, 0x02, 0x03, 0x04, 0x05, 0xC0],
        address_type: embassy_stm32_wpan::bluetooth::gap_init::AddressType::RandomStatic,
        privacy_enabled: false,
        ..embassy_stm32_wpan::bluetooth::gap_init::GapInitParams::default()
    };
    let mut ble = embassy_stm32_wpan::bluetooth::HCI::new_with_gap_params(platform, runtime, Irqs, gap_params)
        .await
        .expect("BLE init failed");

    // ── Security ──────────────────────────────────────────────────────────────
    let mut security = ble.security_manager();

    // DisplayYesNo + MITM + SC Optional — matches ST's BLE_p2pServer reference.
    // SC Optional (not Required) lets iOS fall back to LE Legacy so reconnect
    // uses EDIV/RAND-based LTK lookup, which the ST stack handles cleanly.
    let params = SecurityParams::new()
        .with_bonding(true)
        .with_mitm_protection(true)
        .with_io_capability(IoCapability::DisplayYesNo)
        .with_secure_connections(SecureConnectionsSupport::Optional)
        .with_key_size_range(7, 16);

    security
        .set_authentication_requirements(params)
        .expect("set security params");

    // Restore bonded peers into the Filter Accept List (matches the
    // `aci_gap_configure_whitelist()` call in ST's BLE_p2pServer). This is what
    // makes bonded reconnect work: on LL_ENC_REQ the controller looks up the
    // LTK by EDIV/RAND directly, with the bond identified via the FAL.
    if let Err(e) = security.configure_filter_accept_list() {
        warn!("configure_filter_accept_list failed: {:?}", e);
    } else {
        info!("Filter Accept List configured from bonded devices");
    }

    // No HCI address resolution / resolving list — ST's BLE_p2pServer pattern
    // relies on LL-layer EDIV/RAND lookup for bonded reconnect, which doesn't
    // require any host-side privacy plumbing.

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
    let adv_params = AdvParams {
        interval_min: 0x0050,
        interval_max: 0x0050,
        adv_type: AdvType::ConnectableUndirected,
        own_addr_type: OwnAddressType::Random,
        ..AdvParams::default()
    };

    fn make_adv_data() -> AdvData {
        let mut d = AdvData::new();
        d.add_flags(0x06).unwrap();
        d.add_name("Embassy-Bond").unwrap();
        d.add_service_uuid_16(BOND_SERVICE_UUID).unwrap();
        d
    }

    ble.start_advertising(adv_params.clone(), make_adv_data(), None)
        .await
        .expect("start advertising");

    info!("Advertising as 'Embassy-Bond' — waiting for connection");

    // ── Event loop ────────────────────────────────────────────────────────────
    loop {
        let event = ble.read_event().await;

        if let Some(gap_event) = ble.process_event(&event) {
            match gap_event {
                GapEvent::Connected(conn) => {
                    info!("Connected: handle=0x{:04X} peer={}", conn.handle.0, conn.peer_address);
                    // Bonded peers re-encrypt with the stored LTK automatically.
                    // Fresh pairings are triggered by reading BEF0 (ENCRY_READ → ATT 0x0F).
                }

                GapEvent::Disconnected { handle, reason } => {
                    info!("Disconnected: handle=0x{:04X} reason=0x{:02X}", handle.0, reason);

                    // Do NOT call set_authentication_requirements here — re-running it
                    // between sessions appears to invalidate the bond's LTK lookup state,
                    // making the next bonded reconnect fail with "SMP unexpected LTK
                    // request". The passkey set at boot remains valid for fresh pairings.

                    ble.start_advertising(adv_params.clone(), make_adv_data(), None)
                        .await
                        .expect("restart advertising");
                    info!("Advertising restarted");
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
            Event::Vendor(VendorEvent::GapPairingComplete(GapPairingComplete {
                conn_handle,
                status,
                reason,
            })) => match status {
                GapPairingStatus::Success => {
                    info!(
                        "PAIRING COMPLETE: handle=0x{:04X} — encrypted and bonded",
                        conn_handle.0
                    );
                }
                GapPairingStatus::Timeout => {
                    warn!("Pairing timed out: handle=0x{:04X}", conn_handle.0);
                }
                GapPairingStatus::Failed => {
                    warn!(
                        "Pairing failed: handle=0x{:04X} reason=0x{:02X} — wrong code entered?",
                        conn_handle.0, reason
                    );
                }
            },

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

            // ── Bond lost: the peer will need to re-pair ─────────────────
            Event::Vendor(VendorEvent::GapBondLost) => {
                info!("Bond lost — peer will need to re-pair with the current code");
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
