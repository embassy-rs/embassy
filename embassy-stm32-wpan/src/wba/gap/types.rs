//! GAP types and constants

use crate::wba::error::BleError;
// Re-export HCI types for convenience
pub use crate::wba::hci::types::OwnAddressType;
use crate::wba::hci::types::{AdvFilterPolicy, AdvType as HciAdvType};

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

impl From<AdvType> for HciAdvType {
    fn from(adv_type: AdvType) -> Self {
        match adv_type {
            AdvType::ConnectableUndirected => HciAdvType::ConnectableUndirected,
            AdvType::ConnectableDirectedHighDuty => HciAdvType::ConnectableDirectedHighDutyCycle,
            AdvType::ScannableUndirected => HciAdvType::ScannableUndirected,
            AdvType::NonConnectableUndirected => HciAdvType::NonConnectableUndirected,
            AdvType::ConnectableDirectedLowDuty => HciAdvType::ConnectableDirectedLowDutyCycle,
        }
    }
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

    /// Advertising channel map (bit 0: channel 37, bit 1: channel 38, bit 2: channel 39)
    /// Default: 0x07 (all channels)
    pub channel_map: u8,
}

impl Default for AdvParams {
    fn default() -> Self {
        Self {
            interval_min: 0x0800, // 1.28 seconds
            interval_max: 0x0800, // 1.28 seconds
            adv_type: AdvType::ConnectableUndirected,
            own_addr_type: OwnAddressType::Public,
            filter_policy: AdvFilterPolicy::All,
            channel_map: 0x07, // All channels
        }
    }
}

/// AD Type constants for advertising data formatting
#[allow(dead_code)]
mod ad_type {
    pub const FLAGS: u8 = 0x01;
    pub const INCOMPLETE_LIST_16BIT_SERVICE_UUIDS: u8 = 0x02;
    pub const COMPLETE_LIST_16BIT_SERVICE_UUIDS: u8 = 0x03;
    pub const INCOMPLETE_LIST_32BIT_SERVICE_UUIDS: u8 = 0x04;
    pub const COMPLETE_LIST_32BIT_SERVICE_UUIDS: u8 = 0x05;
    pub const INCOMPLETE_LIST_128BIT_SERVICE_UUIDS: u8 = 0x06;
    pub const COMPLETE_LIST_128BIT_SERVICE_UUIDS: u8 = 0x07;
    pub const SHORTENED_LOCAL_NAME: u8 = 0x08;
    pub const COMPLETE_LOCAL_NAME: u8 = 0x09;
    pub const TX_POWER_LEVEL: u8 = 0x0A;
    pub const MANUFACTURER_SPECIFIC_DATA: u8 = 0xFF;
}

/// Advertising data builder
///
/// Constructs advertising data according to the Bluetooth Core Specification.
/// Each data element consists of:
/// - Length (1 byte): length of type + data
/// - Type (1 byte): AD type
/// - Data (0-29 bytes): type-specific data
///
/// Maximum advertising data length is 31 bytes.
#[derive(Clone)]
pub struct AdvData {
    data: heapless::Vec<u8, 31>,
}

impl AdvData {
    /// Create a new empty advertising data builder
    pub fn new() -> Self {
        Self {
            data: heapless::Vec::new(),
        }
    }

    /// Add flags to advertising data
    ///
    /// Common flag values:
    /// - 0x01: LE Limited Discoverable Mode
    /// - 0x02: LE General Discoverable Mode
    /// - 0x04: BR/EDR Not Supported
    /// - 0x05: Simultaneous LE and BR/EDR to Same Device Capable (Controller)
    /// - 0x06: Simultaneous LE and BR/EDR to Same Device Capable (Host)
    ///
    /// Typical value: 0x06 (General Discoverable + BR/EDR Not Supported)
    pub fn add_flags(&mut self, flags: u8) -> Result<&mut Self, BleError> {
        self.add_field(ad_type::FLAGS, &[flags])
    }

    /// Add complete local name to advertising data
    pub fn add_name(&mut self, name: &str) -> Result<&mut Self, BleError> {
        self.add_field(ad_type::COMPLETE_LOCAL_NAME, name.as_bytes())
    }

    /// Add shortened local name to advertising data
    pub fn add_short_name(&mut self, name: &str) -> Result<&mut Self, BleError> {
        self.add_field(ad_type::SHORTENED_LOCAL_NAME, name.as_bytes())
    }

    /// Add 16-bit service UUID (complete list)
    pub fn add_service_uuid_16(&mut self, uuid: u16) -> Result<&mut Self, BleError> {
        let uuid_bytes = uuid.to_le_bytes();
        self.add_field(ad_type::COMPLETE_LIST_16BIT_SERVICE_UUIDS, &uuid_bytes)
    }

    /// Add multiple 16-bit service UUIDs (complete list)
    pub fn add_service_uuids_16(&mut self, uuids: &[u16]) -> Result<&mut Self, BleError> {
        let mut uuid_bytes = heapless::Vec::<u8, 30>::new();
        for uuid in uuids {
            uuid_bytes
                .extend_from_slice(&uuid.to_le_bytes())
                .map_err(|_| BleError::BufferFull)?;
        }
        self.add_field(ad_type::COMPLETE_LIST_16BIT_SERVICE_UUIDS, &uuid_bytes)
    }

    /// Add 128-bit service UUID (complete list)
    pub fn add_service_uuid_128(&mut self, uuid: &[u8; 16]) -> Result<&mut Self, BleError> {
        self.add_field(ad_type::COMPLETE_LIST_128BIT_SERVICE_UUIDS, uuid)
    }

    /// Add TX power level
    pub fn add_tx_power(&mut self, power: i8) -> Result<&mut Self, BleError> {
        self.add_field(ad_type::TX_POWER_LEVEL, &[power as u8])
    }

    /// Add manufacturer-specific data
    ///
    /// Format: 2-byte company ID (little-endian) followed by data
    pub fn add_manufacturer_data(&mut self, company_id: u16, data: &[u8]) -> Result<&mut Self, BleError> {
        let mut mfg_data = heapless::Vec::<u8, 29>::new();
        mfg_data
            .extend_from_slice(&company_id.to_le_bytes())
            .map_err(|_| BleError::BufferFull)?;
        mfg_data.extend_from_slice(data).map_err(|_| BleError::BufferFull)?;
        self.add_field(ad_type::MANUFACTURER_SPECIFIC_DATA, &mfg_data)
    }

    /// Add raw field to advertising data
    fn add_field(&mut self, ad_type: u8, data: &[u8]) -> Result<&mut Self, BleError> {
        let field_len = 1 + data.len(); // Type + data

        if field_len > 255 {
            return Err(BleError::InvalidParameter);
        }

        if self.data.len() + field_len + 1 > 31 {
            return Err(BleError::BufferFull);
        }

        // Add length byte
        self.data.push(field_len as u8).map_err(|_| BleError::BufferFull)?;

        // Add type byte
        self.data.push(ad_type).map_err(|_| BleError::BufferFull)?;

        // Add data
        self.data.extend_from_slice(data).map_err(|_| BleError::BufferFull)?;

        Ok(self)
    }

    /// Get the advertising data as a byte slice
    pub fn build(&self) -> &[u8] {
        &self.data
    }

    /// Get the length of the advertising data
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the advertising data is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for AdvData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adv_data_flags() {
        let mut adv_data = AdvData::new();
        adv_data.add_flags(0x06).unwrap();

        assert_eq!(adv_data.build(), &[0x02, 0x01, 0x06]);
    }

    #[test]
    fn test_adv_data_name() {
        let mut adv_data = AdvData::new();
        adv_data.add_name("Test").unwrap();

        assert_eq!(adv_data.build(), &[0x05, 0x09, b'T', b'e', b's', b't']);
    }

    #[test]
    fn test_adv_data_multiple_fields() {
        let mut adv_data = AdvData::new();
        adv_data.add_flags(0x06).unwrap();
        adv_data.add_name("BLE").unwrap();

        assert_eq!(
            adv_data.build(),
            &[
                0x02, 0x01, 0x06, // Flags
                0x04, 0x09, b'B', b'L', b'E' // Name
            ]
        );
    }

    #[test]
    fn test_adv_data_buffer_full() {
        let mut adv_data = AdvData::new();
        let long_name = "This is a very long name that exceeds the maximum advertising data length";

        assert!(adv_data.add_name(long_name).is_err());
    }
}
