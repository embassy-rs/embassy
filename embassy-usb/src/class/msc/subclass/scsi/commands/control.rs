use crate::packed_struct;

packed_struct! {
    pub struct Control<1> {
        #[offset = 0, size = 2]
        vendor_specific: u8,
        #[offset = 5, size = 1]
        naca: bool,
    }
}
