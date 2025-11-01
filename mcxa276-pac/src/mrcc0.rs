#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    mrcc_glb_rst0: MrccGlbRst0,
    mrcc_glb_rst0_set: MrccGlbRst0Set,
    mrcc_glb_rst0_clr: MrccGlbRst0Clr,
    _reserved3: [u8; 0x04],
    mrcc_glb_rst1: MrccGlbRst1,
    mrcc_glb_rst1_set: MrccGlbRst1Set,
    mrcc_glb_rst1_clr: MrccGlbRst1Clr,
    _reserved6: [u8; 0x04],
    mrcc_glb_rst2: MrccGlbRst2,
    mrcc_glb_rst2_set: MrccGlbRst2Set,
    mrcc_glb_rst2_clr: MrccGlbRst2Clr,
    _reserved9: [u8; 0x14],
    mrcc_glb_cc0: MrccGlbCc0,
    mrcc_glb_cc0_set: MrccGlbCc0Set,
    mrcc_glb_cc0_clr: MrccGlbCc0Clr,
    _reserved12: [u8; 0x04],
    mrcc_glb_cc1: MrccGlbCc1,
    mrcc_glb_cc1_set: MrccGlbCc1Set,
    mrcc_glb_cc1_clr: MrccGlbCc1Clr,
    _reserved15: [u8; 0x04],
    mrcc_glb_cc2: MrccGlbCc2,
    mrcc_glb_cc2_set: MrccGlbCc2Set,
    mrcc_glb_cc2_clr: MrccGlbCc2Clr,
    _reserved18: [u8; 0x14],
    mrcc_glb_acc0: MrccGlbAcc0,
    mrcc_glb_acc1: MrccGlbAcc1,
    mrcc_glb_acc2: MrccGlbAcc2,
    _reserved21: [u8; 0x14],
    mrcc_i3c0_fclk_clksel: MrccI3c0FclkClksel,
    mrcc_i3c0_fclk_clkdiv: MrccI3c0FclkClkdiv,
    mrcc_ctimer0_clksel: MrccCtimer0Clksel,
    mrcc_ctimer0_clkdiv: MrccCtimer0Clkdiv,
    mrcc_ctimer1_clksel: MrccCtimer1Clksel,
    mrcc_ctimer1_clkdiv: MrccCtimer1Clkdiv,
    mrcc_ctimer2_clksel: MrccCtimer2Clksel,
    mrcc_ctimer2_clkdiv: MrccCtimer2Clkdiv,
    mrcc_ctimer3_clksel: MrccCtimer3Clksel,
    mrcc_ctimer3_clkdiv: MrccCtimer3Clkdiv,
    mrcc_ctimer4_clksel: MrccCtimer4Clksel,
    mrcc_ctimer4_clkdiv: MrccCtimer4Clkdiv,
    _reserved33: [u8; 0x04],
    mrcc_wwdt0_clkdiv: MrccWwdt0Clkdiv,
    mrcc_flexio0_clksel: MrccFlexio0Clksel,
    mrcc_flexio0_clkdiv: MrccFlexio0Clkdiv,
    mrcc_lpi2c0_clksel: MrccLpi2c0Clksel,
    mrcc_lpi2c0_clkdiv: MrccLpi2c0Clkdiv,
    mrcc_lpi2c1_clksel: MrccLpi2c1Clksel,
    mrcc_lpi2c1_clkdiv: MrccLpi2c1Clkdiv,
    mrcc_lpspi0_clksel: MrccLpspi0Clksel,
    mrcc_lpspi0_clkdiv: MrccLpspi0Clkdiv,
    mrcc_lpspi1_clksel: MrccLpspi1Clksel,
    mrcc_lpspi1_clkdiv: MrccLpspi1Clkdiv,
    mrcc_lpuart0_clksel: MrccLpuart0Clksel,
    mrcc_lpuart0_clkdiv: MrccLpuart0Clkdiv,
    mrcc_lpuart1_clksel: MrccLpuart1Clksel,
    mrcc_lpuart1_clkdiv: MrccLpuart1Clkdiv,
    mrcc_lpuart2_clksel: MrccLpuart2Clksel,
    mrcc_lpuart2_clkdiv: MrccLpuart2Clkdiv,
    mrcc_lpuart3_clksel: MrccLpuart3Clksel,
    mrcc_lpuart3_clkdiv: MrccLpuart3Clkdiv,
    mrcc_lpuart4_clksel: MrccLpuart4Clksel,
    mrcc_lpuart4_clkdiv: MrccLpuart4Clkdiv,
    mrcc_usb0_clksel: MrccUsb0Clksel,
    mrcc_usb0_clkdiv: MrccUsb0Clkdiv,
    mrcc_lptmr0_clksel: MrccLptmr0Clksel,
    mrcc_lptmr0_clkdiv: MrccLptmr0Clkdiv,
    mrcc_ostimer0_clksel: MrccOstimer0Clksel,
    _reserved59: [u8; 0x04],
    mrcc_adc_clksel: MrccAdcClksel,
    mrcc_adc_clkdiv: MrccAdcClkdiv,
    _reserved61: [u8; 0x04],
    mrcc_cmp0_func_clkdiv: MrccCmp0FuncClkdiv,
    mrcc_cmp0_rr_clksel: MrccCmp0RrClksel,
    mrcc_cmp0_rr_clkdiv: MrccCmp0RrClkdiv,
    _reserved64: [u8; 0x04],
    mrcc_cmp1_func_clkdiv: MrccCmp1FuncClkdiv,
    mrcc_cmp1_rr_clksel: MrccCmp1RrClksel,
    mrcc_cmp1_rr_clkdiv: MrccCmp1RrClkdiv,
    _reserved67: [u8; 0x04],
    mrcc_cmp2_func_clkdiv: MrccCmp2FuncClkdiv,
    mrcc_cmp2_rr_clksel: MrccCmp2RrClksel,
    mrcc_cmp2_rr_clkdiv: MrccCmp2RrClkdiv,
    mrcc_dac0_clksel: MrccDac0Clksel,
    mrcc_dac0_clkdiv: MrccDac0Clkdiv,
    mrcc_flexcan0_clksel: MrccFlexcan0Clksel,
    mrcc_flexcan0_clkdiv: MrccFlexcan0Clkdiv,
    mrcc_flexcan1_clksel: MrccFlexcan1Clksel,
    mrcc_flexcan1_clkdiv: MrccFlexcan1Clkdiv,
    mrcc_lpi2c2_clksel: MrccLpi2c2Clksel,
    mrcc_lpi2c2_clkdiv: MrccLpi2c2Clkdiv,
    mrcc_lpi2c3_clksel: MrccLpi2c3Clksel,
    mrcc_lpi2c3_clkdiv: MrccLpi2c3Clkdiv,
    mrcc_lpuart5_clksel: MrccLpuart5Clksel,
    mrcc_lpuart5_clkdiv: MrccLpuart5Clkdiv,
    mrcc_dbg_trace_clksel: MrccDbgTraceClksel,
    mrcc_dbg_trace_clkdiv: MrccDbgTraceClkdiv,
    mrcc_clkout_clksel: MrccClkoutClksel,
    mrcc_clkout_clkdiv: MrccClkoutClkdiv,
    mrcc_systick_clksel: MrccSystickClksel,
    mrcc_systick_clkdiv: MrccSystickClkdiv,
}
impl RegisterBlock {
    #[doc = "0x00 - Peripheral Reset Control 0"]
    #[inline(always)]
    pub const fn mrcc_glb_rst0(&self) -> &MrccGlbRst0 {
        &self.mrcc_glb_rst0
    }
    #[doc = "0x04 - Peripheral Reset Control Set 0"]
    #[inline(always)]
    pub const fn mrcc_glb_rst0_set(&self) -> &MrccGlbRst0Set {
        &self.mrcc_glb_rst0_set
    }
    #[doc = "0x08 - Peripheral Reset Control Clear 0"]
    #[inline(always)]
    pub const fn mrcc_glb_rst0_clr(&self) -> &MrccGlbRst0Clr {
        &self.mrcc_glb_rst0_clr
    }
    #[doc = "0x10 - Peripheral Reset Control 1"]
    #[inline(always)]
    pub const fn mrcc_glb_rst1(&self) -> &MrccGlbRst1 {
        &self.mrcc_glb_rst1
    }
    #[doc = "0x14 - Peripheral Reset Control Set 1"]
    #[inline(always)]
    pub const fn mrcc_glb_rst1_set(&self) -> &MrccGlbRst1Set {
        &self.mrcc_glb_rst1_set
    }
    #[doc = "0x18 - Peripheral Reset Control Clear 1"]
    #[inline(always)]
    pub const fn mrcc_glb_rst1_clr(&self) -> &MrccGlbRst1Clr {
        &self.mrcc_glb_rst1_clr
    }
    #[doc = "0x20 - Peripheral Reset Control 2"]
    #[inline(always)]
    pub const fn mrcc_glb_rst2(&self) -> &MrccGlbRst2 {
        &self.mrcc_glb_rst2
    }
    #[doc = "0x24 - Peripheral Reset Control Set 2"]
    #[inline(always)]
    pub const fn mrcc_glb_rst2_set(&self) -> &MrccGlbRst2Set {
        &self.mrcc_glb_rst2_set
    }
    #[doc = "0x28 - Peripheral Reset Control Clear 2"]
    #[inline(always)]
    pub const fn mrcc_glb_rst2_clr(&self) -> &MrccGlbRst2Clr {
        &self.mrcc_glb_rst2_clr
    }
    #[doc = "0x40 - AHB Clock Control 0"]
    #[inline(always)]
    pub const fn mrcc_glb_cc0(&self) -> &MrccGlbCc0 {
        &self.mrcc_glb_cc0
    }
    #[doc = "0x44 - AHB Clock Control Set 0"]
    #[inline(always)]
    pub const fn mrcc_glb_cc0_set(&self) -> &MrccGlbCc0Set {
        &self.mrcc_glb_cc0_set
    }
    #[doc = "0x48 - AHB Clock Control Clear 0"]
    #[inline(always)]
    pub const fn mrcc_glb_cc0_clr(&self) -> &MrccGlbCc0Clr {
        &self.mrcc_glb_cc0_clr
    }
    #[doc = "0x50 - AHB Clock Control 1"]
    #[inline(always)]
    pub const fn mrcc_glb_cc1(&self) -> &MrccGlbCc1 {
        &self.mrcc_glb_cc1
    }
    #[doc = "0x54 - AHB Clock Control Set 1"]
    #[inline(always)]
    pub const fn mrcc_glb_cc1_set(&self) -> &MrccGlbCc1Set {
        &self.mrcc_glb_cc1_set
    }
    #[doc = "0x58 - AHB Clock Control Clear 1"]
    #[inline(always)]
    pub const fn mrcc_glb_cc1_clr(&self) -> &MrccGlbCc1Clr {
        &self.mrcc_glb_cc1_clr
    }
    #[doc = "0x60 - AHB Clock Control 2"]
    #[inline(always)]
    pub const fn mrcc_glb_cc2(&self) -> &MrccGlbCc2 {
        &self.mrcc_glb_cc2
    }
    #[doc = "0x64 - AHB Clock Control Set 2"]
    #[inline(always)]
    pub const fn mrcc_glb_cc2_set(&self) -> &MrccGlbCc2Set {
        &self.mrcc_glb_cc2_set
    }
    #[doc = "0x68 - AHB Clock Control Clear 2"]
    #[inline(always)]
    pub const fn mrcc_glb_cc2_clr(&self) -> &MrccGlbCc2Clr {
        &self.mrcc_glb_cc2_clr
    }
    #[doc = "0x80 - Control Automatic Clock Gating 0"]
    #[inline(always)]
    pub const fn mrcc_glb_acc0(&self) -> &MrccGlbAcc0 {
        &self.mrcc_glb_acc0
    }
    #[doc = "0x84 - Control Automatic Clock Gating 1"]
    #[inline(always)]
    pub const fn mrcc_glb_acc1(&self) -> &MrccGlbAcc1 {
        &self.mrcc_glb_acc1
    }
    #[doc = "0x88 - Control Automatic Clock Gating 2"]
    #[inline(always)]
    pub const fn mrcc_glb_acc2(&self) -> &MrccGlbAcc2 {
        &self.mrcc_glb_acc2
    }
    #[doc = "0xa0 - I3C0_FCLK clock selection control"]
    #[inline(always)]
    pub const fn mrcc_i3c0_fclk_clksel(&self) -> &MrccI3c0FclkClksel {
        &self.mrcc_i3c0_fclk_clksel
    }
    #[doc = "0xa4 - I3C0_FCLK clock divider control"]
    #[inline(always)]
    pub const fn mrcc_i3c0_fclk_clkdiv(&self) -> &MrccI3c0FclkClkdiv {
        &self.mrcc_i3c0_fclk_clkdiv
    }
    #[doc = "0xa8 - CTIMER0 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_ctimer0_clksel(&self) -> &MrccCtimer0Clksel {
        &self.mrcc_ctimer0_clksel
    }
    #[doc = "0xac - CTIMER0 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_ctimer0_clkdiv(&self) -> &MrccCtimer0Clkdiv {
        &self.mrcc_ctimer0_clkdiv
    }
    #[doc = "0xb0 - CTIMER1 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_ctimer1_clksel(&self) -> &MrccCtimer1Clksel {
        &self.mrcc_ctimer1_clksel
    }
    #[doc = "0xb4 - CTIMER1 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_ctimer1_clkdiv(&self) -> &MrccCtimer1Clkdiv {
        &self.mrcc_ctimer1_clkdiv
    }
    #[doc = "0xb8 - CTIMER2 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_ctimer2_clksel(&self) -> &MrccCtimer2Clksel {
        &self.mrcc_ctimer2_clksel
    }
    #[doc = "0xbc - CTIMER2 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_ctimer2_clkdiv(&self) -> &MrccCtimer2Clkdiv {
        &self.mrcc_ctimer2_clkdiv
    }
    #[doc = "0xc0 - CTIMER3 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_ctimer3_clksel(&self) -> &MrccCtimer3Clksel {
        &self.mrcc_ctimer3_clksel
    }
    #[doc = "0xc4 - CTIMER3 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_ctimer3_clkdiv(&self) -> &MrccCtimer3Clkdiv {
        &self.mrcc_ctimer3_clkdiv
    }
    #[doc = "0xc8 - CTIMER4 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_ctimer4_clksel(&self) -> &MrccCtimer4Clksel {
        &self.mrcc_ctimer4_clksel
    }
    #[doc = "0xcc - CTIMER4 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_ctimer4_clkdiv(&self) -> &MrccCtimer4Clkdiv {
        &self.mrcc_ctimer4_clkdiv
    }
    #[doc = "0xd4 - WWDT0 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_wwdt0_clkdiv(&self) -> &MrccWwdt0Clkdiv {
        &self.mrcc_wwdt0_clkdiv
    }
    #[doc = "0xd8 - FLEXIO0 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_flexio0_clksel(&self) -> &MrccFlexio0Clksel {
        &self.mrcc_flexio0_clksel
    }
    #[doc = "0xdc - FLEXIO0 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_flexio0_clkdiv(&self) -> &MrccFlexio0Clkdiv {
        &self.mrcc_flexio0_clkdiv
    }
    #[doc = "0xe0 - LPI2C0 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_lpi2c0_clksel(&self) -> &MrccLpi2c0Clksel {
        &self.mrcc_lpi2c0_clksel
    }
    #[doc = "0xe4 - LPI2C0 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_lpi2c0_clkdiv(&self) -> &MrccLpi2c0Clkdiv {
        &self.mrcc_lpi2c0_clkdiv
    }
    #[doc = "0xe8 - LPI2C1 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_lpi2c1_clksel(&self) -> &MrccLpi2c1Clksel {
        &self.mrcc_lpi2c1_clksel
    }
    #[doc = "0xec - LPI2C1 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_lpi2c1_clkdiv(&self) -> &MrccLpi2c1Clkdiv {
        &self.mrcc_lpi2c1_clkdiv
    }
    #[doc = "0xf0 - LPSPI0 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_lpspi0_clksel(&self) -> &MrccLpspi0Clksel {
        &self.mrcc_lpspi0_clksel
    }
    #[doc = "0xf4 - LPSPI0 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_lpspi0_clkdiv(&self) -> &MrccLpspi0Clkdiv {
        &self.mrcc_lpspi0_clkdiv
    }
    #[doc = "0xf8 - LPSPI1 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_lpspi1_clksel(&self) -> &MrccLpspi1Clksel {
        &self.mrcc_lpspi1_clksel
    }
    #[doc = "0xfc - LPSPI1 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_lpspi1_clkdiv(&self) -> &MrccLpspi1Clkdiv {
        &self.mrcc_lpspi1_clkdiv
    }
    #[doc = "0x100 - LPUART0 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_lpuart0_clksel(&self) -> &MrccLpuart0Clksel {
        &self.mrcc_lpuart0_clksel
    }
    #[doc = "0x104 - LPUART0 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_lpuart0_clkdiv(&self) -> &MrccLpuart0Clkdiv {
        &self.mrcc_lpuart0_clkdiv
    }
    #[doc = "0x108 - LPUART1 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_lpuart1_clksel(&self) -> &MrccLpuart1Clksel {
        &self.mrcc_lpuart1_clksel
    }
    #[doc = "0x10c - LPUART1 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_lpuart1_clkdiv(&self) -> &MrccLpuart1Clkdiv {
        &self.mrcc_lpuart1_clkdiv
    }
    #[doc = "0x110 - LPUART2 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_lpuart2_clksel(&self) -> &MrccLpuart2Clksel {
        &self.mrcc_lpuart2_clksel
    }
    #[doc = "0x114 - LPUART2 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_lpuart2_clkdiv(&self) -> &MrccLpuart2Clkdiv {
        &self.mrcc_lpuart2_clkdiv
    }
    #[doc = "0x118 - LPUART3 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_lpuart3_clksel(&self) -> &MrccLpuart3Clksel {
        &self.mrcc_lpuart3_clksel
    }
    #[doc = "0x11c - LPUART3 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_lpuart3_clkdiv(&self) -> &MrccLpuart3Clkdiv {
        &self.mrcc_lpuart3_clkdiv
    }
    #[doc = "0x120 - LPUART4 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_lpuart4_clksel(&self) -> &MrccLpuart4Clksel {
        &self.mrcc_lpuart4_clksel
    }
    #[doc = "0x124 - LPUART4 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_lpuart4_clkdiv(&self) -> &MrccLpuart4Clkdiv {
        &self.mrcc_lpuart4_clkdiv
    }
    #[doc = "0x128 - USB0 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_usb0_clksel(&self) -> &MrccUsb0Clksel {
        &self.mrcc_usb0_clksel
    }
    #[doc = "0x12c - USB0 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_usb0_clkdiv(&self) -> &MrccUsb0Clkdiv {
        &self.mrcc_usb0_clkdiv
    }
    #[doc = "0x130 - LPTMR0 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_lptmr0_clksel(&self) -> &MrccLptmr0Clksel {
        &self.mrcc_lptmr0_clksel
    }
    #[doc = "0x134 - LPTMR0 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_lptmr0_clkdiv(&self) -> &MrccLptmr0Clkdiv {
        &self.mrcc_lptmr0_clkdiv
    }
    #[doc = "0x138 - OSTIMER0 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_ostimer0_clksel(&self) -> &MrccOstimer0Clksel {
        &self.mrcc_ostimer0_clksel
    }
    #[doc = "0x140 - ADCx clock selection control"]
    #[inline(always)]
    pub const fn mrcc_adc_clksel(&self) -> &MrccAdcClksel {
        &self.mrcc_adc_clksel
    }
    #[doc = "0x144 - ADCx clock divider control"]
    #[inline(always)]
    pub const fn mrcc_adc_clkdiv(&self) -> &MrccAdcClkdiv {
        &self.mrcc_adc_clkdiv
    }
    #[doc = "0x14c - CMP0_FUNC clock divider control"]
    #[inline(always)]
    pub const fn mrcc_cmp0_func_clkdiv(&self) -> &MrccCmp0FuncClkdiv {
        &self.mrcc_cmp0_func_clkdiv
    }
    #[doc = "0x150 - CMP0_RR clock selection control"]
    #[inline(always)]
    pub const fn mrcc_cmp0_rr_clksel(&self) -> &MrccCmp0RrClksel {
        &self.mrcc_cmp0_rr_clksel
    }
    #[doc = "0x154 - CMP0_RR clock divider control"]
    #[inline(always)]
    pub const fn mrcc_cmp0_rr_clkdiv(&self) -> &MrccCmp0RrClkdiv {
        &self.mrcc_cmp0_rr_clkdiv
    }
    #[doc = "0x15c - CMP1_FUNC clock divider control"]
    #[inline(always)]
    pub const fn mrcc_cmp1_func_clkdiv(&self) -> &MrccCmp1FuncClkdiv {
        &self.mrcc_cmp1_func_clkdiv
    }
    #[doc = "0x160 - CMP1_RR clock selection control"]
    #[inline(always)]
    pub const fn mrcc_cmp1_rr_clksel(&self) -> &MrccCmp1RrClksel {
        &self.mrcc_cmp1_rr_clksel
    }
    #[doc = "0x164 - CMP1_RR clock divider control"]
    #[inline(always)]
    pub const fn mrcc_cmp1_rr_clkdiv(&self) -> &MrccCmp1RrClkdiv {
        &self.mrcc_cmp1_rr_clkdiv
    }
    #[doc = "0x16c - CMP2_FUNC clock divider control"]
    #[inline(always)]
    pub const fn mrcc_cmp2_func_clkdiv(&self) -> &MrccCmp2FuncClkdiv {
        &self.mrcc_cmp2_func_clkdiv
    }
    #[doc = "0x170 - CMP2_RR clock selection control"]
    #[inline(always)]
    pub const fn mrcc_cmp2_rr_clksel(&self) -> &MrccCmp2RrClksel {
        &self.mrcc_cmp2_rr_clksel
    }
    #[doc = "0x174 - CMP2_RR clock divider control"]
    #[inline(always)]
    pub const fn mrcc_cmp2_rr_clkdiv(&self) -> &MrccCmp2RrClkdiv {
        &self.mrcc_cmp2_rr_clkdiv
    }
    #[doc = "0x178 - DAC0 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_dac0_clksel(&self) -> &MrccDac0Clksel {
        &self.mrcc_dac0_clksel
    }
    #[doc = "0x17c - DAC0 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_dac0_clkdiv(&self) -> &MrccDac0Clkdiv {
        &self.mrcc_dac0_clkdiv
    }
    #[doc = "0x180 - FLEXCAN0 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_flexcan0_clksel(&self) -> &MrccFlexcan0Clksel {
        &self.mrcc_flexcan0_clksel
    }
    #[doc = "0x184 - FLEXCAN0 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_flexcan0_clkdiv(&self) -> &MrccFlexcan0Clkdiv {
        &self.mrcc_flexcan0_clkdiv
    }
    #[doc = "0x188 - FLEXCAN1 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_flexcan1_clksel(&self) -> &MrccFlexcan1Clksel {
        &self.mrcc_flexcan1_clksel
    }
    #[doc = "0x18c - FLEXCAN1 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_flexcan1_clkdiv(&self) -> &MrccFlexcan1Clkdiv {
        &self.mrcc_flexcan1_clkdiv
    }
    #[doc = "0x190 - LPI2C2 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_lpi2c2_clksel(&self) -> &MrccLpi2c2Clksel {
        &self.mrcc_lpi2c2_clksel
    }
    #[doc = "0x194 - LPI2C2 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_lpi2c2_clkdiv(&self) -> &MrccLpi2c2Clkdiv {
        &self.mrcc_lpi2c2_clkdiv
    }
    #[doc = "0x198 - LPI2C3 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_lpi2c3_clksel(&self) -> &MrccLpi2c3Clksel {
        &self.mrcc_lpi2c3_clksel
    }
    #[doc = "0x19c - LPI2C3 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_lpi2c3_clkdiv(&self) -> &MrccLpi2c3Clkdiv {
        &self.mrcc_lpi2c3_clkdiv
    }
    #[doc = "0x1a0 - LPUART5 clock selection control"]
    #[inline(always)]
    pub const fn mrcc_lpuart5_clksel(&self) -> &MrccLpuart5Clksel {
        &self.mrcc_lpuart5_clksel
    }
    #[doc = "0x1a4 - LPUART5 clock divider control"]
    #[inline(always)]
    pub const fn mrcc_lpuart5_clkdiv(&self) -> &MrccLpuart5Clkdiv {
        &self.mrcc_lpuart5_clkdiv
    }
    #[doc = "0x1a8 - DBG_TRACE clock selection control"]
    #[inline(always)]
    pub const fn mrcc_dbg_trace_clksel(&self) -> &MrccDbgTraceClksel {
        &self.mrcc_dbg_trace_clksel
    }
    #[doc = "0x1ac - DBG_TRACE clock divider control"]
    #[inline(always)]
    pub const fn mrcc_dbg_trace_clkdiv(&self) -> &MrccDbgTraceClkdiv {
        &self.mrcc_dbg_trace_clkdiv
    }
    #[doc = "0x1b0 - CLKOUT clock selection control"]
    #[inline(always)]
    pub const fn mrcc_clkout_clksel(&self) -> &MrccClkoutClksel {
        &self.mrcc_clkout_clksel
    }
    #[doc = "0x1b4 - CLKOUT clock divider control"]
    #[inline(always)]
    pub const fn mrcc_clkout_clkdiv(&self) -> &MrccClkoutClkdiv {
        &self.mrcc_clkout_clkdiv
    }
    #[doc = "0x1b8 - SYSTICK clock selection control"]
    #[inline(always)]
    pub const fn mrcc_systick_clksel(&self) -> &MrccSystickClksel {
        &self.mrcc_systick_clksel
    }
    #[doc = "0x1bc - SYSTICK clock divider control"]
    #[inline(always)]
    pub const fn mrcc_systick_clkdiv(&self) -> &MrccSystickClkdiv {
        &self.mrcc_systick_clkdiv
    }
}
#[doc = "MRCC_GLB_RST0 (rw) register accessor: Peripheral Reset Control 0\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_glb_rst0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_rst0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_rst0`] module"]
#[doc(alias = "MRCC_GLB_RST0")]
pub type MrccGlbRst0 = crate::Reg<mrcc_glb_rst0::MrccGlbRst0Spec>;
#[doc = "Peripheral Reset Control 0"]
pub mod mrcc_glb_rst0;
#[doc = "MRCC_GLB_RST0_SET (w) register accessor: Peripheral Reset Control Set 0\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_rst0_set::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_rst0_set`] module"]
#[doc(alias = "MRCC_GLB_RST0_SET")]
pub type MrccGlbRst0Set = crate::Reg<mrcc_glb_rst0_set::MrccGlbRst0SetSpec>;
#[doc = "Peripheral Reset Control Set 0"]
pub mod mrcc_glb_rst0_set;
#[doc = "MRCC_GLB_RST0_CLR (w) register accessor: Peripheral Reset Control Clear 0\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_rst0_clr::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_rst0_clr`] module"]
#[doc(alias = "MRCC_GLB_RST0_CLR")]
pub type MrccGlbRst0Clr = crate::Reg<mrcc_glb_rst0_clr::MrccGlbRst0ClrSpec>;
#[doc = "Peripheral Reset Control Clear 0"]
pub mod mrcc_glb_rst0_clr;
#[doc = "MRCC_GLB_RST1 (rw) register accessor: Peripheral Reset Control 1\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_glb_rst1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_rst1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_rst1`] module"]
#[doc(alias = "MRCC_GLB_RST1")]
pub type MrccGlbRst1 = crate::Reg<mrcc_glb_rst1::MrccGlbRst1Spec>;
#[doc = "Peripheral Reset Control 1"]
pub mod mrcc_glb_rst1;
#[doc = "MRCC_GLB_RST1_SET (w) register accessor: Peripheral Reset Control Set 1\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_rst1_set::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_rst1_set`] module"]
#[doc(alias = "MRCC_GLB_RST1_SET")]
pub type MrccGlbRst1Set = crate::Reg<mrcc_glb_rst1_set::MrccGlbRst1SetSpec>;
#[doc = "Peripheral Reset Control Set 1"]
pub mod mrcc_glb_rst1_set;
#[doc = "MRCC_GLB_RST1_CLR (w) register accessor: Peripheral Reset Control Clear 1\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_rst1_clr::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_rst1_clr`] module"]
#[doc(alias = "MRCC_GLB_RST1_CLR")]
pub type MrccGlbRst1Clr = crate::Reg<mrcc_glb_rst1_clr::MrccGlbRst1ClrSpec>;
#[doc = "Peripheral Reset Control Clear 1"]
pub mod mrcc_glb_rst1_clr;
#[doc = "MRCC_GLB_RST2 (rw) register accessor: Peripheral Reset Control 2\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_glb_rst2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_rst2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_rst2`] module"]
#[doc(alias = "MRCC_GLB_RST2")]
pub type MrccGlbRst2 = crate::Reg<mrcc_glb_rst2::MrccGlbRst2Spec>;
#[doc = "Peripheral Reset Control 2"]
pub mod mrcc_glb_rst2;
#[doc = "MRCC_GLB_RST2_SET (w) register accessor: Peripheral Reset Control Set 2\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_rst2_set::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_rst2_set`] module"]
#[doc(alias = "MRCC_GLB_RST2_SET")]
pub type MrccGlbRst2Set = crate::Reg<mrcc_glb_rst2_set::MrccGlbRst2SetSpec>;
#[doc = "Peripheral Reset Control Set 2"]
pub mod mrcc_glb_rst2_set;
#[doc = "MRCC_GLB_RST2_CLR (w) register accessor: Peripheral Reset Control Clear 2\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_rst2_clr::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_rst2_clr`] module"]
#[doc(alias = "MRCC_GLB_RST2_CLR")]
pub type MrccGlbRst2Clr = crate::Reg<mrcc_glb_rst2_clr::MrccGlbRst2ClrSpec>;
#[doc = "Peripheral Reset Control Clear 2"]
pub mod mrcc_glb_rst2_clr;
#[doc = "MRCC_GLB_CC0 (rw) register accessor: AHB Clock Control 0\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_glb_cc0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_cc0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_cc0`] module"]
#[doc(alias = "MRCC_GLB_CC0")]
pub type MrccGlbCc0 = crate::Reg<mrcc_glb_cc0::MrccGlbCc0Spec>;
#[doc = "AHB Clock Control 0"]
pub mod mrcc_glb_cc0;
#[doc = "MRCC_GLB_CC0_SET (w) register accessor: AHB Clock Control Set 0\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_cc0_set::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_cc0_set`] module"]
#[doc(alias = "MRCC_GLB_CC0_SET")]
pub type MrccGlbCc0Set = crate::Reg<mrcc_glb_cc0_set::MrccGlbCc0SetSpec>;
#[doc = "AHB Clock Control Set 0"]
pub mod mrcc_glb_cc0_set;
#[doc = "MRCC_GLB_CC0_CLR (w) register accessor: AHB Clock Control Clear 0\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_cc0_clr::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_cc0_clr`] module"]
#[doc(alias = "MRCC_GLB_CC0_CLR")]
pub type MrccGlbCc0Clr = crate::Reg<mrcc_glb_cc0_clr::MrccGlbCc0ClrSpec>;
#[doc = "AHB Clock Control Clear 0"]
pub mod mrcc_glb_cc0_clr;
#[doc = "MRCC_GLB_CC1 (rw) register accessor: AHB Clock Control 1\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_glb_cc1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_cc1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_cc1`] module"]
#[doc(alias = "MRCC_GLB_CC1")]
pub type MrccGlbCc1 = crate::Reg<mrcc_glb_cc1::MrccGlbCc1Spec>;
#[doc = "AHB Clock Control 1"]
pub mod mrcc_glb_cc1;
#[doc = "MRCC_GLB_CC1_SET (w) register accessor: AHB Clock Control Set 1\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_cc1_set::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_cc1_set`] module"]
#[doc(alias = "MRCC_GLB_CC1_SET")]
pub type MrccGlbCc1Set = crate::Reg<mrcc_glb_cc1_set::MrccGlbCc1SetSpec>;
#[doc = "AHB Clock Control Set 1"]
pub mod mrcc_glb_cc1_set;
#[doc = "MRCC_GLB_CC1_CLR (w) register accessor: AHB Clock Control Clear 1\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_cc1_clr::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_cc1_clr`] module"]
#[doc(alias = "MRCC_GLB_CC1_CLR")]
pub type MrccGlbCc1Clr = crate::Reg<mrcc_glb_cc1_clr::MrccGlbCc1ClrSpec>;
#[doc = "AHB Clock Control Clear 1"]
pub mod mrcc_glb_cc1_clr;
#[doc = "MRCC_GLB_CC2 (rw) register accessor: AHB Clock Control 2\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_glb_cc2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_cc2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_cc2`] module"]
#[doc(alias = "MRCC_GLB_CC2")]
pub type MrccGlbCc2 = crate::Reg<mrcc_glb_cc2::MrccGlbCc2Spec>;
#[doc = "AHB Clock Control 2"]
pub mod mrcc_glb_cc2;
#[doc = "MRCC_GLB_CC2_SET (w) register accessor: AHB Clock Control Set 2\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_cc2_set::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_cc2_set`] module"]
#[doc(alias = "MRCC_GLB_CC2_SET")]
pub type MrccGlbCc2Set = crate::Reg<mrcc_glb_cc2_set::MrccGlbCc2SetSpec>;
#[doc = "AHB Clock Control Set 2"]
pub mod mrcc_glb_cc2_set;
#[doc = "MRCC_GLB_CC2_CLR (w) register accessor: AHB Clock Control Clear 2\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_cc2_clr::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_cc2_clr`] module"]
#[doc(alias = "MRCC_GLB_CC2_CLR")]
pub type MrccGlbCc2Clr = crate::Reg<mrcc_glb_cc2_clr::MrccGlbCc2ClrSpec>;
#[doc = "AHB Clock Control Clear 2"]
pub mod mrcc_glb_cc2_clr;
#[doc = "MRCC_GLB_ACC0 (rw) register accessor: Control Automatic Clock Gating 0\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_glb_acc0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_acc0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_acc0`] module"]
#[doc(alias = "MRCC_GLB_ACC0")]
pub type MrccGlbAcc0 = crate::Reg<mrcc_glb_acc0::MrccGlbAcc0Spec>;
#[doc = "Control Automatic Clock Gating 0"]
pub mod mrcc_glb_acc0;
#[doc = "MRCC_GLB_ACC1 (rw) register accessor: Control Automatic Clock Gating 1\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_glb_acc1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_acc1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_acc1`] module"]
#[doc(alias = "MRCC_GLB_ACC1")]
pub type MrccGlbAcc1 = crate::Reg<mrcc_glb_acc1::MrccGlbAcc1Spec>;
#[doc = "Control Automatic Clock Gating 1"]
pub mod mrcc_glb_acc1;
#[doc = "MRCC_GLB_ACC2 (rw) register accessor: Control Automatic Clock Gating 2\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_glb_acc2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_acc2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_glb_acc2`] module"]
#[doc(alias = "MRCC_GLB_ACC2")]
pub type MrccGlbAcc2 = crate::Reg<mrcc_glb_acc2::MrccGlbAcc2Spec>;
#[doc = "Control Automatic Clock Gating 2"]
pub mod mrcc_glb_acc2;
#[doc = "MRCC_I3C0_FCLK_CLKSEL (rw) register accessor: I3C0_FCLK clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_i3c0_fclk_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_i3c0_fclk_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_i3c0_fclk_clksel`] module"]
#[doc(alias = "MRCC_I3C0_FCLK_CLKSEL")]
pub type MrccI3c0FclkClksel = crate::Reg<mrcc_i3c0_fclk_clksel::MrccI3c0FclkClkselSpec>;
#[doc = "I3C0_FCLK clock selection control"]
pub mod mrcc_i3c0_fclk_clksel;
#[doc = "MRCC_I3C0_FCLK_CLKDIV (rw) register accessor: I3C0_FCLK clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_i3c0_fclk_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_i3c0_fclk_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_i3c0_fclk_clkdiv`] module"]
#[doc(alias = "MRCC_I3C0_FCLK_CLKDIV")]
pub type MrccI3c0FclkClkdiv = crate::Reg<mrcc_i3c0_fclk_clkdiv::MrccI3c0FclkClkdivSpec>;
#[doc = "I3C0_FCLK clock divider control"]
pub mod mrcc_i3c0_fclk_clkdiv;
#[doc = "MRCC_CTIMER0_CLKSEL (rw) register accessor: CTIMER0 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_ctimer0_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_ctimer0_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_ctimer0_clksel`] module"]
#[doc(alias = "MRCC_CTIMER0_CLKSEL")]
pub type MrccCtimer0Clksel = crate::Reg<mrcc_ctimer0_clksel::MrccCtimer0ClkselSpec>;
#[doc = "CTIMER0 clock selection control"]
pub mod mrcc_ctimer0_clksel;
#[doc = "MRCC_CTIMER0_CLKDIV (rw) register accessor: CTIMER0 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_ctimer0_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_ctimer0_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_ctimer0_clkdiv`] module"]
#[doc(alias = "MRCC_CTIMER0_CLKDIV")]
pub type MrccCtimer0Clkdiv = crate::Reg<mrcc_ctimer0_clkdiv::MrccCtimer0ClkdivSpec>;
#[doc = "CTIMER0 clock divider control"]
pub mod mrcc_ctimer0_clkdiv;
#[doc = "MRCC_CTIMER1_CLKSEL (rw) register accessor: CTIMER1 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_ctimer1_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_ctimer1_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_ctimer1_clksel`] module"]
#[doc(alias = "MRCC_CTIMER1_CLKSEL")]
pub type MrccCtimer1Clksel = crate::Reg<mrcc_ctimer1_clksel::MrccCtimer1ClkselSpec>;
#[doc = "CTIMER1 clock selection control"]
pub mod mrcc_ctimer1_clksel;
#[doc = "MRCC_CTIMER1_CLKDIV (rw) register accessor: CTIMER1 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_ctimer1_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_ctimer1_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_ctimer1_clkdiv`] module"]
#[doc(alias = "MRCC_CTIMER1_CLKDIV")]
pub type MrccCtimer1Clkdiv = crate::Reg<mrcc_ctimer1_clkdiv::MrccCtimer1ClkdivSpec>;
#[doc = "CTIMER1 clock divider control"]
pub mod mrcc_ctimer1_clkdiv;
#[doc = "MRCC_CTIMER2_CLKSEL (rw) register accessor: CTIMER2 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_ctimer2_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_ctimer2_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_ctimer2_clksel`] module"]
#[doc(alias = "MRCC_CTIMER2_CLKSEL")]
pub type MrccCtimer2Clksel = crate::Reg<mrcc_ctimer2_clksel::MrccCtimer2ClkselSpec>;
#[doc = "CTIMER2 clock selection control"]
pub mod mrcc_ctimer2_clksel;
#[doc = "MRCC_CTIMER2_CLKDIV (rw) register accessor: CTIMER2 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_ctimer2_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_ctimer2_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_ctimer2_clkdiv`] module"]
#[doc(alias = "MRCC_CTIMER2_CLKDIV")]
pub type MrccCtimer2Clkdiv = crate::Reg<mrcc_ctimer2_clkdiv::MrccCtimer2ClkdivSpec>;
#[doc = "CTIMER2 clock divider control"]
pub mod mrcc_ctimer2_clkdiv;
#[doc = "MRCC_CTIMER3_CLKSEL (rw) register accessor: CTIMER3 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_ctimer3_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_ctimer3_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_ctimer3_clksel`] module"]
#[doc(alias = "MRCC_CTIMER3_CLKSEL")]
pub type MrccCtimer3Clksel = crate::Reg<mrcc_ctimer3_clksel::MrccCtimer3ClkselSpec>;
#[doc = "CTIMER3 clock selection control"]
pub mod mrcc_ctimer3_clksel;
#[doc = "MRCC_CTIMER3_CLKDIV (rw) register accessor: CTIMER3 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_ctimer3_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_ctimer3_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_ctimer3_clkdiv`] module"]
#[doc(alias = "MRCC_CTIMER3_CLKDIV")]
pub type MrccCtimer3Clkdiv = crate::Reg<mrcc_ctimer3_clkdiv::MrccCtimer3ClkdivSpec>;
#[doc = "CTIMER3 clock divider control"]
pub mod mrcc_ctimer3_clkdiv;
#[doc = "MRCC_CTIMER4_CLKSEL (rw) register accessor: CTIMER4 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_ctimer4_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_ctimer4_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_ctimer4_clksel`] module"]
#[doc(alias = "MRCC_CTIMER4_CLKSEL")]
pub type MrccCtimer4Clksel = crate::Reg<mrcc_ctimer4_clksel::MrccCtimer4ClkselSpec>;
#[doc = "CTIMER4 clock selection control"]
pub mod mrcc_ctimer4_clksel;
#[doc = "MRCC_CTIMER4_CLKDIV (rw) register accessor: CTIMER4 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_ctimer4_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_ctimer4_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_ctimer4_clkdiv`] module"]
#[doc(alias = "MRCC_CTIMER4_CLKDIV")]
pub type MrccCtimer4Clkdiv = crate::Reg<mrcc_ctimer4_clkdiv::MrccCtimer4ClkdivSpec>;
#[doc = "CTIMER4 clock divider control"]
pub mod mrcc_ctimer4_clkdiv;
#[doc = "MRCC_WWDT0_CLKDIV (rw) register accessor: WWDT0 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_wwdt0_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_wwdt0_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_wwdt0_clkdiv`] module"]
#[doc(alias = "MRCC_WWDT0_CLKDIV")]
pub type MrccWwdt0Clkdiv = crate::Reg<mrcc_wwdt0_clkdiv::MrccWwdt0ClkdivSpec>;
#[doc = "WWDT0 clock divider control"]
pub mod mrcc_wwdt0_clkdiv;
#[doc = "MRCC_FLEXIO0_CLKSEL (rw) register accessor: FLEXIO0 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_flexio0_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_flexio0_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_flexio0_clksel`] module"]
#[doc(alias = "MRCC_FLEXIO0_CLKSEL")]
pub type MrccFlexio0Clksel = crate::Reg<mrcc_flexio0_clksel::MrccFlexio0ClkselSpec>;
#[doc = "FLEXIO0 clock selection control"]
pub mod mrcc_flexio0_clksel;
#[doc = "MRCC_FLEXIO0_CLKDIV (rw) register accessor: FLEXIO0 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_flexio0_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_flexio0_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_flexio0_clkdiv`] module"]
#[doc(alias = "MRCC_FLEXIO0_CLKDIV")]
pub type MrccFlexio0Clkdiv = crate::Reg<mrcc_flexio0_clkdiv::MrccFlexio0ClkdivSpec>;
#[doc = "FLEXIO0 clock divider control"]
pub mod mrcc_flexio0_clkdiv;
#[doc = "MRCC_LPI2C0_CLKSEL (rw) register accessor: LPI2C0 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpi2c0_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpi2c0_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpi2c0_clksel`] module"]
#[doc(alias = "MRCC_LPI2C0_CLKSEL")]
pub type MrccLpi2c0Clksel = crate::Reg<mrcc_lpi2c0_clksel::MrccLpi2c0ClkselSpec>;
#[doc = "LPI2C0 clock selection control"]
pub mod mrcc_lpi2c0_clksel;
#[doc = "MRCC_LPI2C0_CLKDIV (rw) register accessor: LPI2C0 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpi2c0_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpi2c0_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpi2c0_clkdiv`] module"]
#[doc(alias = "MRCC_LPI2C0_CLKDIV")]
pub type MrccLpi2c0Clkdiv = crate::Reg<mrcc_lpi2c0_clkdiv::MrccLpi2c0ClkdivSpec>;
#[doc = "LPI2C0 clock divider control"]
pub mod mrcc_lpi2c0_clkdiv;
#[doc = "MRCC_LPI2C1_CLKSEL (rw) register accessor: LPI2C1 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpi2c1_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpi2c1_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpi2c1_clksel`] module"]
#[doc(alias = "MRCC_LPI2C1_CLKSEL")]
pub type MrccLpi2c1Clksel = crate::Reg<mrcc_lpi2c1_clksel::MrccLpi2c1ClkselSpec>;
#[doc = "LPI2C1 clock selection control"]
pub mod mrcc_lpi2c1_clksel;
#[doc = "MRCC_LPI2C1_CLKDIV (rw) register accessor: LPI2C1 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpi2c1_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpi2c1_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpi2c1_clkdiv`] module"]
#[doc(alias = "MRCC_LPI2C1_CLKDIV")]
pub type MrccLpi2c1Clkdiv = crate::Reg<mrcc_lpi2c1_clkdiv::MrccLpi2c1ClkdivSpec>;
#[doc = "LPI2C1 clock divider control"]
pub mod mrcc_lpi2c1_clkdiv;
#[doc = "MRCC_LPSPI0_CLKSEL (rw) register accessor: LPSPI0 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpspi0_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpspi0_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpspi0_clksel`] module"]
#[doc(alias = "MRCC_LPSPI0_CLKSEL")]
pub type MrccLpspi0Clksel = crate::Reg<mrcc_lpspi0_clksel::MrccLpspi0ClkselSpec>;
#[doc = "LPSPI0 clock selection control"]
pub mod mrcc_lpspi0_clksel;
#[doc = "MRCC_LPSPI0_CLKDIV (rw) register accessor: LPSPI0 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpspi0_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpspi0_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpspi0_clkdiv`] module"]
#[doc(alias = "MRCC_LPSPI0_CLKDIV")]
pub type MrccLpspi0Clkdiv = crate::Reg<mrcc_lpspi0_clkdiv::MrccLpspi0ClkdivSpec>;
#[doc = "LPSPI0 clock divider control"]
pub mod mrcc_lpspi0_clkdiv;
#[doc = "MRCC_LPSPI1_CLKSEL (rw) register accessor: LPSPI1 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpspi1_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpspi1_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpspi1_clksel`] module"]
#[doc(alias = "MRCC_LPSPI1_CLKSEL")]
pub type MrccLpspi1Clksel = crate::Reg<mrcc_lpspi1_clksel::MrccLpspi1ClkselSpec>;
#[doc = "LPSPI1 clock selection control"]
pub mod mrcc_lpspi1_clksel;
#[doc = "MRCC_LPSPI1_CLKDIV (rw) register accessor: LPSPI1 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpspi1_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpspi1_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpspi1_clkdiv`] module"]
#[doc(alias = "MRCC_LPSPI1_CLKDIV")]
pub type MrccLpspi1Clkdiv = crate::Reg<mrcc_lpspi1_clkdiv::MrccLpspi1ClkdivSpec>;
#[doc = "LPSPI1 clock divider control"]
pub mod mrcc_lpspi1_clkdiv;
#[doc = "MRCC_LPUART0_CLKSEL (rw) register accessor: LPUART0 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpuart0_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpuart0_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpuart0_clksel`] module"]
#[doc(alias = "MRCC_LPUART0_CLKSEL")]
pub type MrccLpuart0Clksel = crate::Reg<mrcc_lpuart0_clksel::MrccLpuart0ClkselSpec>;
#[doc = "LPUART0 clock selection control"]
pub mod mrcc_lpuart0_clksel;
#[doc = "MRCC_LPUART0_CLKDIV (rw) register accessor: LPUART0 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpuart0_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpuart0_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpuart0_clkdiv`] module"]
#[doc(alias = "MRCC_LPUART0_CLKDIV")]
pub type MrccLpuart0Clkdiv = crate::Reg<mrcc_lpuart0_clkdiv::MrccLpuart0ClkdivSpec>;
#[doc = "LPUART0 clock divider control"]
pub mod mrcc_lpuart0_clkdiv;
#[doc = "MRCC_LPUART1_CLKSEL (rw) register accessor: LPUART1 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpuart1_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpuart1_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpuart1_clksel`] module"]
#[doc(alias = "MRCC_LPUART1_CLKSEL")]
pub type MrccLpuart1Clksel = crate::Reg<mrcc_lpuart1_clksel::MrccLpuart1ClkselSpec>;
#[doc = "LPUART1 clock selection control"]
pub mod mrcc_lpuart1_clksel;
#[doc = "MRCC_LPUART1_CLKDIV (rw) register accessor: LPUART1 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpuart1_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpuart1_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpuart1_clkdiv`] module"]
#[doc(alias = "MRCC_LPUART1_CLKDIV")]
pub type MrccLpuart1Clkdiv = crate::Reg<mrcc_lpuart1_clkdiv::MrccLpuart1ClkdivSpec>;
#[doc = "LPUART1 clock divider control"]
pub mod mrcc_lpuart1_clkdiv;
#[doc = "MRCC_LPUART2_CLKSEL (rw) register accessor: LPUART2 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpuart2_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpuart2_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpuart2_clksel`] module"]
#[doc(alias = "MRCC_LPUART2_CLKSEL")]
pub type MrccLpuart2Clksel = crate::Reg<mrcc_lpuart2_clksel::MrccLpuart2ClkselSpec>;
#[doc = "LPUART2 clock selection control"]
pub mod mrcc_lpuart2_clksel;
#[doc = "MRCC_LPUART2_CLKDIV (rw) register accessor: LPUART2 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpuart2_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpuart2_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpuart2_clkdiv`] module"]
#[doc(alias = "MRCC_LPUART2_CLKDIV")]
pub type MrccLpuart2Clkdiv = crate::Reg<mrcc_lpuart2_clkdiv::MrccLpuart2ClkdivSpec>;
#[doc = "LPUART2 clock divider control"]
pub mod mrcc_lpuart2_clkdiv;
#[doc = "MRCC_LPUART3_CLKSEL (rw) register accessor: LPUART3 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpuart3_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpuart3_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpuart3_clksel`] module"]
#[doc(alias = "MRCC_LPUART3_CLKSEL")]
pub type MrccLpuart3Clksel = crate::Reg<mrcc_lpuart3_clksel::MrccLpuart3ClkselSpec>;
#[doc = "LPUART3 clock selection control"]
pub mod mrcc_lpuart3_clksel;
#[doc = "MRCC_LPUART3_CLKDIV (rw) register accessor: LPUART3 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpuart3_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpuart3_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpuart3_clkdiv`] module"]
#[doc(alias = "MRCC_LPUART3_CLKDIV")]
pub type MrccLpuart3Clkdiv = crate::Reg<mrcc_lpuart3_clkdiv::MrccLpuart3ClkdivSpec>;
#[doc = "LPUART3 clock divider control"]
pub mod mrcc_lpuart3_clkdiv;
#[doc = "MRCC_LPUART4_CLKSEL (rw) register accessor: LPUART4 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpuart4_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpuart4_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpuart4_clksel`] module"]
#[doc(alias = "MRCC_LPUART4_CLKSEL")]
pub type MrccLpuart4Clksel = crate::Reg<mrcc_lpuart4_clksel::MrccLpuart4ClkselSpec>;
#[doc = "LPUART4 clock selection control"]
pub mod mrcc_lpuart4_clksel;
#[doc = "MRCC_LPUART4_CLKDIV (rw) register accessor: LPUART4 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpuart4_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpuart4_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpuart4_clkdiv`] module"]
#[doc(alias = "MRCC_LPUART4_CLKDIV")]
pub type MrccLpuart4Clkdiv = crate::Reg<mrcc_lpuart4_clkdiv::MrccLpuart4ClkdivSpec>;
#[doc = "LPUART4 clock divider control"]
pub mod mrcc_lpuart4_clkdiv;
#[doc = "MRCC_USB0_CLKSEL (rw) register accessor: USB0 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_usb0_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_usb0_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_usb0_clksel`] module"]
#[doc(alias = "MRCC_USB0_CLKSEL")]
pub type MrccUsb0Clksel = crate::Reg<mrcc_usb0_clksel::MrccUsb0ClkselSpec>;
#[doc = "USB0 clock selection control"]
pub mod mrcc_usb0_clksel;
#[doc = "MRCC_USB0_CLKDIV (rw) register accessor: USB0 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_usb0_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_usb0_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_usb0_clkdiv`] module"]
#[doc(alias = "MRCC_USB0_CLKDIV")]
pub type MrccUsb0Clkdiv = crate::Reg<mrcc_usb0_clkdiv::MrccUsb0ClkdivSpec>;
#[doc = "USB0 clock divider control"]
pub mod mrcc_usb0_clkdiv;
#[doc = "MRCC_LPTMR0_CLKSEL (rw) register accessor: LPTMR0 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lptmr0_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lptmr0_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lptmr0_clksel`] module"]
#[doc(alias = "MRCC_LPTMR0_CLKSEL")]
pub type MrccLptmr0Clksel = crate::Reg<mrcc_lptmr0_clksel::MrccLptmr0ClkselSpec>;
#[doc = "LPTMR0 clock selection control"]
pub mod mrcc_lptmr0_clksel;
#[doc = "MRCC_LPTMR0_CLKDIV (rw) register accessor: LPTMR0 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lptmr0_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lptmr0_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lptmr0_clkdiv`] module"]
#[doc(alias = "MRCC_LPTMR0_CLKDIV")]
pub type MrccLptmr0Clkdiv = crate::Reg<mrcc_lptmr0_clkdiv::MrccLptmr0ClkdivSpec>;
#[doc = "LPTMR0 clock divider control"]
pub mod mrcc_lptmr0_clkdiv;
#[doc = "MRCC_OSTIMER0_CLKSEL (rw) register accessor: OSTIMER0 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_ostimer0_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_ostimer0_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_ostimer0_clksel`] module"]
#[doc(alias = "MRCC_OSTIMER0_CLKSEL")]
pub type MrccOstimer0Clksel = crate::Reg<mrcc_ostimer0_clksel::MrccOstimer0ClkselSpec>;
#[doc = "OSTIMER0 clock selection control"]
pub mod mrcc_ostimer0_clksel;
#[doc = "MRCC_ADC_CLKSEL (rw) register accessor: ADCx clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_adc_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_adc_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_adc_clksel`] module"]
#[doc(alias = "MRCC_ADC_CLKSEL")]
pub type MrccAdcClksel = crate::Reg<mrcc_adc_clksel::MrccAdcClkselSpec>;
#[doc = "ADCx clock selection control"]
pub mod mrcc_adc_clksel;
#[doc = "MRCC_ADC_CLKDIV (rw) register accessor: ADCx clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_adc_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_adc_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_adc_clkdiv`] module"]
#[doc(alias = "MRCC_ADC_CLKDIV")]
pub type MrccAdcClkdiv = crate::Reg<mrcc_adc_clkdiv::MrccAdcClkdivSpec>;
#[doc = "ADCx clock divider control"]
pub mod mrcc_adc_clkdiv;
#[doc = "MRCC_CMP0_FUNC_CLKDIV (rw) register accessor: CMP0_FUNC clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_cmp0_func_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_cmp0_func_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_cmp0_func_clkdiv`] module"]
#[doc(alias = "MRCC_CMP0_FUNC_CLKDIV")]
pub type MrccCmp0FuncClkdiv = crate::Reg<mrcc_cmp0_func_clkdiv::MrccCmp0FuncClkdivSpec>;
#[doc = "CMP0_FUNC clock divider control"]
pub mod mrcc_cmp0_func_clkdiv;
#[doc = "MRCC_CMP0_RR_CLKSEL (rw) register accessor: CMP0_RR clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_cmp0_rr_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_cmp0_rr_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_cmp0_rr_clksel`] module"]
#[doc(alias = "MRCC_CMP0_RR_CLKSEL")]
pub type MrccCmp0RrClksel = crate::Reg<mrcc_cmp0_rr_clksel::MrccCmp0RrClkselSpec>;
#[doc = "CMP0_RR clock selection control"]
pub mod mrcc_cmp0_rr_clksel;
#[doc = "MRCC_CMP0_RR_CLKDIV (rw) register accessor: CMP0_RR clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_cmp0_rr_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_cmp0_rr_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_cmp0_rr_clkdiv`] module"]
#[doc(alias = "MRCC_CMP0_RR_CLKDIV")]
pub type MrccCmp0RrClkdiv = crate::Reg<mrcc_cmp0_rr_clkdiv::MrccCmp0RrClkdivSpec>;
#[doc = "CMP0_RR clock divider control"]
pub mod mrcc_cmp0_rr_clkdiv;
#[doc = "MRCC_CMP1_FUNC_CLKDIV (rw) register accessor: CMP1_FUNC clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_cmp1_func_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_cmp1_func_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_cmp1_func_clkdiv`] module"]
#[doc(alias = "MRCC_CMP1_FUNC_CLKDIV")]
pub type MrccCmp1FuncClkdiv = crate::Reg<mrcc_cmp1_func_clkdiv::MrccCmp1FuncClkdivSpec>;
#[doc = "CMP1_FUNC clock divider control"]
pub mod mrcc_cmp1_func_clkdiv;
#[doc = "MRCC_CMP1_RR_CLKSEL (rw) register accessor: CMP1_RR clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_cmp1_rr_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_cmp1_rr_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_cmp1_rr_clksel`] module"]
#[doc(alias = "MRCC_CMP1_RR_CLKSEL")]
pub type MrccCmp1RrClksel = crate::Reg<mrcc_cmp1_rr_clksel::MrccCmp1RrClkselSpec>;
#[doc = "CMP1_RR clock selection control"]
pub mod mrcc_cmp1_rr_clksel;
#[doc = "MRCC_CMP1_RR_CLKDIV (rw) register accessor: CMP1_RR clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_cmp1_rr_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_cmp1_rr_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_cmp1_rr_clkdiv`] module"]
#[doc(alias = "MRCC_CMP1_RR_CLKDIV")]
pub type MrccCmp1RrClkdiv = crate::Reg<mrcc_cmp1_rr_clkdiv::MrccCmp1RrClkdivSpec>;
#[doc = "CMP1_RR clock divider control"]
pub mod mrcc_cmp1_rr_clkdiv;
#[doc = "MRCC_CMP2_FUNC_CLKDIV (rw) register accessor: CMP2_FUNC clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_cmp2_func_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_cmp2_func_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_cmp2_func_clkdiv`] module"]
#[doc(alias = "MRCC_CMP2_FUNC_CLKDIV")]
pub type MrccCmp2FuncClkdiv = crate::Reg<mrcc_cmp2_func_clkdiv::MrccCmp2FuncClkdivSpec>;
#[doc = "CMP2_FUNC clock divider control"]
pub mod mrcc_cmp2_func_clkdiv;
#[doc = "MRCC_CMP2_RR_CLKSEL (rw) register accessor: CMP2_RR clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_cmp2_rr_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_cmp2_rr_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_cmp2_rr_clksel`] module"]
#[doc(alias = "MRCC_CMP2_RR_CLKSEL")]
pub type MrccCmp2RrClksel = crate::Reg<mrcc_cmp2_rr_clksel::MrccCmp2RrClkselSpec>;
#[doc = "CMP2_RR clock selection control"]
pub mod mrcc_cmp2_rr_clksel;
#[doc = "MRCC_CMP2_RR_CLKDIV (rw) register accessor: CMP2_RR clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_cmp2_rr_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_cmp2_rr_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_cmp2_rr_clkdiv`] module"]
#[doc(alias = "MRCC_CMP2_RR_CLKDIV")]
pub type MrccCmp2RrClkdiv = crate::Reg<mrcc_cmp2_rr_clkdiv::MrccCmp2RrClkdivSpec>;
#[doc = "CMP2_RR clock divider control"]
pub mod mrcc_cmp2_rr_clkdiv;
#[doc = "MRCC_DAC0_CLKSEL (rw) register accessor: DAC0 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_dac0_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_dac0_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_dac0_clksel`] module"]
#[doc(alias = "MRCC_DAC0_CLKSEL")]
pub type MrccDac0Clksel = crate::Reg<mrcc_dac0_clksel::MrccDac0ClkselSpec>;
#[doc = "DAC0 clock selection control"]
pub mod mrcc_dac0_clksel;
#[doc = "MRCC_DAC0_CLKDIV (rw) register accessor: DAC0 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_dac0_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_dac0_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_dac0_clkdiv`] module"]
#[doc(alias = "MRCC_DAC0_CLKDIV")]
pub type MrccDac0Clkdiv = crate::Reg<mrcc_dac0_clkdiv::MrccDac0ClkdivSpec>;
#[doc = "DAC0 clock divider control"]
pub mod mrcc_dac0_clkdiv;
#[doc = "MRCC_FLEXCAN0_CLKSEL (rw) register accessor: FLEXCAN0 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_flexcan0_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_flexcan0_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_flexcan0_clksel`] module"]
#[doc(alias = "MRCC_FLEXCAN0_CLKSEL")]
pub type MrccFlexcan0Clksel = crate::Reg<mrcc_flexcan0_clksel::MrccFlexcan0ClkselSpec>;
#[doc = "FLEXCAN0 clock selection control"]
pub mod mrcc_flexcan0_clksel;
#[doc = "MRCC_FLEXCAN0_CLKDIV (rw) register accessor: FLEXCAN0 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_flexcan0_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_flexcan0_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_flexcan0_clkdiv`] module"]
#[doc(alias = "MRCC_FLEXCAN0_CLKDIV")]
pub type MrccFlexcan0Clkdiv = crate::Reg<mrcc_flexcan0_clkdiv::MrccFlexcan0ClkdivSpec>;
#[doc = "FLEXCAN0 clock divider control"]
pub mod mrcc_flexcan0_clkdiv;
#[doc = "MRCC_FLEXCAN1_CLKSEL (rw) register accessor: FLEXCAN1 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_flexcan1_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_flexcan1_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_flexcan1_clksel`] module"]
#[doc(alias = "MRCC_FLEXCAN1_CLKSEL")]
pub type MrccFlexcan1Clksel = crate::Reg<mrcc_flexcan1_clksel::MrccFlexcan1ClkselSpec>;
#[doc = "FLEXCAN1 clock selection control"]
pub mod mrcc_flexcan1_clksel;
#[doc = "MRCC_FLEXCAN1_CLKDIV (rw) register accessor: FLEXCAN1 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_flexcan1_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_flexcan1_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_flexcan1_clkdiv`] module"]
#[doc(alias = "MRCC_FLEXCAN1_CLKDIV")]
pub type MrccFlexcan1Clkdiv = crate::Reg<mrcc_flexcan1_clkdiv::MrccFlexcan1ClkdivSpec>;
#[doc = "FLEXCAN1 clock divider control"]
pub mod mrcc_flexcan1_clkdiv;
#[doc = "MRCC_LPI2C2_CLKSEL (rw) register accessor: LPI2C2 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpi2c2_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpi2c2_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpi2c2_clksel`] module"]
#[doc(alias = "MRCC_LPI2C2_CLKSEL")]
pub type MrccLpi2c2Clksel = crate::Reg<mrcc_lpi2c2_clksel::MrccLpi2c2ClkselSpec>;
#[doc = "LPI2C2 clock selection control"]
pub mod mrcc_lpi2c2_clksel;
#[doc = "MRCC_LPI2C2_CLKDIV (rw) register accessor: LPI2C2 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpi2c2_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpi2c2_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpi2c2_clkdiv`] module"]
#[doc(alias = "MRCC_LPI2C2_CLKDIV")]
pub type MrccLpi2c2Clkdiv = crate::Reg<mrcc_lpi2c2_clkdiv::MrccLpi2c2ClkdivSpec>;
#[doc = "LPI2C2 clock divider control"]
pub mod mrcc_lpi2c2_clkdiv;
#[doc = "MRCC_LPI2C3_CLKSEL (rw) register accessor: LPI2C3 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpi2c3_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpi2c3_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpi2c3_clksel`] module"]
#[doc(alias = "MRCC_LPI2C3_CLKSEL")]
pub type MrccLpi2c3Clksel = crate::Reg<mrcc_lpi2c3_clksel::MrccLpi2c3ClkselSpec>;
#[doc = "LPI2C3 clock selection control"]
pub mod mrcc_lpi2c3_clksel;
#[doc = "MRCC_LPI2C3_CLKDIV (rw) register accessor: LPI2C3 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpi2c3_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpi2c3_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpi2c3_clkdiv`] module"]
#[doc(alias = "MRCC_LPI2C3_CLKDIV")]
pub type MrccLpi2c3Clkdiv = crate::Reg<mrcc_lpi2c3_clkdiv::MrccLpi2c3ClkdivSpec>;
#[doc = "LPI2C3 clock divider control"]
pub mod mrcc_lpi2c3_clkdiv;
#[doc = "MRCC_LPUART5_CLKSEL (rw) register accessor: LPUART5 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpuart5_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpuart5_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpuart5_clksel`] module"]
#[doc(alias = "MRCC_LPUART5_CLKSEL")]
pub type MrccLpuart5Clksel = crate::Reg<mrcc_lpuart5_clksel::MrccLpuart5ClkselSpec>;
#[doc = "LPUART5 clock selection control"]
pub mod mrcc_lpuart5_clksel;
#[doc = "MRCC_LPUART5_CLKDIV (rw) register accessor: LPUART5 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_lpuart5_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_lpuart5_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_lpuart5_clkdiv`] module"]
#[doc(alias = "MRCC_LPUART5_CLKDIV")]
pub type MrccLpuart5Clkdiv = crate::Reg<mrcc_lpuart5_clkdiv::MrccLpuart5ClkdivSpec>;
#[doc = "LPUART5 clock divider control"]
pub mod mrcc_lpuart5_clkdiv;
#[doc = "MRCC_DBG_TRACE_CLKSEL (rw) register accessor: DBG_TRACE clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_dbg_trace_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_dbg_trace_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_dbg_trace_clksel`] module"]
#[doc(alias = "MRCC_DBG_TRACE_CLKSEL")]
pub type MrccDbgTraceClksel = crate::Reg<mrcc_dbg_trace_clksel::MrccDbgTraceClkselSpec>;
#[doc = "DBG_TRACE clock selection control"]
pub mod mrcc_dbg_trace_clksel;
#[doc = "MRCC_DBG_TRACE_CLKDIV (rw) register accessor: DBG_TRACE clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_dbg_trace_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_dbg_trace_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_dbg_trace_clkdiv`] module"]
#[doc(alias = "MRCC_DBG_TRACE_CLKDIV")]
pub type MrccDbgTraceClkdiv = crate::Reg<mrcc_dbg_trace_clkdiv::MrccDbgTraceClkdivSpec>;
#[doc = "DBG_TRACE clock divider control"]
pub mod mrcc_dbg_trace_clkdiv;
#[doc = "MRCC_CLKOUT_CLKSEL (rw) register accessor: CLKOUT clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_clkout_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_clkout_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_clkout_clksel`] module"]
#[doc(alias = "MRCC_CLKOUT_CLKSEL")]
pub type MrccClkoutClksel = crate::Reg<mrcc_clkout_clksel::MrccClkoutClkselSpec>;
#[doc = "CLKOUT clock selection control"]
pub mod mrcc_clkout_clksel;
#[doc = "MRCC_CLKOUT_CLKDIV (rw) register accessor: CLKOUT clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_clkout_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_clkout_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_clkout_clkdiv`] module"]
#[doc(alias = "MRCC_CLKOUT_CLKDIV")]
pub type MrccClkoutClkdiv = crate::Reg<mrcc_clkout_clkdiv::MrccClkoutClkdivSpec>;
#[doc = "CLKOUT clock divider control"]
pub mod mrcc_clkout_clkdiv;
#[doc = "MRCC_SYSTICK_CLKSEL (rw) register accessor: SYSTICK clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_systick_clksel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_systick_clksel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_systick_clksel`] module"]
#[doc(alias = "MRCC_SYSTICK_CLKSEL")]
pub type MrccSystickClksel = crate::Reg<mrcc_systick_clksel::MrccSystickClkselSpec>;
#[doc = "SYSTICK clock selection control"]
pub mod mrcc_systick_clksel;
#[doc = "MRCC_SYSTICK_CLKDIV (rw) register accessor: SYSTICK clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_systick_clkdiv::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_systick_clkdiv::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mrcc_systick_clkdiv`] module"]
#[doc(alias = "MRCC_SYSTICK_CLKDIV")]
pub type MrccSystickClkdiv = crate::Reg<mrcc_systick_clkdiv::MrccSystickClkdivSpec>;
#[doc = "SYSTICK clock divider control"]
pub mod mrcc_systick_clkdiv;
