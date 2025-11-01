#[doc = "Register `CTL` reader"]
pub type R = crate::R<CtlSpec>;
#[doc = "Register `CTL` writer"]
pub type W = crate::W<CtlSpec>;
#[doc = "USB Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Usbensofen {
    #[doc = "0: Disable"]
    DisUsbSof = 0,
    #[doc = "1: Enable"]
    EnUsbSof = 1,
}
impl From<Usbensofen> for bool {
    #[inline(always)]
    fn from(variant: Usbensofen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `USBENSOFEN` reader - USB Enable"]
pub type UsbensofenR = crate::BitReader<Usbensofen>;
impl UsbensofenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Usbensofen {
        match self.bits {
            false => Usbensofen::DisUsbSof,
            true => Usbensofen::EnUsbSof,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_usb_sof(&self) -> bool {
        *self == Usbensofen::DisUsbSof
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_usb_sof(&self) -> bool {
        *self == Usbensofen::EnUsbSof
    }
}
#[doc = "Field `USBENSOFEN` writer - USB Enable"]
pub type UsbensofenW<'a, REG> = crate::BitWriter<'a, REG, Usbensofen>;
impl<'a, REG> UsbensofenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_usb_sof(self) -> &'a mut crate::W<REG> {
        self.variant(Usbensofen::DisUsbSof)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_usb_sof(self) -> &'a mut crate::W<REG> {
        self.variant(Usbensofen::EnUsbSof)
    }
}
#[doc = "Field `ODDRST` reader - Odd Reset"]
pub type OddrstR = crate::BitReader;
#[doc = "Field `ODDRST` writer - Odd Reset"]
pub type OddrstW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RESUME` reader - Resume"]
pub type ResumeR = crate::BitReader;
#[doc = "Field `RESUME` writer - Resume"]
pub type ResumeW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Host Mode Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hostmodeen {
    #[doc = "0: USBFS operates in Device mode."]
    EnDeviceMode = 0,
    #[doc = "1: USBFS operates in Host mode. In Host mode, USBFS performs USB transactions under the programmed control of the host processor."]
    EnHostMode = 1,
}
impl From<Hostmodeen> for bool {
    #[inline(always)]
    fn from(variant: Hostmodeen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HOSTMODEEN` reader - Host Mode Enable"]
pub type HostmodeenR = crate::BitReader<Hostmodeen>;
impl HostmodeenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hostmodeen {
        match self.bits {
            false => Hostmodeen::EnDeviceMode,
            true => Hostmodeen::EnHostMode,
        }
    }
    #[doc = "USBFS operates in Device mode."]
    #[inline(always)]
    pub fn is_en_device_mode(&self) -> bool {
        *self == Hostmodeen::EnDeviceMode
    }
    #[doc = "USBFS operates in Host mode. In Host mode, USBFS performs USB transactions under the programmed control of the host processor."]
    #[inline(always)]
    pub fn is_en_host_mode(&self) -> bool {
        *self == Hostmodeen::EnHostMode
    }
}
#[doc = "Field `HOSTMODEEN` writer - Host Mode Enable"]
pub type HostmodeenW<'a, REG> = crate::BitWriter<'a, REG, Hostmodeen>;
impl<'a, REG> HostmodeenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "USBFS operates in Device mode."]
    #[inline(always)]
    pub fn en_device_mode(self) -> &'a mut crate::W<REG> {
        self.variant(Hostmodeen::EnDeviceMode)
    }
    #[doc = "USBFS operates in Host mode. In Host mode, USBFS performs USB transactions under the programmed control of the host processor."]
    #[inline(always)]
    pub fn en_host_mode(self) -> &'a mut crate::W<REG> {
        self.variant(Hostmodeen::EnHostMode)
    }
}
#[doc = "Reset Signaling Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reset {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Reset> for bool {
    #[inline(always)]
    fn from(variant: Reset) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RESET` reader - Reset Signaling Enable"]
pub type ResetR = crate::BitReader<Reset>;
impl ResetR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Reset {
        match self.bits {
            false => Reset::Disable,
            true => Reset::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Reset::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Reset::Enable
    }
}
#[doc = "Field `RESET` writer - Reset Signaling Enable"]
pub type ResetW<'a, REG> = crate::BitWriter<'a, REG, Reset>;
impl<'a, REG> ResetW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Reset::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Reset::Enable)
    }
}
#[doc = "Field `TXSUSPENDTOKENBUSY` reader - TXD Suspend And Token Busy"]
pub type TxsuspendtokenbusyR = crate::BitReader;
#[doc = "Field `TXSUSPENDTOKENBUSY` writer - TXD Suspend And Token Busy"]
pub type TxsuspendtokenbusyW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SE0` reader - Live USB Single-Ended Zero signal"]
pub type Se0R = crate::BitReader;
#[doc = "Field `SE0` writer - Live USB Single-Ended Zero signal"]
pub type Se0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `JSTATE` reader - Live USB Differential Receiver JSTATE Signal"]
pub type JstateR = crate::BitReader;
#[doc = "Field `JSTATE` writer - Live USB Differential Receiver JSTATE Signal"]
pub type JstateW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - USB Enable"]
    #[inline(always)]
    pub fn usbensofen(&self) -> UsbensofenR {
        UsbensofenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Odd Reset"]
    #[inline(always)]
    pub fn oddrst(&self) -> OddrstR {
        OddrstR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Resume"]
    #[inline(always)]
    pub fn resume(&self) -> ResumeR {
        ResumeR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Host Mode Enable"]
    #[inline(always)]
    pub fn hostmodeen(&self) -> HostmodeenR {
        HostmodeenR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Reset Signaling Enable"]
    #[inline(always)]
    pub fn reset(&self) -> ResetR {
        ResetR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - TXD Suspend And Token Busy"]
    #[inline(always)]
    pub fn txsuspendtokenbusy(&self) -> TxsuspendtokenbusyR {
        TxsuspendtokenbusyR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Live USB Single-Ended Zero signal"]
    #[inline(always)]
    pub fn se0(&self) -> Se0R {
        Se0R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Live USB Differential Receiver JSTATE Signal"]
    #[inline(always)]
    pub fn jstate(&self) -> JstateR {
        JstateR::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - USB Enable"]
    #[inline(always)]
    pub fn usbensofen(&mut self) -> UsbensofenW<CtlSpec> {
        UsbensofenW::new(self, 0)
    }
    #[doc = "Bit 1 - Odd Reset"]
    #[inline(always)]
    pub fn oddrst(&mut self) -> OddrstW<CtlSpec> {
        OddrstW::new(self, 1)
    }
    #[doc = "Bit 2 - Resume"]
    #[inline(always)]
    pub fn resume(&mut self) -> ResumeW<CtlSpec> {
        ResumeW::new(self, 2)
    }
    #[doc = "Bit 3 - Host Mode Enable"]
    #[inline(always)]
    pub fn hostmodeen(&mut self) -> HostmodeenW<CtlSpec> {
        HostmodeenW::new(self, 3)
    }
    #[doc = "Bit 4 - Reset Signaling Enable"]
    #[inline(always)]
    pub fn reset(&mut self) -> ResetW<CtlSpec> {
        ResetW::new(self, 4)
    }
    #[doc = "Bit 5 - TXD Suspend And Token Busy"]
    #[inline(always)]
    pub fn txsuspendtokenbusy(&mut self) -> TxsuspendtokenbusyW<CtlSpec> {
        TxsuspendtokenbusyW::new(self, 5)
    }
    #[doc = "Bit 6 - Live USB Single-Ended Zero signal"]
    #[inline(always)]
    pub fn se0(&mut self) -> Se0W<CtlSpec> {
        Se0W::new(self, 6)
    }
    #[doc = "Bit 7 - Live USB Differential Receiver JSTATE Signal"]
    #[inline(always)]
    pub fn jstate(&mut self) -> JstateW<CtlSpec> {
        JstateW::new(self, 7)
    }
}
#[doc = "Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ctl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CtlSpec;
impl crate::RegisterSpec for CtlSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`ctl::R`](R) reader structure"]
impl crate::Readable for CtlSpec {}
#[doc = "`write(|w| ..)` method takes [`ctl::W`](W) writer structure"]
impl crate::Writable for CtlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CTL to value 0"]
impl crate::Resettable for CtlSpec {}
