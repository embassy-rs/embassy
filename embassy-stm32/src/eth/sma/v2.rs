use embassy_hal_internal::Peri;
pub(crate) use regs::{Macmdioar as AddressRegister, Macmdiodr as DataRegister};
use stm32_metapac::eth::regs;

use super::{Instance, StationManagement};
use crate::eth::{MDCPin, MDIOPin};
use crate::gpio::{AfType, Flex, OutputType, Speed};

/// Station Management Agent.
///
/// This peripheral is used for SMI reads and writes to the connected
/// ethernet PHY/device(s).
pub struct Sma<'d, T: Instance> {
    _peri: Peri<'d, T>,
    _pins: [Flex<'d>; 2],
    clock_range: u8,
}

impl<'d, T: Instance> Sma<'d, T> {
    /// Create a new instance of this peripheral.
    pub fn new(peri: Peri<'d, T>, mdio: Peri<'d, impl MDIOPin<T>>, mdc: Peri<'d, impl MDCPin<T>>) -> Self {
        // Enable necessary clocks.
        critical_section::with(|_| {
            #[cfg(not(eth_v2a))]
            crate::pac::RCC.ahb1enr().modify(|w| {
                w.set_ethen(true);
            });
            #[cfg(eth_v2a)]
            crate::pac::RCC.ahb5enr().modify(|w| {
                w.set_eth1en(true);
                w.set_eth1macen(true);
            });
        });

        let hclk = unsafe { crate::rcc::get_freqs().hclk1.to_hertz() };
        let hclk = unwrap!(hclk, "SMA requires HCLK to be enabled, but it was not.");
        let hclk_mhz = hclk.0 / 1_000_000;

        // Set the MDC clock frequency in the range 1MHz - 2.5MHz
        #[cfg(not(eth_v2a))]
        let clock_range = match hclk_mhz {
            0..=34 => 2,    // Divide by 16
            35..=59 => 3,   // Divide by 26
            60..=99 => 0,   // Divide by 42
            100..=149 => 1, // Divide by 62
            150..=249 => 4, // Divide by 102
            250..=310 => 5, // Divide by 124
            _ => {
                panic!("HCLK results in MDC clock > 2.5MHz even for the highest CSR clock divider")
            }
        };
        // The eth_v2a MAC supports two extra dividers for higher HCLK
        // frequencies. Boundaries match the ST HAL (HAL_ETH_SetMDIOClockRange).
        #[cfg(eth_v2a)]
        let clock_range = match hclk_mhz {
            0..=34 => 2,    // Divide by 16
            35..=59 => 3,   // Divide by 26
            60..=99 => 0,   // Divide by 42
            100..=149 => 1, // Divide by 62
            150..=249 => 4, // Divide by 102
            250..=299 => 5, // Divide by 124
            300..=499 => 6, // Divide by 204
            _ => 7,         // Divide by 324
        };

        Self {
            _peri: peri,
            clock_range,
            _pins: [
                new_pin!(mdio, AfType::output(OutputType::PushPull, Speed::VeryHigh)).unwrap(),
                new_pin!(mdc, AfType::output(OutputType::PushPull, Speed::VeryHigh)).unwrap(),
            ],
        }
    }
}

impl<T: Instance> StationManagement for Sma<'_, T> {
    fn smi_read(&mut self, phy_addr: u8, reg: u8) -> u16 {
        let (macmdioar, macmdiodr) = T::regs();

        macmdioar.modify(|w| {
            w.set_pa(phy_addr);
            w.set_rda(reg);
            w.set_goc(0b11); // read
            w.set_cr(self.clock_range);
            // eth_v2a renamed the MDIO busy/data fields (MB/MD -> GB/GD).
            #[cfg(not(eth_v2a))]
            w.set_mb(true);
            #[cfg(eth_v2a)]
            w.set_gb(true);
        });

        #[cfg(not(eth_v2a))]
        while macmdioar.read().mb() {}
        #[cfg(eth_v2a)]
        while macmdioar.read().gb() {}

        #[cfg(not(eth_v2a))]
        let data = macmdiodr.read().md();
        #[cfg(eth_v2a)]
        let data = macmdiodr.read().gd();
        data
    }

    fn smi_write(&mut self, phy_addr: u8, reg: u8, val: u16) {
        let (macmdioar, macmdiodr) = T::regs();

        #[cfg(not(eth_v2a))]
        macmdiodr.write(|w| w.set_md(val));
        #[cfg(eth_v2a)]
        macmdiodr.write(|w| w.set_gd(val));
        macmdioar.modify(|w| {
            w.set_pa(phy_addr);
            w.set_rda(reg);
            w.set_goc(0b01); // write
            w.set_cr(self.clock_range);
            #[cfg(not(eth_v2a))]
            w.set_mb(true);
            #[cfg(eth_v2a)]
            w.set_gb(true);
        });

        #[cfg(not(eth_v2a))]
        while macmdioar.read().mb() {}
        #[cfg(eth_v2a)]
        while macmdioar.read().gb() {}
    }
}
