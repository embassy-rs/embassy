/// A region in flash used by the bootloader.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Partition {
    /// Start of the flash region.
    pub from: usize,
    /// End of the flash region.
    pub to: usize,
}

impl Partition {
    /// Create a new partition with the provided range
    pub const fn new(from: usize, to: usize) -> Self {
        Self { from, to }
    }

    /// Return the length of the partition
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        self.to - self.from
    }
}
