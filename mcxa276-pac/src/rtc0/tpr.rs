#[doc = "Register `TPR` reader"]
pub type R = crate::R<TprSpec>;
#[doc = "Register `TPR` writer"]
pub type W = crate::W<TprSpec>;
#[doc = "Field `TPR` reader - Time Prescaler Register"]
pub type TprR = crate::FieldReader<u16>;
#[doc = "Field `TPR` writer - Time Prescaler Register"]
pub type TprW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Time Prescaler Register"]
    #[inline(always)]
    pub fn tpr(&self) -> TprR {
        TprR::new((self.bits & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Time Prescaler Register"]
    #[inline(always)]
    pub fn tpr(&mut self) -> TprW<TprSpec> {
        TprW::new(self, 0)
    }
}
#[doc = "RTC Time Prescaler\n\nYou can [`read`](crate::Reg::read) this register and get [`tpr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tpr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TprSpec;
impl crate::RegisterSpec for TprSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`tpr::R`](R) reader structure"]
impl crate::Readable for TprSpec {}
#[doc = "`write(|w| ..)` method takes [`tpr::W`](W) writer structure"]
impl crate::Writable for TprSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TPR to value 0"]
impl crate::Resettable for TprSpec {}
