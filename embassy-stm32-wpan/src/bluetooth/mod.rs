//! High-level BLE API for STM32WBA

pub mod error;
pub mod gap;
pub mod gap_init;
pub mod gatt;
pub mod hci;
pub mod security;

use bt_hci::cmd::controller_baseband::{Reset, SetEventMask};
use bt_hci::cmd::info::ReadLocalVersionInformation;
use bt_hci::cmd::le::{
    LeConnCteRequestEnable, LeConnCteResponseEnable, LeConnUpdate, LeCreateConn, LeCreateConnCancel,
    LeReadAntennaInformation, LeReadBufferSizeV2, LeReadLocalSupportedFeatures, LeReadPhy, LeReceiverTest,
    LeReceiverTestV2, LeRemoteConnectionParameterRequestReply, LeSetConnCteReceiveParams, LeSetConnCteTransmitParams,
    LeSetDataLength, LeSetEventMask, LeSetPhy, LeSetRandomAddr, LeTestEnd, LeTransmitterTest, LeTransmitterTestV2,
};
use bt_hci::cmd::link_control::Disconnect;
use bt_hci::cmd::{AsyncCmd, SyncCmd};
use bt_hci::controller::Controller as _;
use bt_hci::event::le::{
    LeConnectionComplete, LeConnectionUpdateComplete, LeDataLengthChange, LeEnhancedConnectionComplete, LeEvent,
    LePhyUpdateComplete, LeRemoteConnectionParameterRequest,
};
use bt_hci::event::{DisconnectionComplete, Event, EventKind, EventPacket};
use bt_hci::param::{
    AddrKind, BdAddr, ConnHandle, CteKind, CteMask, DisconnectReason as HciDisconnectReason, Duration, EventMask,
    LeEventMask, PhyOptions,
};
use bt_hci::{ControllerToHostPacket, FromHciBytes};
use embassy_futures::yield_now;
use embassy_stm32::interrupt;
use stm32wb_hci::aci::flags::{RadioActivityMask, Role};
use stm32wb_hci::aci::gatt::GattInit;
use stm32wb_hci::aci::hal::{HalLeTxTestPacketNumber, HalReadRssi, HalRssi, HalSetRadioActivityMask};
use stm32wb_hci::aci::l2cap::L2capConnectionParameterUpdateReq;
pub use stm32wb_hci::event::BleEvent;

use crate::bluetooth::error::BleError;
use crate::bluetooth::gap::connection::{
    Connection, ConnectionInitParams, ConnectionInterval, ConnectionManager, DisconnectReason, GapEvent,
    MAX_CONNECTIONS, PhyKind,
};
use crate::bluetooth::gap::scanner::{ScanParams, ScanProcedure, Scanner};
use crate::bluetooth::gap::types::{AdvData, AdvParams, BdAddrType};
use crate::bluetooth::gap_init::{GapInitParams, init_gap_and_hal};
use crate::bluetooth::gatt::{
    GattClient, GattClientEvent, GattEvent, GattServer, client_events_from_vendor_event, from_vendor_event,
};
use crate::bluetooth::hci::types::DtmPacketPayload;
use crate::bluetooth::hci::{DtmRxPhy, DtmTxPhy, all_phys_from_raw, phy_mask_from_raw};
use crate::bluetooth::security::{SecurityEvent, SecurityManager, from_vendor_event as security_from_vendor_event};
use crate::controller::{Controller, ControllerAdapter};
use crate::{HighInterruptHandler, LowInterruptHandler, Platform, Runtime};

trait SealedMode {}
#[allow(private_bounds)]
pub trait Mode: SealedMode {}

pub struct Normal;
pub struct Test;

impl SealedMode for Normal {}
impl Mode for Normal {}

impl SealedMode for Test {}
impl Mode for Test {}

/// Main BLE interface
///
/// This struct provides the primary interface to the BLE stack.
///
/// # Example
///
/// ```no_run
/// use embassy_stm32_wpan::{HCI, gap::{AdvData, AdvParams}};
///
/// // Spawn the BLE runner task (required for proper BLE operation)
/// spawner.spawn(ble_runner_task(platform).expect("Failed to spawn BLE runner"));
///
/// // Initialize BLE stack (runner must be spawned first)
/// let mut ble = HCI::new(platform, runtime, irqs).await.unwrap();
///
/// // Create advertising data
/// let adv_data = AdvData::new()
///     .flags(AdvFlags::LE_GENERAL_DISCOVERABLE | AdvFlags::BR_EDR_NOT_SUPPORTED)
///     .unwrap()
///     .complete_local_name(b"MyDevice")
///     .unwrap();
///
/// // Start advertising
/// ble.start_advertising(AdvParams::default(), adv_data, None).await.unwrap();
///
/// // Event loop
/// let mut event_buf = EventBuffer::new();
/// loop {
///     let event = ble.read_event(&mut event_buf).await;
///     // Handle BLE events
///     if let Some(gap_event) = ble.process_event(&event) {
///         // ...
///     }
/// }
/// ```
///
/// # Crypto
///
/// Full BLE operation requires `embassy-crypto` drivers registered for the
/// operations the BLE stack uses: random numbers ([`embassy_crypto::Rng`]),
/// AES-128 ECB ([`embassy_crypto::Aes128`]), AES-128 CMAC
/// ([`embassy_crypto::Aes128Cmac`]), AES-128 CCM
/// ([`embassy_crypto::Aes128Ccm`]) and P-256 arithmetic
/// ([`embassy_crypto::p256`]). The drivers are selected by the final binary,
/// e.g. via the matching `embassy-crypto-*` features of `embassy-stm32` or via
/// `embassy-crypto-rustcrypto`.
pub struct HCI<'d, M: Mode> {
    controller: ControllerAdapter<'d>,
    connections: ConnectionManager<MAX_CONNECTIONS>,
    is_advertising: bool,
    active_scan_proc: Option<ScanProcedure>,
    _mode: M,
}

impl<'d> HCI<'d, Normal> {
    /// Create a new BLE instance
    ///
    /// Requires the shared [`Platform`] and `embassy-crypto` drivers for
    /// RNG, AES-128 and P-256; see the [type-level documentation](Self#crypto).
    pub async fn new(
        platform: &'static Platform,
        runtime: &'d mut Runtime,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::RADIO, HighInterruptHandler>
        + interrupt::typelevel::Binding<interrupt::typelevel::HASH, LowInterruptHandler>,
    ) -> Result<Self, BleError> {
        Self::new_with_role(platform, runtime, irq, Role::PERIPHERAL).await
    }

    /// Like `new`, but lets you specify the GAP role.
    ///
    /// Use `GapRole::Observer` for a scanner-only device (required for
    /// `ACI_GAP_START_OBSERVATION_PROC` to succeed).
    pub async fn new_with_role(
        platform: &'static Platform,
        runtime: &'d mut Runtime,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::RADIO, HighInterruptHandler>
        + interrupt::typelevel::Binding<interrupt::typelevel::HASH, LowInterruptHandler>,
        role: Role,
    ) -> Result<Self, BleError> {
        let uid = embassy_stm32::uid::uid();
        let mut gap_params = GapInitParams::default();
        gap_params.role = role;
        gap_params.bd_addr.copy_from_slice(&uid[0..6]);
        Self::new_with_gap_params(platform, runtime, irq, gap_params).await
    }

    /// Like `new`, but accepts fully custom `GapInitParams`.
    ///
    /// Use this to override the address type, BD address, or any other GAP
    /// init parameter — e.g. a fixed public address.
    pub async fn new_with_gap_params(
        platform: &'static Platform,
        runtime: &'d mut Runtime,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::RADIO, HighInterruptHandler>
        + interrupt::typelevel::Binding<interrupt::typelevel::HASH, LowInterruptHandler>,
        gap_params: GapInitParams,
    ) -> Result<Self, BleError> {
        let controller = Controller::new(platform, runtime, irq)
            .await
            .map_err(|_| BleError::InitializationFailed)?;

        let mut this = Self {
            connections: ConnectionManager::new(),
            is_advertising: false,
            active_scan_proc: None,
            controller: ControllerAdapter::new(controller),
            _mode: Normal,
        };

        this.init_with_gap_params(gap_params).await?;

        yield_now().await;

        Ok(this)
    }

    async fn init_with_gap_params(&mut self, mut gap_params: GapInitParams) -> Result<(), BleError> {
        info!("Ble::init: BLE stack initialized, sending HCI reset");

        // 1. Reset BLE controller
        Reset::new().exec(&self.controller).await?;

        // 2. Read local version information
        let version = ReadLocalVersionInformation::new().exec(&self.controller).await?;

        let (hci_version, hci_subversion, lmp_version, company_identifier) = (
            version.hci_version,
            version.hci_subversion,
            version.lmp_version,
            version.company_identifier,
        );
        info!(
            "BLE Controller: HCI Version {:?}, Revision: 0x{:04X}, LMP Version: {:?}, Manufacturer: 0x{:04X}",
            hci_version, hci_subversion, lmp_version, company_identifier
        );

        // 3. Set event mask (enable all events)
        // Note: The ST BLE stack handles event masks internally, so these calls
        // may not be needed. Skip if they fail with UnknownCommand.

        info!("Calling set_event_mask...");
        let event_mask = EventMask::from_hci_bytes(&[0xFF; 8]).expect("valid event mask").0;
        if let Err(e) = SetEventMask::new(event_mask).exec(&self.controller).await {
            warn!(
                "set_event_mask failed: {:?} (may be handled internally)",
                BleError::from(e)
            );
        } else {
            info!("set_event_mask OK");
        }

        info!("Calling le_set_event_mask...");
        let le_event_mask = LeEventMask::from_hci_bytes(&[0xFF; 8]).expect("valid LE event mask").0;
        if let Err(e) = LeSetEventMask::new(le_event_mask).exec(&self.controller).await {
            warn!(
                "le_set_event_mask failed: {:?} (may be handled internally)",
                BleError::from(e)
            );
        } else {
            info!("le_set_event_mask OK");
        }

        // 4. Read buffer sizes (optional - skip if not available)
        info!("Calling le_read_buffer_size...");
        match LeReadBufferSizeV2::new().exec(&self.controller).await {
            Ok(r) => {
                // The return parameters are packed: copy them out before
                // borrowing the fields to log them.
                let (acl_len, acl_packets, iso_len, iso_packets) = (
                    r.le_acl_data_packet_length,
                    r.total_num_le_acl_data_packets,
                    r.iso_data_packet_length,
                    r.total_num_iso_data_packets,
                );
                info!(
                    "Buffer sizes - ACL: {} bytes x {} packets, ISO: {} bytes x {} packets",
                    acl_len, acl_packets, iso_len, iso_packets
                );
            }
            Err(e) => warn!("le_read_buffer_size failed: {:?} (skipping)", BleError::from(e)),
        }

        // 5. Read supported features (optional - skip if not available)
        info!("Calling le_read_local_supported_features...");
        match LeReadLocalSupportedFeatures::new().exec(&self.controller).await {
            Ok(features) => info!("Supported LE features: {}", features),
            Err(e) => warn!("le_read_local_supported_features failed: {:?} (skipping)", e),
        }

        // 6. Initialize GATT layer (MUST be done BEFORE GAP initialization!)
        // Per ST's BLE_HeartRate: aci_gatt_init() is called before aci_gap_init()
        info!("Initializing GATT layer...");

        // Call aci_gatt_init from gatt module
        GattInit::new()
            .exec(&self.controller)
            .await
            .inspect_err(|e| error!("aci_gatt_init failed: {}", e))?;

        info!("GATT layer initialized");

        // 7. Initialize GAP and HAL (AFTER GATT!)
        // This is the critical step that ST's BLE_HeartRate does in Ble_Hci_Gap_Gatt_Init().
        // It configures BD address, IR/ER keys, TX power, PHY, and initializes the GAP layer.

        info!("Initializing GAP and HAL...");

        init_gap_and_hal(&self.controller, &mut gap_params).await?;

        info!("GAP and HAL initialized");

        info!("BLE stack initialized successfully");

        Ok(())
    }

    /// Create a new GATT server instance
    pub fn gatt_server(&mut self) -> GattServer {
        GattServer::new()
    }

    /// Create a new minimal GATT client instance.
    pub fn gatt_client(&self) -> GattClient {
        GattClient::new()
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

        // Configure host-stack advertising parameters/data. `configure` also
        // applies the full AD payload and the scan response, and the GAP command
        // it issues starts advertising by itself.
        //
        // Deliberately no HCI_LE_Set_Advertising_Enable here. ST's interface
        // documentation states it "must not be used when the Host stack is
        // active (see ACI GAP commands instead)", and their reference
        // applications never call it. Driving the link layer directly after GAP
        // has already armed advertising splits the controller's advertising and
        // filter state from GAP's, which stays invisible until the resolving and
        // filter accept lists are populated and then makes the controller refuse
        // every connection while still advertising -- no HCI event, nothing to log.
        gap::advertiser::configure(&self.controller, &params, &adv_data, scan_rsp_data.as_ref()).await?;
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

        // `aci_gap_set_non_discoverable` stops advertising through GAP; see
        // `start_advertising` for why the raw HCI enable/disable is not used.
        gap::advertiser::unconfigure(&self.controller).await?;
        yield_now().await;

        self.is_advertising = false;
        Ok(())
    }

    /// Check if currently advertising
    pub fn is_advertising(&self) -> bool {
        self.is_advertising
    }

    /// Update advertising data without stopping advertising.
    pub async fn update_adv_data(&mut self, adv_data: AdvData) -> Result<(), BleError> {
        gap::advertiser::update_adv_data(&self.controller, &adv_data).await
    }

    /// Update scan response data without stopping advertising.
    pub async fn update_scan_rsp_data(&mut self, scan_rsp_data: AdvData) -> Result<(), BleError> {
        gap::advertiser::update_scan_rsp_data(&self.controller, &scan_rsp_data).await
    }

    /// Set a random address for the device
    ///
    /// This must be called before advertising with `OwnAddressType::StaticRandom`.
    /// The random address must follow Bluetooth specification requirements.
    ///
    /// # Parameters
    ///
    /// - `address`: 6-byte random address
    pub async fn set_random_address(&self, address: BdAddr) -> Result<(), BleError> {
        LeSetRandomAddr::new(address).exec(&self.controller).await?;
        Ok(())
    }

    /// Get a reference to the controller.
    ///
    /// This allows the GATT/Security/Scanner helpers to issue their commands,
    /// and gives access to the raw HCI command interface for advanced use cases.
    pub fn controller(&self) -> &ControllerAdapter<'d> {
        &self.controller
    }

    /// Set data length for a connection (`HCI_LE_Set_Data_Length`).
    ///
    /// Allows the Host to suggest maximum transmission packet size and maximum packet transmission time.
    ///
    /// # Parameters
    ///
    /// - `handle`: Connection handle
    /// - `tx_octets`: Preferred maximum number of payload octets (27-251)
    /// - `tx_time`: Preferred maximum number of microseconds for TX (328-17040)
    pub async fn le_set_data_length(&self, handle: ConnHandle, tx_octets: u16, tx_time: u16) -> Result<(), BleError> {
        LeSetDataLength::new(handle, tx_octets, tx_time)
            .exec(&self.controller)
            .await?;
        Ok(())
    }

    /// Request a PHY change for a connection (`HCI_LE_Set_PHY`).
    ///
    /// - `all_phys`: Bit field: bit 0 = no TX PHY preference, bit 1 = no RX PHY preference
    /// - `tx_phys`: Bitmask of preferred TX PHYs (0x01=1M, 0x02=2M, 0x04=Coded)
    /// - `rx_phys`: Bitmask of preferred RX PHYs
    /// - `phy_options`: Coded PHY options (0=no preference, 1=S=2, 2=S=8)
    pub async fn le_set_phy(
        &self,
        handle: ConnHandle,
        all_phys: u8,
        tx_phys: u8,
        rx_phys: u8,
        phy_options: u16,
    ) -> Result<(), BleError> {
        let phy_options = match phy_options {
            0 => PhyOptions::NoPreferredCoding,
            1 => PhyOptions::S2CodingPreferred,
            2 => PhyOptions::S8CodingPreferred,
            _ => return Err(BleError::InvalidParameter),
        };
        LeSetPhy::new(
            handle,
            all_phys_from_raw(all_phys),
            phy_mask_from_raw(tx_phys),
            phy_mask_from_raw(rx_phys),
            phy_options,
        )
        .exec(&self.controller)
        .await?;
        Ok(())
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
    pub fn scanner(&self) -> Scanner {
        Scanner::new()
    }

    /// Start observer scanning (observation procedure).
    pub async fn start_scan_observation(&mut self, params: ScanParams) -> Result<(), BleError> {
        if self.active_scan_proc.is_some() {
            self.stop_scan().await?;
        }
        gap::aci_gap::start_observation(
            &self.controller,
            params.scan_interval,
            params.scan_window,
            params.scan_type as u8,
            params.own_address_type as u8,
            params.filter_duplicates,
            params.filter_policy as u8,
        )
        .await?;
        self.active_scan_proc = Some(ScanProcedure::GenericScan);
        Ok(())
    }

    /// Start active general discovery procedure.
    pub async fn start_scan_general_discovery(&mut self, params: ScanParams) -> Result<(), BleError> {
        if self.active_scan_proc.is_some() {
            self.stop_scan().await?;
        }
        gap::aci_gap::start_general_discovery(
            &self.controller,
            params.scan_interval,
            params.scan_window,
            params.own_address_type as u8,
            params.filter_duplicates,
        )
        .await?;
        self.active_scan_proc = Some(ScanProcedure::GeneralDiscovery);
        Ok(())
    }

    /// Start active limited discovery procedure.
    pub async fn start_scan_limited_discovery(&mut self, params: ScanParams) -> Result<(), BleError> {
        if self.active_scan_proc.is_some() {
            self.stop_scan().await?;
        }
        gap::aci_gap::start_limited_discovery(
            &self.controller,
            params.scan_interval,
            params.scan_window,
            params.own_address_type as u8,
            params.filter_duplicates,
        )
        .await?;
        self.active_scan_proc = Some(ScanProcedure::LimitedDiscovery);
        Ok(())
    }

    /// Stop whichever GAP scanning/discovery procedure is currently active.
    pub async fn stop_scan(&mut self) -> Result<(), BleError> {
        if let Some(proc) = self.active_scan_proc {
            let proc = gap::aci_gap::to_gap_procedure(proc);
            gap::aci_gap::terminate_gap_proc(&self.controller, proc).await?;
            self.active_scan_proc = None;
        }
        Ok(())
    }

    /// Return whether a scan/discovery procedure is currently active.
    pub fn is_scanning(&self) -> bool {
        self.active_scan_proc.is_some()
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
    pub fn get_connection(&self, handle: ConnHandle) -> Option<&Connection> {
        self.connections.get_by_handle(handle)
    }

    /// Get a mutable connection by handle
    pub fn get_connection_mut(&mut self, handle: ConnHandle) -> Option<&mut Connection> {
        self.connections.get_by_handle_mut(handle)
    }

    /// Disconnect a connection
    ///
    /// # Parameters
    ///
    /// - `handle`: Connection handle to disconnect
    /// - `reason`: Reason for disconnection
    pub async fn disconnect(&self, handle: ConnHandle, reason: DisconnectReason) -> Result<(), BleError> {
        let reason =
            HciDisconnectReason::from_hci_bytes_complete(&[reason.as_u8()]).map_err(|_| BleError::InvalidParameter)?;
        Disconnect::new(handle, reason).exec(&self.controller).await?;
        Ok(())
    }

    /// Initiate a connection to a peripheral device (Central role)
    ///
    /// This starts the connection process. The connection complete event
    /// will be received when the connection is established.
    ///
    /// # Parameters
    ///
    /// - `params`: Connection initiation parameters
    pub async fn connect(&self, params: &ConnectionInitParams) -> Result<(), BleError> {
        let peer_addr = params.peer_address.bytes();
        LeCreateConn::new(
            Duration::<625>::from_u16(params.scan_interval),
            Duration::<625>::from_u16(params.scan_window),
            params.use_filter_accept_list,
            AddrKind::new(params.peer_address.kind()),
            BdAddr::new(peer_addr),
            AddrKind::new(params.own_address_type as u8),
            Duration::<1_250>::from_u16(params.conn_interval_min),
            Duration::<1_250>::from_u16(params.conn_interval_max),
            params.max_latency,
            Duration::<10_000>::from_u16(params.supervision_timeout),
            Duration::<625>::from_u16(params.min_ce_length),
            Duration::<625>::from_u16(params.max_ce_length),
        )
        .exec(&self.controller)
        .await?;
        Ok(())
    }

    /// Cancel an ongoing connection attempt
    pub async fn cancel_connect(&self) -> Result<(), BleError> {
        LeCreateConnCancel::new().exec(&self.controller).await?;
        Ok(())
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
    pub async fn update_connection_params(
        &self,
        handle: ConnHandle,
        interval_min: u16,
        interval_max: u16,
        latency: u16,
        supervision_timeout: u16,
    ) -> Result<(), BleError> {
        LeConnUpdate::new(
            handle,
            Duration::<1_250>::from_u16(interval_min),
            Duration::<1_250>::from_u16(interval_max),
            latency,
            Duration::<10_000>::from_u16(supervision_timeout),
            Duration::<625>::from_u16(0),      // min CE length
            Duration::<625>::from_u16(0xFFFF), // max CE length
        )
        .exec(&self.controller)
        .await?;
        Ok(())
    }

    /// Ask the central to change the connection parameters, from the peripheral role.
    ///
    /// [`update_connection_params`](Self::update_connection_params) issues
    /// `HCI_LE_Connection_Update`, which is a central-role command; a peripheral
    /// has to route the request through L2CAP instead. This is the call ST's
    /// reference peripherals use for their connection-parameter-update button
    /// (`aci_l2cap_connection_parameter_update_req`).
    ///
    /// The central answers asynchronously with an L2CAP connection update
    /// response, and applies the new parameters only if it accepts them.
    ///
    /// # Parameters
    ///
    /// - `handle`: Connection handle
    /// - `interval_min`: Minimum connection interval (units of 1.25ms)
    /// - `interval_max`: Maximum connection interval (units of 1.25ms)
    /// - `latency`: Peripheral latency, in connection events
    /// - `timeout_multiplier`: Supervision timeout (units of 10ms)
    pub async fn request_connection_params(
        &self,
        handle: ConnHandle,
        interval_min: u16,
        interval_max: u16,
        latency: u16,
        timeout_multiplier: u16,
    ) -> Result<(), BleError> {
        use stm32wb_hci::aci::durations::ConnInterval;
        use stm32wb_hci::aci::ranges::ConnLatency;

        L2capConnectionParameterUpdateReq::new(
            handle,
            ConnInterval::from_units(interval_min).ok_or(BleError::InvalidParameter)?,
            ConnInterval::from_units(interval_max).ok_or(BleError::InvalidParameter)?,
            ConnLatency::new(latency).ok_or(BleError::InvalidParameter)?,
            timeout_multiplier,
        )
        .exec(&self.controller)
        .await?;
        Ok(())
    }

    /// Read the current PHY for a connection
    ///
    /// # Returns
    ///
    /// Tuple of (tx_phy, rx_phy)
    pub async fn read_phy(&self, handle: ConnHandle) -> Result<(PhyKind, PhyKind), BleError> {
        let resp = LeReadPhy::new(handle).exec(&self.controller).await?;
        Ok((resp.tx_phy, resp.rx_phy))
    }

    /// Read the RSSI (dBm) of the most recently received packet.
    ///
    /// Returns `Ok(None)` when the controller reports that RSSI is not
    /// available (raw value 127).
    pub async fn read_rssi(&self) -> Result<HalRssi, BleError> {
        HalReadRssi::new().exec(&self.controller).await.map_err(|e| e.into())
    }

    /// Select which radio activities are reported through
    /// `ACI_HAL_END_OF_RADIO_ACTIVITY_EVENT`.
    ///
    /// See [`RadioActivityMask`] for the available bits.
    pub async fn set_radio_activity_mask(&self, mask: RadioActivityMask) -> Result<(), BleError> {
        HalSetRadioActivityMask::new(mask)
            .exec(&self.controller)
            .await
            .map_err(|e| e.into())
    }

    // ===== Direction Finding / CTE Commands =====

    /// Set CTE transmit parameters for a connection (peripheral/tag side).
    ///
    /// Call this after connecting, before `le_set_connection_cte_transmit_enable`.
    ///
    /// - `cte_types`: Bit field — bit 0=AoA, bit 1=AoD 1μs slots, bit 2=AoD 2μs slots
    /// - `antenna_ids`: Antenna switching pattern IDs (2–75 elements)
    pub async fn le_set_connection_cte_transmit_parameters(
        &self,
        handle: ConnHandle,
        cte_types: u8,
        antenna_ids: &[u8],
    ) -> Result<(), BleError> {
        if antenna_ids.len() < 2 || antenna_ids.len() > 75 {
            return Err(BleError::InvalidParameter);
        }
        // The setters take and return the mask by value.
        let mask = CteMask::new()
            .set_aoa_cte(cte_types & 0x01 != 0)
            .set_aod_1us_cte(cte_types & 0x02 != 0)
            .set_aod_2us_cte(cte_types & 0x04 != 0);
        LeSetConnCteTransmitParams::new(handle, mask, antenna_ids)
            .exec(&self.controller)
            .await?;
        Ok(())
    }

    /// Enable or disable CTE response for a connection (peripheral/tag side).
    ///
    /// BT spec 7.8.86 `HCI_LE_Connection_CTE_Response_Enable`. Call
    /// `le_set_connection_cte_transmit_parameters` before enabling.
    pub async fn le_connection_cte_response_enable(&self, handle: ConnHandle, enable: bool) -> Result<(), BleError> {
        LeConnCteResponseEnable::new(handle, enable)
            .exec(&self.controller)
            .await?;
        Ok(())
    }

    /// Set CTE receive (IQ sampling) parameters for a connection (central/locator side).
    ///
    /// - `slot_durations`: 0x01 = 1μs slots, 0x02 = 2μs slots
    /// - `antenna_ids`: Antenna switching pattern (2–75 elements; ignored if sampling disabled)
    pub async fn le_set_connection_cte_receive_parameters(
        &self,
        handle: ConnHandle,
        sampling_enable: bool,
        slot_durations: u8,
        antenna_ids: &[u8],
    ) -> Result<(), BleError> {
        if sampling_enable && (antenna_ids.len() < 2 || antenna_ids.len() > 75) {
            return Err(BleError::InvalidParameter);
        }
        LeSetConnCteReceiveParams::new(handle, sampling_enable, slot_durations, antenna_ids)
            .exec(&self.controller)
            .await?;
        Ok(())
    }

    /// Enable or disable CTE requests for a connection (central/locator side).
    ///
    /// - `request_interval`: 0 = request once; N = request every N connection events
    /// - `requested_cte_length`: Requested CTE length in 8μs units (range: 2–20)
    /// - `requested_cte_type`: 0x00=AoA, 0x01=AoD 1μs, 0x02=AoD 2μs
    pub async fn le_connection_cte_request_enable(
        &self,
        handle: ConnHandle,
        enable: bool,
        request_interval: u16,
        requested_cte_length: u8,
        requested_cte_type: u8,
    ) -> Result<(), BleError> {
        let requested_cte_kind = match requested_cte_type {
            0x00 => CteKind::AoA,
            0x01 => CteKind::AoD1Us,
            0x02 => CteKind::AoD2Us,
            _ => return Err(BleError::InvalidParameter),
        };
        LeConnCteRequestEnable::new(
            handle,
            enable,
            request_interval,
            requested_cte_length,
            requested_cte_kind,
        )
        .exec(&self.controller)
        .await?;
        Ok(())
    }

    /// Read antenna information from the controller.
    ///
    /// Returns `(switching_sampling_rates, num_antennae, max_pattern_length, max_cte_length)`.
    pub async fn le_read_antenna_information(&self) -> Result<(u8, u8, u8, u8), BleError> {
        let resp = LeReadAntennaInformation::new().exec(&self.controller).await?;
        Ok((
            resp.supported_switching_sampling_rates.into_inner(),
            resp.num_antennae,
            resp.max_switching_pattern_len,
            resp.max_cte_len,
        ))
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
    pub async fn process_event(&mut self, event: &BleEvent<'_>) -> Option<GapEvent> {
        let BleEvent::Core(event) = event else {
            return None;
        };
        match event {
            Event::Le(LeEvent::LeConnectionComplete(LeConnectionComplete {
                status,
                handle,
                role,
                peer_addr_kind,
                peer_addr,
                conn_interval,
                peripheral_latency,
                supervision_timeout,
                central_clock_accuracy: _,
            })) => {
                if status.to_result().is_ok() {
                    let interval = ConnectionInterval::new(*conn_interval, *peripheral_latency, *supervision_timeout);
                    let peer_address = BdAddrType::new(*peer_addr_kind, *peer_addr);
                    let conn = Connection::new(*handle, *role, peer_address, interval);
                    self.on_connected(conn).await
                } else {
                    None
                }
            }
            Event::Le(LeEvent::LeEnhancedConnectionComplete(LeEnhancedConnectionComplete {
                status,
                handle,
                role,
                peer_addr_kind,
                peer_addr,
                local_resolvable_private_addr,
                peer_resolvable_private_addr,
                conn_interval,
                peripheral_latency,
                supervision_timeout,
                central_clock_accuracy: _,
            })) => {
                if status.to_result().is_ok() {
                    let interval = ConnectionInterval::new(*conn_interval, *peripheral_latency, *supervision_timeout);
                    let peer_address = BdAddrType::new(*peer_addr_kind, *peer_addr);
                    let conn = Connection::new_enhanced(
                        *handle,
                        *role,
                        peer_address,
                        *local_resolvable_private_addr,
                        *peer_resolvable_private_addr,
                        interval,
                    );
                    self.on_connected(conn).await
                } else {
                    None
                }
            }
            Event::DisconnectionComplete(DisconnectionComplete { status, handle, reason }) => {
                if status.to_result().is_ok() {
                    self.connections.remove(*handle);

                    Some(GapEvent::Disconnected {
                        handle: *handle,
                        reason: DisconnectReason::from(reason.into_inner()),
                    })
                } else {
                    None
                }
            }
            Event::Le(LeEvent::LeRemoteConnectionParameterRequest(LeRemoteConnectionParameterRequest {
                handle,
                interval_min,
                interval_max,
                max_latency,
                timeout,
            })) => {
                // When this event is unmasked the controller waits for a host reply. Accept the
                // requested parameters so pairing is not blocked (Android sends this immediately
                // after connect).
                match LeRemoteConnectionParameterRequestReply::new(
                    *handle,
                    Duration::<1_250>::from_u16(interval_min.as_u16()),
                    Duration::<1_250>::from_u16(interval_max.as_u16()),
                    *max_latency,
                    Duration::<10_000>::from_u16(timeout.as_u16()),
                    Duration::<625>::from_u16(0),
                    Duration::<625>::from_u16(0),
                )
                .exec(&self.controller)
                .await
                {
                    Ok(_) => info!("accepted remote connection parameter request"),
                    Err(e) => warn!("conn param request reply failed: {:?}", BleError::from(e)),
                }
                None
            }
            Event::Le(LeEvent::LeConnectionUpdateComplete(LeConnectionUpdateComplete {
                status,
                handle,
                conn_interval,
                peripheral_latency,
                supervision_timeout,
            })) => {
                if status.to_result().is_ok() {
                    let interval = ConnectionInterval::new(*conn_interval, *peripheral_latency, *supervision_timeout);
                    if let Some(conn) = self.connections.get_by_handle_mut(*handle) {
                        conn.update_interval(interval);
                    }
                    Some(GapEvent::ConnectionParamsUpdated {
                        handle: *handle,
                        interval,
                    })
                } else {
                    None
                }
            }
            Event::Le(LeEvent::LePhyUpdateComplete(LePhyUpdateComplete {
                status,
                handle,
                tx_phy,
                rx_phy,
            })) => {
                if status.to_result().is_ok() {
                    if let Some(conn) = self.connections.get_by_handle_mut(*handle) {
                        conn.update_phy(*tx_phy, *rx_phy);
                    }
                    Some(GapEvent::PhyUpdated {
                        handle: *handle,
                        tx_phy: *tx_phy,
                        rx_phy: *rx_phy,
                    })
                } else {
                    None
                }
            }
            Event::Le(LeEvent::LeDataLengthChange(LeDataLengthChange {
                handle,
                max_tx_octets,
                max_tx_time,
                max_rx_octets,
                max_rx_time,
            })) => Some(GapEvent::DataLengthChanged {
                handle: *handle,
                max_tx_octets: *max_tx_octets,
                max_tx_time: *max_tx_time,
                max_rx_octets: *max_rx_octets,
                max_rx_time: *max_rx_time,
            }),
            _ => None,
        }
    }

    async fn on_connected(&mut self, conn: Connection) -> Option<GapEvent> {
        if let Some(stored_conn) = self.connections.allocate(conn.clone()) {
            // Read PHY after connection
            if let Ok(resp) = LeReadPhy::new(conn.handle).exec(&self.controller).await {
                stored_conn.update_phy(resp.tx_phy, resp.rx_phy);
            }
        }
        // LL stops advertising automatically on connection
        self.is_advertising = false;
        Some(GapEvent::Connected(conn))
    }

    /// Convert a raw HCI event into a high-level GATT event when applicable.
    pub fn process_gatt_event(&self, event: &BleEvent<'_>) -> Option<GattEvent> {
        match event {
            BleEvent::Vendor(v) => from_vendor_event(v),
            _ => None,
        }
    }

    /// Convert a raw HCI event into one or more high-level GATT client events.
    pub fn process_gatt_client_events(&self, event: &BleEvent<'_>) -> heapless::Vec<GattClientEvent, 16> {
        match event {
            BleEvent::Vendor(v) => client_events_from_vendor_event(v),
            _ => heapless::Vec::new(),
        }
    }

    /// Return the first terminal GATT client event for a procedure, if any.
    ///
    /// Useful for "start procedure + pump events until terminal" patterns.
    pub fn process_gatt_client_terminal_event(&self, event: &BleEvent<'_>) -> Option<GattClientEvent> {
        let events = self.process_gatt_client_events(event);
        events.into_iter().find(|e| e.is_terminal())
    }

    /// Convert a raw HCI event into a high-level security event when applicable.
    pub fn process_security_event(&self, event: &BleEvent<'_>) -> Option<SecurityEvent> {
        match event {
            BleEvent::Vendor(v) => security_from_vendor_event(v),
            _ => None,
        }
    }
}

impl<'d> HCI<'d, Test> {
    /// Create a BLE instance for Direct Test Mode (DTM) only.
    ///
    /// Use this for FCC DTM (TX test, RX test, tone) where no pairing or crypto
    /// is used. Full BLE (advertising, connections, GATT) uses `new` instead,
    /// which additionally initializes GATT and GAP.
    ///
    /// Performs the minimum initialization required before issuing DTM commands
    /// (HCI_LE_Transmitter_Test, HCI_LE_Receiver_Test, HCI_LE_Test_End).
    /// Does not initialize GATT or GAP — those layers are not used in DTM.
    pub async fn new_dtm(
        platform: &'static Platform,
        runtime: &'d mut Runtime,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::RADIO, HighInterruptHandler>
        + interrupt::typelevel::Binding<interrupt::typelevel::HASH, LowInterruptHandler>,
    ) -> Result<Self, BleError> {
        let controller = Controller::new(platform, runtime, irq)
            .await
            .map_err(|_| BleError::InitializationFailed)?;

        let mut this = Self {
            connections: ConnectionManager::new(),
            is_advertising: false,
            active_scan_proc: None,
            controller: ControllerAdapter::new(controller),
            _mode: Test,
        };

        this.dtm_init().await?;

        Ok(this)
    }

    /// Initialize the BLE stack for Direct Test Mode (DTM) only.
    ///
    /// Performs the minimum initialization required before issuing DTM commands
    /// (HCI_LE_Transmitter_Test, HCI_LE_Receiver_Test, HCI_LE_Test_End).
    /// Does not initialize GATT or GAP — those layers are not used in DTM.
    async fn dtm_init(&mut self) -> Result<(), BleError> {
        // 1. Reset BLE controller
        Reset::new().exec(&self.controller).await?;

        // 2. Read local version information
        let version = ReadLocalVersionInformation::new().exec(&self.controller).await?;

        let (hci_version, hci_subversion, lmp_version, company_identifier) = (
            version.hci_version,
            version.hci_subversion,
            version.lmp_version,
            version.company_identifier,
        );
        info!(
            "BLE Controller: HCI Version {:?}, Revision: 0x{:04X}, LMP Version: {:?}, Manufacturer: 0x{:04X}",
            hci_version, hci_subversion, lmp_version, company_identifier
        );

        let event_mask = EventMask::from_hci_bytes(&[0xFF; 8]).expect("valid event mask").0;
        if let Err(e) = SetEventMask::new(event_mask).exec(&self.controller).await {
            warn!(
                "set_event_mask failed: {:?} (may be handled internally)",
                BleError::from(e)
            );
        }

        let le_event_mask = LeEventMask::from_hci_bytes(&[0xFF; 8]).expect("valid LE event mask").0;
        if let Err(e) = LeSetEventMask::new(le_event_mask).exec(&self.controller).await {
            warn!(
                "le_set_event_mask failed: {:?} (may be handled internally)",
                BleError::from(e)
            );
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
    pub async fn dtm_transmit(&mut self, channel: u8, length: u8, payload: DtmPacketPayload) -> Result<(), BleError> {
        LeTransmitterTest::new(channel, length, payload as u8)
            .exec(&self.controller)
            .await
            .map_err(|e| e.into())
    }

    /// Start a DTM receiver test on the given channel.
    ///
    /// Counts received test packets until `dtm_end()` is called.
    /// Call `Ble::deinit()` then `Ble::new_dtm()` first to ensure the LL is idle.
    ///
    /// `channel`: 0–39, maps to 2402 + (2 × N) MHz.
    pub async fn dtm_receive(&mut self, channel: u8) -> Result<(), BleError> {
        LeReceiverTest::new(channel)
            .exec(&self.controller)
            .await
            .map_err(|e| e.into())
    }

    pub async fn dtm_receive_v2(&mut self, channel: u8, phy: DtmRxPhy, modulation_index: u8) -> Result<(), BleError> {
        LeReceiverTestV2::new(channel, phy as u8, modulation_index)
            .exec(&self.controller)
            .await
            .map_err(|e| e.into())
    }

    pub async fn le_transmitter_test_v2(
        &mut self,
        tx_channel: u8,
        test_data_length: u8,
        packet_payload: DtmPacketPayload,
        phy: DtmTxPhy,
    ) -> Result<(), BleError> {
        LeTransmitterTestV2::new(tx_channel, test_data_length, packet_payload as u8, phy as u8)
            .exec(&self.controller)
            .await
            .map_err(|e| e.into())
    }

    pub async fn aci_hal_tx_test_packet_number(&mut self) -> Result<u32, BleError> {
        Ok(HalLeTxTestPacketNumber::new()
            .exec(&self.controller)
            .await?
            .number_of_packets)
    }

    /// End a DTM test and return the received packet count.
    ///
    /// For a receiver test: returns the number of packets received.
    /// For a transmitter test: always returns 0 per BLE spec Vol 4 Part E §7.8.30.
    pub async fn dtm_end(&mut self) -> Result<u16, BleError> {
        Ok(LeTestEnd::new().exec(&self.controller).await?)
    }
}

impl<'d, M: Mode> HCI<'d, M> {
    /// Fully tear down the BLE stack and return the controller state.
    ///
    /// Terminates all connections, resets the HCI controller (which resets the radio
    /// hardware to its initial state), and zeroes the host stack memory buffers so
    /// `init_ble_stack()` can reinitialize cleanly on the next `HCI::new()` call.
    ///
    /// The returned [`Runtime`] can be passed directly to the next
    /// `HCI::new()` or `HCI::new_dtm()` call, enabling multiple DTM cycles per boot
    /// without re-initializing the underlying static buffers.
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success
    /// - `Err(BleError)` if the HCI reset failed
    pub async fn deinit(mut self) -> Result<(), BleError> {
        // Terminate all active connections cleanly
        for conn in self.connections.iter() {
            // 0x16 = "connection terminated by local host"
            if let Ok(reason) = HciDisconnectReason::from_hci_bytes_complete(&[0x16]) {
                let _ = Disconnect::new(conn.handle, reason).exec(&self.controller).await;
            }
        }

        // Reset the HCI controller — this resets the radio hardware to its
        // initial state, which is required before re-calling init_ble_stack().
        Reset::new().exec(&self.controller).await?;

        self.is_advertising = false;
        self.active_scan_proc = None;
        Ok(())
    }

    /// Read the next BLE event
    ///
    /// This function blocks until an event is available, copies it into `buf`
    /// and decodes it. Events include connection complete, disconnection, etc.
    /// Packets that are not events, or that fail to decode, are logged and
    /// skipped.
    ///
    /// The returned event borrows `buf`, not `self`, so it can be passed to
    /// [`Self::process_event`] and the other `process_*` methods.
    ///
    /// # Note
    ///
    /// Most applications don't need to call this directly. Events are
    /// processed automatically by the stack for operations like advertising
    /// and scanning. This is provided for applications that need to handle
    /// raw events (e.g., for connection management).
    pub async fn read_event<'b>(&mut self, buf: &'b mut EventBuffer) -> BleEvent<'b> {
        loop {
            let mut rx = ();
            match self.controller.read(&mut rx).await {
                Ok(ControllerToHostPacket::Event(EventPacket { kind, data })) => {
                    // Decode in place first so that a packet that fails to decode
                    // does not overwrite the caller's buffer.
                    if BleEvent::from_packet(EventPacket { kind, data }).is_err() {
                        // An unparsable LE Enhanced Connection Complete means the link is up
                        // in the controller but the application never learns about it, so
                        // the peer sits at "connecting" until it times out: log it.
                        error!("HCI event dropped: decode failed ({:?})", kind);
                        continue;
                    }
                    buf.kind = kind;
                    buf.len = data.len();
                    buf.data[..buf.len].copy_from_slice(data);
                    break;
                }
                Ok(_) => debug!("HCI packet ignored: not an event"),
                Err(_) => error!("HCI packet dropped: read failed"),
            }
        }
        buf.event().expect("event decoded above")
    }
}

/// Storage for an HCI event read with [`HCI::read_event`]
pub struct EventBuffer {
    kind: EventKind,
    len: usize,
    data: [u8; 255],
}

impl EventBuffer {
    /// Create an empty buffer
    pub const fn new() -> Self {
        Self {
            kind: EventKind::Vendor,
            len: 0,
            data: [0; 255],
        }
    }

    /// Decode the event held in the buffer
    pub fn event(&self) -> Result<BleEvent<'_>, bt_hci::FromHciBytesError> {
        BleEvent::from_packet(EventPacket {
            kind: self.kind,
            data: &self.data[..self.len],
        })
    }
}

impl Default for EventBuffer {
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

pub mod config_data;
