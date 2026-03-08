//! GAP and HAL Initialization Functions
//!
//! This module provides the low-level initialization functions required
//! to properly configure the BLE stack's GAP layer and HAL settings.
//!
//! Based on ST's Ble_Hci_Gap_Gatt_Init() from BLE_HeartRate example.

use crate::wba::error::BleError;
use crate::wba::hci::types::Status;

#[allow(non_camel_case_types)]
type tBleStatus = u8;

const BLE_STATUS_SUCCESS: u8 = 0x00;

// Config data offsets and lengths (from ST's ble_const.h)
const CONFIG_DATA_PUBADDR_OFFSET: u8 = 0x00;
const CONFIG_DATA_PUBADDR_LEN: u8 = 6;
const CONFIG_DATA_ER_OFFSET: u8 = 0x08;
const CONFIG_DATA_ER_LEN: u8 = 16;
const CONFIG_DATA_IR_OFFSET: u8 = 0x18;
const CONFIG_DATA_IR_LEN: u8 = 16;
const CONFIG_DATA_GAP_ADD_REC_NBR_OFFSET: u8 = 0x2C;
const CONFIG_DATA_GAP_ADD_REC_NBR_LEN: u8 = 1;

// GAP roles
const GAP_PERIPHERAL_ROLE: u8 = 0x01;
#[allow(dead_code)]
const GAP_CENTRAL_ROLE: u8 = 0x04;

#[link(name = "stm32wba_ble_stack_basic")]
unsafe extern "C" {
    /// Initialize GAP layer
    #[link_name = "ACI_GAP_INIT"]
    fn aci_gap_init(
        role: u8,
        privacy_enabled: u8,
        device_name_char_len: u8,
        gap_service_handle: *mut u16,
        gap_dev_name_char_handle: *mut u16,
        gap_appearance_char_handle: *mut u16,
    ) -> tBleStatus;

    /// Write configuration data to BLE stack
    #[link_name = "ACI_HAL_WRITE_CONFIG_DATA"]
    fn aci_hal_write_config_data(offset: u8, length: u8, value: *const u8) -> tBleStatus;

    /// Set transmission power level
    #[link_name = "ACI_HAL_SET_TX_POWER_LEVEL"]
    fn aci_hal_set_tx_power_level(en_high_power: u8, pa_level: u8) -> tBleStatus;

    /// Set default PHY
    #[link_name = "HCI_LE_SET_DEFAULT_PHY"]
    fn hci_le_set_default_phy(all_phys: u8, tx_phys: u8, rx_phys: u8) -> tBleStatus;

    /// Set IO capability
    #[link_name = "ACI_GAP_SET_IO_CAPABILITY"]
    fn aci_gap_set_io_capability(io_capability: u8) -> tBleStatus;

    /// Set authentication requirement
    #[link_name = "ACI_GAP_SET_AUTHENTICATION_REQUIREMENT"]
    fn aci_gap_set_authentication_requirement(
        bonding_mode: u8,
        mitm_mode: u8,
        sc_support: u8,
        key_press_notification_support: u8,
        min_encryption_key_size: u8,
        max_encryption_key_size: u8,
        use_fixed_pin: u8,
        fixed_pin: u32,
        identity_address_type: u8,
    ) -> tBleStatus;

    /// Update characteristic value (for device name and appearance)
    #[link_name = "ACI_GATT_UPDATE_CHAR_VALUE"]
    fn aci_gatt_update_char_value(
        service_handle: u16,
        char_handle: u16,
        val_offset: u8,
        char_value_length: u8,
        char_value: *const u8,
    ) -> tBleStatus;
}

/// GAP initialization parameters
pub struct GapInitParams {
    /// Device role (peripheral, central, or both)
    pub role: GapRole,
    /// Privacy mode enabled
    pub privacy_enabled: bool,
    /// Device name
    pub device_name: &'static [u8],
    /// GAP appearance value
    pub appearance: u16,
    /// BD address (6 bytes)
    pub bd_addr: [u8; 6],
    /// Identity Root key (16 bytes) for IRK derivation
    pub ir_value: [u8; 16],
    /// Encryption Root key (16 bytes) for LTK derivation
    pub er_value: [u8; 16],
    /// TX power level (0-31, default 25 for +6dBm)
    pub tx_power: u8,
    /// PHY preferences
    pub phy_prefs: PhyPrefs,
    /// IO capability
    pub io_capability: IoCapability,
    /// Security parameters
    pub security: SecurityParams,
}

/// GAP role
#[derive(Debug, Clone, Copy)]
pub enum GapRole {
    Peripheral,
    Central,
    Both,
}

impl GapRole {
    fn to_bits(self) -> u8 {
        match self {
            GapRole::Peripheral => GAP_PERIPHERAL_ROLE,
            GapRole::Central => GAP_CENTRAL_ROLE,
            GapRole::Both => GAP_PERIPHERAL_ROLE | GAP_CENTRAL_ROLE,
        }
    }
}

/// PHY preferences
#[derive(Debug, Clone, Copy)]
pub struct PhyPrefs {
    /// All PHYs (0 = host has preference, 1 = host has no preference)
    pub all_phys: u8,
    /// TX PHY preference (1M, 2M, Coded)
    pub tx_phys: u8,
    /// RX PHY preference (1M, 2M, Coded)
    pub rx_phys: u8,
}

impl Default for PhyPrefs {
    fn default() -> Self {
        Self {
            all_phys: 0x00,  // Host has preference
            tx_phys: 0x03,   // 1M and 2M
            rx_phys: 0x03,   // 1M and 2M
        }
    }
}

/// IO Capability
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum IoCapability {
    DisplayOnly = 0x00,
    DisplayYesNo = 0x01,
    KeyboardOnly = 0x02,
    NoInputNoOutput = 0x03,
    KeyboardDisplay = 0x04,
}

/// Security parameters
#[derive(Debug, Clone, Copy)]
pub struct SecurityParams {
    /// Bonding mode (0 = no bonding, 1 = bonding)
    pub bonding_mode: u8,
    /// MITM protection required
    pub mitm_mode: u8,
    /// Secure Connections support
    pub sc_support: u8,
    /// Key press notification support
    pub key_press_notification_support: u8,
    /// Minimum encryption key size (7-16)
    pub min_encryption_key_size: u8,
    /// Maximum encryption key size (7-16)
    pub max_encryption_key_size: u8,
    /// Use fixed PIN
    pub use_fixed_pin: u8,
    /// Fixed PIN value (0-999999)
    pub fixed_pin: u32,
    /// Identity address type (0 = public, 1 = random static)
    pub identity_address_type: u8,
}

impl Default for SecurityParams {
    fn default() -> Self {
        Self {
            bonding_mode: 1,
            mitm_mode: 1,
            sc_support: 1,
            key_press_notification_support: 0,
            min_encryption_key_size: 8,
            max_encryption_key_size: 16,
            use_fixed_pin: 1,
            fixed_pin: 111111,
            identity_address_type: 0,  // Public address
        }
    }
}

impl Default for GapInitParams {
    fn default() -> Self {
        Self {
            role: GapRole::Peripheral,
            privacy_enabled: false,
            device_name: b"Embassy-BLE",
            appearance: 0,  // Unknown
            bd_addr: [0xE1, 0x80, 0xE1, 0x26, 0x1A, 0x00], // Default address
            ir_value: [0x12; 16],
            er_value: [0x34; 16],
            tx_power: 25,  // +6 dBm
            phy_prefs: PhyPrefs::default(),
            io_capability: IoCapability::NoInputNoOutput,
            security: SecurityParams::default(),
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
pub fn init_gap_and_hal(params: &GapInitParams) -> Result<GapHandles, BleError> {
    unsafe {
        // 1. Write additional GAP service record number (for Device Info service, etc.)
        let additional_records: [u8; 1] = [0x03];
        let status = aci_hal_write_config_data(
            CONFIG_DATA_GAP_ADD_REC_NBR_OFFSET,
            CONFIG_DATA_GAP_ADD_REC_NBR_LEN,
            additional_records.as_ptr(),
        );
        if status != BLE_STATUS_SUCCESS {
            #[cfg(feature = "defmt")]
            defmt::warn!("aci_hal_write_config_data (GAP_ADD_REC_NBR) failed: 0x{:02X}", status);
        }

        // 2. Write BD address
        let status = aci_hal_write_config_data(
            CONFIG_DATA_PUBADDR_OFFSET,
            CONFIG_DATA_PUBADDR_LEN,
            params.bd_addr.as_ptr(),
        );
        if status != BLE_STATUS_SUCCESS {
            #[cfg(feature = "defmt")]
            defmt::error!("aci_hal_write_config_data (PUBADDR) failed: 0x{:02X}", status);
            return Err(BleError::CommandFailed(Status::from_u8(status)));
        }

        #[cfg(feature = "defmt")]
        defmt::info!(
            "BD Address configured: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            params.bd_addr[5],
            params.bd_addr[4],
            params.bd_addr[3],
            params.bd_addr[2],
            params.bd_addr[1],
            params.bd_addr[0]
        );

        // 3. Write IR (Identity Root) value
        let status = aci_hal_write_config_data(
            CONFIG_DATA_IR_OFFSET,
            CONFIG_DATA_IR_LEN,
            params.ir_value.as_ptr(),
        );
        if status != BLE_STATUS_SUCCESS {
            #[cfg(feature = "defmt")]
            defmt::warn!("aci_hal_write_config_data (IR) failed: 0x{:02X}", status);
        }

        // 4. Write ER (Encryption Root) value
        let status = aci_hal_write_config_data(
            CONFIG_DATA_ER_OFFSET,
            CONFIG_DATA_ER_LEN,
            params.er_value.as_ptr(),
        );
        if status != BLE_STATUS_SUCCESS {
            #[cfg(feature = "defmt")]
            defmt::warn!("aci_hal_write_config_data (ER) failed: 0x{:02X}", status);
        }

        // 5. Set TX power level
        let status = aci_hal_set_tx_power_level(1, params.tx_power);
        if status != BLE_STATUS_SUCCESS {
            #[cfg(feature = "defmt")]
            defmt::warn!("aci_hal_set_tx_power_level failed: 0x{:02X}", status);
        }

        // 6. Initialize GAP layer
        let mut gap_service_handle: u16 = 0;
        let mut gap_dev_name_char_handle: u16 = 0;
        let mut gap_appearance_char_handle: u16 = 0;

        let status = aci_gap_init(
            params.role.to_bits(),
            params.privacy_enabled as u8,
            params.device_name.len() as u8,
            &mut gap_service_handle,
            &mut gap_dev_name_char_handle,
            &mut gap_appearance_char_handle,
        );

        if status != BLE_STATUS_SUCCESS {
            #[cfg(feature = "defmt")]
            defmt::error!("aci_gap_init failed: 0x{:02X}", status);
            return Err(BleError::CommandFailed(Status::from_u8(status)));
        }

        #[cfg(feature = "defmt")]
        defmt::info!(
            "GAP initialized - service: 0x{:04X}, name: 0x{:04X}, appearance: 0x{:04X}",
            gap_service_handle,
            gap_dev_name_char_handle,
            gap_appearance_char_handle
        );

        // 7. Update device name characteristic
        let status = aci_gatt_update_char_value(
            gap_service_handle,
            gap_dev_name_char_handle,
            0,
            params.device_name.len() as u8,
            params.device_name.as_ptr(),
        );
        if status != BLE_STATUS_SUCCESS {
            #[cfg(feature = "defmt")]
            defmt::warn!("aci_gatt_update_char_value (device name) failed: 0x{:02X}", status);
        }

        // 8. Update appearance characteristic
        let appearance_bytes = params.appearance.to_le_bytes();
        let status = aci_gatt_update_char_value(
            gap_service_handle,
            gap_appearance_char_handle,
            0,
            2,
            appearance_bytes.as_ptr(),
        );
        if status != BLE_STATUS_SUCCESS {
            #[cfg(feature = "defmt")]
            defmt::warn!("aci_gatt_update_char_value (appearance) failed: 0x{:02X}", status);
        }

        // 9. Set default PHY
        let status = hci_le_set_default_phy(
            params.phy_prefs.all_phys,
            params.phy_prefs.tx_phys,
            params.phy_prefs.rx_phys,
        );
        if status != BLE_STATUS_SUCCESS {
            #[cfg(feature = "defmt")]
            defmt::warn!("hci_le_set_default_phy failed: 0x{:02X}", status);
        }

        // 10. Set IO capability
        let status = aci_gap_set_io_capability(params.io_capability as u8);
        if status != BLE_STATUS_SUCCESS {
            #[cfg(feature = "defmt")]
            defmt::warn!("aci_gap_set_io_capability failed: 0x{:02X}", status);
        }

        // 11. Set authentication requirements
        let status = aci_gap_set_authentication_requirement(
            params.security.bonding_mode,
            params.security.mitm_mode,
            params.security.sc_support,
            params.security.key_press_notification_support,
            params.security.min_encryption_key_size,
            params.security.max_encryption_key_size,
            params.security.use_fixed_pin,
            params.security.fixed_pin,
            params.security.identity_address_type,
        );
        if status != BLE_STATUS_SUCCESS {
            #[cfg(feature = "defmt")]
            defmt::warn!("aci_gap_set_authentication_requirement failed: 0x{:02X}", status);
        }

        #[cfg(feature = "defmt")]
        defmt::info!("GAP and HAL configuration completed successfully");

        Ok(GapHandles {
            gap_service_handle,
            gap_dev_name_char_handle,
            gap_appearance_char_handle,
        })
    }
}
