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

use stm32wb_hci::vendor::event::{GapPairingReason, GapPairingStatus, KeypressNotificationType, VendorEvent};

use crate::bluetooth::error::BleError;
use crate::bluetooth::hci::types::Status;

// C library exports uppercase function names
#[allow(non_camel_case_types)]
type tBleStatus = u8;

unsafe extern "C" {
    #[link_name = "ACI_GAP_SET_IO_CAPABILITY"]
    fn aci_gap_set_io_capability(io_capability: u8) -> tBleStatus;

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

    #[link_name = "ACI_GAP_PASSKEY_INPUT"]
    fn aci_gap_passkey_input(connection_handle: u16, input_type: u8) -> tBleStatus;

    #[link_name = "ACI_GAP_AUTHORIZATION_RESP"]
    fn aci_gap_authorization_resp(connection_handle: u16, authorize: u8) -> tBleStatus;

    #[link_name = "ACI_GAP_NUMERIC_COMPARISON_VALUE_CONFIRM_YESNO"]
    fn aci_gap_numeric_comparison_value_confirm_yesno(connection_handle: u16, confirm_yes_no: u8) -> tBleStatus;

    #[link_name = "ACI_GAP_PERIPHERAL_SECURITY_REQ"]
    fn aci_gap_peripheral_security_req(connection_handle: u16) -> tBleStatus;

    #[link_name = "ACI_GAP_SEND_PAIRING_REQ"]
    fn aci_gap_send_pairing_req(connection_handle: u16, force_rebond: u8) -> tBleStatus;

    #[link_name = "ACI_GAP_PAIRING_REQUEST_REPLY"]
    fn aci_gap_pairing_request_reply(connection_handle: u16, accept: u8) -> tBleStatus;

    #[link_name = "ACI_GAP_ALLOW_REBOND"]
    fn aci_gap_allow_rebond(connection_handle: u16) -> tBleStatus;

    #[link_name = "ACI_GAP_CLEAR_SECURITY_DB"]
    fn aci_gap_clear_security_db() -> tBleStatus;

    #[link_name = "ACI_GAP_REMOVE_BONDED_DEVICE"]
    fn aci_gap_remove_bonded_device(peer_identity_address_type: u8, peer_identity_address: *const u8) -> tBleStatus;

    #[link_name = "ACI_GAP_IS_DEVICE_BONDED"]
    fn aci_gap_is_device_bonded(peer_identity_address_type: u8, peer_identity_address: *const u8) -> tBleStatus;

    #[link_name = "ACI_HAL_WRITE_CONFIG_DATA"]
    fn aci_hal_write_config_data(offset: u8, length: u8, value: *const u8) -> tBleStatus;

    #[link_name = "ACI_GAP_CHECK_BONDED_DEVICE"]
    fn aci_gap_check_bonded_device(
        peer_address_type: u8,
        peer_address: *const u8,
        id_address_type: *mut u8,
        id_address: *mut u8,
    ) -> tBleStatus;

    #[link_name = "HCI_LE_SET_ADDRESS_RESOLUTION_ENABLE"]
    fn hci_le_set_address_resolution_enable(enable: u8) -> tBleStatus;

    #[link_name = "HCI_LE_SET_RESOLVABLE_PRIVATE_ADDRESS_TIMEOUT"]
    fn hci_le_set_resolvable_private_address_timeout(timeout: u16) -> tBleStatus;

    #[link_name = "ACI_GAP_GET_BONDED_DEVICES"]
    fn aci_gap_get_bonded_devices(num_of_addresses: *mut u8, bonded_device_entry: *mut BondedDeviceEntry)
    -> tBleStatus;

    #[link_name = "ACI_GAP_ADD_DEVICES_TO_LIST"]
    fn aci_gap_add_devices_to_list(num_of_list_entries: u8, list_entry: *const ListEntry, mode: u8) -> tBleStatus;

    #[link_name = "HCI_LE_CLEAR_RESOLVING_LIST"]
    fn hci_le_clear_resolving_list() -> tBleStatus;

    #[link_name = "HCI_LE_CLEAR_FILTER_ACCEPT_LIST"]
    fn hci_le_clear_filter_accept_list() -> tBleStatus;

    #[allow(dead_code)]
    #[link_name = "HCI_LE_SET_PRIVACY_MODE"]
    fn hci_le_set_privacy_mode(
        peer_identity_address_type: u8,
        peer_identity_address: *const u8,
        privacy_mode: u8,
    ) -> tBleStatus;

    // Diagnostic-only readers, behind the defmt feature so they don't trip dead-code-lints
    // when log_resolving_list_diagnostics is compiled out.
    #[cfg(feature = "defmt")]
    #[link_name = "HCI_LE_READ_RESOLVING_LIST_SIZE"]
    fn hci_le_read_resolving_list_size(resolving_list_size: *mut u8) -> tBleStatus;

    #[cfg(feature = "defmt")]
    #[link_name = "HCI_LE_READ_PEER_RESOLVABLE_ADDRESS"]
    fn hci_le_read_peer_resolvable_address(
        peer_identity_address_type: u8,
        peer_identity_address: *const u8,
        peer_resolvable_address: *mut u8,
    ) -> tBleStatus;

    #[cfg(feature = "defmt")]
    #[link_name = "HCI_LE_READ_LOCAL_RESOLVABLE_ADDRESS"]
    fn hci_le_read_local_resolvable_address(
        peer_identity_address_type: u8,
        peer_identity_address: *const u8,
        local_resolvable_address: *mut u8,
    ) -> tBleStatus;
}

// Matches `Bonded_Device_Entry_t` / `List_Entry_t` in the ST headers (packed: 7 bytes).
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct BondedDeviceEntry {
    address_type: u8,
    address: [u8; 6],
}
type ListEntry = BondedDeviceEntry;

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

/// Passkey-entry keypress type for [`SecurityManager::passkey_input`].
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PasskeyInputType {
    /// Passkey entry started.
    Started = 0x00,
    /// A passkey digit was entered.
    DigitEntered = 0x01,
    /// A passkey digit was erased.
    DigitErased = 0x02,
    /// The entered passkey was cleared.
    Cleared = 0x03,
    /// Passkey entry completed.
    Completed = 0x04,
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
    /// IO capability.
    ///
    /// Must match what the device can actually do. Determines which pairing
    /// method the stack selects. If `mitm_protection` is `true`, this must be
    /// something other than `NoInputNoOutput`; otherwise pairing always fails
    /// with `AuthRequirements` because "Just Works" (the only method available
    /// for NoInputNoOutput) provides no MITM protection.
    pub io_capability: IoCapability,
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
            io_capability: IoCapability::NoInputNoOutput,
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

    /// Set IO capability.
    ///
    /// Determines which pairing method the BLE stack selects. When
    /// `mitm_protection` is enabled, use anything other than
    /// `NoInputNoOutput`; otherwise the stack can only do "Just Works"
    /// and pairing will fail with `AuthRequirements`.
    pub fn with_io_capability(mut self, cap: IoCapability) -> Self {
        self.io_capability = cap;
        self
    }

    /// Preset for "Just Works" pairing (no MITM protection)
    pub fn just_works() -> Self {
        Self {
            io_capability: IoCapability::NoInputNoOutput,
            ..Self::default()
        }
    }

    /// Preset for passkey entry pairing (device displays passkey, peer enters it)
    pub fn passkey_entry() -> Self {
        Self {
            mitm_protection: true,
            io_capability: IoCapability::DisplayOnly,
            ..Self::default()
        }
    }

    /// Preset for secure connections with numeric comparison
    pub fn secure_numeric_comparison() -> Self {
        Self {
            mitm_protection: true,
            secure_connections: SecureConnectionsSupport::Required,
            min_encryption_key_size: 16,
            max_encryption_key_size: 16,
            io_capability: IoCapability::DisplayYesNo,
            ..Self::default()
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
    /// Pairing request received (when using SMP mode bit 3).
    ///
    /// `auth_req` is the raw AuthReq octet from the peer's Pairing Request or
    /// Security Request (Core Spec Vol 3, Part H, 3.5.1): bit 0-1 bonding flags,
    /// bit 2 MITM, bit 3 SC, bit 4 keypress.
    PairingRequest {
        conn_handle: u16,
        is_bonded: bool,
        auth_req: u8,
    },
    /// Application authorization response is required.
    AuthorizationRequest { conn_handle: u16 },
    /// Peripheral-side security procedure has started successfully.
    PeripheralSecurityInitiated,
    /// Peer address could not be resolved with current privacy data.
    AddressNotResolved { conn_handle: u16 },
    /// Peer keypress-notification during passkey entry.
    KeypressNotification {
        conn_handle: u16,
        notification_type: KeypressNotificationType,
    },
}

/// Flags for [`SecurityManager::set_smp_mode`] (`CONFIG_DATA_SMP_MODE`).
pub struct SmpMode;

impl SmpMode {
    /// Deactivate the SMP; controller events that would go to it are passed
    /// straight to the application.
    pub const BYPASS: u8 = 0x01;
    /// Disable the "Repeated Attempts" protection.
    pub const NO_BLACKLIST: u8 = 0x02;
    /// Forbid the peer from using the standard Secure Connections debug key.
    pub const NO_PEER_DEBUG_KEY: u8 = 0x04;
    /// Deliver [`SecurityEvent::PairingRequest`] on an incoming Pairing Request
    /// or Security Request. Requires the application to answer with
    /// [`SecurityManager::pairing_request_reply`].
    pub const PAIRING_REQUEST_EVENT: u8 = 0x08;
    /// Forbid the Just Works association model.
    pub const NO_JUST_WORKS: u8 = 0x10;
    /// Forbid the Passkey Entry association model.
    pub const NO_PASSKEY_ENTRY: u8 = 0x20;
    /// Forbid the Out of Band association model.
    pub const NO_OOB: u8 = 0x40;
    /// Forbid the Numeric Comparison association model.
    pub const NO_NUMERIC_COMPARISON: u8 = 0x80;
}

/// Convert an STM32 vendor-specific event into a high-level security event.
///
/// Returns `None` for vendor events that do not map to [`SecurityEvent`].
pub fn from_vendor_event(event: &VendorEvent) -> Option<SecurityEvent> {
    match event {
        VendorEvent::GapPairingComplete(e) => {
            let (status, reason) = match e.status {
                GapPairingStatus::Success => (PairingStatus::Success, 0),
                GapPairingStatus::Timeout(r) => (PairingStatus::Timeout, pairing_reason_to_u8(r)),
                GapPairingStatus::Failed(r) => (PairingStatus::Failed, pairing_reason_to_u8(r)),
                // Keep these apart: encryption failing against a stored bond is a
                // different fault from a refused pairing negotiation, and the
                // pairing-complete reason code is only defined for the latter.
                GapPairingStatus::EncryptionFailed(_) => (PairingStatus::EncryptionFailed, 0),
            };
            Some(SecurityEvent::PairingComplete {
                conn_handle: e.conn_handle.0,
                status,
                reason,
            })
        }
        VendorEvent::GapPassKeyRequest(conn_handle) => Some(SecurityEvent::PasskeyRequest {
            conn_handle: conn_handle.0,
        }),
        VendorEvent::GapNumericComparisonValue(e) => Some(SecurityEvent::NumericComparisonRequest {
            conn_handle: e.connection_handle.0,
            numeric_value: e.numeric_value,
        }),
        VendorEvent::GapBondLost(conn_handle) => Some(SecurityEvent::BondLost {
            conn_handle: conn_handle.0,
        }),
        VendorEvent::GapPairingRequest(e) => Some(SecurityEvent::PairingRequest {
            conn_handle: e.connection_handle.0,
            is_bonded: e.bonded,
            auth_req: e.auth_req,
        }),
        VendorEvent::GapAuthorizationRequest(conn_handle) => Some(SecurityEvent::AuthorizationRequest {
            conn_handle: conn_handle.0,
        }),
        VendorEvent::GapPeripheralSecurityInitiated => Some(SecurityEvent::PeripheralSecurityInitiated),
        VendorEvent::GapAddressNotResolved(conn_handle) => Some(SecurityEvent::AddressNotResolved {
            conn_handle: conn_handle.0,
        }),
        VendorEvent::GapKeypressNotification(e) => Some(SecurityEvent::KeypressNotification {
            conn_handle: e.connection_handle.0,
            notification_type: e.notification_type,
        }),
        _ => None,
    }
}

fn pairing_reason_to_u8(reason: GapPairingReason) -> u8 {
    reason as u8
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
    /// Pairing failed. The pairing-complete `Reason` field is only meaningful for
    /// this status.
    Failed = 0x02,
    /// Encryption failed.
    ///
    /// This is what a bonded peer that cannot be encrypted reports, so it must be
    /// distinguished from [`Self::Failed`]: it means the link key was rejected or
    /// could not be found rather than that pairing was negotiated and refused. The
    /// `Reason` field does not apply and holds no useful value here.
    EncryptionFailed = 0x03,
}

impl PairingStatus {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x00 => Self::Success,
            0x01 => Self::Timeout,
            0x03 => Self::EncryptionFailed,
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
    pub(crate) const fn new() -> Self {
        Self { initialized: false }
    }

    /// Set authentication requirements
    ///
    /// This configures the security parameters for all future pairing operations.
    /// Must be called before connections are established.
    pub fn set_authentication_requirements(&mut self, params: SecurityParams) -> Result<(), BleError> {
        unsafe {
            // IO capability determines the pairing method. Must be set before
            // the authentication requirement so the stack uses the right method.
            let io_status = aci_gap_set_io_capability(params.io_capability as u8);
            if io_status != BLE_STATUS_SUCCESS {
                return Err(BleError::CommandFailed(Status::from_u8(io_status)));
            }

            let status = aci_gap_set_authentication_requirement(
                params.bonding_mode as u8,
                params.mitm_protection as u8,
                params.secure_connections as u8,
                params.keypress_notification as u8,
                params.min_encryption_key_size,
                params.max_encryption_key_size,
                // Inverted on purpose: ST's parameter is a *prohibition*, not a
                // request. USE_FIXED_PIN_FOR_PAIRING_ALLOWED is 0x00 and
                // ..._FORBIDDEN is 0x01, so passing this flag through unchanged
                // made the default (`use_fixed_pin: false`) enable the deprecated
                // fixed-PIN pairing with PIN 000000 instead of disabling it.
                !params.use_fixed_pin as u8,
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

    /// Signal a passkey-entry keypress to the stack.
    ///
    /// During passkey entry the stack can forward keypress notifications to the
    /// peer (requires `keypress_notification` in [`SecurityParams`]). Call this
    /// as the user interacts with the local keypad; the 6-digit value itself is
    /// sent with [`pass_key_response`](Self::pass_key_response) in response to
    /// [`SecurityEvent::PasskeyRequest`].
    pub fn passkey_input(&self, conn_handle: u16, input: PasskeyInputType) -> Result<(), BleError> {
        unsafe {
            let status = aci_gap_passkey_input(conn_handle, input as u8);
            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(Status::from_u8(status)))
            }
        }
    }

    /// Respond to an authorization request event.
    ///
    /// Call this when you receive a `SecurityEvent::AuthorizationRequest`.
    pub fn authorization_response(&self, conn_handle: u16, authorize: bool) -> Result<(), BleError> {
        unsafe {
            let status = aci_gap_authorization_resp(conn_handle, if authorize { 0x01 } else { 0x02 });
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

    /// Request pairing/encryption from the peripheral side.
    ///
    /// Sends an SMP Security Request to the central. Call this immediately
    /// after connection when the device has security requirements, rather than
    /// waiting for the central to trigger pairing via an insufficient-security
    /// GATT error. Matches ST's recommended pattern (aci_gap_slave_security_req
    /// in BLE_HeartRate).
    pub fn request_pairing(&self, conn_handle: u16) -> Result<(), BleError> {
        unsafe {
            let status = aci_gap_peripheral_security_req(conn_handle);
            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(Status::from_u8(status)))
            }
        }
    }

    /// Request pairing/encryption from the central side.
    ///
    /// Sends a pairing request to the peer peripheral. Set `force_rebond` to
    /// restart pairing even when a bond entry exists.
    pub fn request_pairing_central(&self, conn_handle: u16, force_rebond: bool) -> Result<(), BleError> {
        unsafe {
            let status = aci_gap_send_pairing_req(conn_handle, force_rebond as u8);
            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(Status::from_u8(status)))
            }
        }
    }

    /// Reply to a pairing request signaled via `SecurityEvent::PairingRequest`.
    ///
    /// Set `accept` to `true` to continue pairing, or `false` to reject it.
    pub fn pairing_request_response(&self, conn_handle: u16, accept: bool) -> Result<(), BleError> {
        unsafe {
            let status = aci_gap_pairing_request_reply(conn_handle, accept as u8);
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
            let status = aci_gap_clear_security_db();

            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(Status::from_u8(status)))
            }
        }
    }

    /// Drop the controller-side copies of every bond: address resolution, the
    /// resolving list and the Filter Accept List.
    ///
    /// [`clear_security_database`](Self::clear_security_database) only empties
    /// the host bond database. The controller keeps its own IRK table, accept
    /// list and per-peer privacy modes, and
    /// [`configure_filter_and_resolving_list`](Self::configure_filter_and_resolving_list)
    /// returns early once the database is empty, so nothing ever tells the
    /// controller to forget them. Call this alongside `clear_security_database`
    /// to leave the stack in the same state a reset would.
    ///
    /// Address resolution is disabled first because the Core Spec forbids
    /// clearing the resolving list while translation is enabled and the radio is
    /// active (Vol 4, Part E, 7.8.40).
    ///
    /// Must NOT be called while advertising, scanning, or initiating is active.
    pub fn clear_bond_lists(&self) -> Result<(), BleError> {
        unsafe {
            let status = hci_le_set_address_resolution_enable(0);
            if status != BLE_STATUS_SUCCESS {
                return Err(BleError::CommandFailed(Status::from_u8(status)));
            }

            let status = hci_le_clear_resolving_list();
            if status != BLE_STATUS_SUCCESS {
                return Err(BleError::CommandFailed(Status::from_u8(status)));
            }

            let status = hci_le_clear_filter_accept_list();
            if status != BLE_STATUS_SUCCESS {
                return Err(BleError::CommandFailed(Status::from_u8(status)));
            }

            Ok(())
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

    /// Set the host's SMP mode bitmap (`CONFIG_DATA_SMP_MODE_OFFSET`).
    ///
    /// Must be called before pairing starts. See [`SmpMode`] for the flags.
    /// Note that setting [`SmpMode::PAIRING_REQUEST_EVENT`] makes the stack wait
    /// for [`pairing_request_response`](Self::pairing_request_response) on every
    /// incoming Pairing Request, so the application must handle
    /// [`SecurityEvent::PairingRequest`] or pairing will stall.
    pub fn set_smp_mode(&self, mode: u8) -> Result<(), BleError> {
        const CONFIG_DATA_SMP_MODE_OFFSET: u8 = 0xB0;

        let status = unsafe { aci_hal_write_config_data(CONFIG_DATA_SMP_MODE_OFFSET, 1, &mode) };
        if status == BLE_STATUS_SUCCESS {
            Ok(())
        } else {
            Err(BleError::CommandFailed(Status::from_u8(status)))
        }
    }

    /// Look a peer up in the bonding table, resolving the address first when it
    /// is a Resolvable Private Address.
    ///
    /// Unlike [`is_device_bonded`](Self::is_device_bonded), which needs the peer's
    /// *identity* address and therefore fails for a peer that reconnects behind a
    /// rotating RPA, this resolves the supplied address against every IRK in the
    /// host's security database. It works even with privacy disabled.
    ///
    /// Returns the identity address the peer distributed during bonding, or `None`
    /// if no IRK in the database matches.
    ///
    /// New in CubeWBA 1.10 (`ACI_GAP_CHECK_BONDED_DEVICE`); it is also what the
    /// deprecated `ACI_GAP_RESOLVE_PRIVATE_ADDR` now forwards to.
    pub fn check_bonded_device(&self, address_type: u8, address: &[u8; 6]) -> Option<(u8, [u8; 6])> {
        let mut id_address_type: u8 = 0;
        let mut id_address = [0u8; 6];

        let status = unsafe {
            aci_gap_check_bonded_device(
                address_type,
                address.as_ptr(),
                &mut id_address_type,
                id_address.as_mut_ptr(),
            )
        };

        if status == BLE_STATUS_SUCCESS {
            Some((id_address_type, id_address))
        } else {
            None
        }
    }

    /// Run [`check_bonded_device`](Self::check_bonded_device) against every bonded
    /// peer's own identity address (debug).
    ///
    /// This is the positive control for the RPA lookups: an identity address that
    /// `aci_gap_get_bonded_devices` just returned needs neither an IRK nor any
    /// crypto to match, so success is the only sane outcome. A failure here means
    /// the command is unusable and its verdict on RPAs carries no information.
    #[cfg(feature = "defmt")]
    pub fn log_check_bonded_identities(&self) {
        const MAX_BONDED: usize = 16;
        let mut entries = [BondedDeviceEntry {
            address_type: 0,
            address: [0; 6],
        }; MAX_BONDED];
        let mut num: u8 = 0;

        unsafe {
            if aci_gap_get_bonded_devices(&mut num, entries.as_mut_ptr()) != BLE_STATUS_SUCCESS {
                return;
            }
        }

        for i in 0..(num as usize).min(MAX_BONDED) {
            let e = &entries[i];
            match self.check_bonded_device(e.address_type, &e.address) {
                Some((id_type, id)) => info!(
                    "  check_bonded_device(identity[{}]) OK -> type={} {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    i, id_type, id[5], id[4], id[3], id[2], id[1], id[0]
                ),
                None => warn!("  check_bonded_device(identity[{}]) FAILED -- probe is unreliable", i),
            }
        }
    }

    #[cfg(not(feature = "defmt"))]
    pub fn log_check_bonded_identities(&self) {}

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

    /// Populate the controller's resolving list with bonded peer identities so
    /// that reconnections from RPA-using peers (e.g. iOS) can be recognised as
    /// the bonded peer.
    ///
    /// Without this, `set_address_resolution_enable(true)` only enables resolution
    /// in the controller — there are no IRKs to resolve against. Incoming
    /// `LL_ENC_REQ` from a bonded peer will fail with "SMP unexpected LTK request"
    /// because the host cannot match an LTK to the unresolved peer identity.
    ///
    /// Uses mode 0x01 (clear and set the **resolving list only**, leaving the
    /// Filter Accept List untouched), so it is idempotent and won't interfere
    /// with `aci_gap_set_discoverable` advertising. Safe to call repeatedly,
    /// e.g. after every disconnect before re-advertising.
    ///
    /// Must NOT be called while advertising, scanning, or connecting are active
    /// when address resolution is enabled (Core Spec restriction). Returns the
    /// number of bonded devices restored to the resolving list.
    pub fn populate_resolving_list_from_bonds(&self) -> Result<usize, BleError> {
        const MAX_BONDED: usize = 16;
        let mut entries = [BondedDeviceEntry {
            address_type: 0,
            address: [0; 6],
        }; MAX_BONDED];
        let mut num: u8 = 0;

        unsafe {
            let status = aci_gap_get_bonded_devices(&mut num, entries.as_mut_ptr());
            if status != BLE_STATUS_SUCCESS {
                return Err(BleError::CommandFailed(Status::from_u8(status)));
            }

            if num == 0 {
                // Clear rather than return: leaving the previous entries in place
                // would keep the controller resolving against IRKs whose bonds are
                // gone. Translation must be off to mutate the list, and with no
                // bonds there is nothing to resolve against anyway.
                let status = hci_le_set_address_resolution_enable(0);
                if status != BLE_STATUS_SUCCESS {
                    return Err(BleError::CommandFailed(Status::from_u8(status)));
                }
                let status = hci_le_clear_resolving_list();
                if status != BLE_STATUS_SUCCESS {
                    return Err(BleError::CommandFailed(Status::from_u8(status)));
                }
                return Ok(0);
            }

            let count = (num as usize).min(MAX_BONDED) as u8;
            // mode 0x01 = clear and set the resolving list only
            let status = aci_gap_add_devices_to_list(count, entries.as_ptr() as *const ListEntry, 0x01);
            if status != BLE_STATUS_SUCCESS {
                return Err(BleError::CommandFailed(Status::from_u8(status)));
            }

            Ok(count as usize)
        }
    }

    /// Populate the controller's Filter Accept List with all bonded devices'
    /// identity addresses (clears it first). This is the maintained ST pattern
    /// from `BLE_p2pServer` for restoring bond-based filtering on boot.
    ///
    /// For LE Legacy bonded reconnect, the controller looks up the LTK by the
    /// EDIV/RAND values in `LL_ENC_REQ`, so populating the Filter Accept List
    /// here lets the controller recognise the bonded peer at the LL layer
    /// without needing RPA resolution at the host level.
    ///
    /// Must NOT be called while advertising/scanning/initiating is active.
    pub fn configure_filter_accept_list(&self) -> Result<(), BleError> {
        // 1.10.0 removed `ACI_GAP_CONFIGURE_FILTER_ACCEPT_LIST`. Its replacement is
        // `ACI_GAP_ADD_DEVICES_TO_LIST` with `Mode = FILTER_ACC_LIST_ONLY | CONFIGURE_FROM_SDB`
        // (`ble_defs.h`), which rebuilds the list from the bond database; no explicit
        // entries are passed for the "configure from SDB" mode.
        const FILTER_ACC_LIST_FROM_SDB: u8 = 0x02 | 0x08;
        unsafe {
            let status = aci_gap_add_devices_to_list(0, core::ptr::null(), FILTER_ACC_LIST_FROM_SDB);
            if status == BLE_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(BleError::CommandFailed(Status::from_u8(status)))
            }
        }
    }

    /// Compatibility alias for ST guide naming (`configure_whitelist`).
    ///
    /// On current stacks this is implemented by `ACI_GAP_CONFIGURE_FILTER_ACCEPT_LIST`.
    pub fn configure_whitelist(&self) -> Result<(), BleError> {
        self.configure_filter_accept_list()
    }

    /// Rebuild the controller's resolving list from the bonds currently in the
    /// security database, and clear the Filter Accept List.
    ///
    /// CubeWBA 1.10 mode 0x0D atomically clears both lists and repopulates them
    /// from the security database. GAP therefore owns the full operation and
    /// keeps its internal privacy state synchronized with the controller.
    ///
    /// Address resolution is then enabled iff at least one bond was programmed,
    /// so the controller's translation state always matches the list contents.
    ///
    /// Must NOT be called while advertising, scanning, or initiating is active.
    /// Returns the number of bonds programmed.
    pub fn configure_filter_and_resolving_list(&self) -> Result<usize, BleError> {
        const GAP_ADD_DEV_MODE_CLEAR_BOTH_FROM_SDB: u8 = 0x0D;

        const MAX_BONDED: usize = 16;
        let mut entries = [BondedDeviceEntry {
            address_type: 0,
            address: [0; 6],
        }; MAX_BONDED];
        let mut num: u8 = 0;

        unsafe {
            let status = aci_gap_get_bonded_devices(&mut num, entries.as_mut_ptr());
            if status != BLE_STATUS_SUCCESS {
                return Err(BleError::CommandFailed(Status::from_u8(status)));
            }

            let status = aci_gap_add_devices_to_list(0, core::ptr::null(), GAP_ADD_DEV_MODE_CLEAR_BOTH_FROM_SDB);
            if status != BLE_STATUS_SUCCESS {
                return Err(BleError::CommandFailed(Status::from_u8(status)));
            }

            let count = (num as usize).min(MAX_BONDED);

            // Mode 0x0D only loads the list; it does not touch the translation
            // enable. `clear_bond_lists` has to disable resolution to mutate the
            // list at all, so without this the controller keeps a correctly
            // populated resolving list that it is not permitted to use, and every
            // bonded reconnect arrives as an unresolved RPA.
            let status = hci_le_set_address_resolution_enable((count > 0) as u8);
            if status != BLE_STATUS_SUCCESS {
                return Err(BleError::CommandFailed(Status::from_u8(status)));
            }

            Ok(count)
        }
    }

    /// Log bonded peer identity addresses from the stack database (debug).
    #[cfg(feature = "defmt")]
    pub fn log_bonded_devices(&self) {
        const MAX_BONDED: usize = 16;
        let mut entries = [BondedDeviceEntry {
            address_type: 0,
            address: [0; 6],
        }; MAX_BONDED];
        let mut num: u8 = 0;

        unsafe {
            let status = aci_gap_get_bonded_devices(&mut num, entries.as_mut_ptr());
            if status != BLE_STATUS_SUCCESS {
                warn!("aci_gap_get_bonded_devices failed: 0x{:02X}", status);
                return;
            }

            for i in 0..(num as usize).min(MAX_BONDED) {
                let e = &entries[i];
                info!(
                    "bond[{}]: type={} addr={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    i,
                    e.address_type,
                    e.address[5],
                    e.address[4],
                    e.address[3],
                    e.address[2],
                    e.address[1],
                    e.address[0]
                );
            }
        }
    }

    #[cfg(not(feature = "defmt"))]
    pub fn log_bonded_devices(&self) {}

    /// Log the controller's resolving-list size and the bonded identity list (debug).
    ///
    /// The peer/local RPA readout is printed **only for the peer that is currently
    /// connected**, if any, because that is the only time those values mean anything:
    /// `HCI_LE_READ_PEER_RESOLVABLE_ADDRESS` reports "the *current* peer Resolvable
    /// Private Address **being used**" for a peer (ST `ble_hci_le.h`, Core Spec
    /// Vol 4 Part E 7.8.42), and with no connection to that peer there is no address
    /// in use — the spec says the controller answers 0x02 (Unknown Connection
    /// Identifier). This controller instead answers SUCCESS with a stale address, the
    /// same one for every entry, which is why every `resolving_list[i]` used to print
    /// an identical `peer_rpa`.
    ///
    /// Pass the peer of the live connection as `(identity_address_type,
    /// identity_address)` in the same byte order `aci_gap_get_bonded_devices` returns,
    /// or `None` when disconnected.
    #[cfg(feature = "defmt")]
    pub fn log_resolving_list_diagnostics(&self, connected_peer: Option<(u8, [u8; 6])>) {
        const MAX_BONDED: usize = 16;
        let mut entries = [BondedDeviceEntry {
            address_type: 0,
            address: [0; 6],
        }; MAX_BONDED];
        let mut num: u8 = 0;

        unsafe {
            let mut list_size: u8 = 0;
            let status = hci_le_read_resolving_list_size(&mut list_size);
            if status != BLE_STATUS_SUCCESS {
                warn!("hci_le_read_resolving_list_size failed: 0x{:02X}", status);
            } else {
                info!("Resolving list capacity: {} entries", list_size);
            }

            let status = aci_gap_get_bonded_devices(&mut num, entries.as_mut_ptr());
            if status != BLE_STATUS_SUCCESS {
                warn!("aci_gap_get_bonded_devices failed: 0x{:02X}", status);
                return;
            }

            if connected_peer.is_none() {
                info!("  (peer/local RPA omitted: only in use while that peer is connected)");
            }

            for i in 0..(num as usize).min(MAX_BONDED) {
                let e = &entries[i];

                let id = [
                    e.address[5],
                    e.address[4],
                    e.address[3],
                    e.address[2],
                    e.address[1],
                    e.address[0],
                ];

                info!(
                    "resolving_list[{}]: identity type={} addr={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    i, e.address_type, id[0], id[1], id[2], id[3], id[4], id[5]
                );

                // Only the connected peer has an address in use; for any other entry
                // the query returns a stale value (see the doc comment above), so do
                // not print one.
                if !matches!(connected_peer, Some((t, a)) if t == e.address_type && a == e.address) {
                    continue;
                }

                let mut peer_rpa = [0u8; 6];
                let peer_status =
                    hci_le_read_peer_resolvable_address(e.address_type, e.address.as_ptr(), peer_rpa.as_mut_ptr());

                let mut local_rpa = [0u8; 6];
                let local_status =
                    hci_le_read_local_resolvable_address(e.address_type, e.address.as_ptr(), local_rpa.as_mut_ptr());

                if peer_status == BLE_STATUS_SUCCESS {
                    info!(
                        "  peer_rpa  = {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        peer_rpa[5], peer_rpa[4], peer_rpa[3], peer_rpa[2], peer_rpa[1], peer_rpa[0]
                    );
                } else {
                    warn!(
                        "  peer_rpa: read failed status=0x{:02X} (0x02 = Unknown Conn Id => IRK missing)",
                        peer_status
                    );
                }

                if local_status == BLE_STATUS_SUCCESS {
                    info!(
                        "  local_rpa = {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        local_rpa[5], local_rpa[4], local_rpa[3], local_rpa[2], local_rpa[1], local_rpa[0]
                    );
                } else {
                    warn!("  local_rpa: read failed status=0x{:02X}", local_status);
                }
            }
        }
    }

    #[cfg(not(feature = "defmt"))]
    pub fn log_resolving_list_diagnostics(&self) {}

    /// Number of bonded peers in the stack database.
    pub fn bonded_device_count(&self) -> Result<u8, BleError> {
        const MAX_BONDED: usize = 16;
        let mut entries = [BondedDeviceEntry {
            address_type: 0,
            address: [0; 6],
        }; MAX_BONDED];
        let mut num: u8 = 0;

        unsafe {
            let status = aci_gap_get_bonded_devices(&mut num, entries.as_mut_ptr());
            if status == BLE_STATUS_SUCCESS {
                Ok(num)
            } else {
                Err(BleError::CommandFailed(Status::from_u8(status)))
            }
        }
    }

    /// Whether `address` belongs to a peer already in the bond database.
    ///
    /// Pass the address reported by the connection-complete event. Once address
    /// resolution is working the controller reports a bonded peer's *identity*
    /// address there, so it can be compared against the bond database directly.
    ///
    /// Useful for deciding whether to send a peripheral security request: a peer
    /// that is already bonded will start encryption from its own side, and asking
    /// it to authenticate again is what ST's reference deliberately avoids.
    pub fn is_bonded_address(&self, address_type: u8, address: &[u8; 6]) -> bool {
        const MAX_BONDED: usize = 16;
        let mut entries = [BondedDeviceEntry {
            address_type: 0,
            address: [0; 6],
        }; MAX_BONDED];
        let mut num: u8 = 0;

        unsafe {
            if aci_gap_get_bonded_devices(&mut num, entries.as_mut_ptr()) != BLE_STATUS_SUCCESS {
                return false;
            }

            entries[..(num as usize).min(MAX_BONDED)]
                .iter()
                .any(|e| e.address_type == address_type && &e.address == address)
        }
    }

    /// Append bonded peers to resolving list + FAL (ST mode `0x04`).
    ///
    /// Used after disconnect in `BLE_Privacy_Peripheral` before RPA advertising.
    pub fn append_bond_lists_for_reconnect(&self) -> Result<usize, BleError> {
        const GAP_ADD_DEV_MODE_APPEND_BOTH: u8 = 0x04;

        const MAX_BONDED: usize = 16;
        let mut entries = [BondedDeviceEntry {
            address_type: 0,
            address: [0; 6],
        }; MAX_BONDED];
        let mut num: u8 = 0;

        unsafe {
            let status = aci_gap_get_bonded_devices(&mut num, entries.as_mut_ptr());
            if status != BLE_STATUS_SUCCESS {
                return Err(BleError::CommandFailed(Status::from_u8(status)));
            }

            if num == 0 {
                return Ok(0);
            }

            let count = (num as usize).min(MAX_BONDED) as u8;

            // No HCI_LE_Set_Address_Resolution_Enable here on purpose; GAP owns the
            // resolution state once `aci_gap_init` runs with privacy 0x02. See
            // `configure_filter_and_resolving_list` for what forcing it breaks.
            let status = aci_gap_add_devices_to_list(
                count,
                entries.as_ptr() as *const ListEntry,
                GAP_ADD_DEV_MODE_APPEND_BOTH,
            );
            if status != BLE_STATUS_SUCCESS {
                return Err(BleError::CommandFailed(Status::from_u8(status)));
            }

            Ok(count as usize)
        }
    }

    /// Restore resolving-list IRKs for bonded RPAs without touching the filter accept list.
    ///
    /// Use before advertising with `AdvFilterPolicy::All` so centrals can still discover
    /// the peripheral name in a scan while iOS reconnects can resolve the peer's RPA.
    pub fn restore_resolving_list_for_privacy(&self) -> Result<usize, BleError> {
        let count = self.populate_resolving_list_from_bonds()?;
        if count > 0 {
            self.set_address_resolution_enable(true)?;
        }
        Ok(count)
    }

    /// Compatibility alias for ST guide naming (`add_devices_to_resolving_list`-style flow).
    ///
    /// Restores bonded peer IRKs into the resolving list.
    pub fn add_devices_to_resolving_list_from_bonds(&self) -> Result<usize, BleError> {
        self.populate_resolving_list_from_bonds()
    }

    /// Restore bond-based connection filtering for RPA centrals (e.g. iOS).
    ///
    /// Alias for [`configure_filter_and_resolving_list`](Self::configure_filter_and_resolving_list).
    pub fn restore_bond_lists(&self) -> Result<usize, BleError> {
        self.configure_filter_and_resolving_list()
    }

    /// ST HAL firmware warning: SMP unexpected LTK request (bond lookup failed).
    pub const FW_ERROR_SMP_UNEXPECTED_LTK: u8 = 0x06;

    /// Check if security has been initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
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
