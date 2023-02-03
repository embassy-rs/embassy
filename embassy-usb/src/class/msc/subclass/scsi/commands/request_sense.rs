use super::control::Control;
use crate::class::msc::subclass::scsi::enums::{ResponseCode, SenseKey};
use crate::packed_struct;

packed_struct! {
    pub struct RequestSenseCommand<6> {
        #[offset = 0, size = 8]
        op_code: u8,
        #[offset = 1*8+5, size = 3]
        lun: u8,
        #[offset = 4*8+0, size = 8]
        allocation_length: u8,
        #[offset = 5*8, size = 8]
        control: Control<[u8; Control::SIZE]>,
    }
}

impl RequestSenseCommand<[u8; RequestSenseCommand::SIZE]> {
    pub const OPCODE: u8 = 0x03;
}

packed_struct! {
    pub struct RequestSenseResponse<8> {
        #[offset = 0, size = 7]
        response_code: ResponseCode,
        #[offset = 1*8+0, size = 4]
        sense_key: SenseKey,
        #[offset = 2*8+0, size = 8]
        additional_sense_code: u8,
        #[offset = 3*8+0, size = 8]
        additional_sense_code_qualifier: u8,
        #[offset = 7*8+0, size = 8]
        additional_sense_length: u8,
    }
}
