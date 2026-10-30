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
//! security.set_authentication_requirements(ble.controller(), params).await?;
//! ```

use bt_hci::cmd::le::{
    LeClearFilterAcceptList, LeClearResolvingList, LeSetAddrResolutionEnable, LeSetResolvablePrivateAddrTimeout,
};
// Reading the resolving list back is what the `defmt` diagnostics do; the
// `not(defmt)` stubs of the `log_*` helpers skip it.
#[cfg(feature = "defmt")]
use bt_hci::cmd::le::{LeReadLocalResolvableAddr, LeReadPeerResolvableAddr, LeReadResolvingListSize};
use bt_hci::cmd::{AsyncCmd, SyncCmd};
#[cfg(feature = "defmt")]
use bt_hci::param::AddrKind;
use bt_hci::param::{BdAddr, ConnHandle, Duration};
use stm32wb_hci::aci::AciEvent;
use stm32wb_hci::aci::gap::{
    GapAddDevicesToList, GapAllowRebond, GapAuthorizationResp, GapCheckBondedDevice, GapClearSecurityDb,
    GapGetBondedDevices, GapNumericComparisonValueConfirmYesno, GapPairingRequestReply, GapPassKeyResp,
    GapPasskeyInput, GapPeripheralSecurityReq, GapRemoveBondedDevice, GapSendPairingReq,
    GapSetAuthenticationRequirement, GapSetIoCapability, ListEntry,
};
use stm32wb_hci::aci::general::AciWriteConfigData;
use stm32wb_hci::aci::ranges::Passkey;
/// The ACI value types the security manager shares with its callers.
pub use stm32wb_hci::aci::values::AddressType as IdentityAddressType;
use stm32wb_hci::aci::values::{AddDevicesMode, AddressType, AuthorizationResponse, ConfigDataOffset, UseFixedPin};
pub use stm32wb_hci::aci::values::{
    IoCapability, PairingFailureReason, PairingStatus, PasskeyInputType, ScSupport as SecureConnectionsSupport,
};
use stm32wb_hci::wire::OrUnknown;

use crate::bluetooth::error::BleError;
use crate::controller::ControllerAdapter;

/// Maximum number of bonded device entries the HCI layer returns
/// (`GapGetBondedDevices` decodes into a `BoundedArray<_, 35>`).
const MAX_BONDED: usize = 35;

fn address_type_from_raw(ty: u8) -> AddressType {
    match ty {
        0x00 => AddressType::Public,
        _ => AddressType::Random,
    }
}

/// Map a bonded-device address type onto an HCI address kind.
///
/// Only the `defmt` diagnostics read the resolving list back, so this exists
/// for them alone.
#[cfg(feature = "defmt")]
fn addr_kind_from_raw(ty: u8) -> AddrKind {
    match ty {
        0x00 => AddrKind::PUBLIC,
        _ => AddrKind::RANDOM,
    }
}

/// Collect the bonded devices from the security database into `ListEntry` form.
fn bonded_list_entries(bonded: &stm32wb_hci::aci::gap::GapBondedDevices) -> ([ListEntry; MAX_BONDED], usize) {
    let mut entries = [ListEntry {
        address_type: AddressType::Public,
        address: BdAddr::new([0; 6]),
    }; MAX_BONDED];
    let mut count = 0;
    for e in bonded.bonded_device_entry.iter() {
        if count >= MAX_BONDED {
            break;
        }
        entries[count] = ListEntry {
            address_type: address_type_from_raw(e.address_type.to_raw()),
            address: e.address,
        };
        count += 1;
    }
    (entries, count)
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
            secure_connections: SecureConnectionsSupport::Mandatory,
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
    /// Peer address could not be resolved with current privacy data.
    AddressNotResolved { conn_handle: u16 },
    /// Peer keypress-notification during passkey entry.
    KeypressNotification {
        conn_handle: u16,
        /// Keypress notification type (Core Spec Vol 3, Part H, 3.5.8): 0 = entry
        /// started, 1 = digit entered, 2 = digit erased, 3 = cleared, 4 = entry completed.
        notification_type: u8,
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
    /// [`SecurityManager::pairing_request_response`].
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
pub fn from_vendor_event(event: &AciEvent<'_>) -> Option<SecurityEvent> {
    match event {
        AciEvent::GapPairingComplete(e) => {
            let status = PairingStatus::try_from(e.status.to_raw()).unwrap_or(PairingStatus::PairingFailed);
            let reason = match status {
                // Keep these apart: encryption failing against a stored bond is a
                // different fault from a refused pairing negotiation, and the
                // pairing-complete reason code is only defined for the latter.
                PairingStatus::Success | PairingStatus::EncryptionFailed => 0,
                PairingStatus::SmpTimeout | PairingStatus::PairingFailed => e.reason.to_raw(),
            };
            Some(SecurityEvent::PairingComplete {
                conn_handle: e.connection_handle.raw(),
                status,
                reason,
            })
        }
        AciEvent::GapPassKeyReq(e) => Some(SecurityEvent::PasskeyRequest {
            conn_handle: e.connection_handle.raw(),
        }),
        AciEvent::GapNumericComparisonValue(e) => Some(SecurityEvent::NumericComparisonRequest {
            conn_handle: e.connection_handle.raw(),
            numeric_value: e.numeric_value,
        }),
        AciEvent::GapBondLost(e) => Some(SecurityEvent::BondLost {
            conn_handle: e.connection_handle.raw(),
        }),
        AciEvent::GapPairingRequest(e) => Some(SecurityEvent::PairingRequest {
            conn_handle: e.connection_handle.raw(),
            is_bonded: matches!(e.bonded, OrUnknown::Known(true)),
            auth_req: e.auth_req,
        }),
        AciEvent::GapAuthorizationReq(e) => Some(SecurityEvent::AuthorizationRequest {
            conn_handle: e.connection_handle.raw(),
        }),
        AciEvent::GapAddrNotResolved(e) => Some(SecurityEvent::AddressNotResolved {
            conn_handle: e.connection_handle.raw(),
        }),
        AciEvent::GapKeypressNotification(e) => Some(SecurityEvent::KeypressNotification {
            conn_handle: e.connection_handle.raw(),
            notification_type: e.notification_type,
        }),
        _ => None,
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
    pub async fn set_authentication_requirements(
        &mut self,
        controller: &ControllerAdapter<'_>,
        params: SecurityParams,
    ) -> Result<(), BleError> {
        // IO capability determines the pairing method. Must be set before
        // the authentication requirement so the stack uses the right method.
        GapSetIoCapability::new(params.io_capability).exec(controller).await?;

        // `use_fixed_pin` maps directly onto ST's `UseFixedPin` (ALLOWED = 0x00,
        // FORBIDDEN = 0x01): the default (`use_fixed_pin: false`) must disable
        // the deprecated fixed-PIN pairing, not enable it with PIN 000000.
        GapSetAuthenticationRequirement::new(
            params.bonding_mode == BondingMode::Bonding,
            params.mitm_protection,
            params.secure_connections,
            params.keypress_notification,
            params.min_encryption_key_size,
            params.max_encryption_key_size,
            if params.use_fixed_pin {
                UseFixedPin::Yes
            } else {
                UseFixedPin::No
            },
            Passkey::new(params.fixed_pin).unwrap_or_else(|| Passkey::new(0).unwrap()),
            params.identity_address_type,
        )
        .exec(controller)
        .await?;

        self.initialized = true;
        Ok(())
    }

    /// Respond to a passkey request
    ///
    /// Call this when you receive a `SecurityEvent::PasskeyRequest`.
    /// The passkey must be a 6-digit decimal number (0-999999).
    pub async fn pass_key_response(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        passkey: u32,
    ) -> Result<(), BleError> {
        if passkey > 999_999 {
            return Err(BleError::InvalidParameter);
        }

        GapPassKeyResp::new(
            ConnHandle::new(conn_handle),
            // Just validated to be at most 999999.
            Passkey::new(passkey).unwrap(),
        )
        .exec(controller)
        .await?;

        Ok(())
    }

    /// Signal a passkey-entry keypress to the stack.
    ///
    /// During passkey entry the stack can forward keypress notifications to the
    /// peer (requires `keypress_notification` in [`SecurityParams`]). Call this
    /// as the user interacts with the local keypad; the 6-digit value itself is
    /// sent with [`pass_key_response`](Self::pass_key_response) in response to
    /// [`SecurityEvent::PasskeyRequest`].
    pub async fn passkey_input(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        input: PasskeyInputType,
    ) -> Result<(), BleError> {
        GapPasskeyInput::new(ConnHandle::new(conn_handle), input)
            .exec(controller)
            .await?;

        Ok(())
    }

    /// Respond to an authorization request event.
    ///
    /// Call this when you receive a `SecurityEvent::AuthorizationRequest`.
    pub async fn authorization_response(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        authorize: bool,
    ) -> Result<(), BleError> {
        let authorize = if authorize {
            AuthorizationResponse::Authorize
        } else {
            AuthorizationResponse::Reject
        };
        GapAuthorizationResp::new(ConnHandle::new(conn_handle), authorize)
            .exec(controller)
            .await?;

        Ok(())
    }

    /// Respond to a numeric comparison request
    ///
    /// Call this when you receive a `SecurityEvent::NumericComparisonRequest`.
    /// Pass `true` if the numeric values match, `false` otherwise.
    pub async fn numeric_comparison_response(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        confirm: bool,
    ) -> Result<(), BleError> {
        GapNumericComparisonValueConfirmYesno::new(ConnHandle::new(conn_handle), confirm)
            .exec(controller)
            .await?;

        Ok(())
    }

    /// Request pairing/encryption from the peripheral side.
    ///
    /// Sends an SMP Security Request to the central. Call this immediately
    /// after connection when the device has security requirements, rather than
    /// waiting for the central to trigger pairing via an insufficient-security
    /// GATT error. Matches ST's recommended pattern (aci_gap_slave_security_req
    /// in BLE_HeartRate).
    pub async fn request_pairing(&self, controller: &ControllerAdapter<'_>, conn_handle: u16) -> Result<(), BleError> {
        GapPeripheralSecurityReq::new(ConnHandle::new(conn_handle))
            .exec(controller)
            .await?;

        Ok(())
    }

    /// Request pairing/encryption from the central side.
    ///
    /// Sends a pairing request to the peer peripheral. Set `force_rebond` to
    /// restart pairing even when a bond entry exists.
    pub async fn request_pairing_central(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        force_rebond: bool,
    ) -> Result<(), BleError> {
        GapSendPairingReq::new(ConnHandle::new(conn_handle), force_rebond)
            .exec(controller)
            .await?;

        Ok(())
    }

    /// Reply to a pairing request signaled via `SecurityEvent::PairingRequest`.
    ///
    /// Set `accept` to `true` to continue pairing, or `false` to reject it.
    pub async fn pairing_request_response(
        &self,
        controller: &ControllerAdapter<'_>,
        conn_handle: u16,
        accept: bool,
    ) -> Result<(), BleError> {
        GapPairingRequestReply::new(ConnHandle::new(conn_handle), accept)
            .exec(controller)
            .await?;

        Ok(())
    }

    /// Allow rebonding after receiving BondLost event
    ///
    /// Call this when you receive a `SecurityEvent::BondLost` to allow
    /// the pairing process to continue.
    pub async fn allow_rebond(&self, controller: &ControllerAdapter<'_>, conn_handle: u16) -> Result<(), BleError> {
        GapAllowRebond::new(ConnHandle::new(conn_handle))
            .exec(controller)
            .await?;

        Ok(())
    }

    /// Clear all bonding information from the security database
    ///
    /// This removes all stored bonds. Use with caution.
    pub async fn clear_security_database(&self, controller: &ControllerAdapter<'_>) -> Result<(), BleError> {
        GapClearSecurityDb::new().exec(controller).await?;

        Ok(())
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
    pub async fn clear_bond_lists(&self, controller: &ControllerAdapter<'_>) -> Result<(), BleError> {
        LeSetAddrResolutionEnable::new(false).exec(controller).await?;
        LeClearResolvingList::new().exec(controller).await?;
        LeClearFilterAcceptList::new().exec(controller).await?;

        Ok(())
    }

    /// Remove a specific bonded device
    pub async fn remove_bonded_device(
        &self,
        controller: &ControllerAdapter<'_>,
        address_type: IdentityAddressType,
        address: &[u8; 6],
    ) -> Result<(), BleError> {
        GapRemoveBondedDevice::new(address_type, BdAddr::new(*address))
            .exec(controller)
            .await?;

        Ok(())
    }

    /// Set the host's SMP mode bitmap (`CONFIG_DATA_SMP_MODE`).
    ///
    /// Must be called before pairing starts. See [`SmpMode`] for the flags.
    /// Note that setting [`SmpMode::PAIRING_REQUEST_EVENT`] makes the stack wait
    /// for [`pairing_request_response`](Self::pairing_request_response) on every
    /// incoming Pairing Request, so the application must handle
    /// [`SecurityEvent::PairingRequest`] or pairing will stall.
    pub async fn set_smp_mode(&self, controller: &ControllerAdapter<'_>, mode: u8) -> Result<(), BleError> {
        AciWriteConfigData::try_new(ConfigDataOffset::SmpMode, &[mode])
            .unwrap()
            .exec(controller)
            .await?;

        Ok(())
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
    pub async fn check_bonded_device(
        &self,
        controller: &ControllerAdapter<'_>,
        address_type: u8,
        address: &[u8; 6],
    ) -> Option<(u8, [u8; 6])> {
        let identity = GapCheckBondedDevice::new(address_type_from_raw(address_type), BdAddr::new(*address))
            .exec(controller)
            .await
            .ok()?;

        let id_address: [u8; 6] = identity.id_address.raw().try_into().ok()?;
        Some((identity.id_address_type, id_address))
    }

    /// Run [`check_bonded_device`](Self::check_bonded_device) against every bonded
    /// peer's own identity address (debug).
    ///
    /// This is the positive control for the RPA lookups: an identity address that
    /// `GapGetBondedDevices` just returned needs neither an IRK nor any
    /// crypto to match, so success is the only sane outcome. A failure here means
    /// the command is unusable and its verdict on RPAs carries no information.
    #[cfg(feature = "defmt")]
    pub async fn log_check_bonded_identities(&self, controller: &ControllerAdapter<'_>) {
        let bonded = match GapGetBondedDevices::new().exec(controller).await {
            Ok(bonded) => bonded,
            Err(e) => {
                warn!("aci_gap_get_bonded_devices failed: {:?}", BleError::from(e));
                return;
            }
        };

        for (i, e) in bonded.bonded_device_entry.iter().enumerate() {
            let address: [u8; 6] = e.address.raw().try_into().unwrap_or([0; 6]);
            match self
                .check_bonded_device(controller, e.address_type.to_raw(), &address)
                .await
            {
                Some((id_type, id)) => info!(
                    "  check_bonded_device(identity[{}]) OK -> type={} {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    i, id_type, id[5], id[4], id[3], id[2], id[1], id[0]
                ),
                None => warn!("  check_bonded_device(identity[{}]) FAILED -- probe is unreliable", i),
            }
        }
    }

    #[cfg(not(feature = "defmt"))]
    pub async fn log_check_bonded_identities(&self, _controller: &ControllerAdapter<'_>) {}

    /// Check if a device is bonded
    ///
    /// Success means bonded; any non-zero status from the stack (e.g. 0x42,
    /// "device not bonded") means it is not, so a failed command maps to
    /// `Ok(false)` rather than an error.
    pub async fn is_device_bonded(
        &self,
        controller: &ControllerAdapter<'_>,
        address_type: IdentityAddressType,
        address: &[u8; 6],
    ) -> Result<bool, BleError> {
        let bonded = GapCheckBondedDevice::new(address_type, BdAddr::new(*address))
            .exec(controller)
            .await
            .is_ok();

        Ok(bonded)
    }

    /// Enable or disable address resolution
    ///
    /// When enabled, the controller will automatically resolve resolvable
    /// private addresses (RPA) using stored IRKs.
    pub async fn set_address_resolution_enable(
        &self,
        controller: &ControllerAdapter<'_>,
        enable: bool,
    ) -> Result<(), BleError> {
        LeSetAddrResolutionEnable::new(enable).exec(controller).await?;

        Ok(())
    }

    /// Set RPA (Resolvable Private Address) timeout
    ///
    /// Sets the time interval after which a new RPA is generated.
    /// Valid range: 1 to 41400 seconds (approximately 11.5 hours).
    /// Default is usually 900 seconds (15 minutes).
    pub async fn set_rpa_timeout(
        &self,
        controller: &ControllerAdapter<'_>,
        timeout_seconds: u16,
    ) -> Result<(), BleError> {
        let timeout = timeout_seconds.clamp(1, 41400);

        LeSetResolvablePrivateAddrTimeout::new(Duration::<1_000_000>::from_secs(timeout as u32))
            .exec(controller)
            .await?;

        Ok(())
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
    /// Uses `AddDevicesMode::SetResolvingList` (clear and set the **resolving list
    /// only**), leaving the Filter Accept List untouched, so it is idempotent and
    /// won't interfere with `GapSetDiscoverable` advertising. Safe to call
    /// repeatedly, e.g. after every disconnect before re-advertising.
    ///
    /// Must NOT be called while advertising, scanning, or connecting are active
    /// when address resolution is enabled (Core Spec restriction). Returns the
    /// number of bonded devices restored to the resolving list.
    pub async fn populate_resolving_list_from_bonds(
        &self,
        controller: &ControllerAdapter<'_>,
    ) -> Result<usize, BleError> {
        let bonded = GapGetBondedDevices::new().exec(controller).await?;

        if bonded.bonded_device_entry.is_empty() {
            // Clear rather than return: leaving the previous entries in place
            // would keep the controller resolving against IRKs whose bonds are
            // gone. Translation must be off to mutate the list, and with no
            // bonds there is nothing to resolve against anyway.
            LeSetAddrResolutionEnable::new(false).exec(controller).await?;
            LeClearResolvingList::new().exec(controller).await?;
            return Ok(0);
        }

        let (entries, count) = bonded_list_entries(&bonded);
        let cmd = GapAddDevicesToList::try_new(&entries[..count], AddDevicesMode::SetResolvingList)
            .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;

        Ok(count)
    }

    /// Populate the controller's Filter Accept List with all bonded devices'
    /// identity addresses. This is the maintained ST pattern
    /// from `BLE_p2pServer` for restoring bond-based filtering on boot.
    ///
    /// For LE Legacy bonded reconnect, the controller looks up the LTK by the
    /// EDIV/RAND values in `LL_ENC_REQ`, so populating the Filter Accept List
    /// here lets the controller recognise the bonded peer at the LL layer
    /// without needing RPA resolution at the host level.
    ///
    /// Must NOT be called while advertising/scanning/initiating is active.
    pub async fn configure_filter_accept_list(&self, controller: &ControllerAdapter<'_>) -> Result<(), BleError> {
        // CubeWBA 1.10 removed `ACI_GAP_CONFIGURE_FILTER_ACCEPT_LIST`. Its
        // replacement is `ACI_GAP_ADD_DEVICES_TO_LIST` mode 0x0A ("append the
        // bonded devices plus the devices provided as parameters to the Filter
        // Accept List only"). stm32wb-hci's `AddDevicesMode` has no variant for
        // the 1.10 SDB modes, so pass the bonded devices read back from
        // `GapGetBondedDevices` explicitly with the equivalent plain
        // `AppendFilterAcceptList` (0x02) mode instead.
        let bonded = GapGetBondedDevices::new().exec(controller).await?;
        let (entries, count) = bonded_list_entries(&bonded);
        let cmd = GapAddDevicesToList::try_new(&entries[..count], AddDevicesMode::AppendFilterAcceptList)
            .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;

        Ok(())
    }

    /// Compatibility alias for ST guide naming (`configure_whitelist`).
    pub async fn configure_whitelist(&self, controller: &ControllerAdapter<'_>) -> Result<(), BleError> {
        self.configure_filter_accept_list(controller).await
    }

    /// Rebuild the controller's resolving list from the bonds currently in the
    /// security database, and clear the Filter Accept List.
    ///
    /// CubeWBA 1.10 mode 0x0D ("clear and add the bonded devices plus the devices
    /// provided as parameters to both lists") atomically clears both lists and
    /// repopulates them from the security database. GAP therefore owns the full
    /// operation and keeps its internal privacy state synchronized with the
    /// controller. stm32wb-hci's `AddDevicesMode` has no variant for the 1.10 SDB
    /// modes, so this passes the bonded devices read back from
    /// `GapGetBondedDevices` explicitly with the equivalent `SetBoth` (0x05)
    /// mode, which clears both lists and adds the given entries to both.
    ///
    /// Address resolution is then enabled iff at least one bond was programmed,
    /// so the controller's translation state always matches the list contents.
    ///
    /// Must NOT be called while advertising, scanning, or initiating is active.
    /// Returns the number of bonds programmed.
    pub async fn configure_filter_and_resolving_list(
        &self,
        controller: &ControllerAdapter<'_>,
    ) -> Result<usize, BleError> {
        let bonded = GapGetBondedDevices::new().exec(controller).await?;
        let count = bonded.bonded_device_entry.len();

        let (entries, n) = bonded_list_entries(&bonded);
        let cmd = GapAddDevicesToList::try_new(&entries[..n], AddDevicesMode::SetBoth)
            .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;

        // Mode 0x05/0x0D only loads the list; it does not touch the translation
        // enable. `clear_bond_lists` has to disable resolution to mutate the
        // list at all, so without this the controller keeps a correctly
        // populated resolving list that it is not permitted to use, and every
        // bonded reconnect arrives as an unresolved RPA.
        LeSetAddrResolutionEnable::new(count > 0).exec(controller).await?;

        Ok(count)
    }

    /// Log bonded peer identity addresses from the stack database (debug).
    #[cfg(feature = "defmt")]
    pub async fn log_bonded_devices(&self, controller: &ControllerAdapter<'_>) {
        let bonded = match GapGetBondedDevices::new().exec(controller).await {
            Ok(bonded) => bonded,
            Err(e) => {
                warn!("aci_gap_get_bonded_devices failed: {:?}", BleError::from(e));
                return;
            }
        };

        for (i, e) in bonded.bonded_device_entry.iter().enumerate() {
            let address = e.address.raw();
            info!(
                "bond[{}]: type={} addr={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                i,
                e.address_type.to_raw(),
                address[5],
                address[4],
                address[3],
                address[2],
                address[1],
                address[0]
            );
        }
    }

    #[cfg(not(feature = "defmt"))]
    pub async fn log_bonded_devices(&self, _controller: &ControllerAdapter<'_>) {}

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
    /// identity_address)` in the same byte order `GapGetBondedDevices` returns,
    /// or `None` when disconnected.
    #[cfg(feature = "defmt")]
    pub async fn log_resolving_list_diagnostics(
        &self,
        controller: &ControllerAdapter<'_>,
        connected_peer: Option<(u8, [u8; 6])>,
    ) {
        match LeReadResolvingListSize::new().exec(controller).await {
            Ok(list_size) => info!("Resolving list capacity: {} entries", list_size),
            Err(e) => warn!("hci_le_read_resolving_list_size failed: {:?}", BleError::from(e)),
        }

        let bonded = match GapGetBondedDevices::new().exec(controller).await {
            Ok(bonded) => bonded,
            Err(e) => {
                warn!("aci_gap_get_bonded_devices failed: {:?}", BleError::from(e));
                return;
            }
        };

        if connected_peer.is_none() {
            info!("  (peer/local RPA omitted: only in use while that peer is connected)");
        }

        for (i, e) in bonded.bonded_device_entry.iter().enumerate() {
            let entry_type = e.address_type.to_raw();
            let entry_address: [u8; 6] = e.address.raw().try_into().unwrap_or([0; 6]);

            let id = [
                entry_address[5],
                entry_address[4],
                entry_address[3],
                entry_address[2],
                entry_address[1],
                entry_address[0],
            ];

            info!(
                "resolving_list[{}]: identity type={} addr={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                i, entry_type, id[0], id[1], id[2], id[3], id[4], id[5]
            );

            // Only the connected peer has an address in use; for any other entry
            // the query returns a stale value (see the doc comment above), so do
            // not print one.
            if !matches!(connected_peer, Some((t, a)) if t == entry_type && a == entry_address) {
                continue;
            }

            let addr_kind = addr_kind_from_raw(entry_type);
            let peer = LeReadPeerResolvableAddr::new(addr_kind, BdAddr::new(entry_address))
                .exec(controller)
                .await;
            let local = LeReadLocalResolvableAddr::new(addr_kind, BdAddr::new(entry_address))
                .exec(controller)
                .await;

            match peer {
                Ok(peer_rpa) => {
                    let a = peer_rpa.raw();
                    info!(
                        "  peer_rpa  = {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        a[5], a[4], a[3], a[2], a[1], a[0]
                    );
                }
                Err(bt_hci::cmd::Error::Hci(status)) => warn!(
                    "  peer_rpa: read failed status=0x{:02X} (0x02 = Unknown Conn Id => IRK missing)",
                    status.to_status().into_inner()
                ),
                Err(e) => warn!("  peer_rpa: read failed: {:?}", BleError::from(e)),
            }

            match local {
                Ok(local_rpa) => {
                    let a = local_rpa.raw();
                    info!(
                        "  local_rpa = {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        a[5], a[4], a[3], a[2], a[1], a[0]
                    );
                }
                Err(bt_hci::cmd::Error::Hci(status)) => {
                    warn!(
                        "  local_rpa: read failed status=0x{:02X}",
                        status.to_status().into_inner()
                    )
                }
                Err(e) => warn!("  local_rpa: read failed: {:?}", BleError::from(e)),
            }
        }
    }

    #[cfg(not(feature = "defmt"))]
    pub async fn log_resolving_list_diagnostics(
        &self,
        _controller: &ControllerAdapter<'_>,
        _connected_peer: Option<(u8, [u8; 6])>,
    ) {
    }

    /// Number of bonded peers in the stack database.
    pub async fn bonded_device_count(&self, controller: &ControllerAdapter<'_>) -> Result<u8, BleError> {
        let bonded = GapGetBondedDevices::new().exec(controller).await?;

        Ok(bonded.bonded_device_entry.len() as u8)
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
    pub async fn is_bonded_address(
        &self,
        controller: &ControllerAdapter<'_>,
        address_type: u8,
        address: &[u8; 6],
    ) -> bool {
        let bonded = match GapGetBondedDevices::new().exec(controller).await {
            Ok(bonded) => bonded,
            Err(_) => return false,
        };

        bonded
            .bonded_device_entry
            .iter()
            .any(|e| e.address_type.to_raw() == address_type && e.address.raw() == address)
    }

    /// Append bonded peers to resolving list + FAL (ST mode `0x04`).
    ///
    /// Used after disconnect in `BLE_Privacy_Peripheral` before RPA advertising.
    pub async fn append_bond_lists_for_reconnect(&self, controller: &ControllerAdapter<'_>) -> Result<usize, BleError> {
        let bonded = GapGetBondedDevices::new().exec(controller).await?;

        if bonded.bonded_device_entry.is_empty() {
            return Ok(0);
        }

        let (entries, count) = bonded_list_entries(&bonded);

        // No LeSetAddrResolutionEnable here on purpose; GAP owns the
        // resolution state once `GapInit` runs with privacy 0x02. See
        // `configure_filter_and_resolving_list` for what forcing it breaks.
        let cmd = GapAddDevicesToList::try_new(&entries[..count], AddDevicesMode::AppendBoth)
            .map_err(|_| BleError::InvalidParameter)?;
        cmd.exec(controller).await?;

        Ok(count)
    }

    /// Restore resolving-list IRKs for bonded RPAs without touching the filter accept list.
    ///
    /// Use before advertising with `AdvFilterPolicy::All` so centrals can still discover
    /// the peripheral name in a scan while iOS reconnects can resolve the peer's RPA.
    pub async fn restore_resolving_list_for_privacy(
        &self,
        controller: &ControllerAdapter<'_>,
    ) -> Result<usize, BleError> {
        let count = self.populate_resolving_list_from_bonds(controller).await?;
        if count > 0 {
            self.set_address_resolution_enable(controller, true).await?;
        }
        Ok(count)
    }

    /// Compatibility alias for ST guide naming (`add_devices_to_resolving_list`-style flow).
    ///
    /// Restores bonded peer IRKs into the resolving list.
    pub async fn add_devices_to_resolving_list_from_bonds(
        &self,
        controller: &ControllerAdapter<'_>,
    ) -> Result<usize, BleError> {
        self.populate_resolving_list_from_bonds(controller).await
    }

    /// Restore bond-based connection filtering for RPA centrals (e.g. iOS).
    ///
    /// Alias for [`configure_filter_and_resolving_list`](Self::configure_filter_and_resolving_list).
    pub async fn restore_bond_lists(&self, controller: &ControllerAdapter<'_>) -> Result<usize, BleError> {
        self.configure_filter_and_resolving_list(controller).await
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
