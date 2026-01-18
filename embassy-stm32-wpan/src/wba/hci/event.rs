//! HCI Event parsing and handling

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use super::types::{Address, AddressType, Handle, Status};

/// HCI Event Codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EventCode {
    DisconnectionComplete = 0x05,
    EncryptionChange = 0x08,
    ReadRemoteVersionInformationComplete = 0x0C,
    CommandComplete = 0x0E,
    CommandStatus = 0x0F,
    HardwareError = 0x10,
    NumberOfCompletedPackets = 0x13,
    DataBufferOverflow = 0x1A,
    EncryptionKeyRefreshComplete = 0x30,
    LeMetaEvent = 0x3E,
    VendorSpecific = 0xFF,
}

impl EventCode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x05 => Some(EventCode::DisconnectionComplete),
            0x08 => Some(EventCode::EncryptionChange),
            0x0C => Some(EventCode::ReadRemoteVersionInformationComplete),
            0x0E => Some(EventCode::CommandComplete),
            0x0F => Some(EventCode::CommandStatus),
            0x10 => Some(EventCode::HardwareError),
            0x13 => Some(EventCode::NumberOfCompletedPackets),
            0x1A => Some(EventCode::DataBufferOverflow),
            0x30 => Some(EventCode::EncryptionKeyRefreshComplete),
            0x3E => Some(EventCode::LeMetaEvent),
            0xFF => Some(EventCode::VendorSpecific),
            _ => None,
        }
    }
}

/// LE Subevent Codes (for LE Meta Event)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeSubevent {
    ConnectionComplete = 0x01,
    AdvertisingReport = 0x02,
    ConnectionUpdateComplete = 0x03,
    ReadRemoteFeaturesComplete = 0x04,
    LongTermKeyRequest = 0x05,
    RemoteConnectionParameterRequest = 0x06,
    DataLengthChange = 0x07,
    ReadLocalP256PublicKeyComplete = 0x08,
    GenerateDhKeyComplete = 0x09,
    EnhancedConnectionComplete = 0x0A,
    DirectedAdvertisingReport = 0x0B,
    PhyUpdateComplete = 0x0C,
}

impl LeSubevent {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(LeSubevent::ConnectionComplete),
            0x02 => Some(LeSubevent::AdvertisingReport),
            0x03 => Some(LeSubevent::ConnectionUpdateComplete),
            0x04 => Some(LeSubevent::ReadRemoteFeaturesComplete),
            0x05 => Some(LeSubevent::LongTermKeyRequest),
            0x06 => Some(LeSubevent::RemoteConnectionParameterRequest),
            0x07 => Some(LeSubevent::DataLengthChange),
            0x08 => Some(LeSubevent::ReadLocalP256PublicKeyComplete),
            0x09 => Some(LeSubevent::GenerateDhKeyComplete),
            0x0A => Some(LeSubevent::EnhancedConnectionComplete),
            0x0B => Some(LeSubevent::DirectedAdvertisingReport),
            0x0C => Some(LeSubevent::PhyUpdateComplete),
            _ => None,
        }
    }
}

/// Parsed HCI Event
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Event {
    pub code: EventCode,
    pub params: EventParams,
}

/// Event Parameters
#[derive(Debug, Clone)]
pub enum EventParams {
    /// Command Complete event
    CommandComplete {
        num_hci_command_packets: u8,
        opcode: u16,
        status: Status,
        return_params: heapless::Vec<u8, 255>,
    },

    /// Command Status event
    CommandStatus {
        status: Status,
        num_hci_command_packets: u8,
        opcode: u16,
    },

    /// Disconnection Complete event
    DisconnectionComplete { status: Status, handle: Handle, reason: u8 },

    /// Encryption Change event
    EncryptionChange {
        status: Status,
        handle: Handle,
        enabled: bool,
    },

    /// LE Connection Complete event
    LeConnectionComplete {
        status: Status,
        handle: Handle,
        role: u8,
        peer_address_type: AddressType,
        peer_address: Address,
        conn_interval: u16,
        conn_latency: u16,
        supervision_timeout: u16,
        master_clock_accuracy: u8,
    },

    /// LE Enhanced Connection Complete event (includes RPA)
    LeEnhancedConnectionComplete {
        status: Status,
        handle: Handle,
        role: u8,
        peer_address_type: AddressType,
        peer_address: Address,
        local_resolvable_private_address: Address,
        peer_resolvable_private_address: Address,
        conn_interval: u16,
        conn_latency: u16,
        supervision_timeout: u16,
        master_clock_accuracy: u8,
    },

    /// LE Advertising Report event
    LeAdvertisingReport {
        reports: heapless::Vec<AdvertisingReport, 10>,
    },

    /// LE Connection Update Complete event
    LeConnectionUpdateComplete {
        status: Status,
        handle: Handle,
        conn_interval: u16,
        conn_latency: u16,
        supervision_timeout: u16,
    },

    /// LE PHY Update Complete event
    LePhyUpdateComplete {
        status: Status,
        handle: Handle,
        tx_phy: u8,
        rx_phy: u8,
    },

    /// LE Data Length Change event
    LeDataLengthChange {
        handle: Handle,
        max_tx_octets: u16,
        max_tx_time: u16,
        max_rx_octets: u16,
        max_rx_time: u16,
    },

    /// Hardware Error event
    HardwareError { hardware_code: u8 },

    /// Number of Completed Packets event
    NumberOfCompletedPackets { handles: heapless::Vec<(Handle, u16), 8> },

    /// Vendor Specific event (unparsed)
    VendorSpecific { data: heapless::Vec<u8, 255> },

    // ===== ACI GATT Events (parsed from Vendor Specific) =====
    /// GATT Attribute Modified event
    GattAttributeModified {
        conn_handle: Handle,
        attr_handle: u16,
        offset: u16,
        data: heapless::Vec<u8, 247>,
    },

    /// GATT Notification Complete event
    GattNotificationComplete { conn_handle: Handle, attr_handle: u16 },

    /// GATT Indication Complete event
    GattIndicationComplete { conn_handle: Handle, attr_handle: u16 },

    /// ATT Exchange MTU Response event
    AttExchangeMtuResponse { conn_handle: Handle, server_mtu: u16 },

    /// GATT Procedure Complete event
    GattProcedureComplete { conn_handle: Handle, error_code: u8 },

    /// GATT Procedure Timeout event
    GattProcedureTimeout { conn_handle: Handle },

    /// GATT TX Pool Available event
    GattTxPoolAvailable {
        conn_handle: Handle,
        available_buffers: u16,
    },

    // ===== ACI GAP Security Events =====
    /// GAP Pairing Complete event
    GapPairingComplete {
        conn_handle: Handle,
        status: u8,
        reason: u8,
    },

    /// GAP Passkey Request event
    GapPasskeyRequest { conn_handle: Handle },

    /// GAP Numeric Comparison Value event
    GapNumericComparisonRequest { conn_handle: Handle, numeric_value: u32 },

    /// GAP Bond Lost event
    GapBondLost { conn_handle: Handle },

    /// GAP Pairing Request event
    GapPairingRequest { conn_handle: Handle, is_bonded: bool },

    /// Unknown/Unparsed event
    Unknown { data: heapless::Vec<u8, 255> },
}

/// Advertising Report (part of LE Advertising Report event)
#[derive(Debug, Clone)]
pub struct AdvertisingReport {
    pub event_type: u8,
    pub address_type: AddressType,
    pub address: Address,
    pub data: heapless::Vec<u8, 31>,
    pub rssi: i8,
}

#[cfg(feature = "defmt")]
impl defmt::Format for AdvertisingReport {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "AdvertisingReport {{ event_type: {}, address_type: {}, address: {}, data: [..{}], rssi: {} }}",
            self.event_type,
            self.address_type,
            self.address,
            self.data.len(),
            self.rssi
        )
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for EventParams {
    fn format(&self, f: defmt::Formatter) {
        match self {
            EventParams::CommandComplete {
                num_hci_command_packets,
                opcode,
                status,
                ..
            } => {
                defmt::write!(
                    f,
                    "CommandComplete {{ opcode: 0x{:04X}, status: {}, num_packets: {} }}",
                    opcode,
                    status,
                    num_hci_command_packets
                )
            }
            EventParams::CommandStatus { status, opcode, .. } => {
                defmt::write!(f, "CommandStatus {{ opcode: 0x{:04X}, status: {} }}", opcode, status)
            }
            EventParams::LeConnectionComplete { status, handle, .. } => {
                defmt::write!(f, "LeConnectionComplete {{ handle: {}, status: {} }}", handle, status)
            }
            EventParams::LeEnhancedConnectionComplete {
                status, handle, role, ..
            } => {
                defmt::write!(
                    f,
                    "LeEnhancedConnectionComplete {{ handle: {}, status: {}, role: {} }}",
                    handle,
                    status,
                    role
                )
            }
            EventParams::DisconnectionComplete { status, handle, reason } => {
                defmt::write!(
                    f,
                    "DisconnectionComplete {{ handle: {}, status: {}, reason: {} }}",
                    handle,
                    status,
                    reason
                )
            }
            EventParams::EncryptionChange {
                status,
                handle,
                enabled,
            } => {
                defmt::write!(
                    f,
                    "EncryptionChange {{ handle: {}, status: {}, enabled: {} }}",
                    handle,
                    status,
                    enabled
                )
            }
            EventParams::LeAdvertisingReport { reports } => {
                defmt::write!(f, "LeAdvertisingReport {{ num_reports: {} }}", reports.len())
            }
            EventParams::LeConnectionUpdateComplete { status, handle, .. } => {
                defmt::write!(
                    f,
                    "LeConnectionUpdateComplete {{ handle: {}, status: {} }}",
                    handle,
                    status
                )
            }
            EventParams::LePhyUpdateComplete {
                status,
                handle,
                tx_phy,
                rx_phy,
            } => {
                defmt::write!(
                    f,
                    "LePhyUpdateComplete {{ handle: {}, status: {}, tx: {}, rx: {} }}",
                    handle,
                    status,
                    tx_phy,
                    rx_phy
                )
            }
            EventParams::LeDataLengthChange {
                handle,
                max_tx_octets,
                max_rx_octets,
                ..
            } => {
                defmt::write!(
                    f,
                    "LeDataLengthChange {{ handle: {}, tx: {}, rx: {} }}",
                    handle,
                    max_tx_octets,
                    max_rx_octets
                )
            }
            EventParams::HardwareError { hardware_code } => {
                defmt::write!(f, "HardwareError {{ code: {} }}", hardware_code)
            }
            EventParams::NumberOfCompletedPackets { handles } => {
                defmt::write!(f, "NumberOfCompletedPackets {{ count: {} }}", handles.len())
            }
            EventParams::VendorSpecific { data } => {
                defmt::write!(f, "VendorSpecific {{ len: {} }}", data.len())
            }
            EventParams::GattAttributeModified {
                conn_handle,
                attr_handle,
                offset,
                data,
            } => {
                defmt::write!(
                    f,
                    "GattAttributeModified {{ conn: {}, attr: 0x{:04X}, offset: {}, len: {} }}",
                    conn_handle,
                    attr_handle,
                    offset,
                    data.len()
                )
            }
            EventParams::GattNotificationComplete {
                conn_handle,
                attr_handle,
            } => {
                defmt::write!(
                    f,
                    "GattNotificationComplete {{ conn: {}, attr: 0x{:04X} }}",
                    conn_handle,
                    attr_handle
                )
            }
            EventParams::GattIndicationComplete {
                conn_handle,
                attr_handle,
            } => {
                defmt::write!(
                    f,
                    "GattIndicationComplete {{ conn: {}, attr: 0x{:04X} }}",
                    conn_handle,
                    attr_handle
                )
            }
            EventParams::AttExchangeMtuResponse {
                conn_handle,
                server_mtu,
            } => {
                defmt::write!(
                    f,
                    "AttExchangeMtuResponse {{ conn: {}, mtu: {} }}",
                    conn_handle,
                    server_mtu
                )
            }
            EventParams::GattProcedureComplete {
                conn_handle,
                error_code,
            } => {
                defmt::write!(
                    f,
                    "GattProcedureComplete {{ conn: {}, error: 0x{:02X} }}",
                    conn_handle,
                    error_code
                )
            }
            EventParams::GattProcedureTimeout { conn_handle } => {
                defmt::write!(f, "GattProcedureTimeout {{ conn: {} }}", conn_handle)
            }
            EventParams::GattTxPoolAvailable {
                conn_handle,
                available_buffers,
            } => {
                defmt::write!(
                    f,
                    "GattTxPoolAvailable {{ conn: {}, buffers: {} }}",
                    conn_handle,
                    available_buffers
                )
            }
            EventParams::GapPairingComplete {
                conn_handle,
                status,
                reason,
            } => {
                defmt::write!(
                    f,
                    "GapPairingComplete {{ conn: {}, status: {}, reason: 0x{:02X} }}",
                    conn_handle,
                    status,
                    reason
                )
            }
            EventParams::GapPasskeyRequest { conn_handle } => {
                defmt::write!(f, "GapPasskeyRequest {{ conn: {} }}", conn_handle)
            }
            EventParams::GapNumericComparisonRequest {
                conn_handle,
                numeric_value,
            } => {
                defmt::write!(
                    f,
                    "GapNumericComparisonRequest {{ conn: {}, value: {} }}",
                    conn_handle,
                    numeric_value
                )
            }
            EventParams::GapBondLost { conn_handle } => {
                defmt::write!(f, "GapBondLost {{ conn: {} }}", conn_handle)
            }
            EventParams::GapPairingRequest { conn_handle, is_bonded } => {
                defmt::write!(
                    f,
                    "GapPairingRequest {{ conn: {}, bonded: {} }}",
                    conn_handle,
                    is_bonded
                )
            }
            EventParams::Unknown { data } => {
                defmt::write!(f, "Unknown {{ len: {} }}", data.len())
            }
        }
    }
}

impl Event {
    /// Parse an HCI event from raw bytes
    ///
    /// Event packet format:
    /// - Byte 0: Event code
    /// - Byte 1: Parameter length
    /// - Bytes 2+: Parameters
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 2 {
            return None;
        }

        let code = EventCode::from_u8(data[0])?;
        let param_len = data[1] as usize;

        if data.len() < 2 + param_len {
            return None;
        }

        let params = &data[2..2 + param_len];

        let event_params = match code {
            EventCode::CommandComplete => Self::parse_command_complete(params)?,
            EventCode::CommandStatus => Self::parse_command_status(params)?,
            EventCode::DisconnectionComplete => Self::parse_disconnection_complete(params)?,
            EventCode::EncryptionChange => Self::parse_encryption_change(params)?,
            EventCode::LeMetaEvent => Self::parse_le_meta_event(params)?,
            EventCode::HardwareError => Self::parse_hardware_error(params)?,
            EventCode::NumberOfCompletedPackets => Self::parse_number_of_completed_packets(params)?,
            EventCode::VendorSpecific => Self::parse_vendor_specific(params)?,
            _ => EventParams::Unknown {
                data: heapless::Vec::from_slice(params).ok()?,
            },
        };

        Some(Event {
            code,
            params: event_params,
        })
    }

    fn parse_command_complete(params: &[u8]) -> Option<EventParams> {
        if params.len() < 3 {
            return None;
        }

        let num_hci_command_packets = params[0];
        let opcode = u16::from_le_bytes([params[1], params[2]]);
        let status = if params.len() > 3 {
            Status::from_u8(params[3])
        } else {
            Status::Success
        };

        let return_params = if params.len() > 4 {
            heapless::Vec::from_slice(&params[4..]).ok()?
        } else {
            heapless::Vec::new()
        };

        Some(EventParams::CommandComplete {
            num_hci_command_packets,
            opcode,
            status,
            return_params,
        })
    }

    fn parse_command_status(params: &[u8]) -> Option<EventParams> {
        if params.len() < 4 {
            return None;
        }

        let status = Status::from_u8(params[0]);
        let num_hci_command_packets = params[1];
        let opcode = u16::from_le_bytes([params[2], params[3]]);

        Some(EventParams::CommandStatus {
            status,
            num_hci_command_packets,
            opcode,
        })
    }

    fn parse_disconnection_complete(params: &[u8]) -> Option<EventParams> {
        if params.len() < 4 {
            return None;
        }

        let status = Status::from_u8(params[0]);
        let handle = Handle::new(u16::from_le_bytes([params[1], params[2]]));
        let reason = params[3];

        Some(EventParams::DisconnectionComplete { status, handle, reason })
    }

    fn parse_encryption_change(params: &[u8]) -> Option<EventParams> {
        // Format: status(1) + handle(2) + enabled(1)
        if params.len() < 4 {
            return None;
        }

        let status = Status::from_u8(params[0]);
        let handle = Handle::new(u16::from_le_bytes([params[1], params[2]]));
        let enabled = params[3] != 0;

        Some(EventParams::EncryptionChange {
            status,
            handle,
            enabled,
        })
    }

    fn parse_le_meta_event(params: &[u8]) -> Option<EventParams> {
        if params.is_empty() {
            return None;
        }

        let subevent = LeSubevent::from_u8(params[0])?;
        let subevent_params = &params[1..];

        match subevent {
            LeSubevent::ConnectionComplete => Self::parse_le_connection_complete(subevent_params),
            LeSubevent::EnhancedConnectionComplete => Self::parse_le_enhanced_connection_complete(subevent_params),
            LeSubevent::AdvertisingReport => Self::parse_le_advertising_report(subevent_params),
            LeSubevent::ConnectionUpdateComplete => Self::parse_le_connection_update_complete(subevent_params),
            LeSubevent::PhyUpdateComplete => Self::parse_le_phy_update_complete(subevent_params),
            LeSubevent::DataLengthChange => Self::parse_le_data_length_change(subevent_params),
            _ => Some(EventParams::Unknown {
                data: heapless::Vec::from_slice(params).ok()?,
            }),
        }
    }

    fn parse_le_connection_complete(params: &[u8]) -> Option<EventParams> {
        if params.len() < 18 {
            return None;
        }

        let status = Status::from_u8(params[0]);
        let handle = Handle::new(u16::from_le_bytes([params[1], params[2]]));
        let role = params[3];
        let peer_address_type = match params[4] {
            0 => AddressType::Public,
            1 => AddressType::Random,
            2 => AddressType::PublicIdentity,
            3 => AddressType::RandomIdentity,
            _ => return None,
        };

        let mut peer_address = [0u8; 6];
        peer_address.copy_from_slice(&params[5..11]);

        let conn_interval = u16::from_le_bytes([params[11], params[12]]);
        let conn_latency = u16::from_le_bytes([params[13], params[14]]);
        let supervision_timeout = u16::from_le_bytes([params[15], params[16]]);
        let master_clock_accuracy = params[17];

        Some(EventParams::LeConnectionComplete {
            status,
            handle,
            role,
            peer_address_type,
            peer_address: Address::new(peer_address),
            conn_interval,
            conn_latency,
            supervision_timeout,
            master_clock_accuracy,
        })
    }

    fn parse_le_advertising_report(params: &[u8]) -> Option<EventParams> {
        if params.is_empty() {
            return None;
        }

        let num_reports = params[0] as usize;
        let mut reports = heapless::Vec::new();
        let mut offset = 1;

        for _ in 0..num_reports {
            if offset + 9 > params.len() {
                break;
            }

            let event_type = params[offset];
            let address_type = match params[offset + 1] {
                0 => AddressType::Public,
                1 => AddressType::Random,
                2 => AddressType::PublicIdentity,
                3 => AddressType::RandomIdentity,
                _ => return None,
            };

            let mut address = [0u8; 6];
            address.copy_from_slice(&params[offset + 2..offset + 8]);

            let data_len = params[offset + 8] as usize;
            offset += 9;

            if offset + data_len + 1 > params.len() {
                break;
            }

            let data = heapless::Vec::from_slice(&params[offset..offset + data_len]).ok()?;
            offset += data_len;

            let rssi = params[offset] as i8;
            offset += 1;

            let _ = reports.push(AdvertisingReport {
                event_type,
                address_type,
                address: Address::new(address),
                data,
                rssi,
            });
        }

        Some(EventParams::LeAdvertisingReport { reports })
    }

    fn parse_le_connection_update_complete(params: &[u8]) -> Option<EventParams> {
        if params.len() < 9 {
            return None;
        }

        let status = Status::from_u8(params[0]);
        let handle = Handle::new(u16::from_le_bytes([params[1], params[2]]));
        let conn_interval = u16::from_le_bytes([params[3], params[4]]);
        let conn_latency = u16::from_le_bytes([params[5], params[6]]);
        let supervision_timeout = u16::from_le_bytes([params[7], params[8]]);

        Some(EventParams::LeConnectionUpdateComplete {
            status,
            handle,
            conn_interval,
            conn_latency,
            supervision_timeout,
        })
    }

    fn parse_le_enhanced_connection_complete(params: &[u8]) -> Option<EventParams> {
        // Enhanced Connection Complete has 30 bytes of parameters
        if params.len() < 30 {
            return None;
        }

        let status = Status::from_u8(params[0]);
        let handle = Handle::new(u16::from_le_bytes([params[1], params[2]]));
        let role = params[3];
        let peer_address_type = match params[4] {
            0 => AddressType::Public,
            1 => AddressType::Random,
            2 => AddressType::PublicIdentity,
            3 => AddressType::RandomIdentity,
            _ => return None,
        };

        let mut peer_address = [0u8; 6];
        peer_address.copy_from_slice(&params[5..11]);

        let mut local_rpa = [0u8; 6];
        local_rpa.copy_from_slice(&params[11..17]);

        let mut peer_rpa = [0u8; 6];
        peer_rpa.copy_from_slice(&params[17..23]);

        let conn_interval = u16::from_le_bytes([params[23], params[24]]);
        let conn_latency = u16::from_le_bytes([params[25], params[26]]);
        let supervision_timeout = u16::from_le_bytes([params[27], params[28]]);
        let master_clock_accuracy = params[29];

        Some(EventParams::LeEnhancedConnectionComplete {
            status,
            handle,
            role,
            peer_address_type,
            peer_address: Address::new(peer_address),
            local_resolvable_private_address: Address::new(local_rpa),
            peer_resolvable_private_address: Address::new(peer_rpa),
            conn_interval,
            conn_latency,
            supervision_timeout,
            master_clock_accuracy,
        })
    }

    fn parse_le_phy_update_complete(params: &[u8]) -> Option<EventParams> {
        if params.len() < 5 {
            return None;
        }

        let status = Status::from_u8(params[0]);
        let handle = Handle::new(u16::from_le_bytes([params[1], params[2]]));
        let tx_phy = params[3];
        let rx_phy = params[4];

        Some(EventParams::LePhyUpdateComplete {
            status,
            handle,
            tx_phy,
            rx_phy,
        })
    }

    fn parse_le_data_length_change(params: &[u8]) -> Option<EventParams> {
        if params.len() < 10 {
            return None;
        }

        let handle = Handle::new(u16::from_le_bytes([params[0], params[1]]));
        let max_tx_octets = u16::from_le_bytes([params[2], params[3]]);
        let max_tx_time = u16::from_le_bytes([params[4], params[5]]);
        let max_rx_octets = u16::from_le_bytes([params[6], params[7]]);
        let max_rx_time = u16::from_le_bytes([params[8], params[9]]);

        Some(EventParams::LeDataLengthChange {
            handle,
            max_tx_octets,
            max_tx_time,
            max_rx_octets,
            max_rx_time,
        })
    }

    fn parse_vendor_specific(params: &[u8]) -> Option<EventParams> {
        // Vendor specific events have format:
        // - Bytes 0-1: Event code (little-endian)
        // - Bytes 2+: Event-specific data
        if params.len() < 2 {
            return Some(EventParams::VendorSpecific {
                data: heapless::Vec::from_slice(params).ok()?,
            });
        }

        let ecode = u16::from_le_bytes([params[0], params[1]]);
        let event_data = &params[2..];

        // ACI GATT event codes
        const ACI_GATT_ATTRIBUTE_MODIFIED: u16 = 0x0C01;
        const ACI_GATT_PROC_COMPLETE: u16 = 0x0C02;
        const ACI_GATT_NOTIFICATION_COMPLETE: u16 = 0x0C03;
        const ACI_GATT_INDICATION_COMPLETE: u16 = 0x0C04;
        const ACI_ATT_EXCHANGE_MTU_RESP: u16 = 0x0802;
        const ACI_GATT_PROC_TIMEOUT: u16 = 0x0C05;
        const ACI_GATT_TX_POOL_AVAILABLE: u16 = 0x0C08;

        // ACI GAP security event codes
        const ACI_GAP_PAIRING_COMPLETE: u16 = 0x0401;
        const ACI_GAP_PASS_KEY_REQ: u16 = 0x0402;
        const ACI_GAP_BOND_LOST: u16 = 0x0405;
        const ACI_GAP_NUMERIC_COMPARISON_VALUE: u16 = 0x0409;
        const ACI_GAP_PAIRING_REQUEST: u16 = 0x040B;

        match ecode {
            ACI_GATT_ATTRIBUTE_MODIFIED => Self::parse_gatt_attribute_modified(event_data),
            ACI_GATT_PROC_COMPLETE => Self::parse_gatt_proc_complete(event_data),
            ACI_GATT_NOTIFICATION_COMPLETE => Self::parse_gatt_notification_complete(event_data),
            ACI_GATT_INDICATION_COMPLETE => Self::parse_gatt_indication_complete(event_data),
            ACI_ATT_EXCHANGE_MTU_RESP => Self::parse_att_exchange_mtu_resp(event_data),
            ACI_GATT_PROC_TIMEOUT => Self::parse_gatt_proc_timeout(event_data),
            ACI_GATT_TX_POOL_AVAILABLE => Self::parse_gatt_tx_pool_available(event_data),
            ACI_GAP_PAIRING_COMPLETE => Self::parse_gap_pairing_complete(event_data),
            ACI_GAP_PASS_KEY_REQ => Self::parse_gap_passkey_request(event_data),
            ACI_GAP_BOND_LOST => Self::parse_gap_bond_lost(event_data),
            ACI_GAP_NUMERIC_COMPARISON_VALUE => Self::parse_gap_numeric_comparison(event_data),
            ACI_GAP_PAIRING_REQUEST => Self::parse_gap_pairing_request(event_data),
            _ => Some(EventParams::VendorSpecific {
                data: heapless::Vec::from_slice(params).ok()?,
            }),
        }
    }

    fn parse_gatt_attribute_modified(params: &[u8]) -> Option<EventParams> {
        // Format: conn_handle(2) + attr_handle(2) + offset(2) + data_length(2) + data(N)
        if params.len() < 8 {
            return None;
        }

        let conn_handle = Handle::new(u16::from_le_bytes([params[0], params[1]]));
        let attr_handle = u16::from_le_bytes([params[2], params[3]]);
        let offset = u16::from_le_bytes([params[4], params[5]]);
        let data_length = u16::from_le_bytes([params[6], params[7]]) as usize;

        // Note: Bit 15 of offset may indicate "more data pending" - mask it off
        let actual_offset = offset & 0x7FFF;

        let data = if params.len() >= 8 + data_length {
            heapless::Vec::from_slice(&params[8..8 + data_length]).ok()?
        } else {
            heapless::Vec::from_slice(&params[8..]).ok()?
        };

        Some(EventParams::GattAttributeModified {
            conn_handle,
            attr_handle,
            offset: actual_offset,
            data,
        })
    }

    fn parse_gatt_proc_complete(params: &[u8]) -> Option<EventParams> {
        // Format: conn_handle(2) + error_code(1)
        if params.len() < 3 {
            return None;
        }

        let conn_handle = Handle::new(u16::from_le_bytes([params[0], params[1]]));
        let error_code = params[2];

        Some(EventParams::GattProcedureComplete {
            conn_handle,
            error_code,
        })
    }

    fn parse_gatt_notification_complete(params: &[u8]) -> Option<EventParams> {
        // Format: conn_handle(2) + attr_handle(2)
        if params.len() < 4 {
            return None;
        }

        let conn_handle = Handle::new(u16::from_le_bytes([params[0], params[1]]));
        let attr_handle = u16::from_le_bytes([params[2], params[3]]);

        Some(EventParams::GattNotificationComplete {
            conn_handle,
            attr_handle,
        })
    }

    fn parse_gatt_indication_complete(params: &[u8]) -> Option<EventParams> {
        // Format: conn_handle(2) + attr_handle(2)
        if params.len() < 4 {
            return None;
        }

        let conn_handle = Handle::new(u16::from_le_bytes([params[0], params[1]]));
        let attr_handle = u16::from_le_bytes([params[2], params[3]]);

        Some(EventParams::GattIndicationComplete {
            conn_handle,
            attr_handle,
        })
    }

    fn parse_att_exchange_mtu_resp(params: &[u8]) -> Option<EventParams> {
        // Format: conn_handle(2) + server_mtu(2)
        if params.len() < 4 {
            return None;
        }

        let conn_handle = Handle::new(u16::from_le_bytes([params[0], params[1]]));
        let server_mtu = u16::from_le_bytes([params[2], params[3]]);

        Some(EventParams::AttExchangeMtuResponse {
            conn_handle,
            server_mtu,
        })
    }

    fn parse_gatt_proc_timeout(params: &[u8]) -> Option<EventParams> {
        // Format: conn_handle(2)
        if params.len() < 2 {
            return None;
        }

        let conn_handle = Handle::new(u16::from_le_bytes([params[0], params[1]]));

        Some(EventParams::GattProcedureTimeout { conn_handle })
    }

    fn parse_gatt_tx_pool_available(params: &[u8]) -> Option<EventParams> {
        // Format: conn_handle(2) + available_buffers(2)
        if params.len() < 4 {
            return None;
        }

        let conn_handle = Handle::new(u16::from_le_bytes([params[0], params[1]]));
        let available_buffers = u16::from_le_bytes([params[2], params[3]]);

        Some(EventParams::GattTxPoolAvailable {
            conn_handle,
            available_buffers,
        })
    }

    fn parse_hardware_error(params: &[u8]) -> Option<EventParams> {
        if params.is_empty() {
            return None;
        }

        Some(EventParams::HardwareError {
            hardware_code: params[0],
        })
    }

    fn parse_number_of_completed_packets(params: &[u8]) -> Option<EventParams> {
        if params.is_empty() {
            return None;
        }

        let num_handles = params[0] as usize;
        let mut handles = heapless::Vec::new();

        for i in 0..num_handles {
            let offset = 1 + i * 4;
            if offset + 4 > params.len() {
                break;
            }

            let handle = Handle::new(u16::from_le_bytes([params[offset], params[offset + 1]]));
            let num_completed = u16::from_le_bytes([params[offset + 2], params[offset + 3]]);

            let _ = handles.push((handle, num_completed));
        }

        Some(EventParams::NumberOfCompletedPackets { handles })
    }

    // ===== Security Event Parsing =====

    fn parse_gap_pairing_complete(params: &[u8]) -> Option<EventParams> {
        // Format: conn_handle(2) + status(1) + reason(1)
        if params.len() < 4 {
            return None;
        }

        let conn_handle = Handle::new(u16::from_le_bytes([params[0], params[1]]));
        let status = params[2];
        let reason = params[3];

        Some(EventParams::GapPairingComplete {
            conn_handle,
            status,
            reason,
        })
    }

    fn parse_gap_passkey_request(params: &[u8]) -> Option<EventParams> {
        // Format: conn_handle(2)
        if params.len() < 2 {
            return None;
        }

        let conn_handle = Handle::new(u16::from_le_bytes([params[0], params[1]]));

        Some(EventParams::GapPasskeyRequest { conn_handle })
    }

    fn parse_gap_bond_lost(params: &[u8]) -> Option<EventParams> {
        // Format: conn_handle(2)
        if params.len() < 2 {
            return None;
        }

        let conn_handle = Handle::new(u16::from_le_bytes([params[0], params[1]]));

        Some(EventParams::GapBondLost { conn_handle })
    }

    fn parse_gap_numeric_comparison(params: &[u8]) -> Option<EventParams> {
        // Format: conn_handle(2) + numeric_value(4)
        if params.len() < 6 {
            return None;
        }

        let conn_handle = Handle::new(u16::from_le_bytes([params[0], params[1]]));
        let numeric_value = u32::from_le_bytes([params[2], params[3], params[4], params[5]]);

        Some(EventParams::GapNumericComparisonRequest {
            conn_handle,
            numeric_value,
        })
    }

    fn parse_gap_pairing_request(params: &[u8]) -> Option<EventParams> {
        // Format: conn_handle(2) + bonded(1) + ...
        if params.len() < 3 {
            return None;
        }

        let conn_handle = Handle::new(u16::from_le_bytes([params[0], params[1]]));
        let is_bonded = params[2] != 0;

        Some(EventParams::GapPairingRequest { conn_handle, is_bonded })
    }
}

/// Global event channel for passing events from C callback to Rust async code
/// Size of 8 events should be sufficient for most cases
pub(crate) static EVENT_CHANNEL: Channel<CriticalSectionRawMutex, Event, 8> = Channel::new();

/// Receive the next HCI event (async)
pub async fn read_event() -> Event {
    EVENT_CHANNEL.receive().await
}

/// Try to send an event to the channel (non-blocking, for use in C callbacks)
pub(crate) fn try_send_event(event: Event) -> Result<(), ()> {
    EVENT_CHANNEL.try_send(event).map_err(|_| ())
}
