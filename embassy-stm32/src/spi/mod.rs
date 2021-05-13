#![macro_use]

#[cfg_attr(feature = "_spi_v1", path = "spi_v1.rs")]
#[cfg_attr(feature = "_spi_v2", path = "spi_v2.rs")]
mod spi;

pub use spi::*;

// TODO move upwards in the tree
pub enum ByteOrder {
    LsbFirst,
    MsbFirst,
}

#[derive(Copy, Clone, PartialOrd, PartialEq)]
enum WordSize {
    EightBit,
    SixteenBit,
}

#[non_exhaustive]
pub struct Config {
    pub mode: Mode,
    pub byte_order: ByteOrder,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: MODE_0,
            byte_order: ByteOrder::MsbFirst,
        }
    }
}
