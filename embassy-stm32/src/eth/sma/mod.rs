//! Station Management Agent (also known as MDIO or SMI).

#![macro_use]

use embassy_hal_internal::PeripheralType;
#[cfg(eth_v2)]
pub(crate) use regs::{Macmdioar as AddressRegister, Macmdiodr as DataRegister};
#[cfg(any(eth_v1a, eth_v1b, eth_v1c))]
pub(crate) use regs::{Macmiiar as AddressRegister, Macmiidr as DataRegister};
use stm32_metapac::common::{RW, Reg};
use stm32_metapac::eth::regs;

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
