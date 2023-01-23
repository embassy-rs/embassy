// use super::control::Control;

// #[bitfield(bytes = 6)]
// pub struct InquiryCommand {
//     /// Always 0x12
//     pub op_code: B8,
//     #[skip]
//     __: B7,
//     /// If set, return vital data related to the page_code field
//     pub enable_vital_product_data: B1,
//     /// What kind of vital data to return
//     pub page_code: B8,
//     /// Amount of bytes allocation for data-in transfer
//     pub allocation_length: B16,
//     /// Control byte
//     pub control: Control,
// }

use super::control::Control;
use crate::class::msc::subclass::scsi::enums::{PeripheralDeviceType, PeripheralQualifier};
use crate::packed::PackedField;
use crate::packed_struct;

packed_struct! {
    pub struct InquiryCommand<6> {
        #[offset = 0, size = 8]
        op_code: u8,
        #[offset = 1*8, size = 1]
        enable_vital_product_data: bool,
        #[offset = 2*8, size = 8]
        page_code: u8,
        #[offset = 3*8, size = 16]
        allocation_length: u16,
        #[offset = 5*8, size = 8]
        control: Control<T>,
    }
}

impl InquiryCommand<[u8; InquiryCommand::SIZE]> {
    pub const OPCODE: u8 = 0x12;
}

// impl<T: AsRef<[u8]>> defmt::Format for InquiryCommand<T> {
//     fn format(&self, fmt: defmt::Formatter) {
//         fmt.
//     }
// }

packed_struct! {
    /// Inquiry response can contain many extensions. We support only the minimum required 36 bytes.
    pub struct InquiryResponse<36> {
        #[offset = 0, size = 5]
        peripheral_qualifier: PeripheralQualifier,
        #[offset = 5, size = 3]
        peripheral_device_type: PeripheralDeviceType,
    }
}
