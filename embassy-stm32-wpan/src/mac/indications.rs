use core::slice;

use super::consts::MAX_PENDING_ADDRESS;
use super::event::ParseableMacEvent;
use super::typedefs::{
    AddressMode, Capabilities, DisassociationReason, KeyIdMode, MacAddress, MacChannel, MacStatus, PanDescriptor,
    PanId, SecurityLevel,
};

/// MLME ASSOCIATE Indication which will be used by the MAC
/// to indicate the reception of an association request command
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AssociateIndication {
    /// Extended address of the device requesting association
    pub device_address: [u8; 8],
    /// Operational capabilities of the device requesting association
    pub capability_information: Capabilities,
    /// Security level purportedly used by the received MAC command frame
    pub security_level: SecurityLevel,
    /// The mode used to identify the key used by the originator of frame
    pub key_id_mode: KeyIdMode,
    /// Index of the key used by the originator of the received frame
    pub key_index: u8,
    /// The originator of the key used by the originator of the received frame
    pub key_source: [u8; 8],
}

impl ParseableMacEvent for AssociateIndication {}

/// MLME DISASSOCIATE indication which will be used to send
/// disassociation indication to the application.
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DisassociateIndication {
    /// Extended address of the device requesting association
    pub device_address: [u8; 8],
    /// The reason for the disassociation
    pub disassociation_reason: DisassociationReason,
    /// The security level to be used
    pub security_level: SecurityLevel,
    /// The mode used to identify the key to be used
    pub key_id_mode: KeyIdMode,
    /// The index of the key to be used
    pub key_index: u8,
    /// The originator of the key to be used
    pub key_source: [u8; 8],
}

impl ParseableMacEvent for DisassociateIndication {}

/// MLME BEACON NOTIIFY Indication which is used to send parameters contained
/// within a beacon frame received by the MAC to the application
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BeaconNotifyIndication {
    /// he set of octets comprising the beacon payload to be transferred
    /// from the MAC sublayer entity to the next higher layer
    pub sdu_ptr: *const u8,
    /// The PAN Descriptor for the received beacon
    pub pan_descriptor: PanDescriptor,
    /// The list of addresses of the devices
    pub addr_list: [MacAddress; MAX_PENDING_ADDRESS],
    /// Beacon Sequence Number
    pub bsn: u8,
    /// The beacon pending address specification
    pub pend_addr_spec: u8,
    /// Number of octets contained in the beacon payload of the beacon frame
    pub sdu_length: u8,
}

impl ParseableMacEvent for BeaconNotifyIndication {}

/// MLME COMM STATUS Indication which is used by the MAC to indicate a communications status
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CommStatusIndication {
    /// The 16-bit PAN identifier of the device from which the frame
    /// was received or to which the frame was being sent
    pub pan_id: PanId,
    /// Source addressing mode
    pub src_addr_mode: AddressMode,
    /// Destination addressing mode
    pub dst_addr_mode: AddressMode,
    /// Source address
    pub src_address: MacAddress,
    /// Destination address
    pub dst_address: MacAddress,
    /// The communications status
    pub status: MacStatus,
    /// Security level to be used
    pub security_level: SecurityLevel,
    /// Mode used to identify the key to be used
    pub key_id_mode: KeyIdMode,
    /// Index of the key to be used
    pub key_index: u8,
    /// Originator of the key to be used
    pub key_source: [u8; 8],
}

impl ParseableMacEvent for CommStatusIndication {}

/// MLME GTS Indication indicates that a GTS has been allocated or that a
/// previously allocated GTS has been deallocated
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GtsIndication {
    /// The short address of the device that has been allocated or deallocated a GTS
    pub device_address: [u8; 2],
    /// The characteristics of the GTS
    pub gts_characteristics: u8,
    /// Security level to be used
    pub security_level: SecurityLevel,
    /// Mode used to identify the key to be used
    pub key_id_mode: KeyIdMode,
    /// Index of the key to be used
    pub key_index: u8,
    /// byte stuffing to keep 32 bit alignment
    a_stuffing: [u8; 2],
    /// Originator of the key to be used
    pub key_source: [u8; 8],
}

impl ParseableMacEvent for GtsIndication {}

/// MLME ORPHAN Indication which is used by the coordinator to notify the
/// application of the presence of an orphaned device
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OrphanIndication {
    /// Extended address of the orphaned device
    pub orphan_address: [u8; 8],
    /// Originator of the key used by the originator of the received frame
    pub key_source: [u8; 8],
    /// Security level purportedly used by the received MAC command frame
    pub security_level: SecurityLevel,
    /// Mode used to identify the key used by originator of received frame
    pub key_id_mode: KeyIdMode,
    /// Index of the key used by the originator of the received frame
    pub key_index: u8,
    /// byte stuffing to keep 32 bit alignment
    a_stuffing: [u8; 1],
}

impl ParseableMacEvent for OrphanIndication {}

/// MLME SYNC LOSS Indication which is used by the MAC to indicate the loss
/// of synchronization with the coordinator
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SyncLossIndication {
    /// The PAN identifier with which the device lost synchronization or to which it was realigned
    pub pan_id: PanId,
    /// The reason that synchronization was lost
    pub loss_reason: u8,
    /// The logical channel on which the device lost synchronization or to whi
    pub channel_number: MacChannel,
    /// The channel page on which the device lost synchronization or to which
    pub channel_page: u8,
    /// The security level used by the received MAC frame
    pub security_level: SecurityLevel,
    /// Mode used to identify the key used by originator of received frame
    pub key_id_mode: KeyIdMode,
    /// Index of the key used by the originator of the received frame
    pub key_index: u8,
    /// Originator of the key used by the originator of the received frame
    pub key_source: [u8; 8],
}

impl ParseableMacEvent for SyncLossIndication {}

/// MLME DPS Indication which indicates the expiration of the DPSIndexDuration
///  and the resetting of the DPS values in the PHY
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DpsIndication {
    /// byte stuffing to keep 32 bit alignment
    a_stuffing: [u8; 4],
}

impl ParseableMacEvent for DpsIndication {}

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DataIndication {
    /// Pointer to the set of octets forming the MSDU being indicated  
    pub msdu_ptr: *const u8,
    /// Source addressing mode used  
    pub src_addr_mode: AddressMode,
    /// Source PAN ID   
    pub src_pan_id: PanId,
    /// Source address  
    pub src_address: MacAddress,
    /// Destination addressing mode used  
    pub dst_addr_mode: AddressMode,
    /// Destination PAN ID   
    pub dst_pan_id: PanId,
    /// Destination address  
    pub dst_address: MacAddress,
    /// The number of octets contained in the MSDU being indicated  
    pub msdu_length: u8,
    /// QI value measured during reception of the MPDU
    pub mpdu_link_quality: u8,
    /// The data sequence number of the received data frame  
    pub dsn: u8,
    /// The time, in symbols, at which the data were received  
    pub time_stamp: [u8; 4],
    /// The security level purportedly used by the received data frame  
    security_level: SecurityLevel,
    /// Mode used to identify the key used by originator of received frame  
    key_id_mode: KeyIdMode,
    /// The originator of the key  
    pub key_source: [u8; 8],
    /// The index of the key  
    pub key_index: u8,
    /// he pulse repetition value of the received PPDU
    pub uwbprf: u8,
    /// The preamble symbol repetitions of the UWB PHY frame  
    pub uwn_preamble_symbol_repetitions: u8,
    /// Indicates the data rate
    pub datrate: u8,
    /// time units corresponding to an RMARKER at the antenna at the end of a ranging exchange,  
    pub ranging_received: u8,
    pub ranging_counter_start: u32,
    pub ranging_counter_stop: u32,
    /// ime units in a message exchange over which the tracking offset was measured
    pub ranging_tracking_interval: u32,
    /// time units slipped or advanced by the radio tracking system  
    pub ranging_offset: u32,
    /// The FoM characterizing the ranging measurement
    pub ranging_fom: u8,
    /// The Received Signal Strength Indicator measured
    pub rssi: u8,
}

impl ParseableMacEvent for DataIndication {}

impl DataIndication {
    pub fn payload<'a>(&'a self) -> &'a mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.msdu_ptr as *mut _, self.msdu_length as usize) }
    }
}

/// MLME POLL Indication which will be used for indicating the Data Request
/// reception to upper layer as defined in Zigbee r22 - D.8.2
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PollIndication {
    /// addressing mode used
    pub addr_mode: AddressMode,
    /// Poll requester address
    pub request_address: MacAddress,
}

impl ParseableMacEvent for PollIndication {}
