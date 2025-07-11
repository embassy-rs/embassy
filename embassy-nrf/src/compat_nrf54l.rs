#![allow(missing_docs)]
use extend::ext;

use crate::pac::common::{Reg, RW, W};
use crate::pac::{gpio, pwm, spim, spis, twim, twis, uarte};

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct SpiFrequencyVals(pub u32);

impl SpiFrequencyVals {
    #[doc = "8 Mbps, SPIM 2x, 3x"]
    pub const M8: Self = Self(0x0000_0002);
    #[doc = "4 Mbps, SPIM 2x, 3x"]
    pub const M4: Self = Self(0x0000_0004);
    #[doc = "2 Mbps, SPIM 2x, 3x"]
    pub const M2: Self = Self(0x0000_0008);
    #[doc = "1 Mbps, SPIM 2x, 3x"]
    pub const M1: Self = Self(0x0000_0010);
    #[doc = "500 kbps, SPIM 2x, 3x"]
    pub const K500: Self = Self(0x0000_0020);
    #[doc = "250 kbps, SPIM 2x, 3x"]
    pub const K250: Self = Self(0x0000_0040);
    #[doc = "125 kbps, SPIM 2x, 3x"]
    pub const K125: Self = Self(0x0000_007E);

    #[doc = "32 Mbps, SPIM 00 only"]
    pub const HS_M32: Self = Self(0x0000_0004);
    #[doc = "16 Mbps, SPIM 00 only"]
    pub const HS_M16: Self = Self(0x0000_0008);
    #[doc = "8 Mbps, SPIM 00 only"]
    pub const HS_M8: Self = Self(0x0000_0010);
    #[doc = "4 Mbps, SPIM 00 only"]
    pub const HS_M4: Self = Self(0x0000_0020);
    #[doc = "2 Mbps, SPIM 00 only"]
    pub const HS_M2: Self = Self(0x0000_0040);
    #[doc = "1 Mbps, SPIM 00 only"]
    pub const HS_M1: Self = Self(0x0000_007E);
}

impl SpiFrequencyVals {
    #[inline(always)]
    pub const fn from_bits(val: u32) -> SpiFrequencyVals {
        Self(val & 0xffff_ffff)
    }
    #[inline(always)]
    pub const fn to_bits(self) -> u32 {
        self.0
    }
}

impl core::fmt::Debug for SpiFrequencyVals {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::write!(f, "CLK/{}", self.0)
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for SpiFrequencyVals {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "CLK/{}", self.0)
    }
}

impl From<u32> for SpiFrequencyVals {
    #[inline(always)]
    fn from(val: u32) -> SpiFrequencyVals {
        SpiFrequencyVals::from_bits(val)
    }
}

impl From<SpiFrequencyVals> for u32 {
    #[inline(always)]
    fn from(val: SpiFrequencyVals) -> u32 {
        SpiFrequencyVals::to_bits(val)
    }
}

#[doc = "SPI frequency. Accuracy depends on the HFCLK source selected."]
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SpiFrequency(pub u32);

impl SpiFrequency {
    #[doc = "SPI clock frequency"]
    #[inline(always)]
    pub const fn frequency(&self) -> SpiFrequencyVals {
        let val = (self.0 >> 0usize) & 0xffff_ffff;
        SpiFrequencyVals::from_bits(val as u32)
    }
    #[doc = "SPI clock frequency"]
    #[inline(always)]
    pub fn set_frequency(&mut self, val: SpiFrequencyVals) {
        self.0 = (self.0 & !(0xffff_ffff << 0usize)) | (((val.to_bits() as u32) & 0xffff_ffff) << 0usize);
    }
}

impl Default for SpiFrequency {
    #[inline(always)]
    fn default() -> SpiFrequency {
        SpiFrequency(0)
    }
}

impl core::fmt::Debug for SpiFrequency {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("SpiFrequency")
            .field("frequency", &self.frequency())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for SpiFrequency {
    fn format(&self, f: defmt::Formatter) {
        #[derive(defmt :: Format)]
        struct SpiFrequency {
            frequency: SpiFrequencyVals,
        }
        let proxy = SpiFrequency {
            frequency: self.frequency(),
        };
        defmt::write!(f, "{}", proxy)
    }
}

pub mod gpiovals {
    pub use crate::pac::gpio::vals::*;

    #[allow(dead_code)]
    pub enum Drive {
        S0S1,
        H0S1,
        S0H1,
        H0H1,
        D0S1,
        D0H1,
        S0D1,
        H0D1,
    }
}

#[ext]
pub impl gpio::regs::PinCnf {
    fn set_drive(&mut self, val: gpiovals::Drive) {
        match val {
            gpiovals::Drive::S0S1 => {
                self.set_drive0(gpio::vals::Drive::S);
                self.set_drive1(gpio::vals::Drive::S);
            }
            gpiovals::Drive::H0S1 => {
                self.set_drive0(gpio::vals::Drive::H);
                self.set_drive1(gpio::vals::Drive::S);
            }
            gpiovals::Drive::S0H1 => {
                self.set_drive0(gpio::vals::Drive::S);
                self.set_drive1(gpio::vals::Drive::H);
            }
            gpiovals::Drive::H0H1 => {
                self.set_drive0(gpio::vals::Drive::H);
                self.set_drive1(gpio::vals::Drive::H);
            }
            gpiovals::Drive::D0S1 => {
                self.set_drive0(gpio::vals::Drive::D);
                self.set_drive1(gpio::vals::Drive::S);
            }
            gpiovals::Drive::D0H1 => {
                self.set_drive0(gpio::vals::Drive::D);
                self.set_drive1(gpio::vals::Drive::H);
            }
            gpiovals::Drive::S0D1 => {
                self.set_drive0(gpio::vals::Drive::S);
                self.set_drive1(gpio::vals::Drive::D);
            }
            gpiovals::Drive::H0D1 => {
                self.set_drive0(gpio::vals::Drive::H);
                self.set_drive1(gpio::vals::Drive::D);
            }
        }
    }
}

#[ext]
pub impl uarte::Uarte {
    #[inline(always)]
    fn rxd(&self) -> uarte::DmaRx {
        self.dma().rx()
    }
    #[inline(always)]
    fn txd(&self) -> uarte::DmaTx {
        self.dma().tx()
    }
    #[inline(always)]
    fn tasks_starttx(&self) -> Reg<u32, W> {
        self.tasks_dma().tx().start()
    }
    #[inline(always)]
    fn tasks_startrx(&self) -> Reg<u32, W> {
        self.tasks_dma().rx().start()
    }
    #[inline(always)]
    fn tasks_stoptx(&self) -> Reg<u32, W> {
        self.tasks_dma().tx().stop()
    }
    #[inline(always)]
    fn events_txstarted(&self) -> Reg<u32, RW> {
        self.events_dma().tx().ready()
    }
    #[inline(always)]
    fn events_endtx(&self) -> Reg<u32, RW> {
        self.events_dma().tx().end()
    }
    #[inline(always)]
    fn tasks_stoprx(&self) -> Reg<u32, W> {
        self.tasks_dma().rx().stop()
    }
    #[inline(always)]
    fn events_rxstarted(&self) -> Reg<u32, RW> {
        self.events_dma().rx().ready()
    }
    #[inline(always)]
    fn events_endrx(&self) -> Reg<u32, RW> {
        self.events_dma().rx().end()
    }
}

#[ext]
pub impl uarte::regs::Int {
    #[inline(always)]
    fn set_endrx(&mut self, val: bool) {
        self.set_dmarxend(val)
    }
    #[inline(always)]
    fn set_endtx(&mut self, val: bool) {
        self.set_dmatxend(val)
    }
    #[inline(always)]
    fn set_txstarted(&mut self, val: bool) {
        self.set_dmatxready(val)
    }
    #[inline(always)]
    fn set_rxstarted(&mut self, val: bool) {
        self.set_dmarxready(val)
    }
}

#[ext]
pub impl spim::Spim {
    #[inline(always)]
    fn rxd(&self) -> spim::DmaRx {
        self.dma().rx()
    }
    #[inline(always)]
    fn txd(&self) -> spim::DmaTx {
        self.dma().tx()
    }
    #[inline(always)]
    fn frequency(&self) -> Reg<SpiFrequency, RW> {
        unsafe { Reg::from_ptr(self.as_ptr().add(0x052cusize) as _) }
    }
}

#[ext]
pub impl spis::Spis {
    #[inline(always)]
    fn rxd(&self) -> spis::DmaRx {
        self.dma().rx()
    }
    #[inline(always)]
    fn txd(&self) -> spis::DmaTx {
        self.dma().tx()
    }
}

#[ext]
pub impl twim::Twim {
    #[inline(always)]
    fn rxd(&self) -> twim::DmaRx {
        self.dma().rx()
    }
    #[inline(always)]
    fn txd(&self) -> twim::DmaTx {
        self.dma().tx()
    }
    #[inline(always)]
    fn tasks_starttx(&self) -> Reg<u32, W> {
        self.tasks_dma().tx().start()
    }
    #[inline(always)]
    fn tasks_startrx(&self) -> Reg<u32, W> {
        self.tasks_dma().rx().start()
    }
}

#[ext]
pub impl twim::regs::Shorts {
    #[inline(always)]
    fn set_lasttx_startrx(&mut self, val: bool) {
        self.set_lasttx_dma_rx_start(val)
    }
    #[inline(always)]
    fn set_lastrx_starttx(&mut self, val: bool) {
        self.set_lastrx_dma_tx_start(val)
    }
}

#[ext]
pub impl twis::Twis {
    #[inline(always)]
    fn rxd(&self) -> twis::DmaRx {
        self.dma().rx()
    }
    #[inline(always)]
    fn txd(&self) -> twis::DmaTx {
        self.dma().tx()
    }
}

#[ext]
pub impl pwm::Pwm {
    #[inline(always)]
    fn tasks_seqstart(&self, idx: usize) -> Reg<u32, W> {
        self.tasks_dma().seq(idx).start()
    }
}

#[ext]
pub impl pwm::regs::Shorts {
    #[inline(always)]
    fn set_loopsdone_seqstart0(&mut self, val: bool) {
        self.set_loopsdone_dma_seq0_start(val)
    }
}

#[ext]
pub impl pwm::PwmSeq {
    #[inline(always)]
    fn parent(&self) -> pwm::Pwm {
        unsafe { pwm::Pwm::from_ptr(self.as_ptr().map_addr(|addr| addr & 0xfffff000)) }
    }
    #[inline(always)]
    fn idx(&self) -> usize {
        ((self.as_ptr() as usize & 0xfff) - 0x520) / 32
    }
    #[inline(always)]
    fn ptr(&self) -> Reg<u32, RW> {
        self.parent().dma().seq(self.idx()).ptr()
    }
    #[inline(always)]
    fn cnt(&self) -> Reg<pwm::regs::Maxcnt, RW> {
        self.parent().dma().seq(self.idx()).maxcnt()
    }
}
