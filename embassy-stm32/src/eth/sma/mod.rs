//! Station Management Agent (also known as MDIO or SMI).

#![macro_use]

#[cfg_attr(eth_v2, path = "v2.rs")]
#[cfg_attr(any(eth_v1a, eth_v1b, eth_v1c), path = "v1.rs")]
mod _version;

use embassy_hal_internal::PeripheralType;
use stm32_metapac::common::{RW, Reg};

pub use self::_version::*;

/// Station Management Interface (SMI).
pub trait StationManagement {
    /// Read a register over SMI.
    fn smi_read(&mut self, phy_addr: u8, reg: u8) -> u16;
    /// Write a register over SMI.
    fn smi_write(&mut self, phy_addr: u8, reg: u8, val: u16);
}

trait SealedInstance {
    fn regs() -> (Reg<AddressRegister, RW>, Reg<DataRegister, RW>);
}

/// MDIO instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + Send + 'static {}

impl SealedInstance for crate::peripherals::ETH_SMA {
    fn regs() -> (Reg<AddressRegister, RW>, Reg<DataRegister, RW>) {
        let mac = crate::pac::ETH.ethernet_mac();

        #[cfg(any(eth_v1a, eth_v1b, eth_v1c))]
        return (mac.macmiiar(), mac.macmiidr());

        #[cfg(eth_v2)]
        return (mac.macmdioar(), mac.macmdiodr());
    }
}

impl Instance for crate::peripherals::ETH_SMA {}
