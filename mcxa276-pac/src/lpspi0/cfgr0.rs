#[doc = "Register `CFGR0` reader"]
pub type R = crate::R<Cfgr0Spec>;
#[doc = "Register `CFGR0` writer"]
pub type W = crate::W<Cfgr0Spec>;
#[doc = "Host Request Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hren {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
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
            false => Hren::Disable,
            true => Hren::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Hren::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Hren::Enable
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
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Hren::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Hren::Enable)
    }
}
#[doc = "Host Request Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hrpol {
    #[doc = "0: Active high"]
    Disabled = 0,
    #[doc = "1: Active low"]
    Enabled = 1,
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
            false => Hrpol::Disabled,
            true => Hrpol::Enabled,
        }
    }
    #[doc = "Active high"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Hrpol::Disabled
    }
    #[doc = "Active low"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Hrpol::Enabled
    }
}
#[doc = "Field `HRPOL` writer - Host Request Polarity"]
pub type HrpolW<'a, REG> = crate::BitWriter<'a, REG, Hrpol>;
impl<'a, REG> HrpolW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Active high"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Hrpol::Disabled)
    }
    #[doc = "Active low"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Hrpol::Enabled)
    }
}
#[doc = "Host Request Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hrsel {
    #[doc = "0: HREQ pin"]
    Hreqpin = 0,
    #[doc = "1: Input trigger"]
    InputTrigger = 1,
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
            false => Hrsel::Hreqpin,
            true => Hrsel::InputTrigger,
        }
    }
    #[doc = "HREQ pin"]
    #[inline(always)]
    pub fn is_hreqpin(&self) -> bool {
        *self == Hrsel::Hreqpin
    }
    #[doc = "Input trigger"]
    #[inline(always)]
    pub fn is_input_trigger(&self) -> bool {
        *self == Hrsel::InputTrigger
    }
}
#[doc = "Field `HRSEL` writer - Host Request Select"]
pub type HrselW<'a, REG> = crate::BitWriter<'a, REG, Hrsel>;
impl<'a, REG> HrselW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "HREQ pin"]
    #[inline(always)]
    pub fn hreqpin(self) -> &'a mut crate::W<REG> {
        self.variant(Hrsel::Hreqpin)
    }
    #[doc = "Input trigger"]
    #[inline(always)]
    pub fn input_trigger(self) -> &'a mut crate::W<REG> {
        self.variant(Hrsel::InputTrigger)
    }
}
#[doc = "Host Request Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hrdir {
    #[doc = "0: Input"]
    Input = 0,
    #[doc = "1: Output"]
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
    #[doc = "Input"]
    #[inline(always)]
    pub fn is_input(&self) -> bool {
        *self == Hrdir::Input
    }
    #[doc = "Output"]
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
    #[doc = "Input"]
    #[inline(always)]
    pub fn input(self) -> &'a mut crate::W<REG> {
        self.variant(Hrdir::Input)
    }
    #[doc = "Output"]
    #[inline(always)]
    pub fn output(self) -> &'a mut crate::W<REG> {
        self.variant(Hrdir::Output)
    }
}
#[doc = "Circular FIFO Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cirfifo {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
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
            false => Cirfifo::Disable,
            true => Cirfifo::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Cirfifo::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Cirfifo::Enable
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
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Cirfifo::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Cirfifo::Enable)
    }
}
#[doc = "Receive Data Match Only\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdmo {
    #[doc = "0: Disable"]
    Stored = 0,
    #[doc = "1: Enable"]
    Discarded = 1,
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
            false => Rdmo::Stored,
            true => Rdmo::Discarded,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_stored(&self) -> bool {
        *self == Rdmo::Stored
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_discarded(&self) -> bool {
        *self == Rdmo::Discarded
    }
}
#[doc = "Field `RDMO` writer - Receive Data Match Only"]
pub type RdmoW<'a, REG> = crate::BitWriter<'a, REG, Rdmo>;
impl<'a, REG> RdmoW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn stored(self) -> &'a mut crate::W<REG> {
        self.variant(Rdmo::Stored)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn discarded(self) -> &'a mut crate::W<REG> {
        self.variant(Rdmo::Discarded)
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
}
impl W {
    #[doc = "Bit 0 - Host Request Enable"]
    #[inline(always)]
    pub fn hren(&mut self) -> HrenW<Cfgr0Spec> {
        HrenW::new(self, 0)
    }
    #[doc = "Bit 1 - Host Request Polarity"]
    #[inline(always)]
    pub fn hrpol(&mut self) -> HrpolW<Cfgr0Spec> {
        HrpolW::new(self, 1)
    }
    #[doc = "Bit 2 - Host Request Select"]
    #[inline(always)]
    pub fn hrsel(&mut self) -> HrselW<Cfgr0Spec> {
        HrselW::new(self, 2)
    }
    #[doc = "Bit 3 - Host Request Direction"]
    #[inline(always)]
    pub fn hrdir(&mut self) -> HrdirW<Cfgr0Spec> {
        HrdirW::new(self, 3)
    }
    #[doc = "Bit 8 - Circular FIFO Enable"]
    #[inline(always)]
    pub fn cirfifo(&mut self) -> CirfifoW<Cfgr0Spec> {
        CirfifoW::new(self, 8)
    }
    #[doc = "Bit 9 - Receive Data Match Only"]
    #[inline(always)]
    pub fn rdmo(&mut self) -> RdmoW<Cfgr0Spec> {
        RdmoW::new(self, 9)
    }
}
#[doc = "Configuration 0\n\nYou can [`read`](crate::Reg::read) this register and get [`cfgr0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfgr0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Cfgr0Spec;
impl crate::RegisterSpec for Cfgr0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cfgr0::R`](R) reader structure"]
impl crate::Readable for Cfgr0Spec {}
#[doc = "`write(|w| ..)` method takes [`cfgr0::W`](W) writer structure"]
impl crate::Writable for Cfgr0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CFGR0 to value 0"]
impl crate::Resettable for Cfgr0Spec {}
