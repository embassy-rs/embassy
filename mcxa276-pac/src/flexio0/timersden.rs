#[doc = "Register `TIMERSDEN` reader"]
pub type R = crate::R<TimersdenSpec>;
#[doc = "Register `TIMERSDEN` writer"]
pub type W = crate::W<TimersdenSpec>;
#[doc = "Field `TSDE` reader - Timer Status DMA Enable"]
pub type TsdeR = crate::FieldReader;
#[doc = "Field `TSDE` writer - Timer Status DMA Enable"]
pub type TsdeW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - Timer Status DMA Enable"]
    #[inline(always)]
    pub fn tsde(&self) -> TsdeR {
        TsdeR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Timer Status DMA Enable"]
    #[inline(always)]
    pub fn tsde(&mut self) -> TsdeW<TimersdenSpec> {
        TsdeW::new(self, 0)
    }
}
#[doc = "Timer Status DMA Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`timersden::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timersden::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TimersdenSpec;
impl crate::RegisterSpec for TimersdenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`timersden::R`](R) reader structure"]
impl crate::Readable for TimersdenSpec {}
#[doc = "`write(|w| ..)` method takes [`timersden::W`](W) writer structure"]
impl crate::Writable for TimersdenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TIMERSDEN to value 0"]
impl crate::Resettable for TimersdenSpec {}
