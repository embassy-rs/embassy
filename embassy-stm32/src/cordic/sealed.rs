use super::*;
use crate::pac::cordic::vals;

/// Cordic instance
pub(super) trait SealedInstance {
    /// Get access to CORDIC registers
    fn regs() -> crate::pac::cordic::Cordic;

    /// Set Function value
    fn set_func(&self, func: Function) {
        Self::regs()
            .csr()
            .modify(|v| v.set_func(vals::Func::from_bits(func as u8)));
    }

    /// Set Precision value
    fn set_precision(&self, precision: Precision) {
        Self::regs()
            .csr()
            .modify(|v| v.set_precision(vals::Precision::from_bits(precision as u8)))
    }

    /// Set Scale value
    fn set_scale(&self, scale: Scale) {
        Self::regs()
            .csr()
            .modify(|v| v.set_scale(vals::Scale::from_bits(scale as u8)))
    }

    /// Enable global interrupt
    fn enable_irq(&self) {
        Self::regs().csr().modify(|v| v.set_ien(true))
    }

    /// Disable global interrupt
    fn disable_irq(&self) {
        Self::regs().csr().modify(|v| v.set_ien(false))
    }

    /// Enable Read DMA
    fn enable_read_dma(&self) {
        Self::regs().csr().modify(|v| {
            v.set_dmaren(true);
        })
    }

    /// Disable Read DMA
    fn disable_read_dma(&self) {
        Self::regs().csr().modify(|v| {
            v.set_dmaren(false);
        })
    }

    /// Enable Write DMA
    fn enable_write_dma(&self) {
        Self::regs().csr().modify(|v| {
            v.set_dmawen(true);
        })
    }

    /// Disable Write DMA
    fn disable_write_dma(&self) {
        Self::regs().csr().modify(|v| {
            v.set_dmawen(false);
        })
    }

    /// Set NARGS value
    fn set_argument_count(&self, n: AccessCount) {
        Self::regs().csr().modify(|v| {
            v.set_nargs(match n {
                AccessCount::One => vals::Num::NUM1,
                AccessCount::Two => vals::Num::NUM2,
            })
        })
    }

    /// Set NRES value
    fn set_result_count(&self, n: AccessCount) {
        Self::regs().csr().modify(|v| {
            v.set_nres(match n {
                AccessCount::One => vals::Num::NUM1,
                AccessCount::Two => vals::Num::NUM2,
            });
        })
    }

    /// Set ARGSIZE and RESSIZE value
    fn set_data_width(&self, arg: Width, res: Width) {
        Self::regs().csr().modify(|v| {
            v.set_argsize(match arg {
                Width::Bits32 => vals::Size::BITS32,
                Width::Bits16 => vals::Size::BITS16,
            });
            v.set_ressize(match res {
                Width::Bits32 => vals::Size::BITS32,
                Width::Bits16 => vals::Size::BITS16,
            })
        })
    }

    /// Read RRDY flag
    fn ready_to_read(&self) -> bool {
        Self::regs().csr().read().rrdy()
    }

    /// Write value to WDATA
    fn write_argument(&self, arg: u32) {
        Self::regs().wdata().write_value(arg)
    }

    /// Read value from RDATA
    fn read_result(&self) -> u32 {
        Self::regs().rdata().read()
    }
}
