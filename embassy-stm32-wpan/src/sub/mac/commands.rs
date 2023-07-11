use super::opcodes::OpcodeM4ToM0;
use super::typedefs::{AddressMode, GtsCharacteristics, MacAddress, PibId};

pub trait MacCommand {
    const OPCODE: OpcodeM4ToM0;
    const SIZE: usize;

    fn copy_into_slice(&self, buf: &mut [u8]) {
        unsafe { core::ptr::copy(self as *const _ as *const u8, buf as *mut _ as *mut u8, 8) };
    }
}

/// MLME ASSOCIATE Request used to request an association
#[repr(C)]
pub struct AssociateRequest {
    /// the logical channel on which to attempt association
    pub channel_number: u8,
    /// the channel page on which to attempt association
    pub channel_page: u8,
    /// coordinator addressing mode
    pub coord_addr_mode: AddressMode,
    /// operational capabilities of the associating device
    pub capability_information: u8,
    /// the identifier of the PAN with which to associate
    pub coord_pan_id: [u8; 2],
    /// the security level to be used
    pub security_level: u8,
    /// the mode used to identify the key to be used
    pub key_id_mode: u8,
    /// the originator of the key to be used
    pub key_source: [u8; 8],
    /// Coordinator address
    pub coord_address: MacAddress,
    /// the index of the key to be used
    pub key_index: u8,
}

impl MacCommand for AssociateRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeAssociateReq;
    const SIZE: usize = 25;
}

/// MLME DISASSOCIATE Request sed to request a disassociation
#[repr(C)]
pub struct DisassociateRequest {
    /// device addressing mode used
    pub device_addr_mode: AddressMode,
    /// the identifier of the PAN of the device
    pub device_pan_id: [u8; 2],
    /// the reason for the disassociation
    pub disassociate_reason: u8,
    /// device address
    pub device_address: MacAddress,
    /// `true` if the disassociation notification command is to be sent indirectly
    pub tx_indirect: bool,
    /// the security level to be used
    pub security_level: u8,
    /// the mode to be used to indetify the key to be used
    pub key_id_mode: u8,
    /// the index of the key to be used
    pub key_index: u8,
    /// the originator of the key to be used
    pub key_source: [u8; 8],
}

impl MacCommand for DisassociateRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeDisassociateReq;
    const SIZE: usize = 24;
}

/// MLME GET Request used to request a PIB value
#[repr(C)]
pub struct GetRequest {
    /// the name of the PIB attribute to read
    pub pib_attribute: PibId,
}

impl MacCommand for GetRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeGetReq;
    const SIZE: usize = 4;
}

/// MLME GTS Request used to request and maintain GTSs
#[repr(C)]
pub struct GtsRequest {
    /// the characteristics of the GTS
    pub characteristics: GtsCharacteristics,
    /// the security level to be used
    pub security_level: u8,
    /// the mode used to identify the key to be used
    pub key_id_mode: u8,
    /// the index of the key to be used
    pub key_index: u8,
    /// the originator of the key to be used
    pub key_source: [u8; 8],
}

impl MacCommand for GtsRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeGetReq;
    const SIZE: usize = 12;
}

#[repr(C)]
pub struct ResetRequest {
    /// MAC PIB attributes are set to their default values or not during reset
    pub set_default_pib: bool,
}

impl MacCommand for ResetRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeResetReq;
    const SIZE: usize = 4;
}

/// MLME RX ENABLE Request used to request that the receiver is either enabled
/// for a finite period of time or disabled
#[repr(C)]
pub struct RxEnableRequest {
    /// the request operation can be deferred or not
    pub defer_permit: bool,
    /// configure the transceiver to RX with ranging for a value of
    /// RANGING_ON or to not enable ranging for RANGING_OFF
    pub ranging_rx_control: u8,
    /// number of symbols measured before the receiver is to be enabled or disabled
    pub rx_on_time: [u8; 4],
    /// number of symbols for which the receiver is to be enabled
    pub rx_on_duration: [u8; 4],
}

impl MacCommand for RxEnableRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeRxEnableReq;
    const SIZE: usize = 12;

    fn copy_into_slice(&self, buf: &mut [u8]) {
        buf[0] = self.defer_permit as u8;
        buf[1] = self.ranging_rx_control as u8;

        // stuffing to keep 32bit alignment
        buf[2] = 0;
        buf[3] = 0;

        buf[4..8].copy_from_slice(&self.rx_on_time);
        buf[8..12].copy_from_slice(&self.rx_on_duration);
    }
}

/// MLME SCAN Request used to initiate a channel scan over a given list of channels
#[repr(C)]
pub struct ScanRequest {
    /// the type of scan to be performed
    pub scan_type: u8,
    /// the time spent on scanning each channel
    pub scan_duration: u8,
    /// channel page on which to perform the scan
    pub channel_page: u8,
    /// security level to be used
    pub security_level: u8,
    /// indicate which channels are to be scanned
    pub scan_channels: [u8; 4],
    /// originator the key to be used
    pub key_source: [u8; 8],
    /// mode used to identify the key to be used
    pub key_id_mode: u8,
    /// index of the key to be used
    pub key_index: u8,
}

impl MacCommand for ScanRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeScanReq;
    const SIZE: usize = 20;
}

/// MLME SET Request used to attempt to write the given value to the indicated PIB attribute
#[repr(C)]
pub struct SetRequest {
    /// the pointer to the value of the PIB attribute to set
    pub pib_attribute_ptr: *const u8,
    /// the name of the PIB attribute to set
    pub pib_attribute: PibId,
}

impl MacCommand for SetRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeSetReq;
    const SIZE: usize = 8;
}

/// MLME START Request used by the FFDs to intiate a new PAN or to begin using a new superframe
/// configuration
#[derive(Default)]
#[repr(C)]
pub struct StartRequest {
    /// PAN indentifier to used by the device
    pub pan_id: [u8; 2],
    /// logical channel on which to begin
    pub channel_number: u8,
    /// channel page on which to begin
    pub channel_page: u8,
    /// time at which to begin transmitting beacons
    pub start_time: [u8; 4],
    /// indicated how often the beacon is to be transmitted
    pub beacon_order: u8,
    /// length of the active portion of the superframe
    pub superframe_order: u8,
    /// indicated wheter the device is a PAN coordinator or not
    pub pan_coordinator: bool,
    /// indicates if the receiver of the beaconing device is disabled or not
    pub battery_life_extension: bool,
    /// indicated if the coordinator realignment command is to be trasmitted
    pub coord_realignment: u8,
    /// indicated if the coordinator realignment command is to be trasmitted
    pub coord_realign_security_level: u8,
    /// index of the key to be used
    pub coord_realign_key_id_index: u8,
    /// originator of the key to be used
    pub coord_realign_key_source: [u8; 8],
    /// security level to be used for beacon frames
    pub beacon_security_level: u8,
    /// mode used to identify the key to be used
    pub beacon_key_id_mode: u8,
    /// index of the key to be used
    pub beacon_key_index: u8,
    /// originator of the key to be used
    pub beacon_key_source: [u8; 8],
}

impl MacCommand for StartRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeStartReq;
    const SIZE: usize = 35;
}

/// MLME SYNC Request used to synchronize with the coordinator by acquiring and, if
/// specified, tracking its beacons
#[repr(C)]
pub struct SyncRequest {
    /// the channel number on which to attempt coordinator synchronization
    pub channel_number: u8,
    /// the channel page on which to attempt coordinator synchronization
    pub channel_page: u8,
    /// `true` if the MLME is to synchronize with the next beacon and attempts
    /// to track all future beacons.
    ///
    /// `false` if the MLME is to synchronize with only the next beacon
    pub track_beacon: bool,
}

impl MacCommand for SyncRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeSyncReq;
    const SIZE: usize = 4;
}

/// MLME POLL Request propmts the device to request data from the coordinator
#[repr(C)]
pub struct PollRequest {
    /// addressing mode of the coordinator
    pub coord_addr_mode: AddressMode,
    /// security level to be used
    pub security_level: u8,
    /// mode used to identify the key to be used
    pub key_id_mode: u8,
    /// index of the key to be used
    pub key_index: u8,
    /// coordinator address
    pub coord_address: MacAddress,
    /// originator of the key to be used
    pub key_source: [u8; 8],
    /// PAN identifier of the coordinator
    pub coord_pan_id: [u8; 2],
}

impl MacCommand for PollRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmePollReq;
    const SIZE: usize = 24;
}

/// MLME DPS Request allows the next higher layer to request that the PHY utilize a
/// given pair of preamble codes for a single use pending expiration of the DPSIndexDuration
#[repr(C)]
pub struct DpsRequest {
    /// the index value for the transmitter
    tx_dps_index: u8,
    /// the index value of the receiver
    rx_dps_index: u8,
    /// the number of symbols for which the transmitter and receiver will utilize the
    /// respective DPS indices
    dps_index_duration: u8,
}

impl MacCommand for DpsRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeDpsReq;
    const SIZE: usize = 4;
}

/// MLME SOUNDING request primitive which is used by the next higher layer to request that
/// the PHY respond with channel sounding information
#[repr(C)]
pub struct SoundingRequest;

impl MacCommand for SoundingRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeSoundingReq;
    const SIZE: usize = 4;
}

/// MLME CALIBRATE request primitive which used  to obtain the results of a ranging
/// calibration request from an RDEV
#[repr(C)]
pub struct CalibrateRequest;

impl MacCommand for CalibrateRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeCalibrateReq;
    const SIZE: usize = 4;
}

/// MCPS DATA Request used for MAC data related requests from the application
#[repr(C)]
pub struct DataRequest {
    /// the handle assocated with the MSDU to be transmitted
    pub msdu_ptr: *const u8,
    /// source addressing mode used
    pub src_addr_mode: AddressMode,
    /// destination addressing mode used
    pub dst_addr_mode: AddressMode,
    /// destination PAN Id
    pub dst_pan_id: [u8; 2],
    /// destination address
    pub dst_address: MacAddress,
    /// the number of octets contained in the MSDU
    pub msdu_length: u8,
    /// the handle assocated with the MSDU to be transmitted
    pub msdu_handle: u8,
    /// the ACK transmittion options for the MSDU
    pub ack_tx: u8,
    /// `true` if a GTS is to be used for transmission
    ///
    /// `false` indicates that the CAP will be used
    pub gts_tx: bool,
    /// the pending bit transmission options for the MSDU
    pub indirect_tx: u8,
    /// the security level to be used
    pub security_level: u8,
    /// the mode used to indentify the key to be used
    pub key_id_mode: u8,
    /// the index of the key to be used
    pub key_index: u8,
    /// the originator of the key to be used
    pub key_source: [u8; 8],
    /// 2011 - the pulse repitition value
    pub uwbprf: u8,
    /// 2011 - the ranging configuration
    pub ranging: u8,
    /// 2011 - the preamble symbol repititions
    pub uwb_preamble_symbol_repetitions: u8,
    /// 2011 - indicates the data rate
    pub datrate: u8,
}

impl MacCommand for DataRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::McpsDataReq;
    const SIZE: usize = 40;
}

/// for MCPS PURGE Request used to purge an MSDU from the transaction queue
#[repr(C)]
pub struct PurgeRequest {
    /// the handle associated with the MSDU to be purged from the transaction
    /// queue
    pub msdu_handle: u8,
}

impl MacCommand for PurgeRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::McpsPurgeReq;
    const SIZE: usize = 4;
}

/// MLME ASSOCIATE Response used to initiate a response to an MLME-ASSOCIATE.indication
#[repr(C)]
pub struct AssociateResponse {
    /// extended address of the device requesting association
    pub device_address: [u8; 8],
    /// 16-bitshort device address allocated by the coordinator on successful
    /// association
    pub assoc_short_address: [u8; 2],
    /// status of the association attempt
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

impl MacCommand for AssociateResponse {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeAssociateRes;
    const SIZE: usize = 24;
}

/// MLME ORPHAN Response used to respond to the MLME ORPHAN Indication
#[repr(C)]
pub struct OrphanResponse {
    /// extended address of the orphaned device
    pub orphan_address: [u8; 8],
    /// short address allocated to the orphaned device
    pub short_address: [u8; 2],
    /// if the orphaned device is associated with coordinator or not
    pub associated_member: bool,
    /// security level to be used
    pub security_level: u8,
    /// the originator of the key to be used
    pub key_source: [u8; 8],
    /// the mode used to identify the key to be used
    pub key_id_mode: u8,
    /// the index of the key to be used
    pub key_index: u8,
}

impl MacCommand for OrphanResponse {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeOrphanRes;
    const SIZE: usize = 24;
}
