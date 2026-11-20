//! GAP types and constants

use bt_hci::param::AddrKind;
pub use bt_hci::param::BdAddr;
/// Own address type used for advertising, scanning and initiating.
///
/// The ACI `Own_Address_Type` values, declared by `stm32wb-hci`.
pub use stm32wb_hci::aci::values::OwnAddressType;
/// Advertising or scan response data (at most 31 bytes), built by
/// `stm32wb-hci`.
pub use stm32wb_hci::adv_data::AdvData;

use crate::bluetooth::hci::AdvFilterPolicy;

/// A Bluetooth device address together with its type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BdAddrType {
    /// Public device address
    Public(BdAddr),
    /// Random device address
    Random(BdAddr),
}

impl BdAddrType {
    /// Build from an HCI address type and address.
    ///
    /// Resolved identity addresses (types 0x02 and 0x03) map to their public or
    /// random identity.
    pub fn new(kind: AddrKind, addr: BdAddr) -> Self {
        if kind == AddrKind::PUBLIC || kind == AddrKind::RESOLVABLE_PRIVATE_OR_PUBLIC {
            Self::Public(addr)
        } else {
            Self::Random(addr)
        }
    }

    /// The address, without its type
    pub fn addr(&self) -> BdAddr {
        match self {
            Self::Public(a) | Self::Random(a) => *a,
        }
    }

    /// The HCI address type: 0x00 for public, 0x01 for random
    pub fn kind(&self) -> u8 {
        match self {
            Self::Public(_) => 0x00,
            Self::Random(_) => 0x01,
        }
    }

    /// The address bytes, least significant byte first
    pub fn bytes(&self) -> [u8; 6] {
        let mut bytes = [0; 6];
        bytes.copy_from_slice(self.addr().raw());
        bytes
    }
}

/// Advertising type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AdvType {
    /// Connectable undirected advertising
    ConnectableUndirected,
    /// Connectable directed advertising (high duty cycle)
    ConnectableDirectedHighDuty,
    /// Scannable undirected advertising
    ScannableUndirected,
    /// Non-connectable undirected advertising
    NonConnectableUndirected,
    /// Connectable directed advertising (low duty cycle)
    ConnectableDirectedLowDuty,
}

/// Advertising parameters
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AdvParams {
    /// Minimum advertising interval in units of 0.625ms (Range: 0x0020 to 0x4000)
    /// Default: 0x0800 (1.28 seconds)
    pub interval_min: u16,

    /// Maximum advertising interval in units of 0.625ms (Range: 0x0020 to 0x4000)
    /// Default: 0x0800 (1.28 seconds)
    pub interval_max: u16,

    /// Advertising type
    pub adv_type: AdvType,

    /// Own address type
    pub own_addr_type: OwnAddressType,

    /// Advertising filter policy
    pub filter_policy: AdvFilterPolicy,

    /// Peer address for directed advertising.
    ///
    /// Required for [`AdvType::ConnectableDirectedHighDuty`] and
    /// [`AdvType::ConnectableDirectedLowDuty`]; ignored for undirected
    /// advertising.
    pub peer_addr: Option<BdAddrType>,

    /// Advertising channel map (bit 0: channel 37, bit 1: channel 38, bit 2: channel 39)
    /// Default: 0x07 (all channels)
    pub channel_map: u8,

    /// Use `aci_gap_set_undirected_connectable` + `aci_gap_update_adv_data` instead of
    /// `aci_gap_set_discoverable` (required for ST controller privacy + RPA advertising).
    pub privacy_undirected: bool,
}

impl Default for AdvParams {
    fn default() -> Self {
        Self {
            interval_min: 0x0800, // 1.28 seconds
            interval_max: 0x0800, // 1.28 seconds
            adv_type: AdvType::ConnectableUndirected,
            // Matches the default address type configured in `GapInitParams`.
            own_addr_type: OwnAddressType::StaticRandom,
            filter_policy: AdvFilterPolicy::All,
            peer_addr: None,
            channel_map: 0x07, // All channels
            privacy_undirected: false,
        }
    }
}
