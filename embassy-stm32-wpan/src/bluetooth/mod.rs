//! High-level BLE API for STM32WBA

pub mod error;
pub mod gap;
pub mod gap_init;
pub mod gatt;
pub mod hci;
pub mod security;

use core::cell::RefCell;

use embassy_futures::yield_now;
use embassy_stm32::aes::Aes;
use embassy_stm32::interrupt;
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals::{AES as AesPeriph, PKA as PkaPeriph, RNG};
use embassy_stm32::pka::Pka;
use embassy_stm32::rng::Rng;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use stm32wb_hci::event::{
    DisconnectionComplete, LeConnectionComplete, LeConnectionUpdateComplete, LeDataLengthChangeEvent,
    LeEnhancedConnectionComplete, LePhyUpdateComplete,
};
use stm32wb_hci::{BdAddr, ConnectionHandle, Event, Status};

use crate::bluetooth::error::BleError;
use crate::bluetooth::gap::connection::{
    Connection, ConnectionInitParams, ConnectionManager, DisconnectReason, GapEvent, LePhy, MAX_CONNECTIONS,
};
use crate::bluetooth::gap::scanner::Scanner;
use crate::bluetooth::gap::types::{AdvData, AdvParams};
use crate::bluetooth::gap_init::{GapInitParams, init_gap_and_hal};
use crate::bluetooth::gatt::GattServer;
use crate::bluetooth::gatt::server::init_gatt_layer;
use crate::bluetooth::hci::command::CommandSender;
use crate::bluetooth::hci::types::DtmPacketPayload;
use crate::bluetooth::hci::{DtmRxPhy, DtmTxPhy};
use crate::bluetooth::security::SecurityManager;
use crate::controller::Controller;
use crate::{ControllerState, HighInterruptHandler, LowInterruptHandler};

trait SealedMode {}
#[allow(private_bounds)]
pub trait Mode: SealedMode {}

impl<T: SealedMode> Mode for T {}

pub struct Normal;
pub struct Test;

impl SealedMode for Normal {}
impl SealedMode for Test {}

/// Main BLE interface
///
/// This struct provides the primary interface to the BLE stack.
///
/// # Example
///
/// ```no_run
/// use embassy_stm32_wpan::{HCI, gap::{AdvData, AdvParams}};
///
///  // Spawn the BLE runner task (required for proper BLE operation)
///  spawner.spawn(ble_runner_task().expect("Failed to spawn BLE runner"));
///
/// // Initialize BLE stack (runner must be spawned first)
/// let mut ble = HCI::new(new_controller_state!(8), rng, aes, pka, irqs).await.unwrap();
///
/// // Create advertising data
/// let mut adv_data = AdvData::new();
/// adv_data.add_flags(0x06).unwrap();
/// adv_data.add_name("MyDevice").unwrap();
///
/// // Start advertising
/// ble.start_advertising(AdvParams::default(), adv_data, None).await.unwrap();
///
/// // Event loop
/// loop {
///     let event = ble.read_event().await;
///     // Handle BLE events
/// }
/// ```
pub struct HCI<M: Mode> {
    controller: Controller,
    cmd_sender: CommandSender,
    connections: ConnectionManager<MAX_CONNECTIONS>,
    is_advertising: bool,
    _mode: M,
}

impl HCI<Normal> {
    /// Create a new BLE instance
    ///
    /// Requires hardware peripheral instances for RNG, AES, and PKA.
    /// These are stored in statics so the BLE stack's `extern "C"` callbacks can access them.
    pub async fn new(
        state: &'static mut ControllerState,
        rng: &'static Mutex<CriticalSectionRawMutex, RefCell<Rng<'static, RNG>>>,
        aes: &'static Mutex<CriticalSectionRawMutex, RefCell<Aes<'static, AesPeriph, Blocking>>>,
        pka: &'static Mutex<CriticalSectionRawMutex, RefCell<Pka<'static, PkaPeriph>>>,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::RADIO, HighInterruptHandler>
        + interrupt::typelevel::Binding<interrupt::typelevel::HASH, LowInterruptHandler>,
    ) -> Result<Self, BleError> {
        let controller = Controller::new(state, rng, Some(aes), Some(pka), irq).await?;

        let mut this = Self {
            cmd_sender: CommandSender::new(),
            connections: ConnectionManager::new(),
            is_advertising: false,
            controller,
            _mode: Normal,
        };

        this.init()?;

        yield_now().await;

        Ok(this)
    }

    /// Initialize the BLE stack
    ///
    /// This function performs the following initialization steps:
    /// 1. Initializes BLE host stack (BleStack_Init)
    /// 2. Resets the BLE controller
    /// 3. Reads and logs the local version information
    /// 4. Reads the BD address
    /// 5. Sets the event mask
    /// 6. Reads buffer sizes
    /// 7. Reads supported features
    /// 8. Initializes GATT layer (aci_gatt_init) - MUST be before GAP!
    /// 9. Initializes GAP and HAL (aci_gap_init, aci_hal_write_config_data, etc.)
    ///
    /// Must be called before any other BLE operations.
    ///
    /// # Initialization Order
    ///
    /// The order is critical: GATT initialization MUST happen before GAP initialization.
    /// This matches ST's BLE_HeartRate example sequence.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if initialization succeeded
    /// - `Err(BleError)` if any initialization step failed
    fn init(&mut self) -> Result<(), BleError> {
        info!("Ble::init: BLE stack initialized, sending HCI reset");

        // 1. Reset BLE controller
        self.cmd_sender.reset()?;

        // 2. Read local version information
        let version = self.cmd_sender.read_local_version()?;

        info!(
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

        info!(
            "BD Address: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            bd_addr[5], bd_addr[4], bd_addr[3], bd_addr[2], bd_addr[1], bd_addr[0]
        );

        // 4. Set event mask (enable all events)
        // Note: The ST BLE stack handles event masks internally, so these calls
        // may not be needed. Skip if they fail with UnknownCommand.

        info!("Calling set_event_mask...");
        if let Err(e) = self.cmd_sender.set_event_mask(0xFFFF_FFFF_FFFF_FFFF) {
            warn!("set_event_mask failed: {:?} (may be handled internally)", e);
        } else {
            info!("set_event_mask OK");
        }

        info!("Calling le_set_event_mask...");
        if let Err(e) = self.cmd_sender.le_set_event_mask(0xFFFF_FFFF_FFFF_FFFF) {
            warn!("le_set_event_mask failed: {:?} (may be handled internally)", e);
        } else {
            info!("le_set_event_mask OK");
        }

        // 5. Read buffer sizes (optional - skip if not available)
        info!("Calling le_read_buffer_size...");
        match self.cmd_sender.le_read_buffer_size() {
            Ok((acl_len, acl_num, iso_len, iso_num)) => info!(
                "Buffer sizes - ACL: {} bytes x {} packets, ISO: {} bytes x {} packets",
                acl_len, acl_num, iso_len, iso_num
            ),
            Err(e) => warn!("le_read_buffer_size failed: {:?} (skipping)", e),
        }

        // 6. Read supported features (optional - skip if not available)
        info!("Calling le_read_local_supported_features...");
        match self.cmd_sender.le_read_local_supported_features() {
            Ok(features) => info!("Supported LE features: {=[u8]:#02X}", features),
            Err(e) => warn!("le_read_local_supported_features failed: {:?} (skipping)", e),
        }

        // 7. Initialize GATT layer (MUST be done BEFORE GAP initialization!)
        // Per ST's BLE_HeartRate: aci_gatt_init() is called before aci_gap_init()
        info!("Initializing GATT layer...");

        // Call aci_gatt_init from gatt module
        init_gatt_layer()?;

        info!("GATT layer initialized");

        // 8. Initialize GAP and HAL (AFTER GATT!)
        // This is the critical step that ST's BLE_HeartRate does in Ble_Hci_Gap_Gatt_Init().
        // It configures BD address, IR/ER keys, TX power, PHY, and initializes the GAP layer.

        info!("Initializing GAP and HAL...");

        // Derive a stable random static address from the chip's unique ID.
        let uid = embassy_stm32::uid::uid();
        let mut gap_params = GapInitParams::default();
        gap_params.bd_addr.copy_from_slice(&uid[0..6]);

        let _gap_handles = init_gap_and_hal(&gap_params)?;

        info!("GAP and HAL initialized");

        info!("BLE stack initialized successfully");

        Ok(())
    }

    /// Create a new GATT server instance
    pub fn gatt_server(&mut self) -> GattServer {
        GattServer::new()
    }

    /// Create a new security manager
    pub fn security_manager(&mut self) -> SecurityManager {
        SecurityManager::new()
    }

    /// Start advertising
    ///
    /// # Parameters
    ///
    /// - `params`: Advertising parameters (interval, type, address type, etc.)
    /// - `adv_data`: Advertising data (up to 31 bytes)
    /// - `scan_rsp_data`: Optional scan response data (up to 31 bytes)
    ///
    /// # Returns
    ///
    /// - `Ok(())` if advertising started successfully
    /// - `Err(BleError)` if an error occurred
    ///
    /// # Notes
    ///
    /// This function will stop any ongoing advertising before starting new advertising.
    pub async fn start_advertising(
        &mut self,
        params: AdvParams,
        adv_data: AdvData,
        scan_rsp_data: Option<AdvData>,
    ) -> Result<(), BleError> {
        if self.is_advertising {
            self.stop_advertising().await?;
        }

        // Configure host-stack advertising parameters and data
        gap::advertiser::configure(&params, &adv_data, scan_rsp_data.as_ref())?;

        // Enable LL advertising
        self.cmd_sender.le_set_advertise_enable(true)?;
        yield_now().await;

        self.is_advertising = true;
        Ok(())
    }

    /// Stop advertising
    ///
    /// # Returns
    ///
    /// - `Ok(())` if advertising stopped successfully
    /// - `Err(BleError)` if an error occurred
    pub async fn stop_advertising(&mut self) -> Result<(), BleError> {
        if !self.is_advertising {
            return Ok(());
        }

        // Disable LL advertising
        self.cmd_sender.le_set_advertise_enable(false)?;
        yield_now().await;

        // Remove advertising configuration from the host stack
        gap::advertiser::unconfigure()?;

        self.is_advertising = false;
        Ok(())
    }

    /// Check if currently advertising
    pub fn is_advertising(&self) -> bool {
        self.is_advertising
    }

    /// Update advertising data without stopping advertising.
    pub fn update_adv_data(&mut self, adv_data: AdvData) -> Result<(), BleError> {
        gap::advertiser::update_adv_data(&self.cmd_sender, &adv_data)
    }

    /// Update scan response data without stopping advertising.
    pub fn update_scan_rsp_data(&mut self, scan_rsp_data: AdvData) -> Result<(), BleError> {
        gap::advertiser::update_scan_rsp_data(&self.cmd_sender, &scan_rsp_data)
    }

    /// Set a random address for the device
    ///
    /// This must be called before advertising with OwnAddressType::Random.
    /// The random address must follow Bluetooth specification requirements.
    ///
    /// # Parameters
    ///
    /// - `address`: 6-byte random address
    pub fn set_random_address(&self, address: BdAddr) -> Result<(), BleError> {
        self.cmd_sender.le_set_random_address(&address.0)
    }

    /// Get a reference to the command sender
    ///
    /// This allows direct access to HCI commands for advanced use cases.
    pub fn command_sender(&self) -> &CommandSender {
        &self.cmd_sender
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
    pub fn get_connection(&self, handle: ConnectionHandle) -> Option<&Connection> {
        self.connections.get_by_handle(handle)
    }

    /// Get a mutable connection by handle
    pub fn get_connection_mut(&mut self, handle: ConnectionHandle) -> Option<&mut Connection> {
        self.connections.get_by_handle_mut(handle)
    }

    /// Disconnect a connection
    ///
    /// # Parameters
    ///
    /// - `handle`: Connection handle to disconnect
    /// - `reason`: Reason for disconnection
    pub fn disconnect(&self, handle: ConnectionHandle, reason: DisconnectReason) -> Result<(), BleError> {
        self.cmd_sender.disconnect(handle.0, reason.as_u8())
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
        self.cmd_sender.le_create_connection(
            params.scan_interval,
            params.scan_window,
            params.use_filter_accept_list,
            params.peer_address,
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
        handle: ConnectionHandle,
        interval_min: u16,
        interval_max: u16,
        latency: u16,
        supervision_timeout: u16,
    ) -> Result<(), BleError> {
        self.cmd_sender.le_connection_update(
            handle.0,
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
    pub fn read_phy(&self, handle: ConnectionHandle) -> Result<(LePhy, LePhy), BleError> {
        let (tx, rx) = self.cmd_sender.le_read_phy(handle.0)?;
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
    pub fn process_event(&mut self, event: &stm32wb_hci::Event) -> Option<GapEvent> {
        match event {
            Event::LeConnectionComplete(LeConnectionComplete {
                status,
                conn_handle,
                role,
                peer_bd_addr,
                conn_interval,
                central_clock_accuracy,
            }) => {
                let _ = central_clock_accuracy;

                if matches!(status, Status::Success) {
                    let conn = Connection::new(*conn_handle, *role, *peer_bd_addr, *conn_interval);

                    if let Some(stored_conn) = self.connections.allocate(conn.clone()) {
                        // Read PHY after connection
                        if let Ok((tx_phy, rx_phy)) = self.cmd_sender.le_read_phy(conn_handle.0) {
                            stored_conn.update_phy(tx_phy.try_into().unwrap(), rx_phy.try_into().unwrap());
                        }
                    }
                    // LL stops advertising automatically on connection
                    self.is_advertising = false;
                    Some(GapEvent::Connected(conn))
                } else {
                    None
                }
            }
            Event::LeEnhancedConnectionComplete(LeEnhancedConnectionComplete {
                status,
                conn_handle,
                role,
                peer_bd_addr,
                local_resolvable_private_address,
                peer_resolvable_private_address,
                conn_interval,
                central_clock_accuracy,
            }) => {
                let _ = central_clock_accuracy;

                if matches!(status, Status::Success) {
                    let conn = Connection::new_enhanced(
                        *conn_handle,
                        *role,
                        *peer_bd_addr,
                        *local_resolvable_private_address,
                        *peer_resolvable_private_address,
                        *conn_interval,
                    );

                    if let Some(stored_conn) = self.connections.allocate(conn.clone()) {
                        // Read PHY after connection
                        if let Ok((tx_phy, rx_phy)) = self.cmd_sender.le_read_phy(conn_handle.0) {
                            stored_conn.update_phy(tx_phy.try_into().unwrap(), rx_phy.try_into().unwrap());
                        }
                    }
                    // LL stops advertising automatically on connection
                    self.is_advertising = false;
                    Some(GapEvent::Connected(conn))
                } else {
                    None
                }
            }
            Event::DisconnectionComplete(DisconnectionComplete {
                status,
                conn_handle,
                reason,
            }) => {
                if matches!(status, Status::Success) {
                    self.connections.remove(*conn_handle);

                    Some(GapEvent::Disconnected {
                        handle: *conn_handle,
                        reason: (*reason).into(),
                    })
                } else {
                    None
                }
            }
            Event::LeConnectionUpdateComplete(LeConnectionUpdateComplete {
                status,
                conn_handle,
                conn_interval,
            }) => {
                if matches!(status, Status::Success) {
                    if let Some(conn) = self.connections.get_by_handle_mut(*conn_handle) {
                        conn.update_interval(*conn_interval);
                    }
                    Some(GapEvent::ConnectionParamsUpdated {
                        handle: *conn_handle,
                        interval: *conn_interval,
                    })
                } else {
                    None
                }
            }
            Event::LePhyUpdateComplete(LePhyUpdateComplete {
                conn_handle,
                status,
                tx_phy,
                rx_phy,
            }) => {
                if matches!(status, Status::Success) {
                    if let Some(conn) = self.connections.get_by_handle_mut(*conn_handle) {
                        conn.update_phy(*tx_phy, *rx_phy);
                    }
                    Some(GapEvent::PhyUpdated {
                        handle: *conn_handle,
                        tx_phy: *tx_phy,
                        rx_phy: *rx_phy,
                    })
                } else {
                    None
                }
            }
            Event::LeDataLengthChangeEvent(LeDataLengthChangeEvent {
                conn_handle,
                max_rx_octets,
                max_rx_time,
                max_tx_octets,
                max_tx_time,
            }) => Some(GapEvent::DataLengthChanged {
                handle: *conn_handle,
                max_tx_octets: *max_tx_octets,
                max_tx_time: *max_tx_time,
                max_rx_octets: *max_rx_octets,
                max_rx_time: *max_rx_time,
            }),
            _ => None,
        }
    }
}

impl HCI<Test> {
    /// Create a BLE instance for Direct Test Mode (DTM) only.
    ///
    /// Only RNG is required; AES and PKA are left unset. Use this for FCC DTM
    /// (TX test, RX test, tone) where no pairing or crypto is used. Do not use
    /// for full BLE (advertising, connections, GATT) as those require AES/PKA.
    ///
    /// Performs the minimum initialization required before issuing DTM commands
    /// (HCI_LE_Transmitter_Test, HCI_LE_Receiver_Test, HCI_LE_Test_End).
    /// Does not initialize GATT or GAP — those layers are not used in DTM.
    pub async fn new_dtm(
        state: &'static mut ControllerState,
        rng: &'static Mutex<CriticalSectionRawMutex, RefCell<Rng<'static, RNG>>>,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::RADIO, HighInterruptHandler>
        + interrupt::typelevel::Binding<interrupt::typelevel::HASH, LowInterruptHandler>,
    ) -> Result<Self, BleError> {
        let controller = Controller::new(state, rng, None, None, irq).await?;

        let mut this = Self {
            cmd_sender: CommandSender::new(),
            connections: ConnectionManager::new(),
            is_advertising: false,
            controller,
            _mode: Test,
        };

        this.dtm_init()?;

        Ok(this)
    }

    /// Initialize the BLE stack for Direct Test Mode (DTM) only.
    ///
    /// Performs the minimum initialization required before issuing DTM commands
    /// (HCI_LE_Transmitter_Test, HCI_LE_Receiver_Test, HCI_LE_Test_End).
    /// Does not initialize GATT or GAP — those layers are not used in DTM.
    fn dtm_init(&mut self) -> Result<(), BleError> {
        self.cmd_sender.reset()?;

        let version = self.cmd_sender.read_local_version()?;

        info!(
            "BLE Controller: HCI Version {}.{}, Manufacturer: 0x{:04X}",
            version.hci_version >> 4,
            version.hci_version & 0x0F,
            version.manufacturer_name
        );

        if let Err(e) = self.cmd_sender.set_event_mask(0xFFFF_FFFF_FFFF_FFFF) {
            warn!("set_event_mask failed: {:?}", e);
        }
        if let Err(e) = self.cmd_sender.le_set_event_mask(0xFFFF_FFFF_FFFF_FFFF) {
            warn!("le_set_event_mask failed: {:?}", e);
        }

        info!("BLE stack initialized for DTM");

        Ok(())
    }

    /// Start a DTM transmitter test on the given channel.
    ///
    /// Transmits test packets continuously until `dtm_end()` is called.
    /// Call `Ble::deinit()` then `Ble::new_dtm()` first to ensure the LL is idle.
    ///
    /// `channel`: 0–39, maps to 2402 + (2 × N) MHz.
    /// `length`: payload bytes per packet, 0–255.
    /// `payload`: bit pattern to transmit.
    pub fn dtm_transmit(&mut self, channel: u8, length: u8, payload: DtmPacketPayload) -> Result<(), BleError> {
        hci::command::le_transmitter_test(channel, length, payload)
    }

    /// Start a DTM receiver test on the given channel.
    ///
    /// Counts received test packets until `dtm_end()` is called.
    /// Call `Ble::deinit()` then `Ble::new_dtm()` first to ensure the LL is idle.
    ///
    /// `channel`: 0–39, maps to 2402 + (2 × N) MHz.
    pub fn dtm_receive(&mut self, channel: u8) -> Result<(), BleError> {
        hci::command::le_receiver_test(channel)
    }

    pub fn dtm_receive_v2(&mut self, rx_channel: u8, phy: DtmRxPhy, modulation_index: u8) -> Result<(), BleError> {
        hci::command::le_receiver_test_v2(rx_channel, phy, modulation_index)
    }

    pub fn le_transmitter_test_v2(
        &mut self,
        tx_channel: u8,
        test_data_length: u8,
        packet_payload: DtmPacketPayload,
        phy: DtmTxPhy,
    ) -> Result<(), BleError> {
        hci::command::le_transmitter_test_v2(tx_channel, test_data_length, packet_payload, phy)
    }

    pub fn aci_hal_tx_test_packet_number(&mut self) -> Result<u32, BleError> {
        hci::command::aci_hal_tx_test_packet_number()
    }

    /// End a DTM test and return the received packet count.
    ///
    /// For a receiver test: returns the number of packets received.
    /// For a transmitter test: always returns 0 per BLE spec Vol 4 Part E §7.8.30.
    pub fn dtm_end(&mut self) -> Result<u16, BleError> {
        hci::command::le_test_end()
    }
}

impl<M: Mode> HCI<M> {
    /// Fully tear down the BLE stack and return the controller state.
    ///
    /// Terminates all connections, resets the HCI controller (which resets the radio
    /// hardware to its initial state), and zeroes the host stack memory buffers so
    /// `init_ble_stack()` can reinitialize cleanly on the next `HCI::new()` call.
    ///
    /// The returned `&'static mut ControllerState` can be passed directly to the next
    /// `HCI::new()` or `HCI::new_dtm()` call, enabling multiple DTM cycles per boot
    /// without re-initializing the underlying static buffers.
    ///
    /// # Returns
    ///
    /// - `Ok(&'static mut ControllerState)` on success
    /// - `Err(BleError)` if the HCI reset failed
    pub fn deinit(mut self) -> Result<&'static mut ControllerState, BleError> {
        // Terminate all active connections cleanly
        for conn in self.connections.iter() {
            // 0x16 = "local host terminated connection"
            let _ = self.cmd_sender.disconnect(conn.handle.0, 0x16);
        }

        // Reset the HCI controller — this resets the radio hardware to its
        // initial state, which is required before re-calling init_ble_stack().
        self.cmd_sender.reset()?;

        self.is_advertising = false;
        Ok(self.controller.release_state())
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
    pub async fn read_event(&mut self) -> stm32wb_hci::Event {
        loop {
            if let Ok(event) = self.controller.read_event().await {
                return event;
            }
        }
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
