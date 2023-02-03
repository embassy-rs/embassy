use super::control::Control;
use crate::packed::BE;
use crate::packed_struct;

packed_struct! {
    pub struct ReadFormatCapacitiesCommand<10> {
        #[offset = 0, size = 8]
        op_code: u8,
        #[offset = 1*8+5, size = 3]
        lun: u8,
        #[offset = 7*8, size = 16]
        allocation_length: BE<u16>,
        #[offset = 9*8, size = 8]
        control: Control<[u8; Control::SIZE]>,
    }
}

impl ReadFormatCapacitiesCommand<[u8; ReadFormatCapacitiesCommand::SIZE]> {
    pub const OPCODE: u8 = 0x23;
}

packed_struct! {
    pub struct ReadFormatCapacitiesResponse<12> {
        /// The Capacity List Length specifies the length in bytes of the Capacity Descriptors that follow. Each Capacity
        /// Descriptor is eight bytes in length, making the Capacity List Length equal to eight times the number of descriptors.
        /// Values of n * 8 are valid, where 0 < n < 32
        #[offset = 3*8, size = 8]
        capacity_list_length: u8,
        #[offset = 4*8, size = 32]
        max_lba: BE<u32>,
        #[offset = 8*8, size = 2]
        descriptor_type: BE<u32>,
        // TODO should be 24 bits
        #[offset = 8*8, size = 32]
        block_size: BE<u32>,
    }
}
