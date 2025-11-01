#[doc = "Register `MCFGR0` reader"]
pub type R = crate::R<Mcfgr0Spec>;
#[doc = "Register `MCFGR0` writer"]
pub type W = crate::W<Mcfgr0Spec>;
#[doc = "Host Request Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hren {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Hren> for bool {
    #[inline(always)]
    fn from(variant: Hren) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HREN` reader - Host Request Enable"]
pub type HrenR = crate::BitReader<Hren>;
impl HrenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hren {
        match self.bits {
            false => Hren::Disabled,
            true => Hren::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Hren::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Hren::Enabled
    }
}
#[doc = "Field `HREN` writer - Host Request Enable"]
pub type HrenW<'a, REG> = crate::BitWriter<'a, REG, Hren>;
impl<'a, REG> HrenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Hren::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Hren::Enabled)
    }
}
#[doc = "Host Request Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hrpol {
    #[doc = "0: Active low"]
    ActiveLow = 0,
    #[doc = "1: Active high"]
    ActiveHigh = 1,
}
impl From<Hrpol> for bool {
    #[inline(always)]
    fn from(variant: Hrpol) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HRPOL` reader - Host Request Polarity"]
pub type HrpolR = crate::BitReader<Hrpol>;
impl HrpolR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hrpol {
        match self.bits {
            false => Hrpol::ActiveLow,
            true => Hrpol::ActiveHigh,
        }
    }
    #[doc = "Active low"]
    #[inline(always)]
    pub fn is_active_low(&self) -> bool {
        *self == Hrpol::ActiveLow
    }
    #[doc = "Active high"]
    #[inline(always)]
    pub fn is_active_high(&self) -> bool {
        *self == Hrpol::ActiveHigh
    }
}
#[doc = "Field `HRPOL` writer - Host Request Polarity"]
pub type HrpolW<'a, REG> = crate::BitWriter<'a, REG, Hrpol>;
impl<'a, REG> HrpolW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Active low"]
    #[inline(always)]
    pub fn active_low(self) -> &'a mut crate::W<REG> {
        self.variant(Hrpol::ActiveLow)
    }
    #[doc = "Active high"]
    #[inline(always)]
    pub fn active_high(self) -> &'a mut crate::W<REG> {
        self.variant(Hrpol::ActiveHigh)
    }
}
#[doc = "Host Request Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hrsel {
    #[doc = "0: Host request input is pin HREQ"]
    Disabled = 0,
    #[doc = "1: Host request input is input trigger"]
    Enabled = 1,
}
impl From<Hrsel> for bool {
    #[inline(always)]
    fn from(variant: Hrsel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HRSEL` reader - Host Request Select"]
pub type HrselR = crate::BitReader<Hrsel>;
impl HrselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hrsel {
        match self.bits {
            false => Hrsel::Disabled,
            true => Hrsel::Enabled,
        }
    }
    #[doc = "Host request input is pin HREQ"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Hrsel::Disabled
    }
    #[doc = "Host request input is input trigger"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Hrsel::Enabled
    }
}
#[doc = "Field `HRSEL` writer - Host Request Select"]
pub type HrselW<'a, REG> = crate::BitWriter<'a, REG, Hrsel>;
impl<'a, REG> HrselW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Host request input is pin HREQ"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Hrsel::Disabled)
    }
    #[doc = "Host request input is input trigger"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Hrsel::Enabled)
    }
}
#[doc = "Host Request Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hrdir {
    #[doc = "0: HREQ pin is input (for LPI2C controller)"]
    Input = 0,
    #[doc = "1: HREQ pin is output (for LPI2C target)"]
    Output = 1,
}
impl From<Hrdir> for bool {
    #[inline(always)]
    fn from(variant: Hrdir) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HRDIR` reader - Host Request Direction"]
pub type HrdirR = crate::BitReader<Hrdir>;
impl HrdirR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hrdir {
        match self.bits {
            false => Hrdir::Input,
            true => Hrdir::Output,
        }
    }
    #[doc = "HREQ pin is input (for LPI2C controller)"]
    #[inline(always)]
    pub fn is_input(&self) -> bool {
        *self == Hrdir::Input
    }
    #[doc = "HREQ pin is output (for LPI2C target)"]
    #[inline(always)]
    pub fn is_output(&self) -> bool {
        *self == Hrdir::Output
    }
}
#[doc = "Field `HRDIR` writer - Host Request Direction"]
pub type HrdirW<'a, REG> = crate::BitWriter<'a, REG, Hrdir>;
impl<'a, REG> HrdirW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "HREQ pin is input (for LPI2C controller)"]
    #[inline(always)]
    pub fn input(self) -> &'a mut crate::W<REG> {
        self.variant(Hrdir::Input)
    }
    #[doc = "HREQ pin is output (for LPI2C target)"]
    #[inline(always)]
    pub fn output(self) -> &'a mut crate::W<REG> {
        self.variant(Hrdir::Output)
    }
}
#[doc = "Circular FIFO Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cirfifo {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Cirfifo> for bool {
    #[inline(always)]
    fn from(variant: Cirfifo) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CIRFIFO` reader - Circular FIFO Enable"]
pub type CirfifoR = crate::BitReader<Cirfifo>;
impl CirfifoR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cirfifo {
        match self.bits {
            false => Cirfifo::Disabled,
            true => Cirfifo::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Cirfifo::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Cirfifo::Enabled
    }
}
#[doc = "Field `CIRFIFO` writer - Circular FIFO Enable"]
pub type CirfifoW<'a, REG> = crate::BitWriter<'a, REG, Cirfifo>;
impl<'a, REG> CirfifoW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cirfifo::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cirfifo::Enabled)
    }
}
#[doc = "Receive Data Match Only\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdmo {
    #[doc = "0: Received data is stored in the receive FIFO"]
    Disabled = 0,
    #[doc = "1: Received data is discarded unless MSR\\[DMF\\] is set"]
    Enabled = 1,
}
impl From<Rdmo> for bool {
    #[inline(always)]
    fn from(variant: Rdmo) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RDMO` reader - Receive Data Match Only"]
pub type RdmoR = crate::BitReader<Rdmo>;
impl RdmoR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rdmo {
        match self.bits {
            false => Rdmo::Disabled,
            true => Rdmo::Enabled,
        }
    }
    #[doc = "Received data is stored in the receive FIFO"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rdmo::Disabled
    }
    #[doc = "Received data is discarded unless MSR\\[DMF\\] is set"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rdmo::Enabled
    }
}
#[doc = "Field `RDMO` writer - Receive Data Match Only"]
pub type RdmoW<'a, REG> = crate::BitWriter<'a, REG, Rdmo>;
impl<'a, REG> RdmoW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Received data is stored in the receive FIFO"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rdmo::Disabled)
    }
    #[doc = "Received data is discarded unless MSR\\[DMF\\] is set"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rdmo::Enabled)
    }
}
#[doc = "Relaxed Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relax {
    #[doc = "0: Normal transfer"]
    NormalTransfer = 0,
    #[doc = "1: Relaxed transfer"]
    RelaxedTransfer = 1,
}
impl From<Relax> for bool {
    #[inline(always)]
    fn from(variant: Relax) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RELAX` reader - Relaxed Mode"]
pub type RelaxR = crate::BitReader<Relax>;
impl RelaxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Relax {
        match self.bits {
            false => Relax::NormalTransfer,
            true => Relax::RelaxedTransfer,
        }
    }
    #[doc = "Normal transfer"]
    #[inline(always)]
    pub fn is_normal_transfer(&self) -> bool {
        *self == Relax::NormalTransfer
    }
    #[doc = "Relaxed transfer"]
    #[inline(always)]
    pub fn is_relaxed_transfer(&self) -> bool {
        *self == Relax::RelaxedTransfer
    }
}
#[doc = "Field `RELAX` writer - Relaxed Mode"]
pub type RelaxW<'a, REG> = crate::BitWriter<'a, REG, Relax>;
impl<'a, REG> RelaxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Normal transfer"]
    #[inline(always)]
    pub fn normal_transfer(self) -> &'a mut crate::W<REG> {
        self.variant(Relax::NormalTransfer)
    }
    #[doc = "Relaxed transfer"]
    #[inline(always)]
    pub fn relaxed_transfer(self) -> &'a mut crate::W<REG> {
        self.variant(Relax::RelaxedTransfer)
    }
}
#[doc = "Abort Transfer\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Abort {
    #[doc = "0: Normal transfer"]
    Disabled = 0,
    #[doc = "1: Abort existing transfer and do not start a new one"]
    Enabled = 1,
}
impl From<Abort> for bool {
    #[inline(always)]
    fn from(variant: Abort) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ABORT` reader - Abort Transfer"]
pub type AbortR = crate::BitReader<Abort>;
impl AbortR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Abort {
        match self.bits {
            false => Abort::Disabled,
            true => Abort::Enabled,
        }
    }
    #[doc = "Normal transfer"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Abort::Disabled
    }
    #[doc = "Abort existing transfer and do not start a new one"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Abort::Enabled
    }
}
#[doc = "Field `ABORT` writer - Abort Transfer"]
pub type AbortW<'a, REG> = crate::BitWriter<'a, REG, Abort>;
impl<'a, REG> AbortW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Normal transfer"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Abort::Disabled)
    }
    #[doc = "Abort existing transfer and do not start a new one"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Abort::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - Host Request Enable"]
    #[inline(always)]
    pub fn hren(&self) -> HrenR {
        HrenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Host Request Polarity"]
    #[inline(always)]
    pub fn hrpol(&self) -> HrpolR {
        HrpolR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Host Request Select"]
    #[inline(always)]
    pub fn hrsel(&self) -> HrselR {
        HrselR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Host Request Direction"]
    #[inline(always)]
    pub fn hrdir(&self) -> HrdirR {
        HrdirR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 8 - Circular FIFO Enable"]
    #[inline(always)]
    pub fn cirfifo(&self) -> CirfifoR {
        CirfifoR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Receive Data Match Only"]
    #[inline(always)]
    pub fn rdmo(&self) -> RdmoR {
        RdmoR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 16 - Relaxed Mode"]
    #[inline(always)]
    pub fn relax(&self) -> RelaxR {
        RelaxR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Abort Transfer"]
    #[inline(always)]
    pub fn abort(&self) -> AbortR {
        AbortR::new(((self.bits >> 17) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Host Request Enable"]
    #[inline(always)]
    pub fn hren(&mut self) -> HrenW<Mcfgr0Spec> {
        HrenW::new(self, 0)
    }
    #[doc = "Bit 1 - Host Request Polarity"]
    #[inline(always)]
    pub fn hrpol(&mut self) -> HrpolW<Mcfgr0Spec> {
        HrpolW::new(self, 1)
    }
    #[doc = "Bit 2 - Host Request Select"]
    #[inline(always)]
    pub fn hrsel(&mut self) -> HrselW<Mcfgr0Spec> {
        HrselW::new(self, 2)
    }
    #[doc = "Bit 3 - Host Request Direction"]
    #[inline(always)]
    pub fn hrdir(&mut self) -> HrdirW<Mcfgr0Spec> {
        HrdirW::new(self, 3)
    }
    #[doc = "Bit 8 - Circular FIFO Enable"]
    #[inline(always)]
    pub fn cirfifo(&mut self) -> CirfifoW<Mcfgr0Spec> {
        CirfifoW::new(self, 8)
    }
    #[doc = "Bit 9 - Receive Data Match Only"]
    #[inline(always)]
    pub fn rdmo(&mut self) -> RdmoW<Mcfgr0Spec> {
        RdmoW::new(self, 9)
    }
    #[doc = "Bit 16 - Relaxed Mode"]
    #[inline(always)]
    pub fn relax(&mut self) -> RelaxW<Mcfgr0Spec> {
        RelaxW::new(self, 16)
    }
    #[doc = "Bit 17 - Abort Transfer"]
    #[inline(always)]
    pub fn abort(&mut self) -> AbortW<Mcfgr0Spec> {
        AbortW::new(self, 17)
    }
}
#[doc = "Controller Configuration 0\n\nYou can [`read`](crate::Reg::read) this register and get [`mcfgr0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mcfgr0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mcfgr0Spec;
impl crate::RegisterSpec for Mcfgr0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mcfgr0::R`](R) reader structure"]
impl crate::Readable for Mcfgr0Spec {}
#[doc = "`write(|w| ..)` method takes [`mcfgr0::W`](W) writer structure"]
impl crate::Writable for Mcfgr0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MCFGR0 to value 0"]
impl crate::Resettable for Mcfgr0Spec {}
