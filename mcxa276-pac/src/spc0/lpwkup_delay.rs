#[doc = "Register `LPWKUP_DELAY` reader"]
pub type R = crate::R<LpwkupDelaySpec>;
#[doc = "Register `LPWKUP_DELAY` writer"]
pub type W = crate::W<LpwkupDelaySpec>;
#[doc = "Field `LPWKUP_DELAY` reader - Low-Power Wake-Up Delay"]
pub type LpwkupDelayR = crate::FieldReader<u16>;
#[doc = "Field `LPWKUP_DELAY` writer - Low-Power Wake-Up Delay"]
pub type LpwkupDelayW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Low-Power Wake-Up Delay"]
    #[inline(always)]
    pub fn lpwkup_delay(&self) -> LpwkupDelayR {
        LpwkupDelayR::new((self.bits & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Low-Power Wake-Up Delay"]
    #[inline(always)]
    pub fn lpwkup_delay(&mut self) -> LpwkupDelayW<LpwkupDelaySpec> {
        LpwkupDelayW::new(self, 0)
    }
}
#[doc = "Low Power Wake-Up Delay\n\nYou can [`read`](crate::Reg::read) this register and get [`lpwkup_delay::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpwkup_delay::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LpwkupDelaySpec;
impl crate::RegisterSpec for LpwkupDelaySpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`lpwkup_delay::R`](R) reader structure"]
impl crate::Readable for LpwkupDelaySpec {}
#[doc = "`write(|w| ..)` method takes [`lpwkup_delay::W`](W) writer structure"]
impl crate::Writable for LpwkupDelaySpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LPWKUP_DELAY to value 0"]
impl crate::Resettable for LpwkupDelaySpec {}
