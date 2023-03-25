pub trait FlashRegion {
    const BASE: usize;
    const SIZE: usize;
    const ERASE_SIZE: usize;
    const WRITE_SIZE: usize;
    const ERASE_VALUE: u8;
}
