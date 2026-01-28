use super::consts::{MAX_ED_SCAN_RESULTS_SUPPORTED, MAX_PAN_DESC_SUPPORTED, MAX_SOUNDING_LIST_SUPPORTED};
use super::event::ParseableMacEvent;
use super::typedefs::{
    AddressMode, AssociationStatus, KeyIdMode, MacAddress, MacStatus, PanDescriptor, PanId, PibId, ScanType,
    SecurityLevel,
};

/// MLME ASSOCIATE Confirm used to inform of the initiating device whether
/// its request to associate was successful or unsuccessful
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AssociateConfirm {
    /// short address allocated by the coordinator on successful association
    pub assoc_short_address: [u8; 2],
    /// status of the association request
    pub status: AssociationStatus,
    /// security level to be used
    pub security_level: SecurityLevel,
    /// the originator of the key to be used
    pub key_source: [u8; 8],
    /// the mode used to identify the key to be used
    pub key_id_mode: KeyIdMode,
    /// the index of the key to be used
    pub key_index: u8,
    /// byte stuffing to keep 32 bit alignment
    a_stuffing: [u8; 2],
}

impl ParseableMacEvent for AssociateConfirm {}

/// MLME DISASSOCIATE Confirm used to send disassociation Confirmation to the application.
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DisassociateConfirm {
    /// status of the disassociation attempt
    pub status: MacStatus,
    /// device addressing mode used
    pub device_addr_mode: AddressMode,
    /// the identifier of the PAN of the device
    pub device_pan_id: PanId,
    /// device address
    pub device_address: MacAddress,
}

impl ParseableMacEvent for DisassociateConfirm {}

///  MLME GET Confirm which requests information about a given PIB attribute
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GetConfirm {
    /// The pointer to the value of the PIB attribute attempted to read
    pub pib_attribute_value_ptr: *const u8,
    /// Status of the GET attempt
    pub status: MacStatus,
    /// The name of the PIB attribute attempted to read
    pub pib_attribute: PibId,
    /// The lenght of the PIB attribute Value return
    pub pib_attribute_value_len: u8,
    /// byte stuffing to keep 32 bit alignment
    a_stuffing: [u8; 1],
}

impl ParseableMacEvent for GetConfirm {}

/// MLME GTS Confirm which eports the results of a request to allocate a new GTS
/// or to deallocate an existing GTS
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GtsConfirm {
    /// The characteristics of the GTS
    pub gts_characteristics: u8,
    /// The status of the GTS reques
    pub status: MacStatus,
    /// byte stuffing to keep 32 bit alignment
    a_stuffing: [u8; 2],
}

impl ParseableMacEvent for GtsConfirm {}

/// MLME RESET Confirm which is used to report the results of the reset operation
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ResetConfirm {
    /// The result of the reset operation
    pub status: MacStatus,
    /// byte stuffing to keep 32 bit alignment
    a_stuffing: [u8; 3],
}

impl ParseableMacEvent for ResetConfirm {}

/// MLME RX ENABLE Confirm which is used to report the results of the attempt
/// to enable or disable the receiver
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RxEnableConfirm {
    /// Result of the request to enable or disable the receiver
    pub status: MacStatus,
    /// byte stuffing to keep 32 bit alignment
    a_stuffing: [u8; 3],
}

impl ParseableMacEvent for RxEnableConfirm {}

/// MLME SCAN Confirm which is used to report the result of the channel scan request
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ScanConfirm {
    /// Status of the scan request
    pub status: MacStatus,
    /// The type of scan performed
    pub scan_type: ScanType,
    /// Channel page on which the scan was performed
    pub channel_page: u8,
    /// Channels given in the request which were not scanned
    pub unscanned_channels: [u8; 4],
    /// Number of elements returned in the appropriate result lists
    pub result_list_size: u8,
    /// List of energy measurements
    pub energy_detect_list: [u8; MAX_ED_SCAN_RESULTS_SUPPORTED],
    /// List of PAN descriptors
    pub pan_descriptor_list: [PanDescriptor; MAX_PAN_DESC_SUPPORTED],
    /// Categorization of energy detected in channel
    pub detected_category: u8,
    ///  For UWB PHYs, the list of energy measurements taken
    pub uwb_energy_detect_list: [u8; MAX_ED_SCAN_RESULTS_SUPPORTED],
}

impl ParseableMacEvent for ScanConfirm {}

/// MLME SET Confirm which reports the result of an attempt to write a value to a PIB attribute
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetConfirm {
    /// The result of the set operation
    pub status: MacStatus,
    /// The name of the PIB attribute that was written
    pub pin_attribute: PibId,
    /// byte stuffing to keep 32 bit alignment
    a_stuffing: [u8; 2],
}

impl ParseableMacEvent for SetConfirm {}

/// MLME START Confirm which is used to report the results of the attempt to
/// start using a new superframe configuration
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StartConfirm {
    /// Result of the attempt to start using an updated superframe configuration
    pub status: MacStatus,
    /// byte stuffing to keep 32 bit alignment
    a_stuffing: [u8; 3],
}

impl ParseableMacEvent for StartConfirm {}

/// MLME POLL Confirm which is used to report the result of a request to poll the coordinator for data
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PollConfirm {
    /// The status of the data request
    pub status: MacStatus,
    /// byte stuffing to keep 32 bit alignment
    a_stuffing: [u8; 3],
}

impl ParseableMacEvent for PollConfirm {}

/// MLME DPS Confirm which  reports the results of the attempt to enable or disable the DPS
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DpsConfirm {
    /// The status of the DPS request
    pub status: MacStatus,
    /// byte stuffing to keep 32 bit alignment
    a_stuffing: [u8; 3],
}

impl ParseableMacEvent for DpsConfirm {}

/// MLME SOUNDING Confirm which  reports the result of a request to the PHY to provide
/// channel sounding information
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SoundingConfirm {
    /// Results of the sounding measurement
    pub sounding_list: [u8; MAX_SOUNDING_LIST_SUPPORTED],

    status: u8,
}

impl ParseableMacEvent for SoundingConfirm {}

/// MLME CALIBRATE Confirm which reports the result of a request to the PHY
/// to provide internal propagation path information
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CalibrateConfirm {
    /// The status of the attempt to return sounding data
    pub status: MacStatus,
    /// byte stuffing to keep 32 bit alignment
    a_stuffing: [u8; 3],
    /// A count of the propagation time from the ranging counter
    /// to the transmit antenna
    pub cal_tx_rmaker_offset: u32,
    /// A count of the propagation time from the receive antenna
    /// to the ranging counter
    pub cal_rx_rmaker_offset: u32,
}

impl ParseableMacEvent for CalibrateConfirm {}

/// MCPS DATA Confirm which will be used for reporting the results of
/// MAC data related requests from the application
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DataConfirm {
    /// The handle associated with the MSDU being confirmed
    pub msdu_handle: u8,
    /// The time, in symbols, at which the data were transmitted
    pub time_stamp: [u8; 4],
    /// ranging status
    pub ranging_received: u8,
    /// The status of the last MSDU transmission
    pub status: MacStatus,
    /// time units corresponding to an RMARKER at the antenna at
    /// the beginning of a ranging exchange
    pub ranging_counter_start: u32,
    /// time units corresponding to an RMARKER at the antenna
    /// at the end of a ranging exchange
    pub ranging_counter_stop: u32,
    /// time units in a message exchange over which the tracking offset was measured
    pub ranging_tracking_interval: u32,
    /// time units slipped or advanced by the radio tracking system
    pub ranging_offset: u32,
    /// The FoM characterizing the ranging measurement
    pub ranging_fom: u8,
    /// byte stuffing to keep 32 bit alignment
    a_stuffing: [u8; 3],
}

impl ParseableMacEvent for DataConfirm {}

/// MCPS PURGE Confirm which will be used by the  MAC to notify the application of
/// the status of its request to purge an MSDU from the transaction queue
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PurgeConfirm {
    /// Handle associated with the MSDU requested to be purged from the transaction queue
    pub msdu_handle: u8,
    /// The status of the request
    pub status: MacStatus,
    /// byte stuffing to keep 32 bit alignment
    a_stuffing: [u8; 2],
}

impl ParseableMacEvent for PurgeConfirm {}
