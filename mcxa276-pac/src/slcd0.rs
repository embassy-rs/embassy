#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    lcd_gcr: LcdGcr,
    lcd_ar: LcdAr,
    lcd_fdcr: LcdFdcr,
    lcd_fdsr: LcdFdsr,
    lcd_pen0: LcdPen0,
    lcd_pen1: LcdPen1,
    lcd_bpen0: LcdBpen0,
    lcd_bpen1: LcdBpen1,
    lcd_wfto: [LcdWfto; 12],
}
impl RegisterBlock {
    #[doc = "0x00 - LCD General Control Register"]
    #[inline(always)]
    pub const fn lcd_gcr(&self) -> &LcdGcr {
        &self.lcd_gcr
    }
    #[doc = "0x04 - LCD Auxiliary Register"]
    #[inline(always)]
    pub const fn lcd_ar(&self) -> &LcdAr {
        &self.lcd_ar
    }
    #[doc = "0x08 - LCD Fault Detect Control Register"]
    #[inline(always)]
    pub const fn lcd_fdcr(&self) -> &LcdFdcr {
        &self.lcd_fdcr
    }
    #[doc = "0x0c - LCD Fault Detect Status Register"]
    #[inline(always)]
    pub const fn lcd_fdsr(&self) -> &LcdFdsr {
        &self.lcd_fdsr
    }
    #[doc = "0x10 - LCD Pin Enable Register 0"]
    #[inline(always)]
    pub const fn lcd_pen0(&self) -> &LcdPen0 {
        &self.lcd_pen0
    }
    #[doc = "0x14 - LCD Pin Enable Register 1"]
    #[inline(always)]
    pub const fn lcd_pen1(&self) -> &LcdPen1 {
        &self.lcd_pen1
    }
    #[doc = "0x18 - LCD Back Plane Enable Register 0"]
    #[inline(always)]
    pub const fn lcd_bpen0(&self) -> &LcdBpen0 {
        &self.lcd_bpen0
    }
    #[doc = "0x1c - LCD Back Plane Enable Register 1"]
    #[inline(always)]
    pub const fn lcd_bpen1(&self) -> &LcdBpen1 {
        &self.lcd_bpen1
    }
    #[doc = "0x20..0x50 - LCD Waveform i * 4 + 3 to i * 4 Register"]
    #[inline(always)]
    pub const fn lcd_wfto(&self, n: usize) -> &LcdWfto {
        &self.lcd_wfto[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x20..0x50 - LCD Waveform i * 4 + 3 to i * 4 Register"]
    #[inline(always)]
    pub fn lcd_wfto_iter(&self) -> impl Iterator<Item = &LcdWfto> {
        self.lcd_wfto.iter()
    }
}
#[doc = "LCD_GCR (rw) register accessor: LCD General Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_gcr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_gcr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lcd_gcr`] module"]
#[doc(alias = "LCD_GCR")]
pub type LcdGcr = crate::Reg<lcd_gcr::LcdGcrSpec>;
#[doc = "LCD General Control Register"]
pub mod lcd_gcr;
#[doc = "LCD_AR (rw) register accessor: LCD Auxiliary Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_ar::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_ar::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lcd_ar`] module"]
#[doc(alias = "LCD_AR")]
pub type LcdAr = crate::Reg<lcd_ar::LcdArSpec>;
#[doc = "LCD Auxiliary Register"]
pub mod lcd_ar;
#[doc = "LCD_FDCR (rw) register accessor: LCD Fault Detect Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_fdcr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_fdcr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lcd_fdcr`] module"]
#[doc(alias = "LCD_FDCR")]
pub type LcdFdcr = crate::Reg<lcd_fdcr::LcdFdcrSpec>;
#[doc = "LCD Fault Detect Control Register"]
pub mod lcd_fdcr;
#[doc = "LCD_FDSR (rw) register accessor: LCD Fault Detect Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_fdsr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_fdsr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lcd_fdsr`] module"]
#[doc(alias = "LCD_FDSR")]
pub type LcdFdsr = crate::Reg<lcd_fdsr::LcdFdsrSpec>;
#[doc = "LCD Fault Detect Status Register"]
pub mod lcd_fdsr;
#[doc = "LCD_PEN0 (rw) register accessor: LCD Pin Enable Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_pen0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_pen0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lcd_pen0`] module"]
#[doc(alias = "LCD_PEN0")]
pub type LcdPen0 = crate::Reg<lcd_pen0::LcdPen0Spec>;
#[doc = "LCD Pin Enable Register 0"]
pub mod lcd_pen0;
#[doc = "LCD_PEN1 (rw) register accessor: LCD Pin Enable Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_pen1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_pen1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lcd_pen1`] module"]
#[doc(alias = "LCD_PEN1")]
pub type LcdPen1 = crate::Reg<lcd_pen1::LcdPen1Spec>;
#[doc = "LCD Pin Enable Register 1"]
pub mod lcd_pen1;
#[doc = "LCD_BPEN0 (rw) register accessor: LCD Back Plane Enable Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_bpen0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_bpen0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lcd_bpen0`] module"]
#[doc(alias = "LCD_BPEN0")]
pub type LcdBpen0 = crate::Reg<lcd_bpen0::LcdBpen0Spec>;
#[doc = "LCD Back Plane Enable Register 0"]
pub mod lcd_bpen0;
#[doc = "LCD_BPEN1 (rw) register accessor: LCD Back Plane Enable Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_bpen1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_bpen1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lcd_bpen1`] module"]
#[doc(alias = "LCD_BPEN1")]
pub type LcdBpen1 = crate::Reg<lcd_bpen1::LcdBpen1Spec>;
#[doc = "LCD Back Plane Enable Register 1"]
pub mod lcd_bpen1;
#[doc = "LCD_WFTO (rw) register accessor: LCD Waveform i * 4 + 3 to i * 4 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_wfto::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_wfto::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lcd_wfto`] module"]
#[doc(alias = "LCD_WFTO")]
pub type LcdWfto = crate::Reg<lcd_wfto::LcdWftoSpec>;
#[doc = "LCD Waveform i * 4 + 3 to i * 4 Register"]
pub mod lcd_wfto;
