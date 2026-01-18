//! High-level BLE API for STM32WBA
//!
//! This module provides the main `Ble` struct that manages the BLE stack lifecycle
//! and provides access to GAP functionality including connection management.

use core::sync::atomic::{AtomicBool, Ordering};

use crate::wba::error::BleError;
use crate::wba::gap::Advertiser;
use crate::wba::gap::connection::{
    Connection, ConnectionInitParams, ConnectionManager, ConnectionRole, DisconnectReason, GapEvent, LePhy,
    MAX_CONNECTIONS,
};
use crate::wba::gap::scanner::Scanner;
use crate::wba::hci::command::CommandSender;
use crate::wba::hci::event::{Event, EventParams, read_event};
use crate::wba::hci::types::{Address, Handle, Status};
use crate::wba::ll_sys::init_ble_stack;

/// Main BLE interface
///
/// This struct provides the primary interface to the BLE stack.
///
/// # Example
///
/// ```no_run
/// use embassy_stm32_wpan::wba::{Ble, gap::{AdvData, AdvParams}};
///
/// let mut ble = Ble::new();
///
/// // Initialize BLE stack
/// ble.init().await.unwrap();
///
/// // Create advertising data
/// let mut adv_data = AdvData::new();
/// adv_data.add_flags(0x06).unwrap();
/// adv_data.add_name("MyDevice").unwrap();
///
/// // Start advertising
/// let mut advertiser = ble.advertiser();
/// advertiser.start(AdvParams::default(), adv_data, None).unwrap();
///
/// // Event loop
/// loop {
///     let event = ble.read_event().await;
///     // Handle BLE events
/// }
/// ```
pub struct Ble {
    cmd_sender: CommandSender,
    initialized: AtomicBool,
    connections: ConnectionManager<MAX_CONNECTIONS>,
}

impl Ble {
    /// Create a new BLE instance
    ///
    /// Note: You must call `init()` before using other BLE functionality.
    pub fn new() -> Self {
        Self {
            cmd_sender: CommandSender::new(),
            initialized: AtomicBool::new(false),
            connections: ConnectionManager::new(),
        }
    }

    /// Initialize the BLE stack
    ///
    /// This function performs the following initialization steps:
    /// 1. Resets the BLE controller
    /// 2. Reads and logs the local version information
    /// 3. Reads the BD address
    /// 4. Sets the event mask
    /// 5. Reads buffer sizes
    /// 6. Reads supported features
    ///
    /// Must be called before any other BLE operations.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if initialization succeeded
    /// - `Err(BleError)` if any initialization step failed
    pub fn init(&mut self) -> Result<(), BleError> {
        if self.initialized.load(Ordering::Acquire) {
            return Ok(());
        }

        // 0. Initialize the BLE stack using BleStack_Init
        // This properly initializes the BLE host stack including memory management,
        // which is required before ll_intf_init can work properly.
        init_ble_stack().map_err(|status| {
            #[cfg(feature = "defmt")]
            defmt::error!("BLE stack initialization failed: 0x{:02X}", status);
            BleError::InitializationFailed
        })?;

        #[cfg(feature = "defmt")]
        defmt::info!("Ble::init: BLE stack initialized, sending HCI reset");

        // 1. Reset BLE controller
        self.cmd_sender.reset()?;

        // 2. Read local version information
        let version = self.cmd_sender.read_local_version()?;
        #[cfg(feature = "defmt")]
        defmt::info!(
            "BLE Controller: HCI Version {}.{}, Revision: 0x{:04X}, LMP Version: {}.{}, Manufacturer: 0x{:04X}",
            version.hci_version >> 4,
            version.hci_version & 0x0F,
            version.hci_revision,
            version.lmp_version >> 4,
            version.lmp_version & 0x0F,
            version.manufacturer_name
        );

        // 3. Read BD address
        let bd_addr = self.cmd_sender.read_bd_addr()?;
        #[cfg(feature = "defmt")]
        defmt::info!(
            "BD Address: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            bd_addr[5],
            bd_addr[4],
            bd_addr[3],
            bd_addr[2],
            bd_addr[1],
            bd_addr[0]
        );

        // 4. Set event mask (enable all events)
        // Note: The ST BLE stack handles event masks internally, so these calls
        // may not be needed. Skip if they fail with UnknownCommand.
        #[cfg(feature = "defmt")]
        defmt::info!("Calling set_event_mask...");
        if let Err(e) = self.cmd_sender.set_event_mask(0xFFFF_FFFF_FFFF_FFFF) {
            #[cfg(feature = "defmt")]
            defmt::warn!("set_event_mask failed: {:?} (may be handled internally)", e);
        } else {
            #[cfg(feature = "defmt")]
            defmt::info!("set_event_mask OK");
        }

        #[cfg(feature = "defmt")]
        defmt::info!("Calling le_set_event_mask...");
        if let Err(e) = self.cmd_sender.le_set_event_mask(0xFFFF_FFFF_FFFF_FFFF) {
            #[cfg(feature = "defmt")]
            defmt::warn!("le_set_event_mask failed: {:?} (may be handled internally)", e);
        } else {
            #[cfg(feature = "defmt")]
            defmt::info!("le_set_event_mask OK");
        }

        // 5. Read buffer sizes (optional - skip if not available)
        #[cfg(feature = "defmt")]
        defmt::info!("Calling le_read_buffer_size...");
        match self.cmd_sender.le_read_buffer_size() {
            Ok((acl_len, acl_num, iso_len, iso_num)) => {
                #[cfg(feature = "defmt")]
                defmt::info!(
                    "Buffer sizes - ACL: {} bytes x {} packets, ISO: {} bytes x {} packets",
                    acl_len,
                    acl_num,
                    iso_len,
                    iso_num
                );
            }
            Err(e) => {
                #[cfg(feature = "defmt")]
                defmt::warn!("le_read_buffer_size failed: {:?} (skipping)", e);
            }
        }

        // 6. Read supported features (optional - skip if not available)
        #[cfg(feature = "defmt")]
        defmt::info!("Calling le_read_local_supported_features...");
        match self.cmd_sender.le_read_local_supported_features() {
            Ok(features) => {
                #[cfg(feature = "defmt")]
                defmt::info!("Supported LE features: {=[u8]:#02X}", features);
            }
            Err(e) => {
                #[cfg(feature = "defmt")]
                defmt::warn!("le_read_local_supported_features failed: {:?} (skipping)", e);
            }
        }

        self.initialized.store(true, Ordering::Release);

        #[cfg(feature = "defmt")]
        defmt::info!("BLE stack initialized successfully");

        Ok(())
    }

    /// Check if the BLE stack is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
    }

    /// Set a random address for the device
    ///
    /// This must be called before advertising with OwnAddressType::Random.
    /// The random address must follow Bluetooth specification requirements.
    ///
    /// # Parameters
    ///
    /// - `address`: 6-byte random address
    pub fn set_random_address(&self, address: Address) -> Result<(), BleError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(BleError::NotInitialized);
        }

        self.cmd_sender.le_set_random_address(&address.0)
    }

    /// Get a reference to the command sender
    ///
    /// This allows direct access to HCI commands for advanced use cases.
    pub fn command_sender(&self) -> &CommandSender {
        &self.cmd_sender
    }

    /// Create an advertiser
    ///
    /// # Returns
    ///
    /// An `Advertiser` instance that can be used to start/stop BLE advertising.
    ///
    /// # Note
    ///
    /// The BLE stack must be initialized before creating an advertiser.
    pub fn advertiser(&self) -> Advertiser<'_> {
        Advertiser::new(&self.cmd_sender)
    }

    /// Create a scanner
    ///
    /// # Returns
    ///
    /// A `Scanner` instance that can be used to scan for nearby BLE devices.
    ///
    /// # Note
    ///
    /// The BLE stack must be initialized before creating a scanner.
    /// Advertising reports will be received through the main event loop
    /// as `LeAdvertisingReport` events.
    pub fn scanner(&self) -> Scanner<'_> {
        Scanner::new(&self.cmd_sender)
    }

    // ===== Connection Management =====

    /// Get a reference to the connection manager
    pub fn connections(&self) -> &ConnectionManager<MAX_CONNECTIONS> {
        &self.connections
    }

    /// Get a mutable reference to the connection manager
    pub fn connections_mut(&mut self) -> &mut ConnectionManager<MAX_CONNECTIONS> {
        &mut self.connections
    }

    /// Get a connection by handle
    pub fn get_connection(&self, handle: Handle) -> Option<&Connection> {
        self.connections.get_by_handle(handle)
    }

    /// Get a mutable connection by handle
    pub fn get_connection_mut(&mut self, handle: Handle) -> Option<&mut Connection> {
        self.connections.get_by_handle_mut(handle)
    }

    /// Disconnect a connection
    ///
    /// # Parameters
    ///
    /// - `handle`: Connection handle to disconnect
    /// - `reason`: Reason for disconnection
    pub fn disconnect(&self, handle: Handle, reason: DisconnectReason) -> Result<(), BleError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(BleError::NotInitialized);
        }

        self.cmd_sender.disconnect(handle.as_u16(), reason.as_u8())
    }

    /// Initiate a connection to a peripheral device (Central role)
    ///
    /// This starts the connection process. The connection complete event
    /// will be received when the connection is established.
    ///
    /// # Parameters
    ///
    /// - `params`: Connection initiation parameters
    pub fn connect(&self, params: &ConnectionInitParams) -> Result<(), BleError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(BleError::NotInitialized);
        }

        self.cmd_sender.le_create_connection(
            params.scan_interval,
            params.scan_window,
            params.use_filter_accept_list,
            params.peer_address_type as u8,
            params.peer_address.as_bytes(),
            params.own_address_type,
            params.conn_interval_min,
            params.conn_interval_max,
            params.max_latency,
            params.supervision_timeout,
            params.min_ce_length,
            params.max_ce_length,
        )
    }

    /// Cancel an ongoing connection attempt
    pub fn cancel_connect(&self) -> Result<(), BleError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(BleError::NotInitialized);
        }

        self.cmd_sender.le_create_connection_cancel()
    }

    /// Request connection parameter update
    ///
    /// # Parameters
    ///
    /// - `handle`: Connection handle
    /// - `interval_min`: Minimum connection interval (units of 1.25ms)
    /// - `interval_max`: Maximum connection interval (units of 1.25ms)
    /// - `latency`: Slave latency
    /// - `supervision_timeout`: Supervision timeout (units of 10ms)
    pub fn update_connection_params(
        &self,
        handle: Handle,
        interval_min: u16,
        interval_max: u16,
        latency: u16,
        supervision_timeout: u16,
    ) -> Result<(), BleError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(BleError::NotInitialized);
        }

        self.cmd_sender.le_connection_update(
            handle.as_u16(),
            interval_min,
            interval_max,
            latency,
            supervision_timeout,
            0,      // min CE length
            0xFFFF, // max CE length
        )
    }

    /// Read the current PHY for a connection
    ///
    /// # Returns
    ///
    /// Tuple of (tx_phy, rx_phy)
    pub fn read_phy(&self, handle: Handle) -> Result<(LePhy, LePhy), BleError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(BleError::NotInitialized);
        }

        let (tx, rx) = self.cmd_sender.le_read_phy(handle.as_u16())?;
        Ok((LePhy::from_u8(tx), LePhy::from_u8(rx)))
    }

    /// Process an HCI event and update internal state
    ///
    /// This method processes connection-related events and updates the
    /// connection manager. It returns a GAP event if the event is
    /// connection-related.
    ///
    /// # Returns
    ///
    /// - `Some(GapEvent)` if this was a connection-related event
    /// - `None` if not a connection event
    pub fn process_event(&mut self, event: &Event) -> Option<GapEvent> {
        match &event.params {
            EventParams::LeConnectionComplete {
                status,
                handle,
                role,
                peer_address_type,
                peer_address,
                conn_interval,
                conn_latency,
                supervision_timeout,
                ..
            } => {
                if *status == Status::Success {
                    let role = ConnectionRole::from_u8(*role)?;
                    let conn = Connection::new(
                        *handle,
                        role,
                        *peer_address_type,
                        *peer_address,
                        *conn_interval,
                        *conn_latency,
                        *supervision_timeout,
                    );
                    if let Some(stored_conn) = self.connections.allocate(conn.clone()) {
                        // Read PHY after connection
                        if let Ok((tx_phy, rx_phy)) = self.cmd_sender.le_read_phy(handle.as_u16()) {
                            stored_conn.update_phy(LePhy::from_u8(tx_phy), LePhy::from_u8(rx_phy));
                        }
                    }
                    Some(GapEvent::Connected(conn))
                } else {
                    None
                }
            }

            EventParams::LeEnhancedConnectionComplete {
                status,
                handle,
                role,
                peer_address_type,
                peer_address,
                local_resolvable_private_address,
                peer_resolvable_private_address,
                conn_interval,
                conn_latency,
                supervision_timeout,
                ..
            } => {
                if *status == Status::Success {
                    let role = ConnectionRole::from_u8(*role)?;
                    let conn = Connection::new_enhanced(
                        *handle,
                        role,
                        *peer_address_type,
                        *peer_address,
                        *local_resolvable_private_address,
                        *peer_resolvable_private_address,
                        *conn_interval,
                        *conn_latency,
                        *supervision_timeout,
                    );
                    if let Some(stored_conn) = self.connections.allocate(conn.clone()) {
                        // Read PHY after connection
                        if let Ok((tx_phy, rx_phy)) = self.cmd_sender.le_read_phy(handle.as_u16()) {
                            stored_conn.update_phy(LePhy::from_u8(tx_phy), LePhy::from_u8(rx_phy));
                        }
                    }
                    Some(GapEvent::Connected(conn))
                } else {
                    None
                }
            }

            EventParams::DisconnectionComplete { status, handle, reason } => {
                if *status == Status::Success {
                    self.connections.remove(*handle);
                    Some(GapEvent::Disconnected {
                        handle: *handle,
                        reason: *reason,
                    })
                } else {
                    None
                }
            }

            EventParams::LeConnectionUpdateComplete {
                status,
                handle,
                conn_interval,
                conn_latency,
                supervision_timeout,
            } => {
                if *status == Status::Success {
                    if let Some(conn) = self.connections.get_by_handle_mut(*handle) {
                        conn.update_params(*conn_interval, *conn_latency, *supervision_timeout);
                    }
                    Some(GapEvent::ConnectionParamsUpdated {
                        handle: *handle,
                        interval: *conn_interval,
                        latency: *conn_latency,
                        supervision_timeout: *supervision_timeout,
                    })
                } else {
                    None
                }
            }

            EventParams::LePhyUpdateComplete {
                status,
                handle,
                tx_phy,
                rx_phy,
            } => {
                if *status == Status::Success {
                    let tx = LePhy::from_u8(*tx_phy);
                    let rx = LePhy::from_u8(*rx_phy);
                    if let Some(conn) = self.connections.get_by_handle_mut(*handle) {
                        conn.update_phy(tx, rx);
                    }
                    Some(GapEvent::PhyUpdated {
                        handle: *handle,
                        tx_phy: tx,
                        rx_phy: rx,
                    })
                } else {
                    None
                }
            }

            EventParams::LeDataLengthChange {
                handle,
                max_tx_octets,
                max_tx_time,
                max_rx_octets,
                max_rx_time,
            } => Some(GapEvent::DataLengthChanged {
                handle: *handle,
                max_tx_octets: *max_tx_octets,
                max_tx_time: *max_tx_time,
                max_rx_octets: *max_rx_octets,
                max_rx_time: *max_rx_time,
            }),

            _ => None,
        }
    }

    /// Read the next BLE event
    ///
    /// This function blocks until an event is available.
    /// Events include connection complete, disconnection, etc.
    ///
    /// # Returns
    ///
    /// The next BLE event from the controller.
    ///
    /// # Note
    ///
    /// Most applications don't need to call this directly. Events are
    /// processed automatically by the stack for operations like advertising
    /// and scanning. This is provided for applications that need to handle
    /// raw events (e.g., for connection management).
    pub async fn read_event(&self) -> Event {
        read_event().await
    }
}

impl Default for Ble {
    fn default() -> Self {
        Self::new()
    }
}

/// Version information from the BLE controller
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VersionInfo {
    pub hci_version: u8,
    pub hci_revision: u16,
    pub lmp_version: u8,
    pub manufacturer_name: u16,
    pub lmp_subversion: u16,
}
