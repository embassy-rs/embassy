//! High-level BLE API for STM32WBA
//!
//! This module provides the main `Ble` struct that manages the BLE stack lifecycle
//! and provides access to GAP functionality.

use core::sync::atomic::{AtomicBool, Ordering};

use crate::wba::error::BleError;
use crate::wba::gap::Advertiser;
use crate::wba::hci::command::CommandSender;
use crate::wba::hci::event::{Event, read_event};
use crate::wba::hci::types::Address;

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
}

impl Ble {
    /// Create a new BLE instance
    ///
    /// Note: You must call `init()` before using other BLE functionality.
    pub fn new() -> Self {
        Self {
            cmd_sender: CommandSender::new(),
            initialized: AtomicBool::new(false),
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
        self.cmd_sender.set_event_mask(0xFFFF_FFFF_FFFF_FFFF)?;
        self.cmd_sender.le_set_event_mask(0xFFFF_FFFF_FFFF_FFFF)?;

        // 5. Read buffer sizes
        let (acl_len, acl_num, iso_len, iso_num) = self.cmd_sender.le_read_buffer_size()?;
        #[cfg(feature = "defmt")]
        defmt::info!(
            "Buffer sizes - ACL: {} bytes x {} packets, ISO: {} bytes x {} packets",
            acl_len,
            acl_num,
            iso_len,
            iso_num
        );

        // 6. Read supported features
        let features = self.cmd_sender.le_read_local_supported_features()?;
        #[cfg(feature = "defmt")]
        defmt::info!("Supported LE features: {=[u8]:#02X}", features);

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
