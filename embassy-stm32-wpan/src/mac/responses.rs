use super::consts::{MAX_ED_SCAN_RESULTS_SUPPORTED, MAX_PAN_DESC_SUPPORTED, MAX_SOUNDING_LIST_SUPPORTED};
use super::event::ParseableMacEvent;
use super::typedefs::{
    AddressMode, AssociationStatus, KeyIdMode, MacAddress, MacStatus, PanDescriptor, PanId, PibId, ScanType,
    SecurityLevel,
};

/// MLME ASSOCIATE Confirm used to inform of the initiating device whether
/// its request to associate was successful or unsuccessful
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
}

impl ParseableMacEvent for AssociateConfirm {
    const SIZE: usize = 16;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            assoc_short_address: [buf[0], buf[1]],
            status: AssociationStatus::try_from(buf[2])?,
            security_level: SecurityLevel::try_from(buf[3])?,
            key_source: [buf[4], buf[5], buf[6], buf[7], buf[8], buf[9], buf[10], buf[11]],
            key_id_mode: KeyIdMode::try_from(buf[12])?,
            key_index: buf[13],
        })
    }
}

/// MLME DISASSOCIATE Confirm used to send disassociation Confirmation to the application.
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

impl ParseableMacEvent for DisassociateConfirm {
    const SIZE: usize = 12;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        let device_addr_mode = AddressMode::try_from(buf[1])?;
        let device_address = match device_addr_mode {
            AddressMode::NoAddress => MacAddress { short: [0, 0] },
            AddressMode::Reserved => MacAddress { short: [0, 0] },
            AddressMode::Short => MacAddress {
                short: [buf[4], buf[5]],
            },
            AddressMode::Extended => MacAddress {
                extended: [buf[4], buf[5], buf[6], buf[7], buf[8], buf[9], buf[10], buf[11]],
            },
        };

        Ok(Self {
            status: MacStatus::try_from(buf[0])?,
            device_addr_mode,
            device_pan_id: PanId([buf[2], buf[3]]),
            device_address,
        })
    }
}

///  MLME GET Confirm which requests information about a given PIB attribute
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
}

impl ParseableMacEvent for GetConfirm {
    const SIZE: usize = 8;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        let address = u32::from_le_bytes(buf[0..4].try_into().unwrap());

        Ok(Self {
            pib_attribute_value_ptr: address as *const u8,
            status: MacStatus::try_from(buf[4])?,
            pib_attribute: PibId::try_from(buf[5])?,
            pib_attribute_value_len: buf[6],
        })
    }
}

/// MLME GTS Confirm which eports the results of a request to allocate a new GTS
/// or to deallocate an existing GTS
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GtsConfirm {
    /// The characteristics of the GTS
    pub gts_characteristics: u8,
    /// The status of the GTS reques
    pub status: MacStatus,
}

impl ParseableMacEvent for GtsConfirm {
    const SIZE: usize = 4;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            gts_characteristics: buf[0],
            status: MacStatus::try_from(buf[1])?,
        })
    }
}

/// MLME RESET Confirm which is used to report the results of the reset operation
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ResetConfirm {
    /// The result of the reset operation
    status: MacStatus,
}

impl ParseableMacEvent for ResetConfirm {
    const SIZE: usize = 4;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            status: MacStatus::try_from(buf[0])?,
        })
    }
}

/// MLME RX ENABLE Confirm which is used to report the results of the attempt
/// to enable or disable the receiver
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RxEnableConfirm {
    /// Result of the request to enable or disable the receiver
    status: MacStatus,
}

impl ParseableMacEvent for RxEnableConfirm {
    const SIZE: usize = 4;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            status: MacStatus::try_from(buf[0])?,
        })
    }
}

/// MLME SCAN Confirm which is used to report the result of the channel scan request
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

impl ParseableMacEvent for ScanConfirm {
    const SIZE: usize = 185;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        // TODO: this is unchecked

        Self::validate(buf)?;

        let mut energy_detect_list = [0; MAX_ED_SCAN_RESULTS_SUPPORTED];
        energy_detect_list.copy_from_slice(&buf[8..24]);

        let pan_descriptor_list = [
            PanDescriptor::try_from(&buf[24..46])?,
            PanDescriptor::try_from(&buf[46..68])?,
            PanDescriptor::try_from(&buf[68..90])?,
            PanDescriptor::try_from(&buf[90..102])?,
            PanDescriptor::try_from(&buf[102..124])?,
            PanDescriptor::try_from(&buf[124..146])?,
        ];

        let mut uwb_energy_detect_list = [0; MAX_ED_SCAN_RESULTS_SUPPORTED];
        uwb_energy_detect_list.copy_from_slice(&buf[147..163]);

        Ok(Self {
            status: MacStatus::try_from(buf[0])?,
            scan_type: ScanType::try_from(buf[1])?,
            channel_page: buf[2],
            unscanned_channels: [buf[3], buf[4], buf[5], buf[6]],
            result_list_size: buf[7],
            energy_detect_list,
            pan_descriptor_list,
            detected_category: buf[146],
            uwb_energy_detect_list,
        })
    }
}

/// MLME SET Confirm which reports the result of an attempt to write a value to a PIB attribute
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetConfirm {
    /// The result of the set operation
    pub status: MacStatus,
    /// The name of the PIB attribute that was written
    pub pin_attribute: PibId,
}

impl ParseableMacEvent for SetConfirm {
    const SIZE: usize = 4;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            status: MacStatus::try_from(buf[0])?,
            pin_attribute: PibId::try_from(buf[1])?,
        })
    }
}

/// MLME START Confirm which is used to report the results of the attempt to
/// start using a new superframe configuration
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StartConfirm {
    /// Result of the attempt to start using an updated superframe configuration
    pub status: MacStatus,
}

impl ParseableMacEvent for StartConfirm {
    const SIZE: usize = 4;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            status: MacStatus::try_from(buf[0])?,
        })
    }
}

/// MLME POLL Confirm which is used to report the result of a request to poll the coordinator for data
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PollConfirm {
    /// The status of the data request
    pub status: MacStatus,
}

impl ParseableMacEvent for PollConfirm {
    const SIZE: usize = 4;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            status: MacStatus::try_from(buf[0])?,
        })
    }
}

/// MLME DPS Confirm which  reports the results of the attempt to enable or disable the DPS
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DpsConfirm {
    /// The status of the DPS request
    pub status: MacStatus,
}

impl ParseableMacEvent for DpsConfirm {
    const SIZE: usize = 4;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            status: MacStatus::try_from(buf[0])?,
        })
    }
}

/// MLME SOUNDING Confirm which  reports the result of a request to the PHY to provide
/// channel sounding information
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SoundingConfirm {
    /// Results of the sounding measurement
    sounding_list: [u8; MAX_SOUNDING_LIST_SUPPORTED],
}

impl ParseableMacEvent for SoundingConfirm {
    const SIZE: usize = 1;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        let mut sounding_list = [0u8; MAX_SOUNDING_LIST_SUPPORTED];
        sounding_list[..buf.len()].copy_from_slice(buf);

        Ok(Self { sounding_list })
    }
}

/// MLME CALIBRATE Confirm which reports the result of a request to the PHY
/// to provide internal propagation path information
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CalibrateConfirm {
    /// The status of the attempt to return sounding data
    pub status: MacStatus,
    /// A count of the propagation time from the ranging counter
    /// to the transmit antenna
    pub cal_tx_rmaker_offset: u32,
    /// A count of the propagation time from the receive antenna
    /// to the ranging counter
    pub cal_rx_rmaker_offset: u32,
}

impl ParseableMacEvent for CalibrateConfirm {
    const SIZE: usize = 12;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            status: MacStatus::try_from(buf[0])?,
            // 3 byte stuffing
            cal_tx_rmaker_offset: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            cal_rx_rmaker_offset: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
        })
    }
}

/// MCPS DATA Confirm which will be used for reporting the results of
/// MAC data related requests from the application
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
}

impl ParseableMacEvent for DataConfirm {
    const SIZE: usize = 28;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            msdu_handle: buf[0],
            time_stamp: [buf[1], buf[2], buf[3], buf[4]],
            ranging_received: buf[5],
            status: MacStatus::try_from(buf[6])?,
            ranging_counter_start: u32::from_le_bytes(buf[7..11].try_into().unwrap()),
            ranging_counter_stop: u32::from_le_bytes(buf[11..15].try_into().unwrap()),
            ranging_tracking_interval: u32::from_le_bytes(buf[15..19].try_into().unwrap()),
            ranging_offset: u32::from_le_bytes(buf[19..23].try_into().unwrap()),
            ranging_fom: buf[24],
        })
    }
}

/// MCPS PURGE Confirm which will be used by the  MAC to notify the application of
/// the status of its request to purge an MSDU from the transaction queue
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PurgeConfirm {
    /// Handle associated with the MSDU requested to be purged from the transaction queue
    pub msdu_handle: u8,
    /// The status of the request
    pub status: MacStatus,
}

impl ParseableMacEvent for PurgeConfirm {
    const SIZE: usize = 4;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            msdu_handle: buf[0],
            status: MacStatus::try_from(buf[1])?,
        })
    }
}
