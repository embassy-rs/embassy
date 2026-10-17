// ! GAP and HAL Initialization Functions
// !
// ! This module provides the low-level initialization functions required
// ! to properly configure the BLE stack's GAP layer and HAL settings.
// !
// ! Based on ST's Ble_Hci_Gap_Gatt_Init() from BLE_HeartRate example.

use bt_hci::cmd::le::LeSetDefaultPhy;
use bt_hci::controller::ControllerCmdSync;
use bt_hci::param::{AllPhys, PhyMask};
use stm32wb_hci::aci::flags::Role;
use stm32wb_hci::aci::gap::{GapInit, GapSetAuthenticationRequirement, GapSetIoCapability};
use stm32wb_hci::aci::gatt::GattUpdateCharValue;
use stm32wb_hci::aci::general::{AciReadConfigData, AciWriteConfigData};
use stm32wb_hci::aci::hal::HalSetTxPowerLevel;
use stm32wb_hci::aci::ranges::{PaLevel, Passkey};
use stm32wb_hci::aci::values::{
    AddressType, ConfigDataOffset, IoCapability, Privacy, ReadableConfigDataOffset, ScSupport, UseFixedPin,
};

use crate::bluetooth::error::BleError;
use crate::controller::ControllerAdapter;

#[allow(dead_code)]
const GAP_CENTRAL_ROLE: u8 = 0x04;
#[allow(dead_code)]
const GAP_OBSERVER_ROLE: u8 = 0x08;

/// GAP initialization parameters
pub struct GapInitParams {
    /// Device role (peripheral, central, or both)
    pub role: Role,
    /// Privacy mode enabled
    pub privacy_enabled: bool,
    /// Device name
    pub device_name: &'static [u8],
    /// GAP appearance value
    pub appearance: u16,
    /// BD address (6 bytes). For `RandomStatic`, `init_gap_and_hal` enforces the top two bits of byte 5 are `0b11`.
    pub bd_addr: [u8; 6],
    /// Whether `bd_addr` is a public or random static address.
    pub address_type: AddressType,
    /// Identity Root key (16 bytes) for IRK derivation
    pub ir_value: [u8; 16],
    /// Encryption Root key (16 bytes) for LTK derivation
    pub er_value: [u8; 16],
    /// TX power level (0-31, expressed as PaLevel). Default chosen to match previous numeric 25.
    pub tx_power: PaLevel,
    /// PHY preferences
    pub phy_prefs: PhyPrefs,
    /// IO capability
    pub io_capability: IoCapability,
    /// Security parameters
    pub security: SecurityParams,
    /// When true, skip `aci_gap_set_authentication_requirement` in GAP init so the
    /// application can configure auth exactly once via [`SecurityManager`].
    pub skip_authentication_setup: bool,
}

/// PHY preferences
#[derive(Debug, Clone, Copy)]
pub struct PhyPrefs {
    /// All PHYs (0 = host has preference, 1 = host has no preference)
    pub all_phys: AllPhys,
    /// TX PHY preference (1M, 2M, Coded)
    pub tx_phys: PhyMask,
    /// RX PHY preference (1M, 2M, Coded)
    pub rx_phys: PhyMask,
}

impl Default for PhyPrefs {
    fn default() -> Self {
        Self {
            all_phys: AllPhys::new(),                                        // Host has preference
            tx_phys: PhyMask::new().set_le_1m_phy(true).set_le_2m_phy(true), // 1M and 2M
            rx_phys: PhyMask::new().set_le_1m_phy(true).set_le_2m_phy(true), // 1M and 2M
        }
    }
}

/// Security parameters
#[derive(Debug, Clone, Copy)]
pub struct SecurityParams {
    /// Bonding mode (0 = no bonding, 1 = bonding)
    pub bonding_mode: bool,
    /// MITM protection required
    pub mitm_mode: bool,
    /// Secure Connections support
    pub sc_support: ScSupport,
    /// Key press notification support
    pub key_press_notification_support: bool,
    /// Minimum encryption key size (7-16)
    pub min_encryption_key_size: u8,
    /// Maximum encryption key size (7-16)
    pub max_encryption_key_size: u8,
    /// Use fixed PIN
    pub use_fixed_pin: UseFixedPin,
    /// Fixed PIN value (0-999999)
    pub fixed_pin: Passkey,
    /// Identity address type (0 = public, 1 = random static)
    pub identity_address_type: AddressType,
}

impl Default for SecurityParams {
    fn default() -> Self {
        Self {
            bonding_mode: true,
            mitm_mode: true,
            sc_support: ScSupport::Optional,
            key_press_notification_support: false,
            min_encryption_key_size: 8,
            max_encryption_key_size: 16,
            use_fixed_pin: UseFixedPin::Yes,
            fixed_pin: Passkey::new(111111).unwrap(),
            identity_address_type: AddressType::Public, // Public address
        }
    }
}

impl Default for GapInitParams {
    fn default() -> Self {
        Self {
            role: Role::PERIPHERAL,
            privacy_enabled: false,
            device_name: b"Embassy-BLE",
            appearance: 0, // Unknown
            // Random static address: top two bits of byte 5 are `0b11` (enforced by init_gap_and_hal).
            // Callers that want a per-chip address should overwrite all 6 bytes, e.g. from the chip UID.
            bd_addr: [0x00, 0x1A, 0x26, 0xE1, 0x80, 0xC1],
            // stm32wb-hci AddressType::Random documents a static identity address (same semantics as the old RandomStatic)
            address_type: AddressType::Random,

            ir_value: [0x12; 16],
            er_value: [0x34; 16],
            // Preserve the previous numeric default (25) by selecting the corresponding PaLevel.
            tx_power: PaLevel::new(25).unwrap(),
            phy_prefs: PhyPrefs::default(),
            io_capability: IoCapability::NoInputNoOutput,
            security: SecurityParams::default(),
            skip_authentication_setup: false,
        }
    }
}

/// GAP initialization result
pub struct GapHandles {
    pub gap_service_handle: u16,
    pub gap_dev_name_char_handle: u16,
    pub gap_appearance_char_handle: u16,
}

/// Initialize GAP layer and configure HAL settings
///
/// This function performs the following steps (based on ST's Ble_Hci_Gap_Gatt_Init):
/// 1. Writes BD address to BLE stack
/// 2. Writes IR and ER values for security
/// 3. Sets TX power level
/// 4. Initializes GAP layer with role and device name
/// 5. Updates device name and appearance characteristics
/// 6. Sets default PHY
/// 7. Sets IO capability
/// 8. Sets authentication requirements
///
/// Must be called after BleStack_Init() and aci_gatt_init().
pub async fn init_gap_and_hal(
    controller: &ControllerAdapter<'_>,
    params: &mut GapInitParams,
) -> Result<GapHandles, BleError> {
    // 1. Write additional GAP service record number (for Device Info service, etc.)
    let _ = controller
        .exec(&AciWriteConfigData::try_new(ConfigDataOffset::GapAdditionalRecordNumber, &[0x03]).unwrap())
        .await
        .inspect_err(|e| warn!("aci_hal_write_config_data (GAP_ADD_REC_NBR) failed: {}", e));

    // 2. Write BD address (only for public addresses).
    //
    // For random static addresses the PUBADDR write is skipped — the WBA
    // stack-basic library rejects it, and the address is handled separately.
    match params.address_type {
        AddressType::Public => {
            controller
                .exec(&AciWriteConfigData::try_new(ConfigDataOffset::PublicAddress, &params.bd_addr).unwrap())
                .await
                .inspect_err(|e| warn!("aci_hal_write_config_data (PUBADDR) failed: {}", e))?;

            info!(
                "Public BD address configured: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                params.bd_addr[5],
                params.bd_addr[4],
                params.bd_addr[3],
                params.bd_addr[2],
                params.bd_addr[1],
                params.bd_addr[0]
            );
        }
        AddressType::Random => {
            // Per BT Core Spec Vol 6 Part B §1.3.2.1: the top two bits of the most
            // significant byte must be `11` to identify this as a random static address,
            // distinguishing it from resolvable (10) and non-resolvable (01) private types.
            params.bd_addr[5] |= 0xC0;

            match controller
                .exec(&AciReadConfigData::new(ReadableConfigDataOffset::StaticRandomAddress))
                .await
            {
                Ok(resp) => {
                    if resp.data.len() >= 6 {
                        let addr: [u8; 6] = resp.data[..6].try_into().unwrap();
                        info!(
                            "Random static address: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                            addr[5], addr[4], addr[3], addr[2], addr[1], addr[0]
                        );
                    } else {
                        warn!(
                            "aci_hal_read_config_data returned too-short random address: {} bytes",
                            resp.data.len()
                        );
                    }
                }
                Err(e) => warn!("aci_hal_read_config_data (StaticRandomAddress) failed: {}", e),
            }
        }
    }

    // 3. Write IR (Identity Root) value (non-fatal)
    let _ = controller
        .exec(&AciWriteConfigData::try_new(ConfigDataOffset::IdentityRoot, &params.ir_value).unwrap())
        .await
        .inspect_err(|e| warn!("aci_hal_write_config_data (IR) failed: {}", e));

    // 4. Write ER (Encryption Root) value (non-fatal)
    let _ = controller
        .exec(&AciWriteConfigData::try_new(ConfigDataOffset::EncryptionRoot, &params.er_value).unwrap())
        .await
        .inspect_err(|e| warn!("aci_hal_write_config_data (ER) failed: {}", e));

    // 5. Set TX power level (non-fatal)
    let _ = controller
        .exec(&HalSetTxPowerLevel::new(true, params.tx_power))
        .await
        .inspect_err(|e| warn!("aci_hal_set_tx_power_level failed: {}", e));

    let service = controller
        .exec(&GapInit::new(
            params.role,
            if params.privacy_enabled {
                Privacy::Enabled
            } else {
                Privacy::Disabled
            },
            params.device_name.len() as u8,
        ))
        .await
        .inspect_err(|e| warn!("aci_gap_init failed: {}", e))?;

    info!(
        "GAP initialized - service: 0x{:04X}, name: 0x{:04X}, appearance: 0x{:04X}",
        service.service_handle, service.dev_name_char_handle, service.appearance_char_handle
    );

    // 7. Update device name characteristic (non-fatal)
    let _ = controller
        .exec(
            &GattUpdateCharValue::try_new(
                service.service_handle,
                service.dev_name_char_handle,
                0,
                params.device_name,
            )
            .unwrap(),
        )
        .await
        .inspect_err(|e| warn!("aci_gatt_update_char_value (device name) failed: {}", e));

    // 8. Update appearance characteristic (non-fatal)
    let _ = controller
        .exec(
            &GattUpdateCharValue::try_new(
                service.service_handle,
                service.appearance_char_handle,
                0,
                &params.appearance.to_le_bytes(),
            )
            .unwrap(),
        )
        .await
        .inspect_err(|e| warn!("aci_gatt_update_char_value (appearance) failed: {}", e));

    // 9. Set default PHY (non-fatal)
    let _ = controller
        .exec(&LeSetDefaultPhy::new(
            params.phy_prefs.all_phys,
            params.phy_prefs.tx_phys,
            params.phy_prefs.rx_phys,
        ))
        .await
        .inspect_err(|e| warn!("hci_le_set_default_phy failed: {}", e));

    // 10. Set IO capability (non-fatal)
    let _ = controller
        .exec(&GapSetIoCapability::new(params.io_capability))
        .await
        .inspect_err(|e| warn!("aci_gap_set_io_capability failed: {}", e));

    // 11. Set authentication requirements (optional — apps that use SecurityManager
    // should set skip_authentication_setup to avoid calling this twice).

    if !params.skip_authentication_setup {
        let _ = controller
            .exec(&GapSetAuthenticationRequirement::new(
                params.security.bonding_mode,
                params.security.mitm_mode,
                params.security.sc_support,
                params.security.key_press_notification_support,
                params.security.min_encryption_key_size,
                params.security.max_encryption_key_size,
                params.security.use_fixed_pin,
                params.security.fixed_pin,
                params.security.identity_address_type,
            ))
            .await
            .inspect_err(|e| warn!("aci_gap_set_authentication_requirement failed: {}", e));
    }

    info!("GAP and HAL configuration completed successfully");

    Ok(GapHandles {
        gap_service_handle: service.service_handle,
        gap_dev_name_char_handle: service.dev_name_char_handle,
        gap_appearance_char_handle: service.appearance_char_handle,
    })
}
