use embassy_hal_internal::Peri;
pub(crate) use regs::{Macmiiar as AddressRegister, Macmiidr as DataRegister};
use stm32_metapac::eth::regs;
use stm32_metapac::eth::vals::{Cr, MbProgress, Mw};

use super::{Instance, StationManagement};
use crate::eth::{MDCPin, MDIOPin};
use crate::gpio::{AfType, Flex, OutputType, Speed};

/// Station Management Agent.
///
/// This peripheral is used for SMI reads and writes to the connected
/// ethernet PHY/device(s).
pub struct Sma<'d, T: Instance> {
    _peri: Peri<'d, T>,
    clock_range: Cr,
    _pins: [Flex<'d>; 2],
}

impl<'d, T: Instance> Sma<'d, T> {
    /// Create a new instance of this peripheral.
    pub fn new<#[cfg(afio)] A>(
        peri: Peri<'d, T>,
        mdio: Peri<'d, if_afio!(impl MDIOPin<T, A>)>,
        mdc: Peri<'d, if_afio!(impl MDCPin<T, A>)>,
    ) -> Self {
        // Enable necessary clocks.
        critical_section::with(|_| {
            #[cfg(eth_v1a)]
            let reg = crate::pac::RCC.ahbenr();

            #[cfg(any(eth_v1b, eth_v1c))]
            let reg = crate::pac::RCC.ahb1enr();

            reg.modify(|w| {
                w.set_ethen(true);
            })
        });

        let hclk = unsafe { crate::rcc::get_freqs().hclk1.to_hertz() };
        let hclk = unwrap!(hclk, "SMA requires HCLK to be enabled, but it was not.");
        let hclk_mhz = hclk.0 / 1_000_000;

        // Set the MDC clock frequency in the range 1MHz - 2.5MHz
        let clock_range = match hclk_mhz {
            0..=24 => panic!("Invalid HCLK frequency - should be at least 25 MHz."),
            25..=34 => Cr::CR_20_35,     // Divide by 16
            35..=59 => Cr::CR_35_60,     // Divide by 26
            60..=99 => Cr::CR_60_100,    // Divide by 42
            100..=149 => Cr::CR_100_150, // Divide by 62
            150..=216 => Cr::CR_150_168, // Divide by 102
            _ => {
                panic!("HCLK results in MDC clock > 2.5MHz even for the highest CSR clock divider")
            }
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
        let (macmiiar, macmiidr) = T::regs();

        macmiiar.modify(|w| {
            w.set_pa(phy_addr);
            w.set_mr(reg);
            w.set_mw(Mw::READ); // read operation
            w.set_cr(self.clock_range);
            w.set_mb(MbProgress::BUSY); // indicate that operation is in progress
        });
        while macmiiar.read().mb() == MbProgress::BUSY {}
        macmiidr.read().md()
    }

    fn smi_write(&mut self, phy_addr: u8, reg: u8, val: u16) {
        let (macmiiar, macmiidr) = T::regs();

        macmiidr.write(|w| w.set_md(val));
        macmiiar.modify(|w| {
            w.set_pa(phy_addr);
            w.set_mr(reg);
            w.set_mw(Mw::WRITE); // write
            w.set_cr(self.clock_range);
            w.set_mb(MbProgress::BUSY);
        });
        while macmiiar.read().mb() == MbProgress::BUSY {}
    }
}
