pub trait FlashRegion {
    const SETTINGS: FlashRegionSettings;
}

pub struct FlashRegionSettings {
    pub base: usize,
    pub size: usize,
    pub erase_size: usize,
    pub write_size: usize,
    pub erase_value: u8,
}
