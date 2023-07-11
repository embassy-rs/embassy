use super::consts::{MAX_ED_SCAN_RESULTS_SUPPORTED, MAX_PAN_DESC_SUPPORTED, MAX_SOUNDING_LIST_SUPPORTED};
use super::typedefs::{AddressMode, MacAddress, PanDescriptor};

pub trait MacResponse {
    const SIZE: usize;

    fn parse(buf: &[u8]) -> Self;
}

/// MLME ASSOCIATE Confirm used to inform of the initiating device whether
/// its request to associate was successful or unsuccessful
#[repr(C)]
pub struct AssociateConfirm {
    /// short address allocated by the coordinator on successful association
    pub assoc_short_address: [u8; 2],
    /// status of the association request
    pub status: u8,
    /// security level to be used
    pub security_level: u8,
    /// the originator of the key to be used
    pub key_source: [u8; 8],
    /// the mode used to identify the key to be used
    pub key_id_mode: u8,
    /// the index of the key to be used
    pub key_index: u8,
}

/// MLME DISASSOCIATE Confirm used to send disassociation Confirmation to the application.
#[repr(C)]
pub struct DisassociateConfirm {
    /// status of the disassociation attempt
    pub status: u8,
    /// device addressing mode used
    pub device_addr_mode: AddressMode,
    /// the identifier of the PAN of the device
    pub device_pan_id: [u8; 2],
    /// device address
    pub device_address: MacAddress,
}

///  MLME GET Confirm which requests information about a given PIB attribute
#[repr(C)]
pub struct GetConfirm {
    /// The pointer to the value of the PIB attribute attempted to read
    pub pib_attribute_value_ptr: *const u8,
    /// Status of the GET attempt
    pub status: u8,
    /// The name of the PIB attribute attempted to read
    pub pib_attribute: u8,
    /// The lenght of the PIB attribute Value return
    pub pib_attribute_value_len: u8,
}

/// MLME GTS Confirm which eports the results of a request to allocate a new GTS
/// or to deallocate an existing GTS
#[repr(C)]
pub struct GtsConfirm {
    /// The characteristics of the GTS
    pub gts_characteristics: u8,
    /// The status of the GTS reques
    pub status: u8,
}

/// MLME RESET Confirm which is used to report the results of the reset operation
#[repr(C)]
pub struct ResetConfirm {
    /// The result of the reset operation
    status: u8,
}

/// MLME RX ENABLE Confirm which is used to report the results of the attempt
/// to enable or disable the receiver
#[repr(C)]
pub struct RxEnableConfirm {
    /// Result of the request to enable or disable the receiver
    status: u8,
}

/// MLME SCAN Confirm which is used to report the result of the channel scan request
#[repr(C)]
pub struct ScanConfirm {
    /// Status of the scan request
    pub status: u8,
    /// The type of scan performed
    pub scan_type: u8,
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

/// MLME SET Confirm which reports the result of an attempt to write a value to a PIB attribute
#[repr(C)]
pub struct SetConfirm {
    /// The result of the set operation
    pub status: u8,
    /// The name of the PIB attribute that was written
    pub pin_attribute: u8,
}

/// MLME START Confirm which is used to report the results of the attempt to
/// start using a new superframe configuration
#[repr(C)]
pub struct StartConfirm {
    /// Result of the attempt to start using an updated superframe configuration
    pub status: u8,
}

/// MLME POLL Confirm which is used to report the result of a request to poll the coordinator for data
#[repr(C)]
pub struct PollConfirm {
    /// The status of the data request
    pub status: u8,
}

/// MLME SOUNDING Confirm which  reports the result of a request to the PHY to provide
/// channel sounding information
#[repr(C)]
pub struct SoundingConfirm {
    /// Results of the sounding measurement
    sounding_list: [u8; MAX_SOUNDING_LIST_SUPPORTED],
}

/// MLME CALIBRATE Confirm which reports the result of a request to the PHY
/// to provide internal propagation path information
#[repr(C)]
pub struct CalibrateConfirm {
    /// The status of the attempt to return sounding data
    pub status: u8,
    /// A count of the propagation time from the ranging counter
    /// to the transmit antenna
    pub cal_tx_rmaker_offset: u32,
    /// A count of the propagation time from the receive antenna
    /// to the ranging counter
    pub cal_rx_rmaker_offset: u32,
}

/// MCPS DATA Confirm which will be used for reporting the results of
/// MAC data related requests from the application
#[repr(C)]
pub struct DataConfirm {
    /// The handle associated with the MSDU being confirmed
    pub msdu_handle: u8,
    /// The time, in symbols, at which the data were transmitted
    pub a_time_stamp: [u8; 4],
    /// ranging status
    pub ranging_received: u8,
    /// The status of the last MSDU transmission
    pub status: u8,
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
}

/// MCPS PURGE Confirm which will be used by the  MAC to notify the application of
/// the status of its request to purge an MSDU from the transaction queue
#[repr(C)]
pub struct PurgeConfirm {
    /// Handle associated with the MSDU requested to be purged from the transaction queue
    pub msdu_handle: u8,
    /// The status of the request
    pub status: u8,
}
