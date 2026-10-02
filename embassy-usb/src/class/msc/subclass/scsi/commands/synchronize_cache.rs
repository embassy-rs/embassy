use super::control::Control;
use crate::packed::BE;
use crate::packed_struct;

packed_struct! {
    pub struct SynchronizeCache10Command<10> {
        #[offset = 0, size = 8]
        op_code: u8,
        /// An IMMED bit set to one requests GOOD status as soon as the CDB is validated, before the flush completes.
        #[offset = 1*8+1, size = 1]
        immediate: bool,
        #[offset = 2*8+0, size = 32]
        lba: BE<u32>,
        #[offset = 6*8+0, size = 5]
        group_number: u8,
        /// Zero means all blocks from `lba` to the end of the medium.
        #[offset = 7*8+0, size = 16]
        number_of_blocks: BE<u16>,
        #[offset = 9*8+0, size = 8]
        control: Control<[u8; Control::SIZE]>,
    }
}

impl SynchronizeCache10Command<[u8; SynchronizeCache10Command::SIZE]> {
    pub const OPCODE: u8 = 0x35;
}
