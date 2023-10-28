#![macro_use]

use stm32_metapac::i2c::vals;

use crate::interrupt;

#[cfg_attr(i2c_v1, path = "v1.rs")]
#[cfg_attr(i2c_v2, path = "v2.rs")]
mod _version;
pub use _version::*;

use crate::peripherals;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Bus,
    Arbitration, // in case of master mode: lost arbitration to another master
    Nack,
    Timeout,
    Crc,
    Overrun,
    ZeroLengthTransfer,
    BufferSize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(usize)]
pub enum AddressType {
    Address1 = 0,
    Address2,
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Address2Mask {
    NOMASK,
    MASK1,
    MASK2,
    MASK3,
    MASK4,
    MASK5,
    MASK6,
    MASK7,
}
impl Address2Mask {
    #[inline(always)]
    pub const fn to_vals_impl(self) -> vals::Oamsk {
        match self {
            Address2Mask::NOMASK => vals::Oamsk::NOMASK,
            Address2Mask::MASK1 => vals::Oamsk::MASK1,
            Address2Mask::MASK2 => vals::Oamsk::MASK2,
            Address2Mask::MASK3 => vals::Oamsk::MASK3,
            Address2Mask::MASK4 => vals::Oamsk::MASK4,
            Address2Mask::MASK5 => vals::Oamsk::MASK5,
            Address2Mask::MASK6 => vals::Oamsk::MASK6,
            Address2Mask::MASK7 => vals::Oamsk::MASK7,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Dir {
    WRITE,
    READ,
}
impl Dir {
    #[inline(always)]
    pub const fn to_vals_impl(self) -> vals::Dir {
        match self {
            Dir::READ => vals::Dir::READ,
            Dir::WRITE => vals::Dir::WRITE,
        }
    }
}

pub(crate) mod sealed {
    use super::*;
    pub trait Instance: crate::rcc::RccPeripheral {
        fn regs() -> crate::pac::i2c::I2c;
        fn state() -> &'static State;
    }
}

pub trait Instance: sealed::Instance + 'static {
    type Interrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(SclPin, Instance);
pin_trait!(SdaPin, Instance);
dma_trait!(RxDma, Instance);
dma_trait!(TxDma, Instance);

foreach_interrupt!(
    ($inst:ident, i2c, $block:ident, EV, $irq:ident) => {
        impl sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::i2c::I2c {
                crate::pac::$inst
            }

            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }

        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
);
