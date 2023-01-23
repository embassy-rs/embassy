// #[bitfield(bytes = 1)]
// #[derive(BitfieldSpecifier)]
// pub struct Control {
//     pub vendor_specific: B2,
//     #[skip]
//     __: B3,
//     pub naca: B1,
//     #[skip]
//     __: B2,
// }

use crate::packed_struct;

packed_struct! {
    pub struct Control<1> {
        #[offset = 0, size = 2]
        vendor_specific: u8,
        #[offset = 5, size = 1]
        naca: bool,
    }
}
