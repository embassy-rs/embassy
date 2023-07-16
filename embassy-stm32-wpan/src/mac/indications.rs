use super::consts::MAX_PENDING_ADDRESS;
use super::event::ParseableMacEvent;
use super::typedefs::{
    AddressMode, Capabilities, DisassociationReason, KeyIdMode, MacAddress, MacChannel, MacStatus, PanDescriptor,
    PanId, SecurityLevel,
};

/// MLME ASSOCIATE Indication which will be used by the MAC
/// to indicate the reception of an association request command
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

impl ParseableMacEvent for AssociateIndication {
    const SIZE: usize = 20;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            device_address: [buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]],
            capability_information: Capabilities::from_bits(buf[8]).ok_or(())?,
            security_level: SecurityLevel::try_from(buf[9])?,
            key_id_mode: KeyIdMode::try_from(buf[10])?,
            key_index: buf[11],
            key_source: [buf[12], buf[13], buf[14], buf[15], buf[16], buf[17], buf[18], buf[19]],
        })
    }
}

/// MLME DISASSOCIATE indication which will be used to send
/// disassociation indication to the application.
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

impl ParseableMacEvent for DisassociateIndication {
    const SIZE: usize = 20;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            device_address: [buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]],
            disassociation_reason: DisassociationReason::try_from(buf[8])?,
            security_level: SecurityLevel::try_from(buf[9])?,
            key_id_mode: KeyIdMode::try_from(buf[10])?,
            key_index: buf[11],
            key_source: [buf[12], buf[13], buf[14], buf[15], buf[16], buf[17], buf[18], buf[19]],
        })
    }
}

/// MLME BEACON NOTIIFY Indication which is used to send parameters contained
/// within a beacon frame received by the MAC to the application
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

impl ParseableMacEvent for BeaconNotifyIndication {
    const SIZE: usize = 88;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        // TODO: this is unchecked

        Self::validate(buf)?;

        let addr_list = [
            MacAddress::try_from(&buf[26..34])?,
            MacAddress::try_from(&buf[34..42])?,
            MacAddress::try_from(&buf[42..50])?,
            MacAddress::try_from(&buf[50..58])?,
            MacAddress::try_from(&buf[58..66])?,
            MacAddress::try_from(&buf[66..74])?,
            MacAddress::try_from(&buf[74..82])?,
        ];

        Ok(Self {
            sdu_ptr: u32::from_le_bytes(buf[0..4].try_into().unwrap()) as *const u8,
            pan_descriptor: PanDescriptor::try_from(&buf[4..26])?,
            addr_list,
            bsn: buf[82],
            pend_addr_spec: buf[83],
            sdu_length: buf[83],
        })
    }
}

/// MLME COMM STATUS Indication which is used by the MAC to indicate a communications status
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

impl ParseableMacEvent for CommStatusIndication {
    const SIZE: usize = 32;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        let src_addr_mode = AddressMode::try_from(buf[2])?;
        let dst_addr_mode = AddressMode::try_from(buf[3])?;

        let src_address = match src_addr_mode {
            AddressMode::NoAddress => MacAddress { short: [0, 0] },
            AddressMode::Reserved => MacAddress { short: [0, 0] },
            AddressMode::Short => MacAddress {
                short: [buf[4], buf[5]],
            },
            AddressMode::Extended => MacAddress {
                extended: [buf[4], buf[5], buf[6], buf[7], buf[8], buf[9], buf[10], buf[11]],
            },
        };

        let dst_address = match dst_addr_mode {
            AddressMode::NoAddress => MacAddress { short: [0, 0] },
            AddressMode::Reserved => MacAddress { short: [0, 0] },
            AddressMode::Short => MacAddress {
                short: [buf[12], buf[13]],
            },
            AddressMode::Extended => MacAddress {
                extended: [buf[12], buf[13], buf[14], buf[15], buf[16], buf[17], buf[18], buf[19]],
            },
        };

        Ok(Self {
            pan_id: PanId([buf[0], buf[1]]),
            src_addr_mode,
            dst_addr_mode,
            src_address,
            dst_address,
            status: MacStatus::try_from(buf[20])?,
            security_level: SecurityLevel::try_from(buf[21])?,
            key_id_mode: KeyIdMode::try_from(buf[22])?,
            key_index: buf[23],
            key_source: [buf[24], buf[25], buf[26], buf[27], buf[28], buf[29], buf[30], buf[31]],
        })
    }
}

/// MLME GTS Indication indicates that a GTS has been allocated or that a
/// previously allocated GTS has been deallocated
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
    /// Originator of the key to be used
    pub key_source: [u8; 8],
}

impl ParseableMacEvent for GtsIndication {
    const SIZE: usize = 16;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            device_address: [buf[0], buf[1]],
            gts_characteristics: buf[2],
            security_level: SecurityLevel::try_from(buf[3])?,
            key_id_mode: KeyIdMode::try_from(buf[4])?,
            key_index: buf[5],
            // 2 byte stuffing
            key_source: [buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15]],
        })
    }
}

/// MLME ORPHAN Indication which is used by the coordinator to notify the
/// application of the presence of an orphaned device
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
}

impl ParseableMacEvent for OrphanIndication {
    const SIZE: usize = 20;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            orphan_address: [buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]],
            key_source: [buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15]],
            security_level: SecurityLevel::try_from(buf[16])?,
            key_id_mode: KeyIdMode::try_from(buf[17])?,
            key_index: buf[18],
            // 1 byte stuffing
        })
    }
}

/// MLME SYNC LOSS Indication which is used by the MAC to indicate the loss
/// of synchronization with the coordinator
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

impl ParseableMacEvent for SyncLossIndication {
    const SIZE: usize = 16;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self {
            pan_id: PanId([buf[0], buf[1]]),
            loss_reason: buf[2],
            channel_number: MacChannel::try_from(buf[3])?,
            channel_page: buf[4],
            security_level: SecurityLevel::try_from(buf[5])?,
            key_id_mode: KeyIdMode::try_from(buf[6])?,
            key_index: buf[7],
            key_source: [buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15]],
        })
    }
}

/// MLME DPS Indication which indicates the expiration of the DPSIndexDuration
///  and the resetting of the DPS values in the PHY
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DpsIndication;

impl ParseableMacEvent for DpsIndication {
    const SIZE: usize = 4;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        Ok(Self)
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C, align(8))]
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
    pub security_level: SecurityLevel,
    /// Mode used to identify the key used by originator of received frame  
    pub key_id_mode: KeyIdMode,
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

impl ParseableMacEvent for DataIndication {
    const SIZE: usize = 68;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        let src_addr_mode = AddressMode::try_from(buf[4])?;
        let src_address = match src_addr_mode {
            AddressMode::NoAddress => MacAddress { short: [0, 0] },
            AddressMode::Reserved => MacAddress { short: [0, 0] },
            AddressMode::Short => MacAddress {
                short: [buf[7], buf[8]],
            },
            AddressMode::Extended => MacAddress {
                extended: [buf[7], buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14]],
            },
        };

        let dst_addr_mode = AddressMode::try_from(buf[15])?;
        let dst_address = match dst_addr_mode {
            AddressMode::NoAddress => MacAddress { short: [0, 0] },
            AddressMode::Reserved => MacAddress { short: [0, 0] },
            AddressMode::Short => MacAddress {
                short: [buf[18], buf[19]],
            },
            AddressMode::Extended => MacAddress {
                extended: [buf[18], buf[19], buf[20], buf[21], buf[22], buf[23], buf[24], buf[25]],
            },
        };

        Ok(Self {
            msdu_ptr: u32::from_le_bytes(buf[0..4].try_into().unwrap()) as *const u8,
            src_addr_mode,
            src_pan_id: PanId([buf[5], buf[6]]),
            src_address,
            dst_addr_mode,
            dst_pan_id: PanId([buf[16], buf[17]]),
            dst_address,
            msdu_length: buf[26],
            mpdu_link_quality: buf[27],
            dsn: buf[28],
            time_stamp: [buf[29], buf[30], buf[31], buf[32]],
            security_level: SecurityLevel::try_from(buf[33]).unwrap_or(SecurityLevel::Unsecure), // TODO: this is totaly wrong, but I'm too smol brain to fix it
            key_id_mode: KeyIdMode::try_from(buf[34]).unwrap_or(KeyIdMode::Implicite), // TODO: this is totaly wrong, but I'm too smol brain to fix it
            key_source: [buf[35], buf[36], buf[37], buf[38], buf[39], buf[40], buf[41], buf[42]],
            key_index: buf[43],
            uwbprf: buf[44],
            uwn_preamble_symbol_repetitions: buf[45],
            datrate: buf[46],
            ranging_received: buf[47],
            ranging_counter_start: u32::from_le_bytes(buf[48..52].try_into().unwrap()),
            ranging_counter_stop: u32::from_le_bytes(buf[52..56].try_into().unwrap()),
            ranging_tracking_interval: u32::from_le_bytes(buf[56..60].try_into().unwrap()),
            ranging_offset: u32::from_le_bytes(buf[60..64].try_into().unwrap()),
            ranging_fom: buf[65],
            rssi: buf[66],
        })
    }
}

/// MLME POLL Indication which will be used for indicating the Data Request
/// reception to upper layer as defined in Zigbee r22 - D.8.2
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PollIndication {
    /// addressing mode used
    pub addr_mode: AddressMode,
    /// Poll requester address
    pub request_address: MacAddress,
}

impl ParseableMacEvent for PollIndication {
    const SIZE: usize = 9;

    fn try_parse(buf: &[u8]) -> Result<Self, ()> {
        Self::validate(buf)?;

        let addr_mode = AddressMode::try_from(buf[0])?;
        let request_address = match addr_mode {
            AddressMode::NoAddress => MacAddress { short: [0, 0] },
            AddressMode::Reserved => MacAddress { short: [0, 0] },
            AddressMode::Short => MacAddress {
                short: [buf[1], buf[2]],
            },
            AddressMode::Extended => MacAddress {
                extended: [buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8]],
            },
        };

        Ok(Self {
            addr_mode,
            request_address,
        })
    }
}
