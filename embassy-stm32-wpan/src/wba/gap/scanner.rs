//! BLE Scanner implementation
//!
//! This module provides functionality for scanning (observer role) to discover
//! nearby BLE devices that are advertising.

use crate::wba::error::BleError;
use crate::wba::gap::types::OwnAddressType;
use crate::wba::hci::command::CommandSender;

/// Scan type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ScanType {
    /// Passive scanning - only listen for advertising packets
    #[default]
    Passive = 0x00,
    /// Active scanning - send scan requests to get scan response data
    Active = 0x01,
}

/// Scan filter policy
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ScanFilterPolicy {
    /// Accept all advertising packets
    #[default]
    AcceptAll = 0x00,
    /// Accept only advertising packets from devices in the filter accept list
    AcceptFilterListOnly = 0x01,
    /// Accept all advertising packets, but filter directed ads not addressed to us
    AcceptAllFilterDirected = 0x02,
    /// Accept only filter accept list, and filter directed ads
    AcceptFilterListFilterDirected = 0x03,
}

/// Scan parameters
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ScanParams {
    /// Type of scanning (passive or active)
    pub scan_type: ScanType,

    /// Scan interval in units of 0.625ms (Range: 0x0004 to 0x4000, i.e., 2.5ms to 10.24s)
    /// Default: 0x0010 (10ms)
    pub scan_interval: u16,

    /// Scan window in units of 0.625ms (Range: 0x0004 to 0x4000)
    /// Must be <= scan_interval
    /// Default: 0x0010 (10ms)
    pub scan_window: u16,

    /// Own address type to use
    pub own_address_type: OwnAddressType,

    /// Scanning filter policy
    pub filter_policy: ScanFilterPolicy,

    /// Filter duplicate advertising reports
    pub filter_duplicates: bool,
}

impl Default for ScanParams {
    fn default() -> Self {
        Self {
            scan_type: ScanType::Passive,
            scan_interval: 0x0010, // 10ms
            scan_window: 0x0010,   // 10ms
            own_address_type: OwnAddressType::Public,
            filter_policy: ScanFilterPolicy::AcceptAll,
            filter_duplicates: true,
        }
    }
}

impl ScanParams {
    /// Create new scan parameters with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set scan type
    pub fn with_scan_type(mut self, scan_type: ScanType) -> Self {
        self.scan_type = scan_type;
        self
    }

    /// Set scan interval (in units of 0.625ms)
    pub fn with_interval(mut self, interval: u16) -> Self {
        self.scan_interval = interval;
        self
    }

    /// Set scan window (in units of 0.625ms)
    pub fn with_window(mut self, window: u16) -> Self {
        self.scan_window = window;
        self
    }

    /// Set filter policy
    pub fn with_filter_policy(mut self, policy: ScanFilterPolicy) -> Self {
        self.filter_policy = policy;
        self
    }

    /// Set duplicate filtering
    pub fn with_filter_duplicates(mut self, filter: bool) -> Self {
        self.filter_duplicates = filter;
        self
    }

    /// Fast scan parameters (shorter intervals for quick discovery)
    pub fn fast() -> Self {
        Self {
            scan_type: ScanType::Active,
            scan_interval: 0x0030, // 30ms
            scan_window: 0x0030,   // 30ms (100% duty cycle)
            own_address_type: OwnAddressType::Public,
            filter_policy: ScanFilterPolicy::AcceptAll,
            filter_duplicates: true,
        }
    }

    /// Low power scan parameters (longer intervals, shorter windows)
    pub fn low_power() -> Self {
        Self {
            scan_type: ScanType::Passive,
            scan_interval: 0x0800, // 1.28s
            scan_window: 0x0012,   // 11.25ms (~0.9% duty cycle)
            own_address_type: OwnAddressType::Public,
            filter_policy: ScanFilterPolicy::AcceptAll,
            filter_duplicates: true,
        }
    }
}

/// BLE Scanner for discovering nearby devices
///
/// The Scanner provides methods for starting and stopping BLE scanning.
/// Advertising reports are received through the main event loop.
pub struct Scanner<'d> {
    cmd: &'d CommandSender,
    is_scanning: bool,
}

impl<'d> Scanner<'d> {
    /// Create a new Scanner
    pub(crate) fn new(cmd: &'d CommandSender) -> Self {
        Self {
            cmd,
            is_scanning: false,
        }
    }

    /// Start scanning with the given parameters
    ///
    /// Advertising reports will be received as `LeAdvertisingReport` events
    /// through the main BLE event loop.
    ///
    /// # Parameters
    ///
    /// - `params`: Scan parameters
    ///
    /// # Example
    ///
    /// ```no_run
    /// let mut scanner = ble.scanner();
    /// scanner.start(ScanParams::fast())?;
    ///
    /// // Advertising reports received in event loop
    /// loop {
    ///     let event = ble.read_event().await;
    ///     if let EventParams::LeAdvertisingReport { reports } = event.params {
    ///         for report in reports {
    ///             // Process advertising report
    ///         }
    ///     }
    /// }
    /// ```
    pub fn start(&mut self, params: ScanParams) -> Result<(), BleError> {
        // Set scan parameters
        self.cmd.le_set_scan_parameters(
            params.scan_type as u8,
            params.scan_interval,
            params.scan_window,
            params.own_address_type,
            params.filter_policy as u8,
        )?;

        // Enable scanning
        self.cmd.le_set_scan_enable(true, params.filter_duplicates)?;

        self.is_scanning = true;
        Ok(())
    }

    /// Stop scanning
    pub fn stop(&mut self) -> Result<(), BleError> {
        if self.is_scanning {
            self.cmd.le_set_scan_enable(false, false)?;
            self.is_scanning = false;
        }
        Ok(())
    }

    /// Check if currently scanning
    pub fn is_scanning(&self) -> bool {
        self.is_scanning
    }
}

/// Parsed advertising data from a scan report
#[derive(Debug, Default)]
pub struct ParsedAdvData<'a> {
    /// Device name (complete or shortened)
    pub name: Option<&'a str>,
    /// TX power level in dBm
    pub tx_power: Option<i8>,
    /// 16-bit service UUIDs
    pub service_uuids_16: heapless::Vec<u16, 8>,
    /// 128-bit service UUIDs
    pub service_uuids_128: heapless::Vec<[u8; 16], 2>,
    /// Manufacturer-specific data (company ID, data)
    pub manufacturer_data: Option<(u16, &'a [u8])>,
    /// Flags byte
    pub flags: Option<u8>,
}

impl<'a> ParsedAdvData<'a> {
    /// Parse advertising data bytes into structured format
    pub fn parse(data: &'a [u8]) -> Self {
        let mut result = Self::default();
        let mut offset = 0;

        while offset < data.len() {
            if offset + 1 >= data.len() {
                break;
            }

            let length = data[offset] as usize;
            if length == 0 || offset + length >= data.len() {
                break;
            }

            let ad_type = data[offset + 1];
            let ad_data = &data[offset + 2..offset + 1 + length];

            match ad_type {
                0x01 => {
                    // Flags
                    if !ad_data.is_empty() {
                        result.flags = Some(ad_data[0]);
                    }
                }
                0x02 | 0x03 => {
                    // Incomplete/Complete list of 16-bit UUIDs
                    for chunk in ad_data.chunks(2) {
                        if chunk.len() == 2 {
                            let uuid = u16::from_le_bytes([chunk[0], chunk[1]]);
                            let _ = result.service_uuids_16.push(uuid);
                        }
                    }
                }
                0x06 | 0x07 => {
                    // Incomplete/Complete list of 128-bit UUIDs
                    for chunk in ad_data.chunks(16) {
                        if chunk.len() == 16 {
                            let mut uuid = [0u8; 16];
                            uuid.copy_from_slice(chunk);
                            let _ = result.service_uuids_128.push(uuid);
                        }
                    }
                }
                0x08 | 0x09 => {
                    // Shortened/Complete local name
                    if let Ok(name) = core::str::from_utf8(ad_data) {
                        result.name = Some(name);
                    }
                }
                0x0A => {
                    // TX Power Level
                    if !ad_data.is_empty() {
                        result.tx_power = Some(ad_data[0] as i8);
                    }
                }
                0xFF => {
                    // Manufacturer Specific Data
                    if ad_data.len() >= 2 {
                        let company_id = u16::from_le_bytes([ad_data[0], ad_data[1]]);
                        result.manufacturer_data = Some((company_id, &ad_data[2..]));
                    }
                }
                _ => {
                    // Unknown AD type, skip
                }
            }

            offset += 1 + length;
        }

        result
    }
}
