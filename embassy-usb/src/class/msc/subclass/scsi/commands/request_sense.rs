use super::control::Control;
use crate::class::msc::subclass::scsi::enums::{ResponseCode, SenseKey};
use crate::packed::BE;
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
    pub struct RequestSenseResponse<20> {
        #[offset = 0, size = 7]
        response_code: ResponseCode,
        #[offset = 1*8+0, size = 8]
        segment_number: u8,
        #[offset = 2*8+0, size = 4]
        sense_key: SenseKey,
        #[offset = 3*8+0, size = 4*8]
        information: BE<u32>,
        #[offset = 7*8+0, size = 8]
        additional_sense_length: u8,
        #[offset = 8*8+0, size = 4*8]
        command_specific_information: BE<u32>,
        #[offset = 12*8+0, size = 8]
        additional_sense_code: u8,
        #[offset = 13*8+0, size = 8]
        additional_sense_code_qualifier: u8,
        #[offset = 14*8+0, size = 8]
        field_replaceable_unit_code: u8,
        #[offset = 15*8+0, size = 3]
        bit_pointer: u8,
        #[offset = 15*8+3, size = 1]
        bpv: bool,
        /// Command/Data
        #[offset = 15*8+6, size = 1]
        cd: bool,
        /// Sense Key Specific Valid
        #[offset = 15*8+7, size = 1]
        sksv: bool,
        #[offset = 16*8+0, size = 2*8]
        field_pointer: BE<u16>,
    }
}
