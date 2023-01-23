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
