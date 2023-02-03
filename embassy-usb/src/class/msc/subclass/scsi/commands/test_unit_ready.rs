use super::control::Control;
use crate::packed_struct;

packed_struct! {
    pub struct TestUnitReadyCommand<6> {
        #[offset = 0, size = 8]
        op_code: u8,
        #[offset = 5*8, size = 8]
        control: Control<[u8; Control::SIZE]>,
    }
}

impl TestUnitReadyCommand<[u8; TestUnitReadyCommand::SIZE]> {
    pub const OPCODE: u8 = 0x00;
}
