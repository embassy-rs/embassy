const ST_VENDOR_OGF: u16 = 0x3F;
const MAC_802_15_4_CMD_OPCODE_OFFSET: u16 = 0x280;

const fn opcode(ocf: u16) -> isize {
    ((ST_VENDOR_OGF << 9) | (MAC_802_15_4_CMD_OPCODE_OFFSET + ocf)) as isize
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OpcodeM4ToM0 {
    MlmeAssociateReq = opcode(0x00),
    MlmeAssociateRes = opcode(0x01),
    MlmeDisassociateReq = opcode(0x02),
    MlmeGetReq = opcode(0x03),
    MlmeGtsReq = opcode(0x04),
    MlmeOrphanRes = opcode(0x05),
    MlmeResetReq = opcode(0x06),
    MlmeRxEnableReq = opcode(0x07),
    MlmeScanReq = opcode(0x08),
    MlmeSetReq = opcode(0x09),
    MlmeStartReq = opcode(0x0A),
    MlmeSyncReq = opcode(0x0B),
    MlmePollReq = opcode(0x0C),
    MlmeDpsReq = opcode(0x0D),
    MlmeSoundingReq = opcode(0x0E),
    MlmeCalibrateReq = opcode(0x0F),
    McpsDataReq = opcode(0x10),
    McpsPurgeReq = opcode(0x11),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OpcodeM0ToM4 {
    MlmeAssociateCnf = 0x00,
    MlmeDisassociateCnf,
    MlmeGetCnf,
    MlmeGtsCnf,
    MlmeResetCnf,
    MlmeRxEnableCnf,
    MlmeScanCnf,
    MlmeSetCnf,
    MlmeStartCnf,
    MlmePollCnf,
    MlmeDpsCnf,
    MlmeSoundingCnf,
    MlmeCalibrateCnf,
    McpsDataCnf,
    McpsPurgeCnf,
    MlmeAssociateInd,
    MlmeDisassociateInd,
    MlmeBeaconNotifyInd,
    MlmeCommStatusInd,
    MlmeGtsInd,
    MlmeOrphanInd,
    MlmeSyncLossInd,
    MlmeDpsInd,
    McpsDataInd,
    MlmePollInd,
}

impl TryFrom<u16> for OpcodeM0ToM4 {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::MlmeAssociateCnf),
            1 => Ok(Self::MlmeDisassociateCnf),
            2 => Ok(Self::MlmeGetCnf),
            3 => Ok(Self::MlmeGtsCnf),
            4 => Ok(Self::MlmeResetCnf),
            5 => Ok(Self::MlmeRxEnableCnf),
            6 => Ok(Self::MlmeScanCnf),
            7 => Ok(Self::MlmeSetCnf),
            8 => Ok(Self::MlmeStartCnf),
            9 => Ok(Self::MlmePollCnf),
            10 => Ok(Self::MlmeDpsCnf),
            11 => Ok(Self::MlmeSoundingCnf),
            12 => Ok(Self::MlmeCalibrateCnf),
            13 => Ok(Self::McpsDataCnf),
            14 => Ok(Self::McpsPurgeCnf),
            15 => Ok(Self::MlmeAssociateInd),
            16 => Ok(Self::MlmeDisassociateInd),
            17 => Ok(Self::MlmeBeaconNotifyInd),
            18 => Ok(Self::MlmeCommStatusInd),
            19 => Ok(Self::MlmeGtsInd),
            20 => Ok(Self::MlmeOrphanInd),
            21 => Ok(Self::MlmeSyncLossInd),
            22 => Ok(Self::MlmeDpsInd),
            23 => Ok(Self::McpsDataInd),
            24 => Ok(Self::MlmePollInd),
            _ => Err(()),
        }
    }
}
