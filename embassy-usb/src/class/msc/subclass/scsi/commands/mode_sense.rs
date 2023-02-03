use super::control::Control;
use crate::class::msc::subclass::scsi::enums::PageControl;
use crate::packed_struct;

packed_struct! {
    pub struct ModeSense6Command<6> {
        #[offset = 0, size = 8]
        op_code: u8,
        /// A disable block descriptors (DBD) bit set to zero specifies that the device server may return zero or more block descriptors in the returned MODE SENSE data.
        ///
        /// A DBD bit set to one specifies that the device server shall not return any block descriptors in the returned MODE SENSE data.
        #[offset = 1*8+3, size = 1]
        disable_block_descriptors: bool,
        /// The PAGE CODE and SUBPAGE CODE fields specify which mode pages and subpages to return
        #[offset = 2*8+0, size = 6]
        page_code: u8,
        #[offset = 2*8+6, size = 2]
        page_control: PageControl,
        #[offset = 3*8+0, size = 8]
        subpage_code: u8,
        #[offset = 4*8+0, size = 8]
        allocation_length: u8,
        #[offset = 5*8, size = 8]
        control: Control<[u8; Control::SIZE]>,
    }
}

impl ModeSense6Command<[u8; ModeSense6Command::SIZE]> {
    pub const OPCODE: u8 = 0x1A;
}
