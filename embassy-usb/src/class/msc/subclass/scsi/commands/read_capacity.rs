use super::control::Control;
use crate::packed::BE;
use crate::packed_struct;

packed_struct! {
    pub struct ReadCapacity10Command<10> {
        #[offset = 0, size = 8]
        op_code: u8,
        #[offset = 2*8, size = 32]
        lba: BE<u32>,
        #[offset = 9*8, size = 8]
        control: Control<[u8; Control::SIZE]>,
    }
}

impl ReadCapacity10Command<[u8; ReadCapacity10Command::SIZE]> {
    pub const OPCODE: u8 = 0x25;
}

packed_struct! {
    pub struct ReadCapacity10Response<8> {
        #[offset = 0*8, size = 32]
        max_lba: BE<u32>,
        #[offset = 4*8, size = 32]
        block_size: BE<u32>,
    }
}
