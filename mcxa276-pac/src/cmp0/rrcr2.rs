#[doc = "Register `RRCR2` reader"]
pub type R = crate::R<Rrcr2Spec>;
#[doc = "Register `RRCR2` writer"]
pub type W = crate::W<Rrcr2Spec>;
#[doc = "Field `RR_TIMER_RELOAD` reader - Number of Sample Clocks"]
pub type RrTimerReloadR = crate::FieldReader<u32>;
#[doc = "Field `RR_TIMER_RELOAD` writer - Number of Sample Clocks"]
pub type RrTimerReloadW<'a, REG> = crate::FieldWriter<'a, REG, 28, u32>;
#[doc = "Round-Robin Internal Timer Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrTimerEn {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<RrTimerEn> for bool {
    #[inline(always)]
    fn from(variant: RrTimerEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_TIMER_EN` reader - Round-Robin Internal Timer Enable"]
pub type RrTimerEnR = crate::BitReader<RrTimerEn>;
impl RrTimerEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrTimerEn {
        match self.bits {
            false => RrTimerEn::Disable,
            true => RrTimerEn::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RrTimerEn::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RrTimerEn::Enable
    }
}
#[doc = "Field `RR_TIMER_EN` writer - Round-Robin Internal Timer Enable"]
pub type RrTimerEnW<'a, REG> = crate::BitWriter<'a, REG, RrTimerEn>;
impl<'a, REG> RrTimerEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RrTimerEn::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RrTimerEn::Enable)
    }
}
impl R {
    #[doc = "Bits 0:27 - Number of Sample Clocks"]
    #[inline(always)]
    pub fn rr_timer_reload(&self) -> RrTimerReloadR {
        RrTimerReloadR::new(self.bits & 0x0fff_ffff)
    }
    #[doc = "Bit 31 - Round-Robin Internal Timer Enable"]
    #[inline(always)]
    pub fn rr_timer_en(&self) -> RrTimerEnR {
        RrTimerEnR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:27 - Number of Sample Clocks"]
    #[inline(always)]
    pub fn rr_timer_reload(&mut self) -> RrTimerReloadW<Rrcr2Spec> {
        RrTimerReloadW::new(self, 0)
    }
    #[doc = "Bit 31 - Round-Robin Internal Timer Enable"]
    #[inline(always)]
    pub fn rr_timer_en(&mut self) -> RrTimerEnW<Rrcr2Spec> {
        RrTimerEnW::new(self, 31)
    }
}
#[doc = "Round Robin Control Register 2\n\nYou can [`read`](crate::Reg::read) this register and get [`rrcr2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rrcr2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Rrcr2Spec;
impl crate::RegisterSpec for Rrcr2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rrcr2::R`](R) reader structure"]
impl crate::Readable for Rrcr2Spec {}
#[doc = "`write(|w| ..)` method takes [`rrcr2::W`](W) writer structure"]
impl crate::Writable for Rrcr2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RRCR2 to value 0"]
impl crate::Resettable for Rrcr2Spec {}
