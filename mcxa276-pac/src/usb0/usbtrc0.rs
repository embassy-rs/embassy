#[doc = "Register `USBTRC0` reader"]
pub type R = crate::R<Usbtrc0Spec>;
#[doc = "Register `USBTRC0` writer"]
pub type W = crate::W<Usbtrc0Spec>;
#[doc = "USB Asynchronous Interrupt\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsbResumeInt {
    #[doc = "0: Not generated"]
    NoAsyncInt = 0,
    #[doc = "1: Generated because of the USB asynchronous interrupt"]
    SyncIntGenerated = 1,
}
impl From<UsbResumeInt> for bool {
    #[inline(always)]
    fn from(variant: UsbResumeInt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `USB_RESUME_INT` reader - USB Asynchronous Interrupt"]
pub type UsbResumeIntR = crate::BitReader<UsbResumeInt>;
impl UsbResumeIntR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> UsbResumeInt {
        match self.bits {
            false => UsbResumeInt::NoAsyncInt,
            true => UsbResumeInt::SyncIntGenerated,
        }
    }
    #[doc = "Not generated"]
    #[inline(always)]
    pub fn is_no_async_int(&self) -> bool {
        *self == UsbResumeInt::NoAsyncInt
    }
    #[doc = "Generated because of the USB asynchronous interrupt"]
    #[inline(always)]
    pub fn is_sync_int_generated(&self) -> bool {
        *self == UsbResumeInt::SyncIntGenerated
    }
}
#[doc = "Synchronous USB Interrupt Detect\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncDet {
    #[doc = "0: Not detected"]
    NoSyncInt = 0,
    #[doc = "1: Detected"]
    SyncIntDetected = 1,
}
impl From<SyncDet> for bool {
    #[inline(always)]
    fn from(variant: SyncDet) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SYNC_DET` reader - Synchronous USB Interrupt Detect"]
pub type SyncDetR = crate::BitReader<SyncDet>;
impl SyncDetR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SyncDet {
        match self.bits {
            false => SyncDet::NoSyncInt,
            true => SyncDet::SyncIntDetected,
        }
    }
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn is_no_sync_int(&self) -> bool {
        *self == SyncDet::NoSyncInt
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn is_sync_int_detected(&self) -> bool {
        *self == SyncDet::SyncIntDetected
    }
}
#[doc = "Field `USB_CLK_RECOVERY_INT` reader - Combined USB Clock Recovery interrupt status"]
pub type UsbClkRecoveryIntR = crate::BitReader;
#[doc = "VREGIN Rising Edge Interrupt Detect\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VredgDet {
    #[doc = "0: Not detected"]
    NoVregReInt = 0,
    #[doc = "1: Detected"]
    VregReIntDetected = 1,
}
impl From<VredgDet> for bool {
    #[inline(always)]
    fn from(variant: VredgDet) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `VREDG_DET` reader - VREGIN Rising Edge Interrupt Detect"]
pub type VredgDetR = crate::BitReader<VredgDet>;
impl VredgDetR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> VredgDet {
        match self.bits {
            false => VredgDet::NoVregReInt,
            true => VredgDet::VregReIntDetected,
        }
    }
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn is_no_vreg_re_int(&self) -> bool {
        *self == VredgDet::NoVregReInt
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn is_vreg_re_int_detected(&self) -> bool {
        *self == VredgDet::VregReIntDetected
    }
}
#[doc = "VREGIN Falling Edge Interrupt Detect\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VfedgDet {
    #[doc = "0: Not detected"]
    NoVregFeInt = 0,
    #[doc = "1: Detected"]
    VregFeIntDetected = 1,
}
impl From<VfedgDet> for bool {
    #[inline(always)]
    fn from(variant: VfedgDet) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `VFEDG_DET` reader - VREGIN Falling Edge Interrupt Detect"]
pub type VfedgDetR = crate::BitReader<VfedgDet>;
impl VfedgDetR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> VfedgDet {
        match self.bits {
            false => VfedgDet::NoVregFeInt,
            true => VfedgDet::VregFeIntDetected,
        }
    }
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn is_no_vreg_fe_int(&self) -> bool {
        *self == VfedgDet::NoVregFeInt
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn is_vreg_fe_int_detected(&self) -> bool {
        *self == VfedgDet::VregFeIntDetected
    }
}
#[doc = "Asynchronous Resume Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Usbresmen {
    #[doc = "0: Disable"]
    DisAsyncWakeup = 0,
    #[doc = "1: Enable"]
    EnAsyncWakeup = 1,
}
impl From<Usbresmen> for bool {
    #[inline(always)]
    fn from(variant: Usbresmen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `USBRESMEN` reader - Asynchronous Resume Interrupt Enable"]
pub type UsbresmenR = crate::BitReader<Usbresmen>;
impl UsbresmenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Usbresmen {
        match self.bits {
            false => Usbresmen::DisAsyncWakeup,
            true => Usbresmen::EnAsyncWakeup,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_dis_async_wakeup(&self) -> bool {
        *self == Usbresmen::DisAsyncWakeup
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_en_async_wakeup(&self) -> bool {
        *self == Usbresmen::EnAsyncWakeup
    }
}
#[doc = "Field `USBRESMEN` writer - Asynchronous Resume Interrupt Enable"]
pub type UsbresmenW<'a, REG> = crate::BitWriter<'a, REG, Usbresmen>;
impl<'a, REG> UsbresmenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn dis_async_wakeup(self) -> &'a mut crate::W<REG> {
        self.variant(Usbresmen::DisAsyncWakeup)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn en_async_wakeup(self) -> &'a mut crate::W<REG> {
        self.variant(Usbresmen::EnAsyncWakeup)
    }
}
#[doc = "Field `VREGIN_STS` reader - VREGIN Status"]
pub type VreginStsR = crate::BitReader;
#[doc = "USB Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Usbreset {
    #[doc = "0: Normal USBFS operation"]
    NormalOperation = 0,
    #[doc = "1: Returns USBFS to its reset state"]
    ForceHardReset = 1,
}
impl From<Usbreset> for bool {
    #[inline(always)]
    fn from(variant: Usbreset) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `USBRESET` writer - USB Reset"]
pub type UsbresetW<'a, REG> = crate::BitWriter<'a, REG, Usbreset>;
impl<'a, REG> UsbresetW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Normal USBFS operation"]
    #[inline(always)]
    pub fn normal_operation(self) -> &'a mut crate::W<REG> {
        self.variant(Usbreset::NormalOperation)
    }
    #[doc = "Returns USBFS to its reset state"]
    #[inline(always)]
    pub fn force_hard_reset(self) -> &'a mut crate::W<REG> {
        self.variant(Usbreset::ForceHardReset)
    }
}
impl R {
    #[doc = "Bit 0 - USB Asynchronous Interrupt"]
    #[inline(always)]
    pub fn usb_resume_int(&self) -> UsbResumeIntR {
        UsbResumeIntR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Synchronous USB Interrupt Detect"]
    #[inline(always)]
    pub fn sync_det(&self) -> SyncDetR {
        SyncDetR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Combined USB Clock Recovery interrupt status"]
    #[inline(always)]
    pub fn usb_clk_recovery_int(&self) -> UsbClkRecoveryIntR {
        UsbClkRecoveryIntR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - VREGIN Rising Edge Interrupt Detect"]
    #[inline(always)]
    pub fn vredg_det(&self) -> VredgDetR {
        VredgDetR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - VREGIN Falling Edge Interrupt Detect"]
    #[inline(always)]
    pub fn vfedg_det(&self) -> VfedgDetR {
        VfedgDetR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Asynchronous Resume Interrupt Enable"]
    #[inline(always)]
    pub fn usbresmen(&self) -> UsbresmenR {
        UsbresmenR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - VREGIN Status"]
    #[inline(always)]
    pub fn vregin_sts(&self) -> VreginStsR {
        VreginStsR::new(((self.bits >> 6) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 5 - Asynchronous Resume Interrupt Enable"]
    #[inline(always)]
    pub fn usbresmen(&mut self) -> UsbresmenW<Usbtrc0Spec> {
        UsbresmenW::new(self, 5)
    }
    #[doc = "Bit 7 - USB Reset"]
    #[inline(always)]
    pub fn usbreset(&mut self) -> UsbresetW<Usbtrc0Spec> {
        UsbresetW::new(self, 7)
    }
}
#[doc = "USB Transceiver Control 0\n\nYou can [`read`](crate::Reg::read) this register and get [`usbtrc0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`usbtrc0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Usbtrc0Spec;
impl crate::RegisterSpec for Usbtrc0Spec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`usbtrc0::R`](R) reader structure"]
impl crate::Readable for Usbtrc0Spec {}
#[doc = "`write(|w| ..)` method takes [`usbtrc0::W`](W) writer structure"]
impl crate::Writable for Usbtrc0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets USBTRC0 to value 0"]
impl crate::Resettable for Usbtrc0Spec {}
