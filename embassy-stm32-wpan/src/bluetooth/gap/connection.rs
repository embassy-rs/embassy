//! BLE Connection Management
//!
//! This module provides types and utilities for managing BLE connections.
//! It supports both peripheral and central roles.

use stm32wb_hci::event::{ConnectionRole, Phy};
use stm32wb_hci::host::OwnAddressType;
use stm32wb_hci::types::FixedConnectionInterval;
use stm32wb_hci::{BdAddr, BdAddrType, ConnectionHandle};

/// Maximum number of simultaneous BLE connections supported
pub const MAX_CONNECTIONS: usize = 4;

/// Physical layer type for BLE connections
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LePhy {
    /// 1 Mbps PHY
    #[default]
    Phy1M = 0x01,
    /// 2 Mbps PHY
    Phy2M = 0x02,
    /// Coded PHY (long range)
    PhyCoded = 0x03,
}

impl LePhy {
    /// Create from raw u8 value
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x01 => LePhy::Phy1M,
            0x02 => LePhy::Phy2M,
            0x03 => LePhy::PhyCoded,
            _ => LePhy::Phy1M,
        }
    }
}

/// Reason for disconnection
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DisconnectReason {
    /// Remote user terminated connection
    RemoteUserTerminated = 0x13,
    /// Remote device terminated due to low resources
    RemoteLowResources = 0x14,
    /// Remote device terminated due to power off
    RemotePowerOff = 0x15,
    /// Connection terminated by local host
    LocalHostTerminated = 0x16,
    /// Unacceptable connection parameters
    UnacceptableParameters = 0x3B,
}

impl DisconnectReason {
    /// Convert to raw u8 value for HCI command
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Connection parameters
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConnectionParams {
    /// Connection interval in units of 1.25ms (Range: 6 to 3200, i.e., 7.5ms to 4s)
    pub interval: u16,
    /// Slave latency (number of connection events the slave can skip)
    pub latency: u16,
    /// Supervision timeout in units of 10ms (Range: 10 to 3200, i.e., 100ms to 32s)
    pub supervision_timeout: u16,
}

impl Default for ConnectionParams {
    fn default() -> Self {
        Self {
            interval: 80,             // 100ms
            latency: 0,               // No latency
            supervision_timeout: 400, // 4s
        }
    }
}

/// Parameters for connection initiation (central role)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConnectionInitParams {
    /// Scan interval in units of 0.625ms
    pub scan_interval: u16,
    /// Scan window in units of 0.625ms
    pub scan_window: u16,
    /// Use filter accept list instead of peer address
    pub use_filter_accept_list: bool,
    /// Peer address
    pub peer_address: BdAddrType,
    /// Own address type
    pub own_address_type: OwnAddressType,
    /// Minimum connection interval in units of 1.25ms
    pub conn_interval_min: u16,
    /// Maximum connection interval in units of 1.25ms
    pub conn_interval_max: u16,
    /// Maximum slave latency
    pub max_latency: u16,
    /// Supervision timeout in units of 10ms
    pub supervision_timeout: u16,
    /// Minimum CE length in units of 0.625ms
    pub min_ce_length: u16,
    /// Maximum CE length in units of 0.625ms
    pub max_ce_length: u16,
}

impl Default for ConnectionInitParams {
    fn default() -> Self {
        Self {
            scan_interval: 0x0010, // 10ms
            scan_window: 0x0010,   // 10ms
            use_filter_accept_list: false,
            peer_address: BdAddrType::Public(BdAddr([0; 6])),
            own_address_type: OwnAddressType::Public,
            conn_interval_min: 0x0018, // 30ms
            conn_interval_max: 0x0028, // 50ms
            max_latency: 0,
            supervision_timeout: 0x01F4, // 5s
            min_ce_length: 0x0000,
            max_ce_length: 0xFFFF,
        }
    }
}

/// Active BLE connection context
///
/// Contains all relevant information about an active connection.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Connection {
    /// Connection handle assigned by the controller
    pub handle: ConnectionHandle,
    /// Role in this connection
    pub role: ConnectionRole,
    /// Peer device address
    pub peer_address: BdAddrType,
    /// Local resolvable private address (if used)
    pub local_rpa: Option<BdAddr>,
    /// Peer resolvable private address (if used)
    pub peer_rpa: Option<BdAddr>,
    /// Current ATT MTU size
    pub mtu: u16,
    /// Connection parameters
    pub interval: FixedConnectionInterval,
    /// TX PHY
    pub tx_phy: Phy,
    /// RX PHY
    pub rx_phy: Phy,
    /// Whether this connection is encrypted
    pub encrypted: bool,
}

impl Connection {
    /// Create a new connection from connection complete event data
    pub fn new(
        handle: ConnectionHandle,
        role: ConnectionRole,
        peer_address: BdAddrType,
        conn_interval: FixedConnectionInterval,
    ) -> Self {
        Self {
            handle,
            role,
            peer_address,
            local_rpa: None,
            peer_rpa: None,
            mtu: 23, // Default ATT MTU
            interval: conn_interval,
            tx_phy: Phy::Le1M,
            rx_phy: Phy::Le2M,
            encrypted: false,
        }
    }

    /// Create from enhanced connection complete event (includes RPA)
    pub fn new_enhanced(
        handle: ConnectionHandle,
        role: ConnectionRole,
        peer_address: BdAddrType,
        local_rpa: BdAddr,
        peer_rpa: BdAddr,
        conn_interval: FixedConnectionInterval,
    ) -> Self {
        let local_rpa_opt = if local_rpa.0 != [0; 6] { Some(local_rpa) } else { None };
        let peer_rpa_opt = if peer_rpa.0 != [0; 6] { Some(peer_rpa) } else { None };

        Self {
            handle,
            role,
            peer_address,
            local_rpa: local_rpa_opt,
            peer_rpa: peer_rpa_opt,
            mtu: 23,
            interval: conn_interval,
            tx_phy: Phy::Le1M,
            rx_phy: Phy::Le1M,
            encrypted: false,
        }
    }

    /// Update connection parameters
    pub fn update_interval(&mut self, conn_interval: FixedConnectionInterval) {
        self.interval = conn_interval
    }

    /// Update MTU
    pub fn update_mtu(&mut self, mtu: u16) {
        self.mtu = mtu;
    }

    /// Update PHY
    pub fn update_phy(&mut self, tx_phy: Phy, rx_phy: Phy) {
        self.tx_phy = tx_phy;
        self.rx_phy = rx_phy;
    }

    /// Set encryption status
    pub fn set_encrypted(&mut self, encrypted: bool) {
        self.encrypted = encrypted;
    }
}

/// Connection manager for tracking active connections
///
/// This maintains a pool of connection contexts and provides
/// methods for looking up connections by handle or address.
pub struct ConnectionManager<const N: usize = MAX_CONNECTIONS> {
    connections: [Option<Connection>; N],
}

impl<const N: usize> ConnectionManager<N> {
    /// Create a new empty connection manager
    pub const fn new() -> Self {
        // const fn compatible array initialization
        Self {
            connections: [const { None }; N],
        }
    }

    /// Get a free slot and initialize it with the given handle
    ///
    /// Returns a mutable reference to the connection context if a slot is available.
    pub fn allocate(&mut self, connection: Connection) -> Option<&mut Connection> {
        for slot in self.connections.iter_mut() {
            if slot.is_none() {
                *slot = Some(connection);
                return slot.as_mut();
            }
        }
        None
    }

    /// Get a connection by its handle
    pub fn get_by_handle(&self, handle: ConnectionHandle) -> Option<&Connection> {
        self.connections
            .iter()
            .filter_map(|c| c.as_ref())
            .find(|c| c.handle == handle)
    }

    /// Get a mutable connection by its handle
    pub fn get_by_handle_mut(&mut self, handle: ConnectionHandle) -> Option<&mut Connection> {
        self.connections
            .iter_mut()
            .filter_map(|c| c.as_mut())
            .find(|c| c.handle == handle)
    }

    /// Get a connection by peer address
    pub fn get_by_address(&self, address: &BdAddrType) -> Option<&Connection> {
        self.connections
            .iter()
            .filter_map(|c| c.as_ref())
            .find(|c| &c.peer_address == address)
    }

    /// Remove a connection by its handle
    ///
    /// Returns the removed connection if it existed.
    pub fn remove(&mut self, handle: ConnectionHandle) -> Option<Connection> {
        for slot in self.connections.iter_mut() {
            if let Some(conn) = slot {
                if conn.handle == handle {
                    return slot.take();
                }
            }
        }
        None
    }

    /// Get the number of active connections
    pub fn count(&self) -> usize {
        self.connections.iter().filter(|c| c.is_some()).count()
    }

    /// Check if there are any active connections
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Check if the connection manager is at capacity
    pub fn is_full(&self) -> bool {
        self.count() >= N
    }

    /// Iterate over all active connections
    pub fn iter(&self) -> impl Iterator<Item = &Connection> {
        self.connections.iter().filter_map(|c| c.as_ref())
    }

    /// Iterate over all active connections mutably
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Connection> {
        self.connections.iter_mut().filter_map(|c| c.as_mut())
    }
}

impl<const N: usize> Default for ConnectionManager<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// GAP events related to connection management
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GapEvent {
    /// A new connection has been established
    Connected(Connection),

    /// A connection has been terminated
    Disconnected {
        /// Handle of the terminated connection
        handle: ConnectionHandle,
        /// Reason for disconnection
        reason: u8,
    },

    /// Connection parameters have been updated
    ConnectionParamsUpdated {
        /// Connection handle
        handle: ConnectionHandle,
        /// New connection interval
        interval: FixedConnectionInterval,
    },

    /// PHY has been updated
    PhyUpdated {
        /// Connection handle
        handle: ConnectionHandle,
        /// New TX PHY
        tx_phy: Phy,
        /// New RX PHY
        rx_phy: Phy,
    },

    /// Data length has changed
    DataLengthChanged {
        /// Connection handle
        handle: ConnectionHandle,
        /// Maximum TX octets
        max_tx_octets: u16,
        /// Maximum TX time in microseconds
        max_tx_time: u16,
        /// Maximum RX octets
        max_rx_octets: u16,
        /// Maximum RX time in microseconds
        max_rx_time: u16,
    },
}
