//! BLE Security Manager implementation
//!
//! This module provides security functionality for BLE connections including:
//! - Pairing and bonding
//! - MITM (Man-in-the-Middle) protection
//! - Secure Connections (SC) support
//! - Passkey entry and numeric comparison
//!
//! # Example
//!
//! ```no_run
//! use embassy_stm32_wpan::security::{SecurityManager, SecurityParams, IoCapability};
//!
//! let mut security = SecurityManager::new();
//!
//! // Configure security requirements
//! let params = SecurityParams::default()
//!     .with_io_capability(IoCapability::DisplayYesNo)
//!     .with_bonding(true)
//!     .with_mitm_protection(true);
//!
//! security.set_authentication_requirements(params)?;
//! ```

use crate::wba::error::BleError;
use crate::wba::hci::types::Status;

// C library exports uppercase function names
#[allow(non_camel_case_types)]
type tBleStatus = u8;

#[link(name = "stm32wba_ble_stack_basic")]
unsafe extern "C" {
    #[link_name = "ACI_GAP_SET_AUTHENTICATION_REQUIREMENT"]
    fn aci_gap_set_authentication_requirement(
        bonding_mode: u8,
        mitm_mode: u8,
        sc_support: u8,
        keypress_notification_support: u8,
        min_encryption_key_size: u8,
        max_encryption_key_size: u8,
        use_fixed_pin: u8,
        fixed_pin: u32,
        identity_address_type: u8,
    ) -> tBleStatus;

    #[link_name = "ACI_GAP_PASS_KEY_RESP"]
    fn aci_gap_pass_key_resp(connection_handle: u16, pass_key: u32) -> tBleStatus;

    #[link_name = "ACI_GAP_NUMERIC_COMPARISON_VALUE_CONFIRM_YESNO"]
    fn aci_gap_numeric_comparison_value_confirm_yesno(connection_handle: u16, confirm_yes_no: u8) -> tBleStatus;

    #[link_name = "ACI_GAP_ALLOW_REBOND"]
    fn aci_gap_allow_rebond(connection_handle: u16) -> tBleStatus;

    #[link_name = "ACI_GAP_CLEAR_SECURITY_DATABASE"]
    fn aci_gap_clear_security_database() -> tBleStatus;

    #[link_name = "ACI_GAP_REMOVE_BONDED_DEVICE"]
    fn aci_gap_remove_bonded_device(peer_identity_address_type: u8, peer_identity_address: *const u8) -> tBleStatus;

    #[link_name = "ACI_GAP_IS_DEVICE_BONDED"]
    fn aci_gap_is_device_bonded(peer_identity_address_type: u8, peer_identity_address: *const u8) -> tBleStatus;

    #[link_name = "HCI_LE_SET_ADDRESS_RESOLUTION_ENABLE"]
    fn hci_le_set_address_resolution_enable(enable: u8) -> tBleStatus;

    #[link_name = "HCI_LE_SET_RESOLVABLE_PRIVATE_ADDRESS_TIMEOUT"]
    fn hci_le_set_resolvable_private_address_timeout(timeout: u16) -> tBleStatus;
}

const BLE_STATUS_SUCCESS: u8 = 0x00;

/// IO Capability for pairing
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum IoCapability {
    /// Display only - can display a passkey but not receive input
    DisplayOnly = 0x00,
    /// Display with Yes/No buttons - can display and confirm numeric comparison
    DisplayYesNo = 0x01,
    /// Keyboard only - can input passkey but no display
    KeyboardOnly = 0x02,
    /// No input or output capability
    #[default]
    NoInputNoOutput = 0x03,
    /// Keyboard and display capability
    KeyboardDisplay = 0x04,
}

/// Secure Connections support mode
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SecureConnectionsSupport {
    /// Secure Connections not supported (legacy pairing only)
    NotSupported = 0x00,
    /// Secure Connections supported but optional (can fall back to legacy)
    #[default]
    Optional = 0x01,
    /// Secure Connections required (will fail if peer doesn't support)
    Required = 0x02,
}

/// Bonding mode
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BondingMode {
    /// No bonding - keys are not stored after pairing
    NoBonding = 0x00,
    /// Bonding enabled - keys are stored for future reconnection
    #[default]
    Bonding = 0x01,
}

/// Identity address type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum IdentityAddressType {
    /// Public identity address
    #[default]
    Public = 0x00,
    /// Static random identity address
    StaticRandom = 0x01,
}

/// Security parameters for authentication
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SecurityParams {
    /// Bonding mode
    pub bonding_mode: BondingMode,
    /// MITM (Man-in-the-Middle) protection required
    pub mitm_protection: bool,
    /// Secure Connections support level
    pub secure_connections: SecureConnectionsSupport,
    /// Keypress notification support
    pub keypress_notification: bool,
    /// Minimum encryption key size (7-16 bytes)
    pub min_encryption_key_size: u8,
    /// Maximum encryption key size (7-16 bytes)
    pub max_encryption_key_size: u8,
    /// Use fixed PIN (for testing only, not recommended for production)
    pub use_fixed_pin: bool,
    /// Fixed PIN value (only used if use_fixed_pin is true)
    pub fixed_pin: u32,
    /// Identity address type
    pub identity_address_type: IdentityAddressType,
}

impl Default for SecurityParams {
    fn default() -> Self {
        Self {
            bonding_mode: BondingMode::Bonding,
            mitm_protection: false,
            secure_connections: SecureConnectionsSupport::Optional,
            keypress_notification: false,
            min_encryption_key_size: 7,
            max_encryption_key_size: 16,
            use_fixed_pin: false,
            fixed_pin: 0,
            identity_address_type: IdentityAddressType::Public,
        }
    }
}

impl SecurityParams {
    /// Create new security parameters with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set bonding mode
    pub fn with_bonding_mode(mut self, mode: BondingMode) -> Self {
        self.bonding_mode = mode;
        self
    }

    /// Enable or disable bonding
    pub fn with_bonding(mut self, enabled: bool) -> Self {
        self.bonding_mode = if enabled {
            BondingMode::Bonding
        } else {
            BondingMode::NoBonding
        };
        self
    }

    /// Set MITM protection requirement
    pub fn with_mitm_protection(mut self, required: bool) -> Self {
        self.mitm_protection = required;
        self
    }

    /// Set Secure Connections support level
    pub fn with_secure_connections(mut self, support: SecureConnectionsSupport) -> Self {
        self.secure_connections = support;
        self
    }

    /// Set encryption key size range
    pub fn with_key_size_range(mut self, min: u8, max: u8) -> Self {
        self.min_encryption_key_size = min.clamp(7, 16);
        self.max_encryption_key_size = max.clamp(7, 16);
        self
    }

    /// Use a fixed PIN (for testing only)
    pub fn with_fixed_pin(mut self, pin: u32) -> Self {
        self.use_fixed_pin = true;
        self.fixed_pin = pin % 1_000_000; // Ensure 6 digits max
        self
    }

    /// Set identity address type
    pub fn with_identity_address_type(mut self, addr_type: IdentityAddressType) -> Self {
        self.identity_address_type = addr_type;
        self
    }

    /// Preset for "Just Works" pairing (no MITM protection)
    pub fn just_works() -> Self {
        Self {
            bonding_mode: BondingMode::Bonding,
            mitm_protection: false,
            secure_connections: SecureConnectionsSupport::Optional,
            keypress_notification: false,
            min_encryption_key_size: 7,
            max_encryption_key_size: 16,
            use_fixed_pin: false,
            fixed_pin: 0,
            identity_address_type: IdentityAddressType::Public,
        }
    }

    /// Preset for passkey entry pairing
    pub fn passkey_entry() -> Self {
        Self {
            bonding_mode: BondingMode::Bonding,
            mitm_protection: true,
            secure_connections: SecureConnectionsSupport::Optional,
            keypress_notification: false,
            min_encryption_key_size: 7,
            max_encryption_key_size: 16,
            use_fixed_pin: false,
            fixed_pin: 0,
            identity_address_type: IdentityAddressType::Public,
        }
    }

    /// Preset for secure connections with numeric comparison
    pub fn secure_numeric_comparison() -> Self {
        Self {
            bonding_mode: BondingMode::Bonding,
            mitm_protection: true,
            secure_connections: SecureConnectionsSupport::Required,
            keypress_notification: false,
            min_encryption_key_size: 16,
            max_encryption_key_size: 16,
            use_fixed_pin: false,
            fixed_pin: 0,
            identity_address_type: IdentityAddressType::Public,
        }
    }
}

/// Security event types
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SecurityEvent {
    /// Pairing has completed (successfully or with error)
    PairingComplete {
        conn_handle: u16,
        status: PairingStatus,
        reason: u8,
    },
    /// Passkey request - application must provide passkey via pass_key_response()
    PasskeyRequest { conn_handle: u16 },
    /// Numeric comparison request - application must confirm via numeric_comparison_response()
    NumericComparisonRequest { conn_handle: u16, numeric_value: u32 },
    /// Bond lost event - need to allow rebond via allow_rebond()
    BondLost { conn_handle: u16 },
    /// Pairing request received (when using SMP mode bit 3)
    PairingRequest { conn_handle: u16, is_bonded: bool },
}

/// Pairing completion status
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PairingStatus {
    /// Pairing completed successfully
    Success = 0x00,
    /// Pairing timed out
    Timeout = 0x01,
    /// Pairing failed
    Failed = 0x02,
}

impl PairingStatus {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x00 => Self::Success,
            0x01 => Self::Timeout,
            _ => Self::Failed,
        }
    }
}

/// Pairing failure reason codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PairingFailureReason {
    PasskeyEntryFailed = 0x01,
    OobNotAvailable = 0x02,
    AuthenticationRequirements = 0x03,
    ConfirmValueFailed = 0x04,
    PairingNotSupported = 0x05,
    EncryptionKeySize = 0x06,
    CommandNotSupported = 0x07,
    UnspecifiedReason = 0x08,
    RepeatedAttempts = 0x09,
    InvalidParameters = 0x0A,
    DhKeyCheckFailed = 0x0B,
    NumericComparisonFailed = 0x0C,
    PairingInProgress = 0x0D,
    CrossTransportKeyDerivationNotAllowed = 0x0E,
    Unknown = 0xFF,
}

impl PairingFailureReason {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x01 => Self::PasskeyEntryFailed,
            0x02 => Self::OobNotAvailable,
            0x03 => Self::AuthenticationRequirements,
            0x04 => Self::ConfirmValueFailed,
            0x05 => Self::PairingNotSupported,
            0x06 => Self::EncryptionKeySize,
            0x07 => Self::CommandNotSupported,
            0x08 => Self::UnspecifiedReason,
            0x09 => Self::RepeatedAttempts,
            0x0A => Self::InvalidParameters,
            0x0B => Self::DhKeyCheckFailed,
            0x0C => Self::NumericComparisonFailed,
            0x0D => Self::PairingInProgress,
            0x0E => Self::CrossTransportKeyDerivationNotAllowed,
            _ => Self::Unknown,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::PasskeyEntryFailed => "Passkey entry failed",
            Self::OobNotAvailable => "OOB data not available",
            Self::AuthenticationRequirements => "Authentication requirements not met",
            Self::ConfirmValueFailed => "Confirm value failed",
            Self::PairingNotSupported => "Pairing not supported",
            Self::EncryptionKeySize => "Encryption key size too short",
            Self::CommandNotSupported => "Command not supported",
            Self::UnspecifiedReason => "Unspecified reason",
            Self::RepeatedAttempts => "Repeated pairing attempts",
            Self::InvalidParameters => "Invalid parameters",
            Self::DhKeyCheckFailed => "DH key check failed",
            Self::NumericComparisonFailed => "Numeric comparison failed",
            Self::PairingInProgress => "Pairing already in progress",
            Self::CrossTransportKeyDerivationNotAllowed => "Cross-transport key derivation not allowed",
            Self::Unknown => "Unknown reason",
        }
    }
}

/// Security Manager
///
/// Manages BLE security including pairing, bonding, and encryption.
pub struct SecurityManager {
    initialized: bool,
}

impl SecurityManager {
    /// Create a new Security Manager
    pub fn new() -> Self {
        Self { initialized: false }
    }

    /// Set authentication requirements
    ///
    /// This configures the security parameters for all future pairing operations.
    /// Must be called before connections are established.
    pub fn set_authentication_requirements(&mut self, params: SecurityParams) -> Result<(), BleError> {
        unsafe {
            let status = aci_gap_set_authentication_requirement(
                params.bonding_mode as u8,
                params.mitm_protection as u8,
                params.secure_connections as u8,
                params.keypress_notification as u8,
                params.min_encryption_key_size,
                params.max_encryption_key_size,
                params.use_fixed_pin as u8,
                params.fixed_pin,
                params.identity_address_type as u8,
            );

            if status == BLE_STATUS_SUCCESS {
                self.initialized = true;
                Ok(())
            } else {
                Err(BleError::CommandFailed(Status::from_u8(status)))
            }
        }
    }

    /// Respond to a passkey request
    ///
    /// Call this when you receive a `SecurityEvent::PasskeyRequest`.
    /// The passkey must be a 6-digit decimal number (0-999999).
    pub fn pass_key_response(&self, conn_handle: u16, passkey: u32) -> Result<(), BleError> {
        if passkey > 999_999 {
            return Err(BleError::InvalidParameter);
        }

        unsafe {
            let status = aci_gap_pass_key_resp(conn_handle, passkey);

            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(Status::from_u8(status)))
            }
        }
    }

    /// Respond to a numeric comparison request
    ///
    /// Call this when you receive a `SecurityEvent::NumericComparisonRequest`.
    /// Pass `true` if the numeric values match, `false` otherwise.
    pub fn numeric_comparison_response(&self, conn_handle: u16, confirm: bool) -> Result<(), BleError> {
        unsafe {
            let status = aci_gap_numeric_comparison_value_confirm_yesno(conn_handle, confirm as u8);

            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(Status::from_u8(status)))
            }
        }
    }

    /// Allow rebonding after receiving BondLost event
    ///
    /// Call this when you receive a `SecurityEvent::BondLost` to allow
    /// the pairing process to continue.
    pub fn allow_rebond(&self, conn_handle: u16) -> Result<(), BleError> {
        unsafe {
            let status = aci_gap_allow_rebond(conn_handle);

            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(Status::from_u8(status)))
            }
        }
    }

    /// Clear all bonding information from the security database
    ///
    /// This removes all stored bonds. Use with caution.
    pub fn clear_security_database(&self) -> Result<(), BleError> {
        unsafe {
            let status = aci_gap_clear_security_database();

            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(Status::from_u8(status)))
            }
        }
    }

    /// Remove a specific bonded device
    pub fn remove_bonded_device(&self, address_type: IdentityAddressType, address: &[u8; 6]) -> Result<(), BleError> {
        unsafe {
            let status = aci_gap_remove_bonded_device(address_type as u8, address.as_ptr());

            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(Status::from_u8(status)))
            }
        }
    }

    /// Check if a device is bonded
    pub fn is_device_bonded(&self, address_type: IdentityAddressType, address: &[u8; 6]) -> Result<bool, BleError> {
        unsafe {
            let status = aci_gap_is_device_bonded(address_type as u8, address.as_ptr());

            // BLE_STATUS_SUCCESS means bonded, error code 0x42 means not bonded
            Ok(status == BLE_STATUS_SUCCESS)
        }
    }

    /// Enable or disable address resolution
    ///
    /// When enabled, the controller will automatically resolve resolvable
    /// private addresses (RPA) using stored IRKs.
    pub fn set_address_resolution_enable(&self, enable: bool) -> Result<(), BleError> {
        unsafe {
            let status = hci_le_set_address_resolution_enable(enable as u8);

            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(Status::from_u8(status)))
            }
        }
    }

    /// Set RPA (Resolvable Private Address) timeout
    ///
    /// Sets the time interval after which a new RPA is generated.
    /// Valid range: 1 to 41400 seconds (approximately 11.5 hours).
    /// Default is usually 900 seconds (15 minutes).
    pub fn set_rpa_timeout(&self, timeout_seconds: u16) -> Result<(), BleError> {
        let timeout = timeout_seconds.clamp(1, 41400);

        unsafe {
            let status = hci_le_set_resolvable_private_address_timeout(timeout);

            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(Status::from_u8(status)))
            }
        }
    }

    /// Check if security has been initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

// ACI event codes for security events
pub mod event_codes {
    pub const ACI_GAP_PAIRING_COMPLETE: u16 = 0x0401;
    pub const ACI_GAP_PASS_KEY_REQ: u16 = 0x0402;
    pub const ACI_GAP_BOND_LOST: u16 = 0x0405;
    pub const ACI_GAP_NUMERIC_COMPARISON_VALUE: u16 = 0x0409;
    pub const ACI_GAP_PAIRING_REQUEST: u16 = 0x040B;
}
