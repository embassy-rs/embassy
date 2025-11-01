#[doc = "Register `USBCTRL` reader"]
pub type R = crate::R<UsbctrlSpec>;
#[doc = "Register `USBCTRL` writer"]
pub type W = crate::W<UsbctrlSpec>;
#[doc = "DP and DM Lane Reversal Control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DpdmLaneReverse {
    #[doc = "0: Standard USB DP and DM package pin assignment"]
    DpDmStandard = 0,
    #[doc = "1: Reverse roles of USB DP and DM package pins"]
    DpDmReversed = 1,
}
impl From<DpdmLaneReverse> for bool {
    #[inline(always)]
    fn from(variant: DpdmLaneReverse) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DPDM_LANE_REVERSE` reader - DP and DM Lane Reversal Control"]
pub type DpdmLaneReverseR = crate::BitReader<DpdmLaneReverse>;
impl DpdmLaneReverseR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> DpdmLaneReverse {
        match self.bits {
            false => DpdmLaneReverse::DpDmStandard,
            true => DpdmLaneReverse::DpDmReversed,
        }
    }
    #[doc = "Standard USB DP and DM package pin assignment"]
    #[inline(always)]
    pub fn is_dp_dm_standard(&self) -> bool {
        *self == DpdmLaneReverse::DpDmStandard
    }
    #[doc = "Reverse roles of USB DP and DM package pins"]
    #[inline(always)]
    pub fn is_dp_dm_reversed(&self) -> bool {
        *self == DpdmLaneReverse::DpDmReversed
    }
}
#[doc = "Field `DPDM_LANE_REVERSE` writer - DP and DM Lane Reversal Control"]
pub type DpdmLaneReverseW<'a, REG> = crate::BitWriter<'a, REG, DpdmLaneReverse>;
impl<'a, REG> DpdmLaneReverseW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Standard USB DP and DM package pin assignment"]
    #[inline(always)]
    pub fn dp_dm_standard(self) -> &'a mut crate::W<REG> {
        self.variant(DpdmLaneReverse::DpDmStandard)
    }
    #[doc = "Reverse roles of USB DP and DM package pins"]
    #[inline(always)]
    pub fn dp_dm_reversed(self) -> &'a mut crate::W<REG> {
        self.variant(DpdmLaneReverse::DpDmReversed)
    }
}
#[doc = "Host-Mode-Only Low-Speed Device EOP Signaling\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostLsEop {
    #[doc = "0: Full-speed device or a low-speed device through a hub"]
    HostFsResumeEop = 0,
    #[doc = "1: Directly-connected low-speed device"]
    HostLsResumeEop = 1,
}
impl From<HostLsEop> for bool {
    #[inline(always)]
    fn from(variant: HostLsEop) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HOST_LS_EOP` reader - Host-Mode-Only Low-Speed Device EOP Signaling"]
pub type HostLsEopR = crate::BitReader<HostLsEop>;
impl HostLsEopR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> HostLsEop {
        match self.bits {
            false => HostLsEop::HostFsResumeEop,
            true => HostLsEop::HostLsResumeEop,
        }
    }
    #[doc = "Full-speed device or a low-speed device through a hub"]
    #[inline(always)]
    pub fn is_host_fs_resume_eop(&self) -> bool {
        *self == HostLsEop::HostFsResumeEop
    }
    #[doc = "Directly-connected low-speed device"]
    #[inline(always)]
    pub fn is_host_ls_resume_eop(&self) -> bool {
        *self == HostLsEop::HostLsResumeEop
    }
}
#[doc = "Field `HOST_LS_EOP` writer - Host-Mode-Only Low-Speed Device EOP Signaling"]
pub type HostLsEopW<'a, REG> = crate::BitWriter<'a, REG, HostLsEop>;
impl<'a, REG> HostLsEopW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Full-speed device or a low-speed device through a hub"]
    #[inline(always)]
    pub fn host_fs_resume_eop(self) -> &'a mut crate::W<REG> {
        self.variant(HostLsEop::HostFsResumeEop)
    }
    #[doc = "Directly-connected low-speed device"]
    #[inline(always)]
    pub fn host_ls_resume_eop(self) -> &'a mut crate::W<REG> {
        self.variant(HostLsEop::HostLsResumeEop)
    }
}
#[doc = "UART Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uartsel {
    #[doc = "0: USB DP and DM external package pins are used for USB signaling."]
    UsbMode = 0,
    #[doc = "1: USB DP and DM external package pins are used for UART signaling."]
    UartMode = 1,
}
impl From<Uartsel> for bool {
    #[inline(always)]
    fn from(variant: Uartsel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `UARTSEL` reader - UART Select"]
pub type UartselR = crate::BitReader<Uartsel>;
impl UartselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Uartsel {
        match self.bits {
            false => Uartsel::UsbMode,
            true => Uartsel::UartMode,
        }
    }
    #[doc = "USB DP and DM external package pins are used for USB signaling."]
    #[inline(always)]
    pub fn is_usb_mode(&self) -> bool {
        *self == Uartsel::UsbMode
    }
    #[doc = "USB DP and DM external package pins are used for UART signaling."]
    #[inline(always)]
    pub fn is_uart_mode(&self) -> bool {
        *self == Uartsel::UartMode
    }
}
#[doc = "Field `UARTSEL` writer - UART Select"]
pub type UartselW<'a, REG> = crate::BitWriter<'a, REG, Uartsel>;
impl<'a, REG> UartselW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "USB DP and DM external package pins are used for USB signaling."]
    #[inline(always)]
    pub fn usb_mode(self) -> &'a mut crate::W<REG> {
        self.variant(Uartsel::UsbMode)
    }
    #[doc = "USB DP and DM external package pins are used for UART signaling."]
    #[inline(always)]
    pub fn uart_mode(self) -> &'a mut crate::W<REG> {
        self.variant(Uartsel::UartMode)
    }
}
#[doc = "UART Signal Channel Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uartchls {
    #[doc = "0: USB DP and DM signals are used as UART TX/RX."]
    UartDpTx = 0,
    #[doc = "1: USB DP and DM signals are used as UART RX/TX."]
    UartDmTx = 1,
}
impl From<Uartchls> for bool {
    #[inline(always)]
    fn from(variant: Uartchls) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `UARTCHLS` reader - UART Signal Channel Select"]
pub type UartchlsR = crate::BitReader<Uartchls>;
impl UartchlsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Uartchls {
        match self.bits {
            false => Uartchls::UartDpTx,
            true => Uartchls::UartDmTx,
        }
    }
    #[doc = "USB DP and DM signals are used as UART TX/RX."]
    #[inline(always)]
    pub fn is_uart_dp_tx(&self) -> bool {
        *self == Uartchls::UartDpTx
    }
    #[doc = "USB DP and DM signals are used as UART RX/TX."]
    #[inline(always)]
    pub fn is_uart_dm_tx(&self) -> bool {
        *self == Uartchls::UartDmTx
    }
}
#[doc = "Field `UARTCHLS` writer - UART Signal Channel Select"]
pub type UartchlsW<'a, REG> = crate::BitWriter<'a, REG, Uartchls>;
impl<'a, REG> UartchlsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "USB DP and DM signals are used as UART TX/RX."]
    #[inline(always)]
    pub fn uart_dp_tx(self) -> &'a mut crate::W<REG> {
        self.variant(Uartchls::UartDpTx)
    }
    #[doc = "USB DP and DM signals are used as UART RX/TX."]
    #[inline(always)]
    pub fn uart_dm_tx(self) -> &'a mut crate::W<REG> {
        self.variant(Uartchls::UartDmTx)
    }
}
#[doc = "Pulldown Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pde {
    #[doc = "0: Disable on D+ and D-"]
    DisPulldowns = 0,
    #[doc = "1: Enable on D+ and D-"]
    EnPulldowns = 1,
}
impl From<Pde> for bool {
    #[inline(always)]
    fn from(variant: Pde) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDE` reader - Pulldown Enable"]
pub type PdeR = crate::BitReader<Pde>;
impl PdeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pde {
        match self.bits {
            false => Pde::DisPulldowns,
            true => Pde::EnPulldowns,
        }
    }
    #[doc = "Disable on D+ and D-"]
    #[inline(always)]
    pub fn is_dis_pulldowns(&self) -> bool {
        *self == Pde::DisPulldowns
    }
    #[doc = "Enable on D+ and D-"]
    #[inline(always)]
    pub fn is_en_pulldowns(&self) -> bool {
        *self == Pde::EnPulldowns
    }
}
#[doc = "Field `PDE` writer - Pulldown Enable"]
pub type PdeW<'a, REG> = crate::BitWriter<'a, REG, Pde>;
impl<'a, REG> PdeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable on D+ and D-"]
    #[inline(always)]
    pub fn dis_pulldowns(self) -> &'a mut crate::W<REG> {
        self.variant(Pde::DisPulldowns)
    }
    #[doc = "Enable on D+ and D-"]
    #[inline(always)]
    pub fn en_pulldowns(self) -> &'a mut crate::W<REG> {
        self.variant(Pde::EnPulldowns)
    }
}
#[doc = "Suspend\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Susp {
    #[doc = "0: Not in Suspend state"]
    XcvrNotSuspend = 0,
    #[doc = "1: In Suspend state"]
    XcvrSuspend = 1,
}
impl From<Susp> for bool {
    #[inline(always)]
    fn from(variant: Susp) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SUSP` reader - Suspend"]
pub type SuspR = crate::BitReader<Susp>;
impl SuspR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Susp {
        match self.bits {
            false => Susp::XcvrNotSuspend,
            true => Susp::XcvrSuspend,
        }
    }
    #[doc = "Not in Suspend state"]
    #[inline(always)]
    pub fn is_xcvr_not_suspend(&self) -> bool {
        *self == Susp::XcvrNotSuspend
    }
    #[doc = "In Suspend state"]
    #[inline(always)]
    pub fn is_xcvr_suspend(&self) -> bool {
        *self == Susp::XcvrSuspend
    }
}
#[doc = "Field `SUSP` writer - Suspend"]
pub type SuspW<'a, REG> = crate::BitWriter<'a, REG, Susp>;
impl<'a, REG> SuspW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not in Suspend state"]
    #[inline(always)]
    pub fn xcvr_not_suspend(self) -> &'a mut crate::W<REG> {
        self.variant(Susp::XcvrNotSuspend)
    }
    #[doc = "In Suspend state"]
    #[inline(always)]
    pub fn xcvr_suspend(self) -> &'a mut crate::W<REG> {
        self.variant(Susp::XcvrSuspend)
    }
}
impl R {
    #[doc = "Bit 2 - DP and DM Lane Reversal Control"]
    #[inline(always)]
    pub fn dpdm_lane_reverse(&self) -> DpdmLaneReverseR {
        DpdmLaneReverseR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Host-Mode-Only Low-Speed Device EOP Signaling"]
    #[inline(always)]
    pub fn host_ls_eop(&self) -> HostLsEopR {
        HostLsEopR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - UART Select"]
    #[inline(always)]
    pub fn uartsel(&self) -> UartselR {
        UartselR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - UART Signal Channel Select"]
    #[inline(always)]
    pub fn uartchls(&self) -> UartchlsR {
        UartchlsR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Pulldown Enable"]
    #[inline(always)]
    pub fn pde(&self) -> PdeR {
        PdeR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Suspend"]
    #[inline(always)]
    pub fn susp(&self) -> SuspR {
        SuspR::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 2 - DP and DM Lane Reversal Control"]
    #[inline(always)]
    pub fn dpdm_lane_reverse(&mut self) -> DpdmLaneReverseW<UsbctrlSpec> {
        DpdmLaneReverseW::new(self, 2)
    }
    #[doc = "Bit 3 - Host-Mode-Only Low-Speed Device EOP Signaling"]
    #[inline(always)]
    pub fn host_ls_eop(&mut self) -> HostLsEopW<UsbctrlSpec> {
        HostLsEopW::new(self, 3)
    }
    #[doc = "Bit 4 - UART Select"]
    #[inline(always)]
    pub fn uartsel(&mut self) -> UartselW<UsbctrlSpec> {
        UartselW::new(self, 4)
    }
    #[doc = "Bit 5 - UART Signal Channel Select"]
    #[inline(always)]
    pub fn uartchls(&mut self) -> UartchlsW<UsbctrlSpec> {
        UartchlsW::new(self, 5)
    }
    #[doc = "Bit 6 - Pulldown Enable"]
    #[inline(always)]
    pub fn pde(&mut self) -> PdeW<UsbctrlSpec> {
        PdeW::new(self, 6)
    }
    #[doc = "Bit 7 - Suspend"]
    #[inline(always)]
    pub fn susp(&mut self) -> SuspW<UsbctrlSpec> {
        SuspW::new(self, 7)
    }
}
#[doc = "USB Control\n\nYou can [`read`](crate::Reg::read) this register and get [`usbctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`usbctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct UsbctrlSpec;
impl crate::RegisterSpec for UsbctrlSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`usbctrl::R`](R) reader structure"]
impl crate::Readable for UsbctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`usbctrl::W`](W) writer structure"]
impl crate::Writable for UsbctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets USBCTRL to value 0xc0"]
impl crate::Resettable for UsbctrlSpec {
    const RESET_VALUE: u8 = 0xc0;
}
