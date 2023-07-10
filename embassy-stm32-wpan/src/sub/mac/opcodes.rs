const ST_VENDOR_OGF: u16 = 0x3F;
const MAC_802_15_4_CMD_OPCODE_OFFSET: u16 = 0x280;

const fn opcode(ocf: u16) -> isize {
    ((ST_VENDOR_OGF << 9) | (MAC_802_15_4_CMD_OPCODE_OFFSET + ocf)) as isize
}

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
